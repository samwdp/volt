impl ShellState {
    pub(crate) fn handle_text_input(&mut self, text: &str) -> Result<(), ShellError> {
        if self.should_suppress_text_input(text) {
            return Ok(());
        }
        if self.text_input_activates_typing_budget()? {
            self.note_text_edit_activity();
        }
        self.last_text_input_profile = None;
        let profile_started = self.typing_profiler.as_ref().map(|_| Instant::now());
        let hover_before = self.ui()?.hover().cloned();
        let result = self.handle_text_input_inner(text);
        let hover_changed = result.is_ok() && self.ui()?.hover().cloned() != hover_before;
        let result = result.and_then(|()| {
            if hover_changed {
                Ok(())
            } else {
                self.refresh_hover_state().map(|_| ())
            }
        });
        if let Some(profile_started) = profile_started {
            self.last_text_input_profile = Some(profile_started.elapsed());
        }
        result
    }

    fn text_input_activates_typing_budget(&self) -> Result<bool, ShellError> {
        if self.ui()?.input_prompt_visible()
            || self.command_line_visible()?
            || self.picker_visible()?
        {
            return Ok(true);
        }
        Ok(matches!(
            self.input_mode()?,
            InputMode::Insert | InputMode::Replace
        ))
    }

    fn handle_text_input_inner(&mut self, text: &str) -> Result<(), ShellError> {
        if self.ui()?.input_prompt_visible() {
            clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
            if let Some(prompt) = self.ui_mut()?.input_prompt_mut() {
                prompt.append_text(text);
            }
            return Ok(());
        }
        if self.command_line_visible()? {
            clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
            if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                command_line.append_text(text);
            }
            return Ok(());
        }
        let acp_inline_picker_active =
            matches!(self.ui()?.picker_kind(), Some(kind) if kind.is_acp_inline());
        if self.picker_visible()? && !acp_inline_picker_active {
            clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
            if let Some(picker) = self.ui_mut()?.picker_mut() {
                picker.append_query(text);
            }
            self.schedule_picker_search_refresh()?;
            return Ok(());
        }
        if acp_inline_picker_active {
            self.ui_mut()?.close_picker();
        }
        let active_buffer =
            active_buffer_event_context(&self.runtime).map_err(ShellError::Runtime)?;

        match self.input_mode()? {
            InputMode::Insert => {
                clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
                if self.ui()?.vim().multicursor.is_some() && !active_buffer.vim_targets_input {
                    apply_multicursor_insert_text(&mut self.runtime, text, false)
                        .map_err(ShellError::Runtime)?;
                    self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                    self.maybe_finish_change_after_input()?;
                    return Ok(());
                }
                if active_buffer.is_terminal {
                    write_active_terminal_text(&mut self.runtime, text)
                        .map_err(ShellError::Runtime)?;
                    return Ok(());
                }
                if active_buffer.vim_targets_input {
                    let buffer_id =
                        active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                    let handled = {
                        let buffer = self.active_buffer_mut()?;
                        if let Some(input) = buffer.input_field_mut() {
                            input.append_text(text);
                            true
                        } else {
                            false
                        }
                    };
                    if handled {
                        acp::maybe_open_acp_input_completion(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        acp::refresh_acp_input_hint(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                        self.maybe_finish_change_after_input()?;
                    }
                    return Ok(());
                }
                if active_shell_buffer_read_only(&self.runtime).map_err(ShellError::Runtime)? {
                    return Ok(());
                }
                let buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let (indent_size, use_tabs) = {
                    let ui = self.ui()?;
                    let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
                    let theme_registry = self.runtime.services().get::<ThemeRegistry>();
                    (
                        theme_lang_indent(theme_registry, language_id),
                        theme_lang_use_tabs(theme_registry, language_id),
                    )
                };
                let normalized = normalize_tabs(text, indent_size, use_tabs);
                let defer_markdown_table_format = should_defer_table_format(
                    shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?,
                    text,
                );
                {
                    let buffer = self.active_buffer_mut()?;
                    buffer.insert_text(normalized.as_ref());
                    if !defer_markdown_table_format {
                        let _ = format_markdown_table_at_cursor(buffer);
                    }
                }
                if should_format_current_line_after_closing_delimiter(
                    shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?,
                    text,
                ) {
                    format_current_line_indent(&mut self.runtime, buffer_id, indent_size, use_tabs)
                        .map_err(ShellError::Runtime)?;
                }
                self.mark_active_buffer_syntax_dirty()?;
                self.schedule_autocomplete_refresh_if_active()?;
                self.schedule_inline_completion_refresh()?;
                self.record_vim_input(VimRecordedInput::Text(normalized.to_string()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            InputMode::Replace => {
                clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
                if self.ui()?.vim().multicursor.is_some() && !active_buffer.vim_targets_input {
                    apply_multicursor_insert_text(&mut self.runtime, text, true)
                        .map_err(ShellError::Runtime)?;
                    self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                    self.maybe_finish_change_after_input()?;
                    return Ok(());
                }
                if active_buffer.is_terminal {
                    write_active_terminal_text(&mut self.runtime, text)
                        .map_err(ShellError::Runtime)?;
                    return Ok(());
                }
                if active_buffer.vim_targets_input {
                    let buffer_id =
                        active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                    let handled = {
                        let buffer = self.active_buffer_mut()?;
                        if let Some(input) = buffer.input_field_mut() {
                            input.append_text(text);
                            true
                        } else {
                            false
                        }
                    };
                    if handled {
                        acp::maybe_open_acp_input_completion(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        acp::refresh_acp_input_hint(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        self.record_vim_input(VimRecordedInput::Text(text.to_owned()))?;
                        self.maybe_finish_change_after_input()?;
                    }
                    return Ok(());
                }
                if active_shell_buffer_read_only(&self.runtime).map_err(ShellError::Runtime)? {
                    return Ok(());
                }
                let buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let (indent_size, use_tabs) = {
                    let ui = self.ui()?;
                    let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
                    let theme_registry = self.runtime.services().get::<ThemeRegistry>();
                    (
                        theme_lang_indent(theme_registry, language_id),
                        theme_lang_use_tabs(theme_registry, language_id),
                    )
                };
                let normalized = normalize_tabs(text, indent_size, use_tabs);
                let defer_markdown_table_format = should_defer_table_format(
                    shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?,
                    text,
                );
                {
                    let buffer = self.active_buffer_mut()?;
                    buffer.replace_mode_text(normalized.as_ref());
                    if !defer_markdown_table_format {
                        let _ = format_markdown_table_at_cursor(buffer);
                    }
                }
                if should_format_current_line_after_closing_delimiter(
                    shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?,
                    text,
                ) {
                    format_current_line_indent(&mut self.runtime, buffer_id, indent_size, use_tabs)
                        .map_err(ShellError::Runtime)?;
                }
                self.mark_active_buffer_syntax_dirty()?;
                self.schedule_autocomplete_refresh_if_active()?;
                self.schedule_inline_completion_refresh()?;
                self.record_vim_input(VimRecordedInput::Text(normalized.to_string()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            _ => {}
        }

        if let Some(chord) = text_chord(text) {
            if self.handle_focused_hover_text_input(&chord)? {
                return Ok(());
            }
            let vim_mode = keymap_vim_mode(self.input_mode()?);
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && active_buffer.is_git_status
                && chord == "V"
                && self.runtime.keymaps().contains_for_mode(
                    &KeymapScope::Workspace,
                    vim_mode,
                    &chord,
                )
            {
                let runtime_surface_before =
                    active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
                self.runtime
                    .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &chord)
                    .map_err(|error| ShellError::Runtime(error.to_string()))?;
                self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                self.clear_stale_vim_count()?;
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && handle_git_status_chord(&mut self.runtime, &chord)
                    .map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.vim_mut().clear_transient();
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && handle_git_view_chord(&mut self.runtime, &chord).map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.vim_mut().clear_transient();
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
                && handle_directory_chord(&mut self.runtime, &chord).map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.vim_mut().clear_transient();
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }
            if self.handle_vim_pending_text(&chord)? || self.handle_vim_count_input(&chord)? {
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }

            let token = normalize_text_token(&chord);
            if self.handle_key_sequence(&token, KeymapScope::Global, vim_mode)? {
                return Ok(());
            }

            let picker_visible = self.ui()?.picker_visible();
            if self.try_plugin_or_overlay_keybinding(&token, vim_mode, picker_visible)? {
                let runtime_surface_before =
                    active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
                self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                self.clear_stale_vim_count()?;
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
                return Ok(());
            }

            if self.ui()?.vim().multicursor.is_some()
                && self.handle_key_sequence(&token, KeymapScope::Multicursor, vim_mode)?
            {
                return Ok(());
            }

            if self.handle_key_sequence(&token, KeymapScope::Workspace, vim_mode)? {
                return Ok(());
            }

            if chord == "." && !self.picker_visible()? {
                self.repeat_last_change()?;
                return Ok(());
            }

            let editing_modes = self.editing_minor_modes()?;
            if self
                .runtime
                .execute_key_binding_with_minor_modes(&editing_modes, vim_mode, &token)
                .map_err(|error| ShellError::Runtime(error.to_string()))?
            {
                let runtime_surface_before =
                    active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
                self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                self.clear_stale_vim_count()?;
                self.record_vim_input(VimRecordedInput::Text(chord.to_owned()))?;
                self.maybe_finish_change_after_input()?;
            }
        }

        Ok(())
    }

    fn schedule_picker_search_refresh(&mut self) -> Result<(), ShellError> {
        enum DynamicPickerSearch {
            Vim {
                buffer_id: BufferId,
                buffer_revision: u64,
                text: TextSnapshot,
                direction: VimSearchDirection,
                query: String,
            },
            Workspace {
                root: PathBuf,
                query: String,
            },
        }

        let pending = {
            let ui = self.ui()?;
            let Some(picker) = ui.picker() else {
                return Ok(());
            };
            let query = picker.session().query().to_owned();
            if let Some(direction) = picker.vim_search_direction() {
                let buffer_id = ui
                    .active_buffer_id()
                    .ok_or_else(|| ShellError::Runtime("active buffer is missing".to_owned()))?;
                let buffer = ui.buffer(buffer_id).ok_or_else(|| {
                    ShellError::Runtime("active shell buffer is missing".to_owned())
                })?;
                Some(DynamicPickerSearch::Vim {
                    buffer_id,
                    buffer_revision: buffer.text.revision(),
                    text: buffer.text.snapshot(),
                    direction,
                    query,
                })
            } else {
                picker
                    .workspace_search_root()
                    .map(|root| DynamicPickerSearch::Workspace {
                        root: root.to_path_buf(),
                        query,
                    })
            }
        };

        match pending {
            Some(DynamicPickerSearch::Vim {
                buffer_id,
                buffer_revision,
                text,
                direction,
                query,
            }) => {
                self.ui_mut()?.vim_search_worker.schedule(
                    buffer_id,
                    buffer_revision,
                    text,
                    direction,
                    query,
                );
            }
            Some(DynamicPickerSearch::Workspace { root, query }) => {
                self.ui_mut()?.workspace_search_worker.schedule(root, query);
            }
            None => {}
        }

        Ok(())
    }

    fn refresh_pending_picker_searches(&mut self) -> Result<bool, ShellError> {
        let now = Instant::now();
        {
            let ui = self.ui_mut()?;
            ui.vim_search_worker.dispatch_due(now);
            ui.workspace_search_worker.dispatch_due(now);
        }

        let mut changed = false;
        if let Some(result) = self.ui()?.vim_search_worker.take_latest_result() {
            let should_apply = {
                let ui = self.ui()?;
                if let Some(picker) = ui.picker()
                    && let Some(buffer) = ui.buffer(result.buffer_id)
                {
                    picker.vim_search_direction() == Some(result.direction)
                        && picker.session().query() == result.query
                        && ui.active_buffer_id() == Some(result.buffer_id)
                        && buffer.text.revision() == result.buffer_revision
                        && result.request_id == ui.vim_search_worker.next_request_id
                } else {
                    false
                }
            };
            if should_apply
                && let Some(picker) = self.ui_mut()?.picker_mut()
                && picker.vim_search_direction() == Some(result.direction)
            {
                picker.set_entries(result.data.entries, result.data.selected_index);
                changed = true;
            }
        }

        if let Some(result) = self.ui()?.workspace_search_worker.take_latest_result() {
            let should_apply = {
                let ui = self.ui()?;
                if let Some(picker) = ui.picker()
                    && let Some(root) = picker.workspace_search_root()
                {
                    picker.session().query() == result.query
                        && root == result.root.as_path()
                        && result.request_id == ui.workspace_search_worker.next_request_id
                } else {
                    false
                }
            };
            if should_apply
                && let Some(picker) = self.ui_mut()?.picker_mut()
                && picker.workspace_search_root().is_some()
            {
                picker.set_entries(result.data.entries, result.data.selected_index);
                changed = true;
            }
        }

        if self.refresh_pending_project_discovery()? {
            changed = true;
        }

        Ok(changed)
    }

    fn refresh_pending_project_discovery(&mut self) -> Result<bool, ShellError> {
        let (source, provider_id) = {
            let Some(picker) = self.ui()?.picker() else {
                return Ok(false);
            };
            let Some(source) = picker.source() else {
                return Ok(false);
            };
            if !matches!(
                source,
                PickerSource::WorkspaceProjects | PickerSource::WorkspaceSwitch
            ) {
                return Ok(false);
            }
            if picker.project_discovery_revision()
                == Some(editor_fs::current_project_discovery_snapshot().revision())
            {
                return Ok(false);
            }
            (
                source,
                picker
                    .provider_id()
                    .unwrap_or(if source == PickerSource::WorkspaceSwitch {
                        "workspace.switch"
                    } else {
                        "workspace.projects"
                    })
                    .to_owned(),
            )
        };
        let entries =
            picker::picker_entries(&self.runtime, &provider_id).map_err(ShellError::Runtime)?;
        let revision = editor_fs::current_project_discovery_snapshot().revision();
        let Some(picker) = self.ui_mut()?.picker_mut() else {
            return Ok(false);
        };
        if picker.source() != Some(source) {
            return Ok(false);
        }
        picker.replace_entries_preserving_selection(entries);
        picker.set_project_discovery_revision(revision);
        Ok(true)
    }

    fn schedule_autocomplete_refresh_if_active(&mut self) -> Result<(), ShellError> {
        let Some(registry) = self
            .runtime
            .services()
            .get::<AutocompleteRegistry>()
            .cloned()
        else {
            return Ok(());
        };
        let lsp_client = self
            .runtime
            .services()
            .get::<Arc<LspClientManager>>()
            .cloned();
        let request = {
            let ui = self.ui()?;
            let Some(buffer_id) = ui.active_buffer_id() else {
                return Ok(());
            };
            let Some(buffer) = ui.buffer(buffer_id) else {
                return Ok(());
            };
            let root = lsp_root_for_buffer(&self.runtime, buffer).map_err(ShellError::Runtime)?;
            if let (Some(lsp_client), Some(path)) = (lsp_client.as_ref(), buffer.lsp_path())
                && let Err(error) = apply_sqls_workspace_settings_for_buffer(
                    &self.runtime,
                    buffer_id,
                    buffer,
                    path,
                    root.as_deref(),
                    lsp_client,
                )
            {
                return Err(ShellError::Runtime(error));
            }
            let token_map_key = ui.autocomplete_worker.token_map_key();
            autocomplete_request_for_buffer(
                &self.runtime,
                buffer_id,
                buffer,
                root,
                &registry,
                lsp_client.clone(),
                false,
            )
            .map(|mut request| {
                attach_token_count_edits(&mut request, &buffer.text, token_map_key);
                request
            })
        };
        let ui = self.ui_mut()?;
        match request {
            Some(request) => {
                if ui
                    .autocomplete()
                    .map(|autocomplete| autocomplete.buffer_id != request.buffer_id)
                    .unwrap_or(true)
                {
                    ui.set_autocomplete(AutocompleteOverlay::new(
                        request.buffer_id,
                        request.buffer_revision,
                        request.query.clone(),
                    ));
                } else if let Some(autocomplete) = ui.autocomplete_mut() {
                    autocomplete.mark_loading(request.buffer_revision, request.query.clone());
                }
                ui.autocomplete_worker.schedule(request);
            }
            None => ui.close_autocomplete(),
        }
        Ok(())
    }

    fn schedule_inline_completion_refresh(&mut self) -> Result<(), ShellError> {
        if self.command_line_visible()?
            || self.picker_visible()?
            || !matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace)
        {
            return Ok(());
        }
        let Some(lsp_client) = self
            .runtime
            .services()
            .get::<Arc<LspClientManager>>()
            .cloned()
        else {
            return Ok(());
        };
        let (buffer_id, buffer_revision, text, path, cursor, language_id, edits) = {
            let ui = self.ui()?;
            let Some(buffer_id) = ui.active_buffer_id() else {
                return Ok(());
            };
            let Some(buffer) = ui.buffer(buffer_id) else {
                return Ok(());
            };
            if buffer.is_read_only() || !buffer.lsp_enabled() {
                return Ok(());
            }
            let Some(path) = buffer.path().map(Path::to_path_buf) else {
                return Ok(());
            };
            if !lsp_client
                .registered_server_ids_for_path(&path)
                .iter()
                .any(|server_id| server_id == "copilot-language-server")
            {
                return Ok(());
            }
            let edits = lsp_edits_since_last_sync(&lsp_client, &path, &buffer.text);
            (
                buffer_id,
                buffer.text.revision(),
                buffer.text.snapshot(),
                path,
                buffer.cursor_point(),
                buffer.language_id().map(str::to_owned),
                edits,
            )
        };
        let root = workspace_root_for_path(&self.runtime, &path).map_err(ShellError::Runtime)?;
        let request = InlineCompletionWorkerRequest {
            request_id: 0,
            buffer_id,
            buffer_revision,
            text,
            path,
            root,
            cursor,
            options: lsp_formatting_options(&self.runtime, language_id.as_deref()),
            lsp_client,
            edits,
        };
        self.ui_mut()?.inline_completion_worker.schedule(request);
        Ok(())
    }

    fn refresh_pending_inline_completion(&mut self) -> Result<bool, ShellError> {
        let now = Instant::now();
        self.ui_mut()?.inline_completion_worker.dispatch_due(now);
        let Some(result) = self.ui()?.inline_completion_worker.take_latest_result() else {
            return Ok(false);
        };
        if let Some(error) = result.error {
            record_runtime_error(
                &mut self.runtime,
                "lsp.inline-completion",
                format!("failed to request inline completion: {error}"),
            );
        }
        let should_apply = {
            let ui = self.ui()?;
            ui.active_buffer_id() == Some(result.buffer_id)
                && result.request_id == ui.inline_completion_worker.next_request_id
                && ui
                    .buffer(result.buffer_id)
                    .map(|buffer| {
                        buffer.text.revision() == result.buffer_revision
                            && buffer.cursor_point() == result.cursor
                    })
                    .unwrap_or(false)
        };
        if !should_apply {
            return Ok(false);
        }
        let lsp_client = self
            .runtime
            .services()
            .get::<Arc<LspClientManager>>()
            .cloned();
        let shown_item = {
            let Some(buffer) = self.ui_mut()?.buffer_mut(result.buffer_id) else {
                return Ok(false);
            };
            if let Some(item) = result.item {
                buffer.set_inline_completion(item);
                buffer.mark_inline_completion_shown()
            } else {
                buffer.clear_inline_completion();
                None
            }
        };
        if let (Some(lsp_client), Some(item)) = (lsp_client, shown_item)
            && let Err(error) = lsp_client.did_show_inline_completion(&item)
        {
            record_runtime_error(
                &mut self.runtime,
                "lsp.inline-completion",
                format!("failed to report shown inline completion: {error}"),
            );
        }
        Ok(true)
    }

    fn accept_inline_completion(&mut self) -> Result<bool, ShellError> {
        let lsp_client = self
            .runtime
            .services()
            .get::<Arc<LspClientManager>>()
            .cloned();
        let Some(buffer_id) = self.ui()?.active_buffer_id() else {
            return Ok(false);
        };
        let Some(item) = self
            .ui_mut()?
            .buffer_mut(buffer_id)
            .and_then(ShellBuffer::take_valid_inline_completion)
        else {
            return Ok(false);
        };
        {
            let buffer = self.active_buffer_mut()?;
            buffer.replace_range(item.range(), item.insert_text());
        }
        self.mark_active_buffer_syntax_dirty()?;
        if let Some(lsp_client) = lsp_client
            && let Err(error) = lsp_client.accept_inline_completion(&item)
        {
            record_runtime_error(
                &mut self.runtime,
                "lsp.inline-completion",
                format!("failed to accept inline completion: {error}"),
            );
        }
        Ok(true)
    }

    fn refresh_pending_autocomplete(&mut self) -> Result<bool, ShellError> {
        let now = Instant::now();
        self.ui_mut()?.autocomplete_worker.dispatch_due(now);
        let Some(result) = self.ui()?.autocomplete_worker.take_latest_result() else {
            return Ok(false);
        };
        let should_apply = {
            let ui = self.ui()?;
            if let Some(autocomplete) = ui.autocomplete()
                && let Some(buffer) = ui.buffer(result.buffer_id)
            {
                autocomplete.buffer_id == result.buffer_id
                    && result.buffer_revision >= autocomplete.buffer_revision
                    && ui.active_buffer_id() == Some(result.buffer_id)
                    && buffer.text.revision() == result.buffer_revision
                    && result.request_id == ui.autocomplete_worker.next_request_id
            } else {
                false
            }
        };
        if !should_apply {
            return Ok(false);
        }
        if result.entries.is_empty() {
            self.ui_mut()?.close_autocomplete();
            return Ok(true);
        }
        if let Some(autocomplete) = self.ui_mut()?.autocomplete_mut() {
            autocomplete.buffer_revision = result.buffer_revision;
            autocomplete.query = result.query;
            autocomplete.set_entries(result.entries);
            return Ok(true);
        }
        Ok(false)
    }

    fn refresh_hover_state(&mut self) -> Result<bool, ShellError> {
        let should_close = {
            let ui = self.ui()?;
            let Some(hover) = ui.hover() else {
                return Ok(false);
            };
            let Some(buffer) = ui.buffer(hover.buffer_id) else {
                return Ok(true);
            };
            buffer.cursor_point() != hover.anchor
        };
        if should_close {
            self.ui_mut()?.close_hover();
            return Ok(true);
        }
        Ok(false)
    }

    fn handle_focused_hover_keydown(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        if !self
            .ui()?
            .hover()
            .map(|hover| hover.focused)
            .unwrap_or(false)
        {
            return Ok(false);
        }
        match keycode {
            Keycode::Escape => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    hover.focused = false;
                    hover.clear_navigation_state();
                }
                Ok(true)
            }
            Keycode::N if keymod.intersects(ctrl_mod()) => {
                cycle_hover_provider(&mut self.runtime, true).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            Keycode::P if keymod.intersects(ctrl_mod()) => {
                cycle_hover_provider(&mut self.runtime, false).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            Keycode::Down => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::Up => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::PageDown => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::PageUp => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::D if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .half_page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::U if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .half_page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::F if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::B if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover
                        .page_scroll_lines()
                        .saturating_mul(hover.take_count_or_one())
                        as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::E if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(lines);
                }
                Ok(true)
            }
            Keycode::Y if keymod.intersects(ctrl_mod()) => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    let lines = hover.take_count_or_one() as i32;
                    hover.pending_g_prefix = false;
                    hover.scroll_by(-lines);
                }
                Ok(true)
            }
            Keycode::Home => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    hover.scroll_to_start();
                }
                Ok(true)
            }
            Keycode::End => {
                if let Some(hover) = self.ui_mut()?.hover_mut() {
                    hover.scroll_to_end();
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_focused_hover_text_input(&mut self, chord: &str) -> Result<bool, ShellError> {
        if matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace) {
            return Ok(false);
        }
        if !self
            .ui()?
            .hover()
            .map(|hover| hover.focused)
            .unwrap_or(false)
        {
            return Ok(false);
        }
        if chord.chars().count() != 1 {
            return Ok(false);
        }
        let Some(character) = chord.chars().next() else {
            return Ok(false);
        };
        let Some(hover) = self.ui_mut()?.hover_mut() else {
            return Ok(false);
        };
        match character {
            '1'..='9' => {
                hover.push_count_digit(character.to_digit(10).unwrap_or_default() as usize);
                Ok(true)
            }
            '0' => {
                if hover.count.is_some() {
                    hover.push_count_digit(0);
                } else {
                    hover.scroll_to_start();
                }
                Ok(true)
            }
            'j' => {
                let lines = hover.take_count_or_one() as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(lines);
                Ok(true)
            }
            'k' => {
                let lines = hover.take_count_or_one() as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(-lines);
                Ok(true)
            }
            'g' => {
                if hover.pending_g_prefix {
                    if let Some(line) = hover.take_count().map(|count| count.saturating_sub(1)) {
                        hover.scroll_to_line(line);
                    } else {
                        hover.scroll_to_start();
                    }
                } else {
                    hover.pending_g_prefix = true;
                }
                Ok(true)
            }
            'G' => {
                if let Some(line) = hover.take_count().map(|count| count.saturating_sub(1)) {
                    hover.scroll_to_line(line);
                } else {
                    hover.scroll_to_end();
                }
                Ok(true)
            }
            '{' | '(' | 'H' => {
                let lines = hover
                    .page_scroll_lines()
                    .saturating_mul(hover.take_count_or_one()) as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(-lines);
                Ok(true)
            }
            '}' | ')' | 'L' | '$' => {
                let lines = hover
                    .page_scroll_lines()
                    .saturating_mul(hover.take_count_or_one()) as i32;
                hover.pending_g_prefix = false;
                hover.scroll_by(lines);
                Ok(true)
            }
            'h' | 'l' | 'w' | 'W' | 'b' | 'B' | 'e' | 'E' | '^' | 'M' => {
                hover.clear_navigation_state();
                Ok(true)
            }
            _ => {
                hover.clear_navigation_state();
                Ok(false)
            }
        }
    }

    fn handle_autocomplete_keydown(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        let Some(chord) = keydown_chord(keycode, keymod) else {
            return Ok(false);
        };
        let handled = {
            let Some(autocomplete) = self.ui_mut()?.autocomplete_mut() else {
                return Ok(false);
            };
            if !autocomplete.is_visible() {
                return Ok(false);
            }
            if chord == AUTOCOMPLETE_NEXT_CHORD {
                autocomplete.select_next();
                true
            } else if chord == AUTOCOMPLETE_PREVIOUS_CHORD {
                autocomplete.select_previous();
                true
            } else {
                false
            }
        };
        if handled {
            self.queue_suppressed_text_input_for_chord(&chord);
        }
        Ok(handled)
    }

    fn handle_input_prompt_keydown(
        &mut self,
        keycode: Keycode,
        _keymod: Mod,
    ) -> Result<bool, ShellError> {
        if !self.ui()?.input_prompt_visible() {
            return Ok(false);
        }
        match keycode {
            Keycode::Escape => {
                self.ui_mut()?.close_input_prompt();
                Ok(true)
            }
            Keycode::Return | Keycode::KpEnter | Keycode::Return2 => {
                let confirmed = self
                    .ui()
                    .ok()
                    .and_then(|ui| ui.input_prompt())
                    .filter(|prompt| {
                        !prompt.text().is_empty()
                            || matches!(
                                prompt.id.as_str(),
                                DAP_BP_CONDITION_PROMPT_ID
                                    | DAP_BP_HIT_CONDITION_PROMPT_ID
                                    | DAP_BP_LOG_MESSAGE_PROMPT_ID
                                    | DAP_REPL_PROMPT_ID
                            )
                    })
                    .map(|prompt| (prompt.id.clone(), prompt.text().to_owned()));
                if let Some((id, text)) = confirmed {
                    self.ui_mut()?.close_input_prompt();
                    dispatch_input_prompt_confirm(&mut self.runtime, &id, &text)
                        .map_err(ShellError::Runtime)?;
                }
                Ok(true)
            }
            Keycode::Backspace => {
                if let Some(prompt) = self.ui_mut()?.input_prompt_mut() {
                    prompt.backspace();
                }
                Ok(true)
            }
            Keycode::Delete => {
                if let Some(prompt) = self.ui_mut()?.input_prompt_mut() {
                    prompt.delete_forward();
                }
                Ok(true)
            }
            Keycode::Left => {
                if let Some(prompt) = self.ui_mut()?.input_prompt_mut() {
                    prompt.move_left();
                }
                Ok(true)
            }
            Keycode::Right => {
                if let Some(prompt) = self.ui_mut()?.input_prompt_mut() {
                    prompt.move_right();
                }
                Ok(true)
            }
            _ => Ok(keydown_chord(keycode, _keymod).is_some()),
        }
    }

    fn handle_command_line_keydown(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        if !self.command_line_visible()? {
            return Ok(false);
        }
        match keycode {
            Keycode::Escape => {
                self.ui_mut()?.close_command_line();
                Ok(true)
            }
            Keycode::Return | Keycode::KpEnter | Keycode::Return2 => {
                submit_vim_command_line(&mut self.runtime).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            Keycode::Backspace => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.backspace();
                }
                Ok(true)
            }
            Keycode::Delete => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.delete_forward();
                }
                Ok(true)
            }
            Keycode::Left => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.move_left();
                }
                Ok(true)
            }
            Keycode::Right => {
                if let Some(command_line) = self.ui_mut()?.command_line_mut() {
                    command_line.move_right();
                }
                Ok(true)
            }
            Keycode::Tab => {
                cycle_vim_command_line_completion(
                    &mut self.runtime,
                    keymod.intersects(shift_mod()),
                )
                .map_err(ShellError::Runtime)?;
                Ok(true)
            }
            _ => Ok(keydown_chord(keycode, keymod).is_some()),
        }
    }
}
