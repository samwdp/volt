impl ShellState {
    fn handle_vim_pending_text(&mut self, chord: &str) -> Result<bool, ShellError> {
        let pending = self.ui()?.vim().pending;
        let Some(pending) = pending else {
            return Ok(false);
        };

        match pending {
            VimPending::Operator { operator, count } => {
                if let Some(digit) = vim_count_digit(chord, self.ui()?.vim().count.is_some()) {
                    self.ui_mut()?.vim_mut().push_count_digit(digit);
                    return Ok(true);
                }

                match (operator, chord) {
                    (VimOperator::Delete, "d")
                    | (VimOperator::Change, "c")
                    | (VimOperator::Yank, "y") => {
                        let lines =
                            count.saturating_mul(self.ui_mut()?.vim_mut().take_count_or_one());
                        apply_linewise_operator(&mut self.runtime, operator, lines)
                            .map_err(ShellError::Runtime)?;
                        return Ok(true);
                    }
                    (_, "i") | (_, "a") => {
                        let around = chord == "a";
                        let count =
                            count.saturating_mul(self.ui_mut()?.vim_mut().take_count_or_one());
                        self.ui_mut()?.vim_mut().pending = Some(VimPending::TextObject {
                            operator,
                            around,
                            count,
                        });
                        return Ok(true);
                    }
                    (_, "g") => {
                        let line_target = self.ui_mut()?.vim_mut().take_count();
                        self.ui_mut()?.vim_mut().pending = Some(VimPending::GPrefix {
                            operator: Some(operator),
                            line_target,
                        });
                        return Ok(true);
                    }
                    _ => {}
                }

                Ok(false)
            }
            VimPending::Format { .. } => {
                if chord == "=" {
                    self.ui_mut()?.vim_mut().clear_transient();
                    emit_workspace_format(&mut self.runtime).map_err(ShellError::Runtime)?;
                    return Ok(true);
                }
                self.ui_mut()?.vim_mut().clear_transient();
                Ok(false)
            }
            VimPending::FindTarget {
                operator,
                kind,
                count,
            } => {
                if let Some(target) = chord.chars().next() {
                    resolve_find_target(&mut self.runtime, operator, kind, count, target)
                        .map_err(ShellError::Runtime)?;
                    return Ok(true);
                }
                Ok(false)
            }
            VimPending::GPrefix {
                operator,
                line_target,
            } => {
                match chord {
                    "g" | "e" | "E" => {
                        if operator.is_none() {
                            self.ui_mut()?.vim_mut().pending_change_prefix = None;
                        }
                        resolve_g_prefix(&mut self.runtime, operator, line_target, chord)
                            .map_err(ShellError::Runtime)?;
                    }
                    "v" if operator.is_none() => {
                        self.ui_mut()?.vim_mut().pending_change_prefix = None;
                        restore_last_visual_selection(&mut self.runtime)
                            .map_err(ShellError::Runtime)?;
                    }
                    "~" | "u" | "U" if operator.is_none() => {
                        let operator = match chord {
                            "~" => VimOperator::ToggleCase,
                            "u" => VimOperator::Lowercase,
                            "U" => VimOperator::Uppercase,
                            _ => VimOperator::ToggleCase,
                        };
                        let prefix = self.ui_mut()?.vim_mut().pending_change_prefix.take();
                        start_change_recording_with_prefix(&mut self.runtime, prefix)
                            .map_err(ShellError::Runtime)?;
                        let count = line_target.unwrap_or(1);
                        self.ui_mut()?.vim_mut().pending =
                            Some(VimPending::Operator { operator, count });
                    }
                    "c" if operator.is_none() && self.input_mode()? == InputMode::Normal => {
                        self.ui_mut()?.vim_mut().pending_change_prefix =
                            Some(VimRecordedInput::Chord("g c".to_owned()));
                        self.ui_mut()?.vim_mut().pending = Some(VimPending::CommentToggle {
                            count: line_target.unwrap_or(1),
                        });
                    }
                    _ => {
                        if self.handle_pending_g_sequence(operator, line_target, chord)? {
                            return Ok(true);
                        }
                        if operator.is_none() {
                            self.ui_mut()?.vim_mut().pending_change_prefix = None;
                        }
                        self.ui_mut()?.vim_mut().clear_transient();
                    }
                }
                Ok(true)
            }
            VimPending::TextObject {
                operator,
                around,
                count,
            } => {
                if let Some(kind) = vim_text_object_kind(chord) {
                    apply_text_object_operator(&mut self.runtime, operator, kind, around, count)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::VisualTextObject { around, count } => {
                if let Some(kind) = vim_text_object_kind(chord) {
                    apply_visual_text_object(&mut self.runtime, kind, around, count)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::CommentToggle { count } => {
                if chord == "c" {
                    let prefix = self.ui_mut()?.vim_mut().pending_change_prefix.take();
                    start_change_recording_with_prefix(&mut self.runtime, prefix)
                        .map_err(ShellError::Runtime)?;
                    toggle_current_line_comment(&mut self.runtime, count)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().pending_change_prefix = None;
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::ReplaceChar { count } => {
                let Some(character) = chord.chars().next() else {
                    self.ui_mut()?.vim_mut().clear_transient();
                    return Ok(true);
                };
                if character != '\n' {
                    if active_shell_buffer_vim_targets_input(&self.runtime)
                        .map_err(ShellError::Runtime)?
                    {
                        if let Some(input) = active_shell_buffer_mut(&mut self.runtime)
                            .map_err(ShellError::Runtime)?
                            .input_field_mut()
                        {
                            let _ = input.replace_chars_at_cursor(character, count);
                        }
                    } else {
                        let replaced = self
                            .active_buffer_mut()?
                            .replace_chars_at_cursor(character, count);
                        if replaced {
                            self.mark_active_buffer_syntax_dirty()?;
                        }
                    }
                }
                self.ui_mut()?.enter_normal_mode();
                apply_directory_edit_queue_if_needed(&mut self.runtime)
                    .map_err(ShellError::Runtime)?;
                schedule_finish_change(&mut self.runtime).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            VimPending::ReplaceVisualSelection => {
                let Some(character) = chord.chars().next() else {
                    self.ui_mut()?.vim_mut().clear_transient();
                    return Ok(true);
                };
                if character != '\n' {
                    replace_visual_selection_chars(&mut self.runtime, character)
                        .map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.enter_normal_mode();
                }
                apply_directory_edit_queue_if_needed(&mut self.runtime)
                    .map_err(ShellError::Runtime)?;
                schedule_finish_change(&mut self.runtime).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            VimPending::Register => {
                if let Some(register) = chord.chars().next() {
                    self.ui_mut()?.vim_mut().active_register = Some(register);
                }
                self.ui_mut()?.vim_mut().clear_transient();
                Ok(true)
            }
            VimPending::MarkSet => {
                if let Some(mark) = chord.chars().next() {
                    let buffer_id =
                        active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                    let point = self.active_buffer_mut()?.cursor_point();
                    self.ui_mut()?
                        .vim_mut()
                        .marks
                        .insert(mark, VimMark { buffer_id, point });
                }
                self.ui_mut()?.vim_mut().clear_transient();
                Ok(true)
            }
            VimPending::MarkJump { linewise } => {
                if let Some(mark) = chord.chars().next() {
                    jump_to_mark(&mut self.runtime, mark, linewise).map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::MacroRecord => {
                if let Some(register) = chord.chars().next() {
                    start_macro_record(&mut self.runtime, register).map_err(ShellError::Runtime)?;
                } else {
                    self.ui_mut()?.vim_mut().clear_transient();
                }
                Ok(true)
            }
            VimPending::MacroPlayback => {
                let repeat = self.ui_mut()?.vim_mut().take_count_or_one();
                let register = chord.chars().next();
                self.ui_mut()?.vim_mut().clear_transient();
                self.play_macro(register, repeat)?;
                Ok(true)
            }
        }
    }

    fn handle_vim_count_input(&mut self, chord: &str) -> Result<bool, ShellError> {
        if matches!(self.input_mode()?, InputMode::Insert | InputMode::Replace) {
            return Ok(false);
        }

        let has_count = self.ui()?.vim().count.is_some();
        let Some(digit) = vim_count_digit(chord, has_count) else {
            return Ok(false);
        };
        self.ui_mut()?.vim_mut().push_count_digit(digit);
        Ok(true)
    }

    fn clear_stale_vim_count(&mut self) -> Result<(), ShellError> {
        let should_clear = {
            let ui = self.ui()?;
            !matches!(ui.input_mode(), InputMode::Insert | InputMode::Replace)
                && ui.vim().pending.is_none()
                && ui.vim().count.is_some()
        };
        if should_clear {
            self.ui_mut()?.vim_mut().count = None;
        }
        Ok(())
    }

    fn record_vim_input(&mut self, input: VimRecordedInput) -> Result<(), ShellError> {
        let input_mode = self.input_mode()?;
        let vim = self.ui_mut()?.vim_mut();
        if vim.replaying {
            return Ok(());
        }
        let skip_macro = matches!(
            (&input, input_mode),
            (VimRecordedInput::Text(text), InputMode::Normal | InputMode::Visual)
                if text == "q"
        );
        if vim.recording_macro.is_some() && !skip_macro {
            if vim.skip_next_macro_input {
                vim.skip_next_macro_input = false;
            } else {
                vim.macro_buffer.push(input.clone());
            }
        }
        if vim.recording_change {
            vim.change_buffer.push(input);
        }
        Ok(())
    }

    fn maybe_finish_change_after_input(&mut self) -> Result<(), ShellError> {
        let finish = self.ui_mut()?.vim_mut().finish_change_after_input;
        if finish {
            self.ui_mut()?.vim_mut().finish_change_after_input = false;
            finish_change_recording(&mut self.runtime).map_err(ShellError::Runtime)?;
        }
        Ok(())
    }

    fn execute_recorded_chord(&mut self, chord: &str) -> Result<(), ShellError> {
        let vim_mode = keymap_vim_mode(self.input_mode()?);
        let runtime_surface_before =
            active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
        let picker_visible = self.ui()?.picker_visible();
        if self.try_plugin_or_overlay_keybinding(chord, vim_mode, picker_visible)? {
            return Ok(());
        }

        let editing_modes = self.editing_minor_modes()?;
        if self
            .runtime
            .execute_key_binding_with_minor_modes(&editing_modes, vim_mode, chord)
            .map_err(|error| ShellError::Runtime(error.to_string()))?
        {
            self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
            self.clear_stale_vim_count()?;
        }

        Ok(())
    }

    fn overlay_minor_modes(&self) -> Result<Vec<KeymapScope>, ShellError> {
        let ui = self.ui()?;
        let mut modes = Vec::new();
        if ui.picker_visible() {
            modes.push(KeymapScope::Popup);
        } else if let Some(popup) =
            active_runtime_popup(&self.runtime).map_err(ShellError::Runtime)?
            && ui.popup_focus_active(&popup)
        {
            modes.push(KeymapScope::Popup);
        } else if ui.workspace_dock_focus_active(&*shell_user_library(&self.runtime)) {
            modes.push(KeymapScope::WorkspaceDock);
        } else if ui.acp_dock_focus_active() {
            modes.push(KeymapScope::AcpDock);
        }
        if ui
            .autocomplete()
            .is_some_and(AutocompleteOverlay::is_visible)
        {
            modes.push(KeymapScope::Autocomplete);
        } else if ui.hover().is_some() {
            modes.push(KeymapScope::Hover);
        }
        if !modes.iter().any(|mode| {
            matches!(
                mode,
                KeymapScope::Popup | KeymapScope::WorkspaceDock | KeymapScope::AcpDock
            )
        }) && active_workspace_has_debug_session(&self.runtime)
        {
            modes.push(KeymapScope::Dap);
        }
        Ok(modes)
    }

    fn editing_minor_modes(&self) -> Result<Vec<KeymapScope>, ShellError> {
        let ui = self.ui()?;
        let mut modes = Vec::new();
        if !ui.picker_visible()
            && !matches!(ui.input_mode(), InputMode::Insert | InputMode::Replace)
        {
            if ui.vim().multicursor.is_some() {
                modes.push(KeymapScope::Multicursor);
            }
            modes.push(KeymapScope::Workspace);
        }
        Ok(modes)
    }

    fn handle_pending_g_sequence(
        &mut self,
        operator: Option<VimOperator>,
        line_target: Option<usize>,
        chord: &str,
    ) -> Result<bool, ShellError> {
        if operator.is_some() {
            return Ok(false);
        }
        let vim_mode = keymap_vim_mode(self.input_mode()?);
        let prefix = match self.ui()?.vim().pending_change_prefix.clone() {
            Some(VimRecordedInput::Chord(chord)) => chord,
            Some(VimRecordedInput::Text(text)) => text,
            None => "g".to_owned(),
        };
        let candidate = format!("{prefix} {chord}");
        if self
            .runtime
            .keymaps()
            .contains_for_mode(&KeymapScope::Workspace, vim_mode, &candidate)
        {
            let runtime_surface_before =
                active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
            self.ui_mut()?.vim_mut().pending_change_prefix = None;
            self.ui_mut()?.vim_mut().clear_transient();
            self.runtime
                .execute_key_binding_for_mode(&KeymapScope::Workspace, vim_mode, &candidate)
                .map_err(|error| ShellError::Runtime(error.to_string()))?;
            self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
            self.clear_stale_vim_count()?;
            return Ok(true);
        }

        let tokens = candidate
            .split_whitespace()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        if self.runtime.keymaps().has_sequence_prefix_for_mode(
            &KeymapScope::Workspace,
            vim_mode,
            &tokens,
        ) {
            self.ui_mut()?.vim_mut().pending_change_prefix =
                Some(VimRecordedInput::Chord(candidate));
            self.ui_mut()?.vim_mut().pending = Some(VimPending::GPrefix {
                operator,
                line_target,
            });
            return Ok(true);
        }

        Ok(false)
    }

    fn handle_key_sequence(
        &mut self,
        token: &str,
        scope: KeymapScope,
        vim_mode: KeymapVimMode,
    ) -> Result<bool, ShellError> {
        let options = key_sequence_options(&*shell_user_library(&self.runtime));
        let taken =
            take_key_sequence(&mut self.runtime, &scope, &options).map_err(ShellError::Runtime)?;
        let pending = match taken {
            TakeKeySequence::Live(pending) => Some(pending),
            TakeKeySequence::None => None,
            TakeKeySequence::FireShort {
                chord,
                vim_mode: pending_mode,
            } => {
                self.execute_sequence_chord(&scope, pending_mode, &chord)?;
                None
            }
        };
        let result = push_key_sequence(
            self.runtime.keymaps(),
            &scope,
            vim_mode,
            pending,
            token,
            0,
            &options,
        );

        match result {
            KeySequencePush::Wait(pending) => {
                set_key_sequence(&mut self.runtime, scope, vim_mode, pending)
                    .map_err(ShellError::Runtime)?;
                Ok(true)
            }
            KeySequencePush::Execute { chord } => {
                self.execute_sequence_chord(&scope, vim_mode, &chord)?;
                Ok(true)
            }
            KeySequencePush::Cancel => {
                clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            KeySequencePush::FireShortThenRetry { chord } => {
                // The new token broke the sequence while an exact short chord
                // (e.g. `g`) was pending: fire the short binding first, then
                // re-process the token so Vim pendings it created (like the
                // g-prefix) can consume it (e.g. `g c c` line comments).
                self.execute_sequence_chord(&scope, vim_mode, &chord)?;
                self.retry_sequence_token(token, scope)
            }
            KeySequencePush::Miss => Ok(false),
        }
    }

    fn retry_sequence_token(
        &mut self,
        token: &str,
        scope: KeymapScope,
    ) -> Result<bool, ShellError> {
        let chord = if token == "Space" {
            " ".to_owned()
        } else {
            token.to_owned()
        };
        if self.handle_vim_pending_text(&chord)? || self.handle_vim_count_input(&chord)? {
            self.record_vim_input(VimRecordedInput::Text(chord))?;
            self.maybe_finish_change_after_input()?;
            return Ok(true);
        }
        // Re-resolve against the keymap as a fresh sequence; the fired short
        // chord may have switched Vim modes, so recompute the mode.
        let vim_mode = keymap_vim_mode(self.input_mode()?);
        self.handle_key_sequence(token, scope, vim_mode)
    }

    fn execute_sequence_chord(
        &mut self,
        scope: &KeymapScope,
        vim_mode: KeymapVimMode,
        chord: &str,
    ) -> Result<(), ShellError> {
        let runtime_surface_before =
            active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
        clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
        self.runtime
            .execute_key_binding_for_mode(scope, vim_mode, chord)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
        self.clear_stale_vim_count()?;
        self.record_vim_input(VimRecordedInput::Chord(chord.to_owned()))?;
        self.maybe_finish_change_after_input()?;
        Ok(())
    }

    fn fire_pending_ambiguous_prefix_timeout(&mut self) -> Result<bool, ShellError> {
        let options = key_sequence_options(&*shell_user_library(&self.runtime));
        let Some((scope, vim_mode, tick)) =
            peek_key_sequence_tick(&self.runtime, &options).map_err(ShellError::Runtime)?
        else {
            return Ok(false);
        };
        match tick {
            KeySequenceTick::Pending => Ok(false),
            KeySequenceTick::Expired => {
                clear_key_sequence(&mut self.runtime).map_err(ShellError::Runtime)?;
                Ok(true)
            }
            KeySequenceTick::Execute { chord } => {
                self.execute_sequence_chord(&scope, vim_mode, &chord)?;
                Ok(true)
            }
        }
    }

    fn replay_recorded_inputs(&mut self, inputs: &[VimRecordedInput]) -> Result<(), ShellError> {
        if inputs.is_empty() {
            return Ok(());
        }

        self.ui_mut()?.vim_mut().replaying = true;
        let result = inputs.iter().try_for_each(|input| match input {
            VimRecordedInput::Text(text) => self.handle_text_input(text),
            VimRecordedInput::Chord(chord) => self.execute_recorded_chord(chord),
        });
        self.ui_mut()?.vim_mut().replaying = false;
        result
    }

    fn repeat_last_change(&mut self) -> Result<(), ShellError> {
        if self.ui()?.vim().replaying {
            return Ok(());
        }
        let repeat = self.ui_mut()?.vim_mut().take_count_or_one();
        let inputs = self.ui()?.vim().last_change.clone();
        if inputs.is_empty() {
            return Ok(());
        }
        for _ in 0..repeat {
            self.replay_recorded_inputs(&inputs)?;
        }
        Ok(())
    }

    fn play_macro(&mut self, register: Option<char>, repeat: usize) -> Result<(), ShellError> {
        if self.ui()?.vim().replaying {
            return Ok(());
        }
        let inputs = {
            let vim = self.ui_mut()?.vim_mut();
            let target = match register {
                Some('@') => vim.last_macro,
                Some(register) => Some(register),
                None => None,
            };
            let Some(register) = target else {
                vim.clear_transient();
                return Ok(());
            };
            let inputs = vim.macros.get(&register).cloned().unwrap_or_default();
            vim.last_macro = Some(register);
            inputs
        };

        if inputs.is_empty() {
            self.ui_mut()?.vim_mut().clear_transient();
            return Ok(());
        }
        for _ in 0..repeat.max(1) {
            self.replay_recorded_inputs(&inputs)?;
        }
        self.ui_mut()?.vim_mut().clear_transient();
        Ok(())
    }
}
