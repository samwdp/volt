impl ShellState {
    #[cfg(test)]
    pub(crate) fn try_runtime_keybinding(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
    ) -> Result<bool, ShellError> {
        if self.handle_input_prompt_keydown(keycode, keymod)? {
            return Ok(true);
        }
        if self.handle_command_line_keydown(keycode, keymod)? {
            return Ok(true);
        }
        let active_buffer =
            active_buffer_event_context(&self.runtime).map_err(ShellError::Runtime)?;
        let (input_mode, picker_visible) = {
            let ui = self.ui()?;
            (ui.input_mode(), ui.picker_visible())
        };
        self.try_runtime_keybinding_cached(
            keycode,
            keymod,
            input_mode,
            picker_visible,
            active_buffer.is_directory,
        )
    }

    #[cfg(test)]
    pub(crate) fn replace_active_buffer_text_for_test(
        &mut self,
        text: &str,
    ) -> Result<(), ShellError> {
        let lines = if text.is_empty() {
            Vec::new()
        } else {
            text.split('\n').map(str::to_owned).collect()
        };
        self.active_buffer_mut()?.replace_with_lines(lines);
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn flush_picker_searches_for_test(&mut self) -> Result<(), ShellError> {
        const SEARCH_WAIT_STEP: Duration = Duration::from_millis(5);
        const SEARCH_WAIT_ATTEMPTS: usize = 40;

        {
            let ui = self.ui_mut()?;
            let due = Instant::now() + Duration::from_secs(1);
            ui.vim_search_worker.dispatch_due(due);
            ui.workspace_search_worker.dispatch_due(due);
        }

        for _ in 0..SEARCH_WAIT_ATTEMPTS {
            if self.refresh_pending_picker_searches()? {
                return Ok(());
            }
            std::thread::sleep(SEARCH_WAIT_STEP);
        }

        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn refresh_pending_project_discovery_for_test(
        &mut self,
    ) -> Result<bool, ShellError> {
        self.refresh_pending_project_discovery()
    }

    fn tick_project_discovery_background(&self) {
        let roots = project_search_roots_from_user_library(&*shell_user_library(&self.runtime));
        if roots.is_empty() {
            return;
        }
        project_discovery_background_tick(&roots);
    }

    fn try_runtime_keybinding_cached(
        &mut self,
        keycode: Keycode,
        keymod: Mod,
        input_mode: InputMode,
        picker_visible: bool,
        active_buffer_is_directory: bool,
    ) -> Result<bool, ShellError> {
        let Some(chord) = keydown_chord(keycode, keymod) else {
            return Ok(false);
        };

        clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
        let vim_mode = keymap_vim_mode(input_mode);
        let in_text_insert_mode = matches!(input_mode, InputMode::Insert | InputMode::Replace);
        let hover_visible = self.ui()?.hover().is_some();

        if !picker_visible && !in_text_insert_mode && hover_visible && chord == "Tab" {
            trigger_hover_focus(&mut self.runtime).map_err(ShellError::Runtime)?;
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if !picker_visible
            && !in_text_insert_mode
            && chord == "Tab"
            && handle_git_status_tab(&mut self.runtime).map_err(ShellError::Runtime)?
        {
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if !picker_visible && !in_text_insert_mode && chord == "Tab" {
            let handled = {
                let buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let buffer =
                    shell_buffer_mut(&mut self.runtime, buffer_id).map_err(ShellError::Runtime)?;
                advance_markdown_table_normal_tab(buffer).is_some()
            };
            if handled {
                self.queue_suppressed_text_input_for_chord(&chord);
                self.record_vim_input(VimRecordedInput::Chord(chord))?;
                self.maybe_finish_change_after_input()?;
                return Ok(true);
            }
        }

        if !picker_visible
            && !in_text_insert_mode
            && active_buffer_is_directory
            && handle_directory_keydown_chord(&mut self.runtime, &chord)
                .map_err(ShellError::Runtime)?
        {
            self.queue_suppressed_text_input_for_chord(&chord);
            self.ui_mut()?.vim_mut().clear_transient();
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        if self.try_plugin_or_overlay_keybinding(&chord, vim_mode, picker_visible)? {
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        let editing_modes = self.editing_minor_modes()?;
        if self
            .runtime
            .execute_key_binding_with_minor_modes(&editing_modes, vim_mode, &chord)
            .map_err(|error| ShellError::Runtime(error.to_string()))?
        {
            self.queue_suppressed_text_input_for_chord(&chord);
            self.record_vim_input(VimRecordedInput::Chord(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }

        Ok(false)
    }

    fn try_picker_extra_keybinding(&mut self, chord: &str) -> Result<bool, ShellError> {
        let dispatch = {
            let ui = self.ui()?;
            let Some(picker) = ui.picker() else {
                return Ok(false);
            };
            picker.resolve_extra(chord)
        };
        let PickerExtraDispatch::Fire {
            command_name,
            context,
            close_picker: _,
        } = dispatch
        else {
            return Ok(false);
        };
        {
            let ui = self.ui_mut()?;
            ui.set_picker_one_shot(context);
            ui.close_picker();
        }
        self.runtime
            .execute_command(&command_name)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        // Handlers should take the context; clear any leftover so it never sticks.
        let _ = self.ui_mut()?.take_picker_one_shot();
        Ok(true)
    }

    fn try_plugin_or_overlay_keybinding(
        &mut self,
        chord: &str,
        vim_mode: KeymapVimMode,
        picker_visible: bool,
    ) -> Result<bool, ShellError> {
        if picker_visible && self.try_picker_extra_keybinding(chord)? {
            return Ok(true);
        }
        if !picker_visible && self.try_plugin_buffer_keybinding(chord, vim_mode)? {
            return Ok(true);
        }
        let overlay_modes = self.overlay_minor_modes()?;
        // Popup Minor Mode keeps Enter bound to picker.submit for picker UIs.
        // When a non-picker popup is focused (terminal, etc.), skip that binding
        // so Enter reaches terminal/input handling instead of erroring with
        // "picker has no selected item" and swallowing the key.
        let skip_picker_submit = !picker_visible
            && self
                .runtime
                .keymaps()
                .find_in_scopes(&overlay_modes, vim_mode, chord)
                .is_some_and(|binding| {
                    binding
                        .command_names()
                        .iter()
                        .any(|name| name.as_str() == "picker.submit")
                });
        if !skip_picker_submit
            && self
                .runtime
                .execute_key_binding_in_scopes(&overlay_modes, vim_mode, chord)
                .map_err(|error| ShellError::Runtime(error.to_string()))?
        {
            return Ok(true);
        }
        if picker_visible && self.try_plugin_buffer_keybinding(chord, vim_mode)? {
            return Ok(true);
        }
        Ok(false)
    }

    fn try_plugin_buffer_keybinding(
        &mut self,
        chord: &str,
        vim_mode: KeymapVimMode,
    ) -> Result<bool, ShellError> {
        let (buffer_id, popup_focused) = {
            let ui = self.ui()?;
            let Some(buffer_id) = ui.focused_buffer_id() else {
                return Ok(false);
            };
            (buffer_id, ui.popup_focus)
        };
        let plugin_kind = {
            let Some(buffer) = self.ui()?.buffer(buffer_id) else {
                return Ok(false);
            };
            match &buffer.kind {
                BufferKind::Plugin(kind) => kind.clone(),
                _ => return Ok(false),
            }
        };
        let binding = shell_user_library(&self.runtime)
            .plugin_buffer_key_bindings(&plugin_kind)
            .into_iter()
            .find(|binding| {
                plugin_buffer_binding_scope_active(binding.scope(), popup_focused)
                    && plugin_vim_mode_matches(binding.vim_mode(), vim_mode)
                    && binding.chord() == chord
            });
        let Some(binding) = binding else {
            return Ok(false);
        };
        for command_name in binding.command_names() {
            self.runtime
                .execute_command(command_name.as_str())
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
        }
        Ok(true)
    }

    #[cfg(test)]
    pub(crate) fn wait_for_autocomplete_results(&mut self) -> Result<(), ShellError> {
        for _ in 0..40 {
            self.ui_mut()?
                .autocomplete_worker
                .dispatch_due(Instant::now());
            if self.refresh_pending_autocomplete()? {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn autocomplete_visible(&self) -> Result<bool, ShellError> {
        Ok(self
            .ui()?
            .autocomplete()
            .is_some_and(AutocompleteOverlay::is_visible))
    }

    #[cfg(test)]
    pub(crate) fn command_line_text(&self) -> Result<Option<String>, ShellError> {
        Ok(self
            .ui()?
            .command_line()
            .map(|command_line| command_line.text().to_owned()))
    }

    #[cfg(test)]
    pub(crate) fn autocomplete_entries(&self) -> Result<Vec<String>, ShellError> {
        Ok(self
            .ui()?
            .autocomplete()
            .filter(|autocomplete| autocomplete.is_visible())
            .map(|autocomplete| {
                autocomplete
                    .entries()
                    .iter()
                    .map(|entry| entry.replacement.clone())
                    .collect()
            })
            .unwrap_or_default())
    }

    #[cfg(test)]
    pub(crate) fn autocomplete_selected(&self) -> Result<Option<String>, ShellError> {
        Ok(self
            .ui()?
            .autocomplete()
            .filter(|autocomplete| autocomplete.is_visible())
            .and_then(|autocomplete| {
                autocomplete
                    .selected()
                    .map(|entry| entry.replacement.clone())
            }))
    }

    #[cfg(test)]
    pub(crate) fn hover_visible(&self) -> Result<bool, ShellError> {
        Ok(self.ui()?.hover().is_some())
    }

    #[cfg(test)]
    pub(crate) fn hover_focused(&self) -> Result<bool, ShellError> {
        Ok(self
            .ui()?
            .hover()
            .map(|hover| hover.focused)
            .unwrap_or(false))
    }

    #[cfg(test)]
    pub(crate) fn hover_provider_label(&self) -> Result<Option<String>, ShellError> {
        Ok(self.ui()?.hover().and_then(|hover| {
            hover
                .current_provider()
                .map(|provider| provider.provider_label.clone())
        }))
    }

    #[cfg(test)]
    pub(crate) fn refresh_pending_file_reloads_for_test(&mut self) -> Result<bool, ShellError> {
        refresh_pending_file_reloads(&mut self.runtime, Instant::now(), true)
            .map_err(ShellError::Runtime)
    }

    #[cfg(test)]
    #[cfg(test)]
    pub(crate) fn flush_pending_syntax_prewarm_for_test(&mut self) -> Result<(), ShellError> {
        while refresh_pending_syntax_prewarm(&mut self.runtime).map_err(ShellError::Runtime)? {}
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn flush_pending_workspace_readme_opens_for_test(
        &mut self,
    ) -> Result<(), ShellError> {
        while refresh_pending_workspace_readme_opens(&mut self.runtime)
            .map_err(ShellError::Runtime)?
        {}
        Ok(())
    }
}
