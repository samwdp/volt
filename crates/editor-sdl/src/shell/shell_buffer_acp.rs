impl ShellBuffer {
    fn is_acp_buffer(&self) -> bool {
        self.acp_state.is_some()
    }

    pub(crate) fn init_acp_view(&mut self, client_label: &str) {
        self.acp_prepare_session_replay(client_label);
        self.acp_push_system_message(format!(
            "{} Connected to {client_label}.",
            editor_icons::symbols::cod::COD_ROCKET
        ));
    }

    /// Reset ACP panes for `session/load` history replay without a fresh Connected banner.
    pub(crate) fn acp_prepare_session_replay(&mut self, client_label: &str) {
        self.text = TextBuffer::new();
        self.undo_tree = UndoTree::new(&self.text);
        self.scroll_row = 0;
        self.wrap_cache = None;
        self.acp_state = Some(AcpBufferState::new(client_label.to_owned()));
    }

    pub(crate) fn acp_switch_pane(&mut self) -> bool {
        let Some(state) = self.acp_state.as_mut() else {
            return false;
        };
        state.active_pane = match state.active_pane {
            AcpPane::Plan => AcpPane::Output,
            AcpPane::Output => AcpPane::Input,
            AcpPane::Input => AcpPane::Footer,
            AcpPane::Footer => AcpPane::Plan,
        };
        true
    }

    fn focus_acp_input(&mut self) -> bool {
        let Some(state) = self.acp_state.as_mut() else {
            return false;
        };
        state.active_pane = AcpPane::Input;
        true
    }

    fn acp_active_pane(&self) -> Option<AcpPane> {
        self.acp_state.as_ref().map(|state| state.active_pane)
    }

    fn acp_plan_viewport_lines(&self) -> usize {
        self.acp_state
            .as_ref()
            .map(|state| state.plan_pane.visible_rows())
            .unwrap_or(1)
    }

    fn acp_output_viewport_lines(&self) -> usize {
        self.acp_state
            .as_ref()
            .map(|state| state.output_pane.visible_rows())
            .unwrap_or(1)
    }

    fn acp_active_pane_state(&self) -> Option<&AcpPaneState> {
        let state = self.acp_state.as_ref()?;
        Some(match state.active_pane {
            AcpPane::Plan => &state.plan_pane,
            AcpPane::Output => &state.output_pane,
            AcpPane::Input | AcpPane::Footer => return None,
        })
    }

    fn acp_active_pane_state_mut(&mut self) -> Option<&mut AcpPaneState> {
        let state = self.acp_state.as_mut()?;
        Some(match state.active_pane {
            AcpPane::Plan => &mut state.plan_pane,
            AcpPane::Output => &mut state.output_pane,
            AcpPane::Input | AcpPane::Footer => return None,
        })
    }

    fn acp_footer_pane(&self) -> Option<&PluginTextPaneState> {
        self.acp_state.as_ref().map(|state| &state.footer_pane)
    }

    fn acp_footer_pane_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.acp_state.as_mut().map(|state| &mut state.footer_pane)
    }

    fn browser_active_pane(&self) -> Option<BrowserPane> {
        self.browser_state.as_ref().map(|state| state.active_pane)
    }

    fn focus_browser_input(&mut self) -> bool {
        let Some(state) = self.browser_state.as_mut() else {
            return false;
        };
        state.active_pane = BrowserPane::Input;
        true
    }

    fn acp_active_pane_is_read_only(&self) -> bool {
        matches!(
            self.acp_active_pane(),
            Some(AcpPane::Plan | AcpPane::Output | AcpPane::Footer)
        )
    }

    fn browser_active_pane_is_read_only(&self) -> bool {
        matches!(self.browser_active_pane(), Some(BrowserPane::Footer))
    }

    fn browser_footer_pane(&self) -> Option<&PluginTextPaneState> {
        self.browser_state.as_ref().map(|state| &state.footer_pane)
    }

    fn browser_footer_pane_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.browser_state
            .as_mut()
            .map(|state| &mut state.footer_pane)
    }

    fn active_aux_text_pane_state(&self) -> Option<&PluginTextPaneState> {
        if let Some(pane) = self.plugin_attached_pane_state() {
            return Some(pane);
        }
        if matches!(self.acp_active_pane(), Some(AcpPane::Footer)) {
            return self.acp_footer_pane();
        }
        if matches!(self.browser_active_pane(), Some(BrowserPane::Footer)) {
            return self.browser_footer_pane();
        }
        None
    }

    fn active_aux_text_pane_state_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        if self
            .plugin_section_state
            .as_ref()
            .is_some_and(PluginSectionBufferState::has_active_attached_section)
        {
            return self.plugin_attached_pane_state_mut();
        }
        if matches!(self.acp_active_pane(), Some(AcpPane::Footer)) {
            return self.acp_footer_pane_mut();
        }
        if matches!(self.browser_active_pane(), Some(BrowserPane::Footer)) {
            return self.browser_footer_pane_mut();
        }
        None
    }

    pub(crate) fn acp_push_user_prompt(&mut self, prompt: impl Into<String>) -> bool {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            state
                .output_items
                .push(AcpOutputItem::UserPrompt(prompt.into()));
        }
        self.acp_rebuild_output_view(follow_output);
        follow_output
    }

    pub(crate) fn acp_push_system_message(&mut self, message: impl Into<String>) -> bool {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            state
                .output_items
                .push(AcpOutputItem::SystemMessage(message.into()));
        }
        self.acp_rebuild_output_view(follow_output);
        follow_output
    }

    pub(crate) fn acp_append_agent_chunk(&mut self, content: ContentBlock) -> bool {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            match state.output_items.last_mut() {
                Some(AcpOutputItem::AgentBlocks(blocks)) => match (blocks.last_mut(), content) {
                    (Some(ContentBlock::Text(existing)), ContentBlock::Text(text)) => {
                        existing.text.push_str(&text.text);
                    }
                    (_, content) => blocks.push(content),
                },
                _ => state
                    .output_items
                    .push(AcpOutputItem::AgentBlocks(vec![content])),
            }
        }
        self.acp_rebuild_output_view(follow_output);
        follow_output
    }

    pub(crate) fn acp_set_plan(&mut self, plan: Plan) {
        if let Some(state) = self.acp_state.as_mut() {
            state.plan_entries = plan.entries;
            normalize_acp_plan_entries(&mut state.plan_entries);
        }
        self.acp_rebuild_plan_view();
    }

    pub(crate) fn acp_complete_plan(&mut self) {
        if let Some(state) = self.acp_state.as_mut() {
            for entry in &mut state.plan_entries {
                if !matches!(entry.status, PlanEntryStatus::Completed) {
                    entry.status = PlanEntryStatus::Completed;
                }
            }
        }
        self.acp_rebuild_plan_view();
    }

    pub(crate) fn acp_set_session_info(&mut self, update: &SessionInfoUpdate) {
        if let Some(state) = self.acp_state.as_mut() {
            match &update.title {
                MaybeUndefined::Value(title) => state.session_title = Some(title.clone()),
                MaybeUndefined::Null => state.session_title = None,
                MaybeUndefined::Undefined => {}
            }
        }
    }

    pub(crate) fn acp_set_session_title(&mut self, title: Option<String>) {
        if let Some(state) = self.acp_state.as_mut() {
            state.session_title = title;
        }
    }

    pub(crate) fn acp_upsert_tool_call(&mut self, tool_call: ToolCall) -> bool {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            let tool_key = tool_call.tool_call_id.to_string();
            if let Some(index) = state.tool_item_indices.get(tool_key.as_str()).copied() {
                state.output_items[index] = AcpOutputItem::ToolCall(tool_call);
            } else {
                let index = state.output_items.len();
                state.tool_item_indices.insert(tool_key, index);
                state.output_items.push(AcpOutputItem::ToolCall(tool_call));
            }
        }
        self.acp_rebuild_output_view(follow_output);
        follow_output
    }

    pub(crate) fn acp_update_tool_call(&mut self, update: ToolCallUpdate) -> bool {
        let follow_output = self
            .acp_state
            .as_ref()
            .map(|state| {
                state
                    .output_pane
                    .should_follow_output(self.acp_output_viewport_lines())
            })
            .unwrap_or(true);
        if let Some(state) = self.acp_state.as_mut() {
            let tool_key = update.tool_call_id.to_string();
            if let Some(index) = state.tool_item_indices.get(tool_key.as_str()).copied() {
                if let Some(AcpOutputItem::ToolCall(tool_call)) = state.output_items.get_mut(index)
                {
                    tool_call.update(update.fields.clone());
                }
            } else {
                let tool_call = ToolCall::try_from(update.clone())
                    .unwrap_or_else(|_| acp_tool_call_from_partial_update(&update));
                let index = state.output_items.len();
                state.tool_item_indices.insert(tool_key, index);
                state.output_items.push(AcpOutputItem::ToolCall(tool_call));
            }
        }
        self.acp_rebuild_output_view(follow_output);
        follow_output
    }

    fn acp_rebuild_plan_view(&mut self) {
        let visible_rows = self.acp_plan_viewport_lines();
        let Some(state) = self.acp_state.as_mut() else {
            return;
        };
        let render_lines = acp_build_plan_lines(&state.plan_entries);
        state
            .plan_pane
            .replace_render_lines(render_lines, false, visible_rows);
    }

    fn acp_rebuild_output_view(&mut self, follow_output: bool) {
        self.acp_rebuild_output_view_with(follow_output, None);
    }

    fn acp_rebuild_output_view_with(
        &mut self,
        follow_output: bool,
        markdown: Option<AcpMarkdownPaint<'_>>,
    ) {
        let visible_rows = self.acp_output_viewport_lines();
        let buffer_enabled = self.markdown_pretty_enabled;
        let Some(state) = self.acp_state.as_ref() else {
            return;
        };
        let render_lines = acp_build_output_lines(&state.output_items, markdown, buffer_enabled);
        let Some(state) = self.acp_state.as_mut() else {
            return;
        };
        state
            .output_pane
            .replace_render_lines(render_lines, follow_output, visible_rows);
    }

    fn input_field(&self) -> Option<&InputField> {
        self.standalone_input_field()
            .or_else(|| self.acp_state.as_ref().map(|state| &state.input))
            .or_else(|| self.browser_state.as_ref().map(|state| &state.input))
    }

    fn input_field_mut(&mut self) -> Option<&mut InputField> {
        if let Some(input) = self.input.as_mut() {
            return Some(input);
        }
        if let Some(state) = self.acp_state.as_mut() {
            return Some(&mut state.input);
        }
        self.browser_state.as_mut().map(|state| &mut state.input)
    }

    fn standalone_input_field(&self) -> Option<&InputField> {
        self.input.as_ref()
    }

    fn clear_input(&mut self) -> bool {
        let cleared = if let Some(input) = self.input_field_mut() {
            input.clear();
            true
        } else {
            false
        };
        if cleared && let Some(state) = self.acp_state.as_mut() {
            state.pasted_images.clear();
        }
        cleared
    }

    fn acp_attach_pasted_image(&mut self, image: ClipboardImage) -> Option<String> {
        let state = self.acp_state.as_mut()?;
        let id = state.next_image_id;
        state.next_image_id = state.next_image_id.saturating_add(1);
        let token = acp::acp_image_mention_token(id, &image.name);
        state.pasted_images.push(AcpPastedImage {
            id,
            name: image.name,
            mime_type: image.mime_type,
            data: base64::engine::general_purpose::STANDARD.encode(image.bytes),
        });
        Some(token)
    }

    fn acp_pasted_images(&self) -> &[AcpPastedImage] {
        self.acp_state
            .as_ref()
            .map(|state| state.pasted_images.as_slice())
            .unwrap_or(&[])
    }

    fn section_state(&self) -> Option<&SectionedBufferState> {
        self.section_state.as_ref()
    }

    fn ensure_section_state(&mut self) -> &mut SectionedBufferState {
        self.section_state
            .get_or_insert_with(SectionedBufferState::default)
    }

    fn section_line_meta(&self, line_index: usize) -> Option<&SectionLineMeta> {
        self.section_state
            .as_ref()
            .and_then(|state| state.lines.get(line_index))
    }

    fn git_snapshot(&self) -> Option<&GitStatusSnapshot> {
        self.git_snapshot.as_ref()
    }

    fn set_git_snapshot(&mut self, snapshot: GitStatusSnapshot) {
        self.git_snapshot = Some(snapshot);
    }

    fn git_status_refresh_due(&self, root: &Path, _now: Instant) -> bool {
        self.git_snapshot.is_none()
            || self.git_status_root.as_deref() != Some(root)
            || self.git_status_probe_revision != Some(git_probe_snapshot(root).revision())
    }

    fn mark_git_status_refreshed(&mut self, root: &Path, _now: Instant) {
        self.git_status_root = Some(root.to_path_buf());
        self.git_status_probe_revision = Some(git_probe_snapshot(root).revision());
    }

    fn git_view(&self) -> Option<&GitViewState> {
        self.git_view.as_ref()
    }

    fn set_git_view(&mut self, view: GitViewState) {
        self.git_view = Some(view);
    }

    fn git_fringe_state(&self) -> Option<&GitFringeState> {
        self.git_fringe.as_ref()
    }

    fn dap_fringe_live(&self) -> bool {
        self.dap_fringe_live
    }

    fn dap_fringe_marker(&self, line_index: usize) -> Option<BreakpointState> {
        self.dap_fringe_markers.get(&line_index).copied()
    }

    fn dap_execution_line(&self) -> Option<usize> {
        self.dap_execution_line
    }

    fn set_dap_fringe(
        &mut self,
        live: bool,
        markers: BTreeMap<usize, BreakpointState>,
        execution_line: Option<usize>,
    ) {
        self.dap_fringe_live = live;
        self.dap_fringe_markers = markers;
        self.dap_execution_line = execution_line;
    }

    fn clear_dap_fringe(&mut self) {
        self.dap_fringe_live = false;
        self.dap_fringe_markers.clear();
        self.dap_execution_line = None;
    }

    fn git_fringe_kind(&self, line_index: usize) -> Option<GitFringeKind> {
        self.git_fringe_state()
            .and_then(|state| state.try_line_kind(line_index))
    }

    fn git_fringe_revision(&self) -> Option<u64> {
        self.git_fringe_state()
            .map(GitFringeState::snapshot_revision)
    }

    fn mark_git_fringe_dirty(&mut self) {
        if matches!(self.kind, BufferKind::File) && self.git_fringe.is_some() {
            self.git_fringe_dirty = true;
            self.git_fringe_last_edit_at = Some(Instant::now());
        }
    }

    fn mark_git_fringe_stale(&mut self) {
        if matches!(self.kind, BufferKind::File) && self.git_fringe.is_some() {
            self.git_fringe_dirty = true;
            self.git_fringe_last_edit_at = None;
        }
    }

    fn git_fringe_refresh_due(&self, now: Instant, typing_active: bool) -> bool {
        !typing_active
            && self.git_fringe_dirty
            && self
                .git_fringe_last_edit_at
                .map(|last| now.duration_since(last) >= GIT_FRINGE_REFRESH_DEBOUNCE)
                .unwrap_or(true)
    }

    fn clear_git_fringe_dirty(&mut self) {
        self.git_fringe_dirty = false;
    }

    fn directory_state(&self) -> Option<&DirectoryViewState> {
        self.directory_state.as_ref()
    }

    fn directory_state_mut(&mut self) -> Option<&mut DirectoryViewState> {
        self.directory_state.as_mut()
    }

    fn set_directory_state(&mut self, state: DirectoryViewState) {
        self.directory_state = Some(state);
    }

    fn clear_directory_state(&mut self) {
        self.directory_state = None;
    }

    fn terminal_render(&self) -> Option<&TerminalRenderSnapshot> {
        self.terminal_render.as_ref()
    }

    fn set_terminal_render(&mut self, snapshot: TerminalRenderSnapshot) {
        self.terminal_render = Some(snapshot);
    }

    fn clear_terminal_render(&mut self) {
        self.terminal_render = None;
    }

    fn set_section_lines(&mut self, lines: Vec<SectionRenderLine>) {
        let is_git_status = buffer_is_git_status(&self.kind);
        let is_directory = buffer_is_directory(&self.kind);
        let mut text_lines = Vec::with_capacity(lines.len());
        let mut meta = Vec::with_capacity(lines.len());
        let mut syntax_lines = BTreeMap::new();
        for (line_index, line) in lines.into_iter().enumerate() {
            let formatted_line = format_section_line(&line);
            if is_git_status {
                let spans = git_status_line_spans(&line, &formatted_line);
                if !spans.is_empty() {
                    syntax_lines.insert(line_index, spans);
                }
            } else if is_directory {
                let spans = oil_directory_line_spans(&line, &formatted_line);
                if !spans.is_empty() {
                    syntax_lines.insert(line_index, spans);
                }
            }
            text_lines.push(formatted_line);
            meta.push(SectionLineMeta {
                section_id: line.section_id,
                kind: line.kind,
                action: line.action,
            });
        }
        let state = self.ensure_section_state();
        state.lines = meta;
        self.replace_with_lines_preserve_view(text_lines);
        if is_git_status || is_directory {
            self.syntax_lines = syntax_lines;
            self.syntax_dirty = false;
            self.last_edit_at = None;
        }
    }

    fn append_output_lines(&mut self, lines: &[String]) {
        if lines.is_empty() {
            return;
        }
        let original_cursor = self.cursor_point();
        let original_scroll = self.scroll_row;
        let follow_output = self.should_follow_output();
        let insert_text = lines.join("\n");
        if self.line_count() == 0 {
            self.text.set_cursor(TextPoint::new(0, 0));
            self.text.insert_text(&insert_text);
        } else {
            let last_line = self.line_count().saturating_sub(1);
            let column = self.line_len_chars(last_line);
            self.text.set_cursor(TextPoint::new(last_line, column));
            self.text.insert_text(&format!("\n{insert_text}"));
        }
        self.text.mark_clean();
        self.text.set_cursor(original_cursor);
        if follow_output {
            self.scroll_output_to_end();
        } else {
            self.scroll_row = original_scroll.min(self.line_count().saturating_sub(1));
        }
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        if self.language_id.is_some() {
            self.mark_syntax_dirty();
        } else {
            self.syntax_dirty = false;
            self.last_edit_at = None;
        }
        self.invalidate_wrap_cache();
    }

    fn language_id(&self) -> Option<&str> {
        self.language_id.as_deref()
    }

    fn kind_label(&self) -> String {
        buffer_kind_label(&self.kind)
    }

    pub(crate) fn cursor_row(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor().line;
        }
        self.acp_active_pane_state()
            .map(|pane| pane.cursor().line)
            .unwrap_or_else(|| self.text.cursor().line)
    }

    pub(crate) fn cursor_col(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor().column;
        }
        self.acp_active_pane_state()
            .map(|pane| pane.cursor().column)
            .unwrap_or_else(|| self.text.cursor().column)
    }

    pub(crate) fn cursor_point(&self) -> TextPoint {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor();
        }
        self.acp_active_pane_state()
            .map(AcpPaneState::cursor)
            .unwrap_or_else(|| self.text.cursor())
    }

    pub(crate) fn char_offset_for_point(&self, point: TextPoint) -> Option<usize> {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return Some(pane.text.point_to_char_index(point));
        }
        self.acp_active_pane_state()
            .map(|pane| pane.text.point_to_char_index(point))
            .or_else(|| Some(self.text.point_to_char_index(point)))
    }

    fn view_state(&self) -> BufferViewState {
        BufferViewState {
            cursor: self.cursor_point(),
            scroll_row: self.scroll_row,
            scroll_col: self.scroll_col,
        }
    }

    fn restore_view_state(&mut self, view_state: BufferViewState) {
        self.set_cursor(view_state.cursor);
        self.scroll_row = view_state
            .scroll_row
            .min(self.line_count().saturating_sub(1));
        self.scroll_col = view_state.scroll_col;
    }

    fn line_count(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.line_count();
        }
        self.acp_active_pane_state()
            .map(AcpPaneState::line_count)
            .unwrap_or_else(|| self.text.line_count())
    }

    fn line_len_chars(&self, line_index: usize) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.line_len_chars(line_index);
        }
        self.acp_active_pane_state()
            .map(|pane| pane.line_len_chars(line_index))
            .unwrap_or_else(|| self.text.line_len_chars(line_index).unwrap_or(0))
    }

    fn should_follow_output(&self) -> bool {
        if let Some(state) = self.acp_state.as_ref() {
            return state
                .output_pane
                .should_follow_output(self.acp_output_viewport_lines());
        }
        if self.line_count() == 0 {
            return true;
        }
        self.line_at_viewport_offset(self.viewport_lines().saturating_sub(1)) + 1
            >= self.line_count()
    }

    fn scroll_output_to_end(&mut self) {
        if let Some(state) = self.acp_state.as_mut() {
            state.output_pane.scroll_visual_row = state.output_pane.max_scroll_row();
            return;
        }
        self.scroll_row = self.line_count().saturating_sub(self.viewport_lines());
    }

    fn path(&self) -> Option<&Path> {
        if self.active_aux_text_pane_state().is_some() {
            return None;
        }
        self.text.path()
    }

    fn lsp_path(&self) -> Option<&Path> {
        if self.active_aux_text_pane_state().is_some() {
            return None;
        }
        self.text.path().or(self.lsp_path.as_deref())
    }

    fn is_dirty(&self) -> bool {
        self.text.is_dirty() || self.pdf_state().is_some_and(|state| state.dirty)
    }

    fn save_to_path(&mut self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(state) = self.pdf_state_mut() {
            state
                .document
                .save(path)
                .map_err(|error| io::Error::other(error.to_string()))?;
            state.dirty = false;
            state.metadata = PdfDocument::load_metadata(path)
                .map_err(|error| io::Error::other(error.to_string()))?;
            self.text.set_path(path.to_path_buf());
            self.backing_file_fingerprint = BackingFileFingerprint::read(path).ok();
            self.backing_file_reload_pending = false;
            self.backing_file_check_in_flight = false;
            self.refresh_pdf_view(true);
            return Ok(());
        }
        self.text.save_to_path(path)?;
        self.backing_file_fingerprint = BackingFileFingerprint::read(path).ok();
        self.backing_file_reload_pending = false;
        self.backing_file_check_in_flight = false;
        Ok(())
    }

    fn mark_backing_file_reload_pending(&mut self) {
        if (self.kind == BufferKind::File || self.is_pdf_buffer()) && self.text.path().is_some() {
            self.backing_file_reload_pending = true;
        }
    }

    fn file_reload_request(&mut self) -> Option<FileReloadWorkerRequest> {
        if self.kind != BufferKind::File
            || self.text.is_dirty()
            || self.backing_file_check_in_flight
            || !self.backing_file_reload_pending
        {
            return None;
        }
        let path = self.text.path().map(Path::to_path_buf)?;
        self.backing_file_reload_pending = false;
        self.backing_file_check_in_flight = true;
        Some(FileReloadWorkerRequest {
            buffer_id: self.id,
            buffer_revision: self.text.revision(),
            path,
            loaded_fingerprint: self.backing_file_fingerprint,
        })
    }

    fn finish_file_reload_request(&mut self) {
        self.backing_file_check_in_flight = false;
    }

    fn apply_reloaded_file_buffer(
        &mut self,
        fingerprint: BackingFileFingerprint,
        reloaded: TextBuffer,
    ) -> bool {
        self.backing_file_fingerprint = Some(fingerprint);
        if !self.text.reload_from_buffer(reloaded) {
            return false;
        }

        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.force_syntax_refresh();
        self.scroll_row = self.scroll_row.min(self.line_count().saturating_sub(1));
        self.invalidate_wrap_cache();
        if self.git_fringe.is_some() {
            self.git_fringe_dirty = true;
            self.git_fringe_last_edit_at = None;
        }
        true
    }

    fn set_syntax_snapshot(&mut self, syntax: Option<SyntaxSnapshot>) {
        let syntax_window = syntax.as_ref().and_then(|_| self.full_syntax_window());
        let syntax_lines = syntax.map(|snapshot| index_syntax_lines(snapshot, &self.text));
        self.set_indexed_syntax_lines(syntax_lines, syntax_window);
    }

    fn set_indexed_syntax_lines(
        &mut self,
        syntax_lines: Option<IndexedSyntaxLines>,
        syntax_window: Option<SyntaxLineWindow>,
    ) {
        self.syntax_lines = syntax_lines.unwrap_or_default();
        self.syntax_dirty = false;
        self.syntax_requested_revision = Some(self.text.revision());
        self.syntax_requested_window = syntax_window;
        self.syntax_requested_at = None;
        self.syntax_applied_window = syntax_window;
        self.last_edit_at = None;
    }

    fn set_language_id(&mut self, language_id: Option<String>) {
        if self.forced_language && language_id.is_none() {
            return;
        }
        self.language_id = language_id;
    }

    fn set_forced_language_id(&mut self, language_id: impl Into<String>) {
        self.language_id = Some(language_id.into());
        self.forced_language = true;
    }

    fn markdown_pretty_enabled(&self) -> Option<bool> {
        self.markdown_pretty_enabled
    }

    fn toggle_markdown_pretty(&mut self, default_enabled: bool) {
        let current = self.markdown_pretty_enabled.unwrap_or(default_enabled);
        self.markdown_pretty_enabled = Some(!current);
    }

    fn rainbow_parens_enabled(&self, default_enabled: bool) -> bool {
        self.rainbow_parens_enabled.unwrap_or(default_enabled)
    }

    fn toggle_rainbow_parens(&mut self, default_enabled: bool) {
        let current = self.rainbow_parens_enabled(default_enabled);
        self.rainbow_parens_enabled = Some(!current);
    }

    fn show_paren_enabled(&self, default_enabled: bool) -> bool {
        self.show_paren_enabled.unwrap_or(default_enabled)
    }

    fn toggle_show_paren(&mut self, default_enabled: bool) {
        let current = self.show_paren_enabled(default_enabled);
        self.show_paren_enabled = Some(!current);
    }

    fn lsp_diagnostics(&self) -> &[LspDiagnostic] {
        &self.lsp_diagnostics
    }

    fn lsp_enabled(&self) -> bool {
        self.lsp_enabled
    }

    fn lsp_diagnostic_line_spans(&self, line_index: usize) -> &[DiagnosticLineSpan] {
        self.lsp_diagnostic_lines
            .get(&line_index)
            .map(Box::as_ref)
            .unwrap_or(&[])
    }

    fn set_lsp_enabled(&mut self, enabled: bool) {
        self.lsp_enabled = enabled;
    }

    fn set_lsp_path(&mut self, path: Option<PathBuf>) {
        self.lsp_path = path;
    }

    fn lsp_diagnostics_revision(&self) -> u64 {
        self.lsp_diagnostics_revision
    }

    fn set_lsp_diagnostics(&mut self, diagnostics: Vec<LspDiagnostic>) -> bool {
        if self.lsp_diagnostics == diagnostics {
            return false;
        }
        self.lsp_diagnostic_lines = diagnostic_line_spans_for_diagnostics(&diagnostics);
        self.lsp_diagnostics = diagnostics;
        self.lsp_diagnostics_revision = self.lsp_diagnostics_revision.saturating_add(1);
        true
    }

    fn lsp_diagnostic_severity(&self, line_index: usize) -> Option<LspDiagnosticSeverity> {
        self.lsp_diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.range().start().line <= line_index)
            .filter(|diagnostic| diagnostic.range().end().line >= line_index)
            .map(LspDiagnostic::severity)
            .min_by_key(|severity| diagnostic_severity_rank(*severity))
    }

    fn invalidate_wrap_cache(&mut self) {
        self.wrap_cache = None;
    }

    fn prepare_wrap_cache_inline_edit(&self, line_index: usize) -> Option<WrapCacheInlineEdit> {
        let cache = self.wrap_cache.as_ref()?;
        if !cache.matches(cache.wrap_cols, cache.indent_size, self.line_count()) {
            return None;
        }
        if line_index >= self.line_count() {
            return None;
        }
        Some(WrapCacheInlineEdit {
            line_index,
            old_row_count: self.line_visual_row_count(
                line_index,
                cache.wrap_cols,
                cache.indent_size,
            ),
        })
    }

    fn apply_wrap_cache_inline_edit(&mut self, edit: Option<WrapCacheInlineEdit>) {
        let Some(edit) = edit else {
            return;
        };
        let (wrap_cols, indent_size, cached_line_count) = match self.wrap_cache.as_ref() {
            Some(cache) => (cache.wrap_cols, cache.indent_size, cache.line_count),
            None => return,
        };
        if cached_line_count != self.line_count() {
            self.wrap_cache = None;
            return;
        }
        let new_row_count = self.line_visual_row_count(edit.line_index, wrap_cols, indent_size);
        if let Some(cache) = self.wrap_cache.as_mut() {
            cache.adjust_for_line_row_delta(edit.line_index, edit.old_row_count, new_row_count);
        }
    }

    fn prepare_wrap_cache_line_splice(
        &self,
        start_line: usize,
        old_span: usize,
    ) -> Option<WrapCacheLineSplice> {
        let cache = self.wrap_cache.as_ref()?;
        if cache.line_count != self.line_count()
            || cache.prefix_rows.len() != cache.line_count.saturating_add(1)
        {
            return None;
        }
        if old_span == 0 || start_line.saturating_add(old_span) > cache.line_count {
            return None;
        }
        Some(WrapCacheLineSplice {
            start_line,
            old_span,
            old_line_count: cache.line_count,
        })
    }

    fn apply_wrap_cache_line_splice(&mut self, splice: Option<WrapCacheLineSplice>) {
        let Some(splice) = splice else {
            return;
        };
        let (wrap_cols, indent_size) = match self.wrap_cache.as_ref() {
            Some(cache) => (cache.wrap_cols, cache.indent_size),
            None => return,
        };
        let new_span = match self
            .line_count()
            .checked_add(splice.old_span)
            .and_then(|sum| sum.checked_sub(splice.old_line_count))
        {
            Some(span) => span,
            None => {
                self.wrap_cache = None;
                return;
            }
        };
        if new_span > MAX_WRAP_CACHE_SPLICE_LINES
            || splice.start_line.saturating_add(new_span) > self.line_count()
        {
            self.wrap_cache = None;
            return;
        }
        let new_row_counts: Vec<usize> = (0..new_span)
            .map(|offset| {
                self.line_visual_row_count(
                    splice.start_line.saturating_add(offset),
                    wrap_cols,
                    indent_size,
                )
            })
            .collect();
        let spliced = self.wrap_cache.as_mut().is_some_and(|cache| {
            cache.splice_lines(splice.start_line, splice.old_span, &new_row_counts)
        });
        if !spliced {
            self.wrap_cache = None;
        }
    }

    fn finish_wrap_cache_line_splice(
        &mut self,
        had_cache: bool,
        splice: Option<WrapCacheLineSplice>,
    ) {
        if splice.is_some() {
            self.apply_wrap_cache_line_splice(splice);
        } else if had_cache {
            self.invalidate_wrap_cache();
        }
    }

    fn plan_wrap_cache_insert(&self, start_line: usize, has_newline: bool) -> WrapCacheInsertPlan {
        WrapCacheInsertPlan {
            inline: (!has_newline)
                .then(|| self.prepare_wrap_cache_inline_edit(start_line))
                .flatten(),
            splice: has_newline
                .then(|| self.prepare_wrap_cache_line_splice(start_line, 1))
                .flatten(),
            had_cache: has_newline && self.wrap_cache.is_some(),
            has_newline,
        }
    }

    fn commit_wrap_cache_insert_plan(&mut self, plan: WrapCacheInsertPlan) {
        if plan.has_newline {
            self.finish_wrap_cache_line_splice(plan.had_cache, plan.splice);
        } else {
            self.apply_wrap_cache_inline_edit(plan.inline);
        }
    }

    fn refresh_wrap_cache(
        &mut self,
        wrap_cols: usize,
        indent_size: usize,
        line_count: usize,
        distance: usize,
        threshold: usize,
    ) {
        let cache_valid = self
            .wrap_cache
            .as_ref()
            .map(|cache| cache.matches(wrap_cols, indent_size, line_count))
            .unwrap_or(false);
        if !cache_valid {
            self.wrap_cache = None;
        }
        if self.wrap_cache.is_none()
            && line_count > 0
            && (line_count >= LARGE_BUFFER_WRAP_CACHE_LINE_THRESHOLD || distance >= threshold)
        {
            self.wrap_cache = Some(WrapRowCache::build(self, wrap_cols, indent_size));
        }
    }

    fn set_syntax_error(&mut self, error: Option<String>) {
        self.syntax_error = error;
    }

    fn reload_from_disk_if_changed(&mut self, force: bool) -> Result<bool, String> {
        if !matches!(self.kind, BufferKind::File) && !self.is_pdf_buffer() {
            return Ok(false);
        }
        if self.kind == BufferKind::File && self.text.is_dirty() {
            return Ok(false);
        }
        if self.is_pdf_buffer() && self.pdf_state().is_some_and(|state| state.dirty) {
            return Ok(false);
        }
        let Some(path) = self.text.path().map(Path::to_path_buf) else {
            return Ok(false);
        };
        if !force && !self.backing_file_reload_pending {
            return Ok(false);
        }
        self.backing_file_reload_pending = false;
        self.backing_file_check_in_flight = false;

        let current_fingerprint = match BackingFileFingerprint::read(&path) {
            Ok(fingerprint) => fingerprint,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(format!("failed to stat `{}`: {error}", path.display()));
            }
        };
        let Some(loaded_fingerprint) = self.backing_file_fingerprint else {
            self.backing_file_fingerprint = Some(current_fingerprint);
            return Ok(false);
        };
        if current_fingerprint == loaded_fingerprint {
            return Ok(false);
        }

        if self.is_pdf_buffer() {
            let current_page = self
                .pdf_state()
                .map(|state| state.current_page)
                .unwrap_or(1);
            let fit_mode = self
                .pdf_state()
                .map(|state| state.fit_mode)
                .unwrap_or(PdfFitMode::Page);
            let zoom_percent = self
                .pdf_state()
                .map(|state| state.zoom_percent)
                .unwrap_or(100);
            let open_mode = self
                .pdf_state()
                .map(|state| state.open_mode)
                .unwrap_or(PdfOpenMode::Rendered);
            let mut state = load_pdf_buffer_state(&path)
                .map_err(|error| format!("failed to reload `{}`: {error}", path.display()))?;
            state.current_page = current_page;
            state.fit_mode = fit_mode;
            state.zoom_percent = zoom_percent;
            state.open_mode = open_mode;
            state.clamp_current_page();
            self.pdf_state = Some(state);
            self.refresh_pdf_view(true);
            self.backing_file_fingerprint = Some(current_fingerprint);
            self.backing_file_reload_pending = false;
            self.backing_file_check_in_flight = false;
            return Ok(true);
        }

        let reloaded = match self.text.reload_from_path() {
            Ok(reloaded) => reloaded,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => {
                return Err(format!("failed to reload `{}`: {error}", path.display()));
            }
        };
        self.backing_file_fingerprint = Some(current_fingerprint);
        if !reloaded {
            return Ok(false);
        }

        self.finish_file_reload_request();
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.force_syntax_refresh();
        self.scroll_row = self.scroll_row.min(self.line_count().saturating_sub(1));
        self.invalidate_wrap_cache();
        if self.git_fringe.is_some() {
            self.git_fringe_dirty = true;
            self.git_fringe_last_edit_at = None;
        }
        Ok(true)
    }

    fn replace_with_lines(&mut self, lines: Vec<String>) {
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text = text;
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.syntax_dirty = false;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.syntax_requested_at = None;
        self.syntax_applied_window = None;
        self.last_edit_at = None;
        self.scroll_row = 0;
        self.invalidate_wrap_cache();
    }

    fn replace_with_lines_preserve_view(&mut self, lines: Vec<String>) {
        let cursor = self.cursor_point();
        let scroll_row = self.scroll_row;
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text = text;
        self.text.mark_clean();
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.syntax_dirty = false;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.syntax_requested_at = None;
        self.syntax_applied_window = None;
        self.last_edit_at = None;
        let line_count = self.line_count();
        if line_count == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_row = 0;
            return;
        }
        let line = cursor.line.min(line_count.saturating_sub(1));
        let column = cursor.column.min(self.line_len_chars(line));
        self.text.set_cursor(TextPoint::new(line, column));
        let max_scroll = line_count.saturating_sub(1);
        self.scroll_row = scroll_row.min(max_scroll);
        self.invalidate_wrap_cache();
    }

    fn replace_with_lines_follow_output(&mut self, lines: Vec<String>) {
        let cursor = self.cursor_point();
        let scroll_row = self.scroll_row;
        let follow_output = self.should_follow_output();
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text = text;
        self.text.mark_clean();
        self.undo_tree = UndoTree::new(&self.text);
        self.syntax_error = None;
        self.syntax_lines.clear();
        self.syntax_dirty = false;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.syntax_requested_at = None;
        self.syntax_applied_window = None;
        self.last_edit_at = None;
        let line_count = self.line_count();
        if line_count == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_row = 0;
            return;
        }
        let line = cursor.line.min(line_count.saturating_sub(1));
        let column = cursor.column.min(self.line_len_chars(line));
        self.text.set_cursor(TextPoint::new(line, column));
        if follow_output {
            self.scroll_output_to_end();
        } else {
            let max_scroll = line_count.saturating_sub(1);
            self.scroll_row = scroll_row.min(max_scroll);
        }
        self.invalidate_wrap_cache();
    }

    fn mark_syntax_dirty(&mut self) {
        if self.kind == BufferKind::File || self.language_id.is_some() {
            self.syntax_dirty = true;
            self.syntax_requested_window = None;
            self.syntax_requested_at = None;
            self.syntax_applied_window = None;
            self.last_edit_at = Some(Instant::now());
            self.mark_git_fringe_dirty();
        }
    }

    fn force_syntax_refresh(&mut self) {
        if self.kind == BufferKind::File || self.language_id.is_some() {
            self.syntax_dirty = true;
            self.syntax_requested_revision = None;
            self.syntax_requested_window = None;
            self.syntax_requested_at = None;
            self.syntax_applied_window = None;
            self.last_edit_at = None;
        }
    }

    fn mark_syntax_refresh_requested(&mut self, syntax_window: Option<SyntaxLineWindow>) {
        self.syntax_requested_revision = Some(self.text.revision());
        self.syntax_requested_window = syntax_window;
        self.syntax_requested_at = Some(Instant::now());
    }

    fn syntax_refresh_due(&self, now: Instant) -> bool {
        const SYNTAX_REFRESH_COLD_DEBOUNCE: Duration = Duration::from_millis(75);
        const SYNTAX_REFRESH_INCREMENTAL_DEBOUNCE: Duration = Duration::from_millis(8);
        let debounce = if self.syntax_applied_window.is_some() {
            SYNTAX_REFRESH_INCREMENTAL_DEBOUNCE
        } else {
            SYNTAX_REFRESH_COLD_DEBOUNCE
        };
        let request_timed_out = self.syntax_requested_revision == Some(self.text.revision())
            && self.syntax_requested_at.is_some_and(|requested_at| {
                now.duration_since(requested_at) >= SYNTAX_REFRESH_REQUEST_TIMEOUT
            });
        self.syntax_dirty
            && (self.syntax_requested_revision != Some(self.text.revision()) || request_timed_out)
            && self
                .last_edit_at
                .map(|last_edit_at| now.duration_since(last_edit_at) >= debounce)
                .unwrap_or(true)
    }

    fn line_syntax_spans(&self, line_index: usize) -> Option<&[LineSyntaxSpan]> {
        self.syntax_lines.get(&line_index).map(Vec::as_slice)
    }

    fn full_syntax_window(&self) -> Option<SyntaxLineWindow> {
        SyntaxLineWindow::new(0, self.line_count())
    }

    fn worker_syntax_window(&self) -> Option<SyntaxLineWindow> {
        self.desired_syntax_window()
            .or_else(|| self.full_syntax_window())
    }

    fn desired_syntax_window(&self) -> Option<SyntaxLineWindow> {
        if self.kind != BufferKind::File && self.language_id.is_none() {
            return None;
        }
        let line_count = self.line_count();
        if line_count == 0 {
            return None;
        }
        let visible_lines = self.viewport_lines();
        let target_lines = visible_lines
            .saturating_add(SYNTAX_WINDOW_MARGIN_LINES.saturating_mul(2))
            .max(SYNTAX_WINDOW_MIN_LINES)
            .min(line_count);
        let centered_margin = target_lines.saturating_sub(visible_lines) / 2;
        let max_start_line = line_count.saturating_sub(target_lines);
        let start_line = self
            .scroll_row
            .saturating_sub(centered_margin)
            .min(max_start_line);
        SyntaxLineWindow::new(start_line, target_lines)
    }

    fn ensure_visible_syntax_window(&mut self) {
        let Some(desired_window) = self.desired_syntax_window() else {
            return;
        };
        let current_revision = self.text.revision();
        let applied_matches = self.syntax_requested_revision == Some(current_revision)
            && self
                .syntax_applied_window
                .map(|window| window.contains(desired_window))
                .unwrap_or(false);
        let requested_matches = self.syntax_requested_revision == Some(current_revision)
            && self
                .syntax_requested_window
                .map(|window| window.contains(desired_window))
                .unwrap_or(false);
        if applied_matches || requested_matches {
            return;
        }
        self.syntax_dirty = true;
        self.syntax_requested_revision = None;
        self.syntax_requested_window = None;
        self.syntax_requested_at = None;
        self.last_edit_at = None;
    }
}
