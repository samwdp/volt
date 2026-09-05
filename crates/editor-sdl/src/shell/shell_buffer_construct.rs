impl ShellBuffer {
    fn from_runtime_buffer(
        buffer: &Buffer,
        lines: Vec<String>,
        user_library: &dyn UserLibrary,
    ) -> Self {
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        let undo_tree = UndoTree::new(&text);
        let (read_only, input) = buffer_interaction(buffer.kind(), user_library);
        let plugin_section_state = plugin_section_state_for_kind(buffer.kind(), user_library);
        let browser_state = browser_state_for_kind(buffer.kind(), user_library);
        let vim_target = default_vim_target(input.is_some() || browser_state.is_some());

        Self {
            id: buffer.id(),
            name: buffer.name().to_owned(),
            kind: buffer.kind().clone(),
            read_only,
            input,
            section_state: None,
            plugin_section_state,
            image_state: None,
            pdf_state: None,
            acp_state: None,
            git_snapshot: None,
            git_status_root: None,
            git_status_probe_revision: None,
            git_view: None,
            git_fringe: None,
            git_fringe_dirty: false,
            git_fringe_last_edit_at: None,
            dap_fringe_live: false,
            dap_fringe_markers: BTreeMap::new(),
            dap_execution_line: None,
            browser_state,
            directory_state: None,
            terminal_render: None,
            text,
            lsp_path: None,
            backing_file_fingerprint: None,
            backing_file_reload_pending: false,
            backing_file_check_in_flight: false,
            undo_tree,
            language_id: None,
            forced_language: false,
            markdown_pretty_enabled: None,
            rainbow_parens_enabled: None,
            show_paren_enabled: None,
            scroll_row: 0,
            scroll_col: 0,
            line_wrap: plugin_buffer_line_wrap(buffer.kind(), user_library),
            viewport_lines: 1,
            content_viewport_lines: 1,
            scroll_wrap_cols: 1,
            scroll_indent_size: 1,
            wrap_cache: None,
            pretty_display_rows: BTreeMap::new(),
            markdown_pretty_plan_cache: Arc::new(Mutex::new(
                editor_markdown::MarkdownPrettyPlanCache::default(),
            )),
            context_overlay_cache: Arc::new(Mutex::new(None)),
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            syntax_requested_revision: None,
            syntax_requested_window: None,
            syntax_requested_at: None,
            syntax_applied_window: None,
            lsp_enabled: true,
            lsp_diagnostics: Vec::new(),
            lsp_diagnostic_lines: BTreeMap::new(),
            lsp_diagnostics_revision: 0,
            inline_completion: None,
            last_edit_at: None,
            vim_buffer_state: VimBufferState {
                target: vim_target,
                ..VimBufferState::default()
            },
        }
    }

    fn from_text_buffer(buffer: &Buffer, text: TextBuffer, user_library: &dyn UserLibrary) -> Self {
        let undo_tree = UndoTree::new(&text);
        let (read_only, input) = buffer_interaction(buffer.kind(), user_library);
        let plugin_section_state = plugin_section_state_for_kind(buffer.kind(), user_library);
        let browser_state = browser_state_for_kind(buffer.kind(), user_library);
        let vim_target = default_vim_target(input.is_some() || browser_state.is_some());
        let git_fringe = if matches!(buffer.kind(), BufferKind::File) && text.path().is_some() {
            Some(GitFringeState::new())
        } else {
            None
        };
        let git_fringe_dirty = git_fringe.is_some();
        let git_fringe_last_edit_at = git_fringe_dirty.then(Instant::now);
        let backing_file_fingerprint = text
            .path()
            .and_then(|path| BackingFileFingerprint::read(path).ok());
        Self {
            id: buffer.id(),
            name: buffer.name().to_owned(),
            kind: buffer.kind().clone(),
            read_only,
            input,
            section_state: None,
            plugin_section_state,
            image_state: None,
            pdf_state: None,
            acp_state: None,
            git_snapshot: None,
            git_status_root: None,
            git_status_probe_revision: None,
            git_view: None,
            git_fringe,
            git_fringe_dirty,
            git_fringe_last_edit_at,
            dap_fringe_live: false,
            dap_fringe_markers: BTreeMap::new(),
            dap_execution_line: None,
            browser_state,
            directory_state: None,
            terminal_render: None,
            text,
            lsp_path: None,
            backing_file_fingerprint,
            backing_file_reload_pending: false,
            backing_file_check_in_flight: false,
            undo_tree,
            language_id: None,
            forced_language: false,
            markdown_pretty_enabled: None,
            rainbow_parens_enabled: None,
            show_paren_enabled: None,
            scroll_row: 0,
            scroll_col: 0,
            line_wrap: plugin_buffer_line_wrap(buffer.kind(), user_library),
            viewport_lines: 1,
            content_viewport_lines: 1,
            scroll_wrap_cols: 1,
            scroll_indent_size: 1,
            wrap_cache: None,
            pretty_display_rows: BTreeMap::new(),
            markdown_pretty_plan_cache: Arc::new(Mutex::new(
                editor_markdown::MarkdownPrettyPlanCache::default(),
            )),
            context_overlay_cache: Arc::new(Mutex::new(None)),
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            syntax_requested_revision: None,
            syntax_requested_window: None,
            syntax_requested_at: None,
            syntax_applied_window: None,
            lsp_enabled: true,
            lsp_diagnostics: Vec::new(),
            lsp_diagnostic_lines: BTreeMap::new(),
            lsp_diagnostics_revision: 0,
            inline_completion: None,
            last_edit_at: None,
            vim_buffer_state: VimBufferState {
                target: vim_target,
                ..VimBufferState::default()
            },
        }
    }

    fn placeholder(
        buffer_id: BufferId,
        name: &str,
        kind: BufferKind,
        user_library: &dyn UserLibrary,
    ) -> Self {
        let lines = placeholder_lines(name, &kind, user_library);
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        let undo_tree = UndoTree::new(&text);
        let (read_only, input) = buffer_interaction(&kind, user_library);
        let browser_state = browser_state_for_kind(&kind, user_library);
        let plugin_section_state = plugin_section_state_for_kind(&kind, user_library);
        let line_wrap = plugin_buffer_line_wrap(&kind, user_library);
        let vim_target = default_vim_target(input.is_some() || browser_state.is_some());

        Self {
            id: buffer_id,
            name: name.to_owned(),
            kind,
            read_only,
            input,
            section_state: None,
            plugin_section_state,
            image_state: None,
            pdf_state: None,
            acp_state: None,
            git_snapshot: None,
            git_status_root: None,
            git_status_probe_revision: None,
            git_view: None,
            git_fringe: None,
            git_fringe_dirty: false,
            git_fringe_last_edit_at: None,
            dap_fringe_live: false,
            dap_fringe_markers: BTreeMap::new(),
            dap_execution_line: None,
            browser_state,
            directory_state: None,
            terminal_render: None,
            text,
            lsp_path: None,
            backing_file_fingerprint: None,
            backing_file_reload_pending: false,
            backing_file_check_in_flight: false,
            undo_tree,
            language_id: None,
            forced_language: false,
            markdown_pretty_enabled: None,
            rainbow_parens_enabled: None,
            show_paren_enabled: None,
            scroll_row: 0,
            scroll_col: 0,
            line_wrap,
            viewport_lines: 1,
            content_viewport_lines: 1,
            scroll_wrap_cols: 1,
            scroll_indent_size: 1,
            wrap_cache: None,
            pretty_display_rows: BTreeMap::new(),
            markdown_pretty_plan_cache: Arc::new(Mutex::new(
                editor_markdown::MarkdownPrettyPlanCache::default(),
            )),
            context_overlay_cache: Arc::new(Mutex::new(None)),
            syntax_error: None,
            syntax_lines: BTreeMap::new(),
            syntax_dirty: false,
            syntax_requested_revision: None,
            syntax_requested_window: None,
            syntax_requested_at: None,
            syntax_applied_window: None,
            lsp_enabled: true,
            lsp_diagnostics: Vec::new(),
            lsp_diagnostic_lines: BTreeMap::new(),
            lsp_diagnostics_revision: 0,
            inline_completion: None,
            last_edit_at: None,
            vim_buffer_state: VimBufferState {
                target: vim_target,
                ..VimBufferState::default()
            },
        }
    }

    pub(crate) fn id(&self) -> BufferId {
        self.id
    }

    pub(crate) fn display_name(&self) -> &str {
        &self.name
    }

    fn context_overlay_snapshot(
        &self,
        user_library: &dyn UserLibrary,
        typing_active: bool,
    ) -> Arc<BufferContextOverlaySnapshot> {
        let key = BufferContextOverlayCacheKey {
            buffer_revision: self.text.revision(),
            buffer_name: self.display_name().to_owned(),
            language_id: self.language_id.clone(),
            viewport_top_line: self.scroll_row,
            cursor_line: self.cursor_row(),
            cursor_column: self.cursor_col(),
        };
        if let Ok(cache) = self.context_overlay_cache.lock()
            && let Some(snapshot) =
                cached_context_overlay_snapshot(cache.as_ref(), &key, typing_active)
        {
            return snapshot;
        }

        let buffer_text = self.text.text();
        let buffer_name = key.buffer_name.clone();
        let language_id = key.language_id.clone();
        let context = HostGhostTextContext {
            buffer_id: self.id().get(),
            buffer_revision: key.buffer_revision,
            buffer_name: &buffer_name,
            language_id: language_id.as_deref(),
            buffer_text: &buffer_text,
            viewport_top_line: key.viewport_top_line,
            cursor_line: key.cursor_line,
            cursor_column: key.cursor_column,
        };
        let mut ghost_text_by_line: BTreeMap<usize, String> = user_library
            .ghost_text_lines(&context)
            .into_iter()
            .map(|line| (line.line, line.text))
            .collect();
        if let Some(ghost_text) = self.inline_completion_ghost_text() {
            ghost_text_by_line.insert(key.cursor_line, ghost_text);
        }
        let snapshot = Arc::new(BufferContextOverlaySnapshot {
            key,
            headerline_lines: user_library.headerline_lines(&context),
            ghost_text_by_line,
        });
        if let Ok(mut cache) = self.context_overlay_cache.lock() {
            *cache = Some(Arc::clone(&snapshot));
        }
        snapshot
    }

    fn clear_context_overlay_cache(&self) {
        if let Ok(mut cache) = self.context_overlay_cache.lock() {
            *cache = None;
        }
    }

    fn is_read_only(&self) -> bool {
        self.read_only
            || self
                .plugin_section_state
                .as_ref()
                .is_some_and(|state| !state.active_section_writable())
            || self.acp_active_pane_is_read_only()
            || self.browser_active_pane_is_read_only()
            || (self.kind == BufferKind::Image && !self.is_svg_source_mode())
    }

    fn inline_completion_ghost_text(&self) -> Option<String> {
        let completion = self.inline_completion.as_ref()?;
        if completion.buffer_revision != self.text.revision()
            || completion.cursor != self.cursor_point()
            || completion.item.range().start() > completion.cursor
            || completion.item.range().end() < completion.cursor
        {
            return None;
        }
        let inserted = completion.item.insert_text();
        let preview = if completion.item.range().start() < completion.cursor {
            let typed_range = TextRange::new(completion.item.range().start(), completion.cursor);
            let typed = self.slice(typed_range);
            inserted.strip_prefix(&typed).unwrap_or(inserted)
        } else {
            inserted
        };
        preview
            .split('\n')
            .next()
            .map(str::to_owned)
            .filter(|line| !line.is_empty())
    }

    fn set_inline_completion(&mut self, item: LspInlineCompletionItem) {
        self.inline_completion = Some(InlineCompletionState {
            item,
            buffer_revision: self.text.revision(),
            cursor: self.cursor_point(),
            shown: false,
        });
        self.clear_context_overlay_cache();
    }

    fn mark_inline_completion_shown(&mut self) -> Option<LspInlineCompletionItem> {
        let completion = self.inline_completion.as_mut()?;
        if completion.shown {
            return None;
        }
        completion.shown = true;
        Some(completion.item.clone())
    }

    fn take_valid_inline_completion(&mut self) -> Option<LspInlineCompletionItem> {
        let completion = self.inline_completion.take()?;
        if completion.buffer_revision == self.text.revision()
            && completion.cursor == self.cursor_point()
            && completion.item.range().start() <= completion.cursor
            && completion.item.range().end() >= completion.cursor
        {
            Some(completion.item)
        } else {
            None
        }
    }

    fn clear_inline_completion(&mut self) {
        if self.inline_completion.take().is_some() {
            self.clear_context_overlay_cache();
        }
    }

    fn has_input_field(&self) -> bool {
        self.input_field().is_some()
    }

    fn has_plugin_sections(&self) -> bool {
        self.plugin_section_state.is_some()
    }

    fn line_wrap(&self) -> bool {
        self.line_wrap
    }

    fn toggle_line_wrap(&mut self) -> bool {
        self.line_wrap = !self.line_wrap;
        if self.line_wrap {
            self.scroll_col = 0;
        }
        self.wrap_cache = None;
        self.line_wrap
    }

    #[cfg(test)]
    fn plugin_active_section_index(&self) -> Option<usize> {
        self.plugin_section_state
            .as_ref()
            .map(|state| state.active_section)
    }

    fn plugin_attached_pane_state(&self) -> Option<&PluginTextPaneState> {
        self.plugin_section_state
            .as_ref()
            .and_then(PluginSectionBufferState::active_attached_section)
    }

    fn plugin_attached_pane_state_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.plugin_section_state
            .as_mut()
            .and_then(PluginSectionBufferState::active_attached_section_mut)
    }

    fn plugin_switch_pane(&mut self) -> bool {
        let Some(state) = self.plugin_section_state.as_mut() else {
            return false;
        };
        if state.section_count() <= 1 {
            return false;
        }
        state.active_section = (state.active_section + 1) % state.section_count();
        true
    }

    fn plugin_focus_section_named(&mut self, name: &str) -> bool {
        self.plugin_section_state
            .as_mut()
            .is_some_and(|state| state.focus_section_named(name))
    }

    fn plugin_active_section_name(&self) -> Option<&str> {
        self.plugin_section_state
            .as_ref()
            .map(PluginSectionBufferState::active_section_name)
    }

    fn plugin_section_index_at_point(
        &self,
        rect: Rect,
        layout: BufferFooterLayout,
        cell_width: i32,
        line_height: i32,
        x: i32,
        y: i32,
    ) -> Option<usize> {
        let section_layout =
            plugin_section_buffer_layout(self, rect, layout, cell_width, line_height)?;
        section_layout.panes.iter().position(|pane| {
            x >= pane.rect.x()
                && y >= pane.rect.y()
                && x < pane.rect.x() + pane.rect.width() as i32
                && y < pane.rect.y() + pane.rect.height() as i32
        })
    }

    fn plugin_focus_section_index(&mut self, index: usize) -> bool {
        let Some(state) = self.plugin_section_state.as_mut() else {
            return false;
        };
        if index >= state.section_count() {
            return false;
        }
        state.active_section = index;
        true
    }

    fn plugin_sections(&self) -> Option<&PluginSectionBufferState> {
        self.plugin_section_state.as_ref()
    }
}
