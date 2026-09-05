impl ShellBuffer {
    fn insert_text(&mut self, text: &str) {
        self.clear_inline_completion();
        let start_line = self.cursor_row();
        let plan = self.plan_wrap_cache_insert(start_line, text.contains('\n'));
        self.preserve_root_cursor_before_text_change();
        self.text.insert_text(text);
        self.commit_wrap_cache_insert_plan(plan);
    }

    fn replace_mode_text(&mut self, text: &str) {
        self.clear_inline_completion();
        let start_line = self.cursor_row();
        let plan = self.plan_wrap_cache_insert(start_line, text.contains('\n'));
        let mut changed = false;
        for character in text.chars() {
            if character == '\n' {
                self.preserve_root_cursor_before_text_change();
                self.text.insert_newline();
                changed = true;
                continue;
            }

            let point = self.cursor_point();
            let Some(next) = self.point_after(point) else {
                self.preserve_root_cursor_before_text_change();
                self.text.insert_text(&character.to_string());
                changed = true;
                continue;
            };

            let current = self.slice(TextRange::new(point, next));
            if current == "\n" {
                self.preserve_root_cursor_before_text_change();
                self.text.insert_text(&character.to_string());
                changed = true;
            } else {
                self.preserve_root_cursor_before_text_change();
                self.text
                    .replace(TextRange::new(point, next), &character.to_string());
                changed = true;
            }
        }
        if changed {
            self.commit_wrap_cache_insert_plan(plan);
        }
    }

    fn backspace(&mut self) {
        self.clear_inline_completion();
        let single_line_edit = self.cursor_col() > 0;
        let join_start = (!single_line_edit && self.cursor_row() > 0)
            .then_some(self.cursor_row().saturating_sub(1));
        let wrap_edit = single_line_edit
            .then(|| self.prepare_wrap_cache_inline_edit(self.cursor_row()))
            .flatten();
        let had_wrap_cache = join_start.is_some() && self.wrap_cache.is_some();
        let wrap_splice =
            join_start.and_then(|start_line| self.prepare_wrap_cache_line_splice(start_line, 2));
        self.preserve_root_cursor_before_text_change();
        if self.text.backspace() {
            if single_line_edit {
                self.apply_wrap_cache_inline_edit(wrap_edit);
            } else {
                self.finish_wrap_cache_line_splice(had_wrap_cache, wrap_splice);
            }
        }
    }

    fn delete_forward(&mut self) {
        self.clear_inline_completion();
        let current = self.cursor_point();
        let next = self.point_after(current);
        let joining_newline = next
            .map(|next| self.slice(TextRange::new(current, next)) == "\n")
            .unwrap_or(false);
        let single_line_edit = next.is_some() && !joining_newline;
        let wrap_edit = single_line_edit
            .then(|| self.prepare_wrap_cache_inline_edit(self.cursor_row()))
            .flatten();
        let had_wrap_cache = joining_newline && self.wrap_cache.is_some();
        let wrap_splice = joining_newline
            .then(|| self.prepare_wrap_cache_line_splice(current.line, 2))
            .flatten();
        self.preserve_root_cursor_before_text_change();
        if self.text.delete_forward() {
            if single_line_edit {
                self.apply_wrap_cache_inline_edit(wrap_edit);
            } else {
                self.finish_wrap_cache_line_splice(had_wrap_cache, wrap_splice);
            }
        }
    }

    fn move_left(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_left();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_left();
        }
        self.text.move_left()
    }

    fn move_right(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_right();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_right();
        }
        self.text.move_right()
    }

    fn move_up(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_up();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.move_visual_row(-1);
        }
        self.text.move_up()
    }

    fn move_down(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_down();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.move_visual_row(1);
        }
        self.text.move_down()
    }

    fn move_word_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_forward();
        }
        self.text.move_word_forward()
    }

    fn move_big_word_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_forward();
        }
        self.text.move_big_word_forward()
    }

    fn move_word_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_backward();
        }
        self.text.move_word_backward()
    }

    fn move_big_word_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_backward();
        }
        self.text.move_big_word_backward()
    }

    fn move_word_end(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_end_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_end_forward();
        }
        self.text.move_word_end_forward()
    }

    fn move_big_word_end(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_end_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_end_forward();
        }
        self.text.move_big_word_end_forward()
    }

    fn move_word_end_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_word_end_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_word_end_backward();
        }
        self.text.move_word_end_backward()
    }

    fn move_big_word_end_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_big_word_end_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_big_word_end_backward();
        }
        self.text.move_big_word_end_backward()
    }

    fn move_matching_delimiter(&mut self) -> bool {
        let language_id = self.language_id.clone();
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_matching_delimiter(language_id.as_deref());
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_matching_delimiter(language_id.as_deref());
        }
        self.text.move_matching_delimiter(language_id.as_deref())
    }

    fn move_sentence_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_sentence_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_sentence_forward();
        }
        self.text.move_sentence_forward()
    }

    fn move_sentence_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_sentence_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_sentence_backward();
        }
        self.text.move_sentence_backward()
    }

    fn move_paragraph_forward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_paragraph_forward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_paragraph_forward();
        }
        self.text.move_paragraph_forward()
    }

    fn move_paragraph_backward(&mut self) -> bool {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            return pane.text.move_paragraph_backward();
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            return pane.text.move_paragraph_backward();
        }
        self.text.move_paragraph_backward()
    }

    pub(crate) fn set_cursor(&mut self, point: TextPoint) {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            pane.set_cursor(point);
            return;
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            pane.set_cursor(point);
            return;
        }
        self.text.set_cursor(point);
    }

    fn point_after(&self, point: TextPoint) -> Option<TextPoint> {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.text.point_after(point);
        }
        self.acp_active_pane_state()
            .map(|pane| pane.text.point_after(point))
            .unwrap_or_else(|| self.text.point_after(point))
    }

    fn move_line_start(&mut self) -> bool {
        let before = self.cursor_point();
        self.set_cursor(editor_buffer::TextPoint::new(self.cursor_row(), 0));
        self.cursor_point() != before
    }

    fn move_line_first_non_blank(&mut self) -> bool {
        let before = self.cursor_point();
        let row = self.cursor_row();
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(row) {
                pane.text.set_cursor(point);
            }
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(row) {
                pane.text.set_cursor(point);
            }
        } else if let Some(point) = self.text.first_non_blank_in_line(row) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn move_line_end(&mut self) -> bool {
        let before = self.cursor_point();
        let line = self.cursor_row();
        let column = self.line_len_chars(line).saturating_sub(1);
        self.set_cursor(editor_buffer::TextPoint::new(line, column));
        self.cursor_point() != before
    }

    fn goto_first_line(&mut self) -> bool {
        let before = self.cursor_point();
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(0) {
                pane.text.set_cursor(point);
            }
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(0) {
                pane.text.set_cursor(point);
            }
        } else if let Some(point) = self.text.first_non_blank_in_line(0) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn goto_last_line(&mut self) -> bool {
        let before = self.cursor_point();
        let line = self.line_count().saturating_sub(1);
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(line) {
                pane.text.set_cursor(point);
            }
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            if let Some(point) = pane.text.first_non_blank_in_line(line) {
                pane.text.set_cursor(point);
            }
        } else if let Some(point) = self.text.first_non_blank_in_line(line) {
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn goto_line(&mut self, line_index: usize) -> bool {
        let before = self.cursor_point();
        let line = line_index.min(self.line_count().saturating_sub(1));
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            let point = pane
                .text
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            pane.text.set_cursor(point);
        } else if let Some(pane) = self.acp_active_pane_state_mut() {
            let point = pane
                .text
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            pane.text.set_cursor(point);
        } else {
            let point = self
                .text
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            self.text.set_cursor(point);
        }
        self.cursor_point() != before
    }

    fn append_after_cursor(&mut self) {
        let line = self.cursor_row();
        let column = self
            .text
            .line_len_chars(line)
            .map(|line_len| {
                if self.cursor_col() < line_len {
                    self.cursor_col() + 1
                } else {
                    line_len
                }
            })
            .unwrap_or(self.cursor_col());
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
    }

    fn append_line_end(&mut self) {
        let line = self.cursor_row();
        let column = self.text.line_len_chars(line).unwrap_or(0);
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
    }

    fn insert_line_start(&mut self) {
        if let Some(point) = self.text.first_non_blank_in_line(self.cursor_row()) {
            self.text.set_cursor(point);
        }
    }

    fn open_line_below(&mut self) {
        let line = self.cursor_row();
        let column = self.text.line_len_chars(line).unwrap_or(0);
        self.text
            .set_cursor(editor_buffer::TextPoint::new(line, column));
        let had_wrap_cache = self.wrap_cache.is_some();
        let wrap_splice = self.prepare_wrap_cache_line_splice(line, 1);
        self.preserve_root_cursor_before_text_change();
        self.text.insert_newline();
        self.finish_wrap_cache_line_splice(had_wrap_cache, wrap_splice);
    }

    fn open_line_above(&mut self) {
        let line = self.cursor_row();
        self.text.set_cursor(editor_buffer::TextPoint::new(line, 0));
        let had_wrap_cache = self.wrap_cache.is_some();
        let wrap_splice = self.prepare_wrap_cache_line_splice(line, 1);
        self.preserve_root_cursor_before_text_change();
        self.text.insert_newline();
        let _ = self.text.move_up();
        self.finish_wrap_cache_line_splice(had_wrap_cache, wrap_splice);
    }

    fn undo(&mut self) {
        let _ = self.undo_tree_undo();
    }

    fn redo(&mut self) {
        let _ = self.undo_tree_redo();
    }

    fn record_undo_snapshot(&mut self) {
        let _ = self.undo_tree.record_snapshot(&self.text);
    }

    fn undo_tree_undo(&mut self) -> bool {
        let Some(snapshot) = self.undo_tree.undo() else {
            return false;
        };
        self.apply_undo_snapshot(&snapshot);
        true
    }

    fn undo_tree_redo(&mut self) -> bool {
        let Some(snapshot) = self
            .undo_tree
            .redo(self.text.cursor(), self.text.revision())
        else {
            return false;
        };
        self.apply_undo_snapshot(&snapshot);
        true
    }

    fn undo_tree_select(&mut self, node_id: usize) -> bool {
        let Some(snapshot) =
            self.undo_tree
                .select(node_id, self.text.cursor(), self.text.revision())
        else {
            return false;
        };
        self.apply_undo_snapshot(&snapshot);
        true
    }

    fn undo_tree_entries(&self) -> (Vec<UndoTreeEntry>, usize) {
        self.undo_tree.picker_entries()
    }

    fn delete_range(&mut self, range: TextRange) {
        self.clear_inline_completion();
        let start_line = range.start().line.min(range.end().line);
        let end_line = range.start().line.max(range.end().line);
        let old_span = end_line.saturating_sub(start_line).saturating_add(1);
        let single_line = start_line == end_line;
        let wrap_edit = single_line
            .then(|| self.prepare_wrap_cache_inline_edit(start_line))
            .flatten();
        let had_wrap_cache = !single_line && self.wrap_cache.is_some();
        let wrap_splice = (!single_line)
            .then(|| self.prepare_wrap_cache_line_splice(start_line, old_span))
            .flatten();
        self.preserve_root_cursor_before_text_change();
        self.text.delete(range);
        if single_line {
            self.apply_wrap_cache_inline_edit(wrap_edit);
        } else {
            self.finish_wrap_cache_line_splice(had_wrap_cache, wrap_splice);
        }
    }

    fn replace_range(&mut self, range: TextRange, text: &str) {
        self.clear_inline_completion();
        let start_line = range.start().line.min(range.end().line);
        let end_line = range.start().line.max(range.end().line);
        let old_span = end_line.saturating_sub(start_line).saturating_add(1);
        let single_line = start_line == end_line && !text.contains('\n');
        let wrap_edit = single_line
            .then(|| self.prepare_wrap_cache_inline_edit(start_line))
            .flatten();
        let had_wrap_cache = !single_line && self.wrap_cache.is_some();
        let wrap_splice = (!single_line)
            .then(|| self.prepare_wrap_cache_line_splice(start_line, old_span))
            .flatten();
        self.preserve_root_cursor_before_text_change();
        self.text.replace(range, text);
        if single_line {
            self.apply_wrap_cache_inline_edit(wrap_edit);
        } else {
            self.finish_wrap_cache_line_splice(had_wrap_cache, wrap_splice);
        }
    }

    fn replace_chars_at_cursor(&mut self, character: char, count: usize) -> bool {
        let original = self.cursor_point();
        let mut replaced = false;
        let mut point = original;
        for _ in 0..count.max(1) {
            let Some(next) = self.point_after(point) else {
                break;
            };
            if self.slice(TextRange::new(point, next)) == "\n" {
                break;
            }
            self.replace_range(TextRange::new(point, next), &character.to_string());
            replaced = true;
            point = next;
        }
        self.set_cursor(original);
        replaced
    }

    fn slice(&self, range: TextRange) -> String {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.text.slice(range);
        }
        self.acp_active_pane_state()
            .map(|pane| pane.text.slice(range))
            .unwrap_or_else(|| self.text.slice(range))
    }

    pub(crate) fn line_range(&self, line_index: usize) -> Option<TextRange> {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.text.line_range(line_index);
        }
        self.acp_active_pane_state()
            .and_then(|pane| pane.text.line_range(line_index))
            .or_else(|| self.text.line_range(line_index))
    }

    pub(crate) fn line_span_range(&self, start_line: usize, count: usize) -> Option<TextRange> {
        if self.line_count() == 0 || count == 0 {
            return None;
        }

        let start_line = start_line.min(self.line_count().saturating_sub(1));
        let end_line =
            (start_line + count.saturating_sub(1)).min(self.line_count().saturating_sub(1));
        Some(TextRange::new(
            self.line_range(start_line)?.start(),
            self.line_range(end_line)?.end(),
        ))
    }

    fn full_range(&self) -> TextRange {
        if self.line_count() == 0 {
            return TextRange::new(TextPoint::default(), TextPoint::default());
        }
        let start = self.line_range(0).map(TextRange::start).unwrap_or_default();
        let end = self
            .line_range(self.line_count().saturating_sub(1))
            .map(TextRange::end)
            .unwrap_or(start);
        TextRange::new(start, end)
    }

    fn apply_undo_snapshot(&mut self, snapshot: &UndoSnapshot) {
        let range = self.full_range();
        self.text.replace(range, &snapshot.text.text());
        self.invalidate_wrap_cache();
        self.set_cursor(snapshot.cursor());
        self.undo_tree.update_revision(self.text.revision());
    }

    fn text_object_range(
        &self,
        kind: VimTextObjectKind,
        around: bool,
        count: usize,
    ) -> Option<TextRange> {
        match kind {
            VimTextObjectKind::Word => self.text.word_range_at(self.cursor_point(), around, count),
            VimTextObjectKind::BigWord => {
                self.text
                    .word_range_at_kind(self.cursor_point(), WordKind::BigWord, around, count)
            }
            VimTextObjectKind::Sentence => {
                self.text
                    .sentence_range_at(self.cursor_point(), around, count)
            }
            VimTextObjectKind::Paragraph => {
                self.text
                    .paragraph_range_at(self.cursor_point(), around, count)
            }
            VimTextObjectKind::Delimited { open, close } => {
                self.text
                    .delimited_range_at(self.cursor_point(), open, close, around)
            }
            VimTextObjectKind::Tag => {
                self.text
                    .tag_range_at(self.cursor_point(), around, self.language_id())
            }
        }
    }

    fn move_find(&mut self, kind: VimFindKind, target: char, count: usize) -> bool {
        let repeat = count.max(1);
        let mut moved = false;
        for _ in 0..repeat {
            let next = match kind {
                VimFindKind::ForwardTo => {
                    self.text.find_forward_in_line(self.cursor_point(), target)
                }
                VimFindKind::BackwardTo => {
                    self.text.find_backward_in_line(self.cursor_point(), target)
                }
                VimFindKind::ForwardBefore => self
                    .text
                    .find_forward_in_line(self.cursor_point(), target)
                    .and_then(|point| self.text.point_before(point)),
                VimFindKind::BackwardAfter => self
                    .text
                    .find_backward_in_line(self.cursor_point(), target)
                    .and_then(|point| self.text.point_after(point)),
            };
            let Some(next) = next else {
                return moved;
            };
            self.text.set_cursor(next);
            moved = true;
        }
        moved
    }

    fn insert_at(&mut self, point: TextPoint, text: &str) {
        self.text.set_cursor(point);
        let plan = self.plan_wrap_cache_insert(point.line, text.contains('\n'));
        self.preserve_root_cursor_before_text_change();
        self.text.insert_text(text);
        self.commit_wrap_cache_insert_plan(plan);
    }

    fn preserve_root_cursor_before_text_change(&mut self) {
        self.undo_tree
            .preserve_root_cursor(self.text.cursor(), self.text.revision());
    }

    fn scroll_by(&mut self, delta: i32) {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            let max_scroll = pane.max_scroll_row() as i32;
            let next = (pane.scroll_row as i32 + delta).clamp(0, max_scroll);
            pane.scroll_row = next as usize;
            return;
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            let max_scroll = pane.max_scroll_row() as i32;
            let next = (pane.scroll_visual_row as i32 + delta).clamp(0, max_scroll);
            pane.scroll_visual_row = next as usize;
            return;
        }
        let max_scroll = self.max_scroll_row() as i32;
        let next = (self.scroll_row as i32 + delta).clamp(0, max_scroll);
        self.scroll_row = next as usize;
    }

    fn max_scroll_row(&self) -> usize {
        let content_rows = self.content_viewport_lines.max(1);
        if self.line_count() == 0 {
            return 0;
        }
        if !self.line_wrap {
            return self.line_count().saturating_sub(content_rows);
        }
        if let Some(cache) = self.wrap_cache.as_ref()
            && cache.matches(
                self.scroll_wrap_cols,
                self.scroll_indent_size,
                self.line_count(),
            )
        {
            return cache.max_scroll_row(content_rows);
        }
        self.max_scroll_row_for_wrapped_rows(
            content_rows,
            self.scroll_wrap_cols,
            self.scroll_indent_size,
        )
    }

    fn set_scroll_layout(
        &mut self,
        content_viewport_lines: usize,
        wrap_cols: usize,
        indent_size: usize,
    ) {
        self.content_viewport_lines = content_viewport_lines.max(1);
        self.scroll_wrap_cols = wrap_cols.max(1);
        self.scroll_indent_size = indent_size.max(1);
    }

    fn line_visual_row_count(
        &self,
        line_index: usize,
        wrap_cols: usize,
        indent_size: usize,
    ) -> usize {
        if let Some(&rows) = self.pretty_display_rows.get(&line_index) {
            return rows.max(1);
        }
        if self.line_wrap {
            line_wrap_row_count(
                &self.text.line(line_index).unwrap_or_default(),
                wrap_cols,
                indent_size,
            )
        } else {
            1
        }
    }

    fn refresh_pretty_display_rows(
        &mut self,
        user_library: &dyn UserLibrary,
        pane_width_px: u32,
        line_height: i32,
        visual_selection: Option<VisualSelection>,
        input_mode: InputMode,
        visible_rows: usize,
    ) {
        if self.language_id() != Some("markdown") {
            if !self.pretty_display_rows.is_empty() {
                self.pretty_display_rows.clear();
                self.wrap_cache = None;
            }
            return;
        }
        let start = self
            .scroll_row
            .min(self.cursor_row())
            .saturating_sub(visible_rows);
        let end = self
            .cursor_row()
            .max(self.scroll_row)
            .saturating_add(visible_rows.saturating_add(8))
            .min(self.line_count().max(1));
        let paint = markdown_pretty_paint_plan(
            self,
            user_library,
            MarkdownPrettyPaintArgs {
                visible_start: start,
                visible_end: end.max(start.saturating_add(1)),
                visual_selection,
                input_mode,
                pane_width_px,
                line_height,
            },
        );
        let mut rows = BTreeMap::new();
        for (line_index, image) in paint.images {
            rows.insert(line_index, image.rows().max(1));
        }
        if self.pretty_display_rows != rows {
            self.pretty_display_rows = rows;
            self.wrap_cache = None;
        }
    }

    pub(crate) fn set_viewport_lines(&mut self, visible_lines: usize) {
        self.viewport_lines = visible_lines.max(1);
        self.content_viewport_lines = self.viewport_lines;
        if let Some(state) = self.plugin_section_state.as_mut() {
            for pane in &mut state.attached_sections {
                let rows = pane.visible_rows();
                let wrap_cols = pane.wrap_cols();
                pane.set_view_metrics(rows, wrap_cols);
            }
        }
        if let Some(state) = self.acp_state.as_mut() {
            let plan_rows = state.plan_pane.visible_rows();
            let plan_wrap_cols = state.plan_pane.wrap_cols();
            state.plan_pane.set_view_metrics(plan_rows, plan_wrap_cols);
            let output_rows = state.output_pane.visible_rows();
            let output_wrap_cols = state.output_pane.wrap_cols();
            state
                .output_pane
                .set_view_metrics(output_rows, output_wrap_cols);
        }
    }

    fn sync_acp_viewport_metrics(
        &mut self,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
        command_line_visible: bool,
    ) {
        let rect = PixelRectToRect::rect(0, 0, width.max(1), height.max(1));
        let layout = buffer_footer_layout_with_command_line(
            self,
            rect,
            line_height,
            cell_width,
            command_line_visible,
        );
        self.viewport_lines = layout.visible_rows.max(1);
        let Some(acp_layout) = acp_buffer_layout(self, rect, layout, cell_width, line_height)
        else {
            return;
        };
        if let Some(state) = self.acp_state.as_mut() {
            state
                .plan_pane
                .set_view_metrics(acp_layout.plan.visible_rows, acp_layout.plan.wrap_cols);
            state
                .output_pane
                .set_view_metrics(acp_layout.output.visible_rows, acp_layout.output.wrap_cols);
        }
    }

    fn sync_plugin_section_viewport_metrics(
        &mut self,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
        command_line_visible: bool,
    ) {
        let rect = PixelRectToRect::rect(0, 0, width.max(1), height.max(1));
        let layout = buffer_footer_layout_with_command_line(
            self,
            rect,
            line_height,
            cell_width,
            command_line_visible,
        );
        let Some(section_layout) =
            plugin_section_buffer_layout(self, rect, layout, cell_width, line_height)
        else {
            return;
        };
        self.viewport_lines = section_layout
            .panes
            .first()
            .map(|pane| pane.visible_rows)
            .unwrap_or(1)
            .max(1);
        if let Some(state) = self.plugin_section_state.as_mut() {
            for (index, pane_layout) in section_layout.panes.iter().enumerate().skip(1) {
                if let Some(pane) = state.attached_section_mut(index) {
                    pane.set_view_metrics(pane_layout.visible_rows, pane_layout.wrap_cols);
                }
            }
        }
    }

    fn viewport_lines(&self) -> usize {
        if let Some(state) = self.plugin_section_state.as_ref() {
            if state.active_section == 0 {
                return self.viewport_lines.max(1);
            }
            return state
                .active_attached_section()
                .map(PluginTextPaneState::visible_rows)
                .unwrap_or(1);
        }
        match self.acp_active_pane() {
            Some(AcpPane::Plan) => self.acp_plan_viewport_lines(),
            Some(AcpPane::Output) => self.acp_output_viewport_lines(),
            Some(AcpPane::Input | AcpPane::Footer) => self.viewport_lines.max(1),
            None => self.viewport_lines.max(1),
        }
    }

    fn line_at_viewport_offset(&self, offset: usize) -> usize {
        let max_line = self.line_count().saturating_sub(1);
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.line_at_viewport_offset(offset).min(max_line);
        }
        if let Some(pane) = self.acp_active_pane_state() {
            return pane.line_at_viewport_offset(offset).min(max_line);
        }
        self.scroll_row.saturating_add(offset).min(max_line)
    }

    fn cursor_viewport_offset(&self) -> usize {
        if let Some(pane) = self.active_aux_text_pane_state() {
            return pane.cursor_viewport_offset();
        }
        self.acp_active_pane_state()
            .map(AcpPaneState::cursor_viewport_offset)
            .unwrap_or_else(|| self.cursor_row().saturating_sub(self.scroll_row))
    }

    fn move_to_viewport_offset(&mut self, offset: usize) -> bool {
        if self.line_count() == 0 {
            return false;
        }
        if let Some(pane) = self.acp_active_pane_state_mut() {
            let target_visual = pane
                .viewport_scroll_top()
                .saturating_add(offset)
                .min(acp_pane_total_visual_rows(pane).saturating_sub(1));
            let before = pane.cursor();
            pane.set_cursor(acp_pane_point_for_visual_row(pane, target_visual));
            return pane.cursor() != before;
        }
        let target_line = self.line_at_viewport_offset(offset);
        self.goto_line(target_line)
    }

    fn move_to_viewport_middle(&mut self) -> bool {
        let middle = self.viewport_lines().saturating_sub(1) / 2;
        self.move_to_viewport_offset(middle)
    }

    pub(crate) fn max_scroll_row_for_wrapped_rows(
        &self,
        visible_rows: usize,
        wrap_cols: usize,
        indent_size: usize,
    ) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let visible_rows = visible_rows.max(1);
        let mut rows = 0usize;
        for line_index in (0..line_count).rev() {
            let row_count = self.line_visual_row_count(line_index, wrap_cols, indent_size);
            if rows.saturating_add(row_count) > visible_rows {
                return if rows == 0 {
                    line_index
                } else {
                    line_index.saturating_add(1)
                };
            }
            rows = rows.saturating_add(row_count);
        }
        0
    }

    fn scroll_row_for_top_margin(
        &self,
        cursor_row: usize,
        cursor_segment_index: usize,
        min_cursor_row: usize,
        wrap_cols: usize,
        indent_size: usize,
    ) -> usize {
        let mut target = cursor_row;
        let mut offset = cursor_segment_index;
        while target > 0 && offset < min_cursor_row {
            target = target.saturating_sub(1);
            offset =
                offset.saturating_add(self.line_visual_row_count(target, wrap_cols, indent_size));
        }
        target
    }

    fn ensure_visible(
        &mut self,
        visible_rows: usize,
        wrap_cols: usize,
        indent_size: usize,
        reserved_top_rows: usize,
        scrolloff: usize,
    ) {
        if let Some(pane) = self.active_aux_text_pane_state_mut() {
            pane.ensure_cursor_visible();
            return;
        }
        if self.is_acp_buffer() {
            if let Some(pane) = self.acp_active_pane_state_mut() {
                pane.ensure_cursor_visible();
            }
            return;
        }
        let visible_rows = visible_rows.max(1);
        let reserved_top_rows = reserved_top_rows.min(visible_rows.saturating_sub(1));
        let content_rows = visible_rows.saturating_sub(reserved_top_rows).max(1);
        let min_cursor_row = scrolloff.min(content_rows.saturating_sub(1) / 2);
        let max_cursor_row = content_rows
            .saturating_sub(1)
            .saturating_sub(min_cursor_row);
        let cursor_row = self.cursor_row();
        if self.line_count() == 0 {
            self.scroll_row = 0;
            self.scroll_col = 0;
            return;
        }
        if !self.line_wrap {
            let max_scroll_row = self.line_count().saturating_sub(content_rows);
            self.scroll_row = self.scroll_row.min(max_scroll_row);
            if cursor_row < self.scroll_row {
                self.scroll_row = cursor_row;
            } else if cursor_row >= self.scroll_row.saturating_add(content_rows) {
                self.scroll_row = cursor_row
                    .saturating_sub(content_rows.saturating_sub(1))
                    .min(max_scroll_row);
            }
            let visible_cols = wrap_cols.max(1);
            let min_cursor_col = scrolloff.min(visible_cols.saturating_sub(1) / 2);
            let max_cursor_col = visible_cols
                .saturating_sub(1)
                .saturating_sub(min_cursor_col);
            let cursor_display_col = LineCharMap::with_tab_width(
                &self.text.line(cursor_row).unwrap_or_default(),
                indent_size,
            )
            .display_col_at(self.cursor_col());
            if cursor_display_col < self.scroll_col.saturating_add(min_cursor_col) {
                self.scroll_col = cursor_display_col.saturating_sub(min_cursor_col);
            } else {
                let current_offset = cursor_display_col.saturating_sub(self.scroll_col);
                if current_offset > max_cursor_col {
                    self.scroll_col = cursor_display_col.saturating_sub(max_cursor_col);
                }
            }
            return;
        }
        let cursor_col = self.cursor_col();
        let cursor_line = self.text.line(cursor_row).unwrap_or_default();
        let cursor_segments = wrap_line_segments_for_line(&cursor_line, wrap_cols, indent_size);
        let cursor_segment_index = segment_index_for_column(&cursor_segments, cursor_col);

        let line_count = self.line_count();
        let distance = cursor_row.abs_diff(self.scroll_row);
        let threshold = content_rows.saturating_mul(4).max(256);
        self.refresh_wrap_cache(wrap_cols, indent_size, line_count, distance, threshold);
        let max_scroll_row = self
            .wrap_cache
            .as_ref()
            .map(|cache| cache.max_scroll_row(content_rows))
            .unwrap_or_else(|| {
                self.max_scroll_row_for_wrapped_rows(content_rows, wrap_cols, indent_size)
            });
        self.scroll_row = self.scroll_row.min(max_scroll_row);

        if let Some(cache) = self.wrap_cache.as_ref() {
            let base = cache
                .prefix_rows
                .get(cursor_row)
                .copied()
                .unwrap_or(0)
                .saturating_add(cursor_segment_index);
            let top_target = cache
                .prefix_rows
                .partition_point(|&value| value <= base.saturating_sub(min_cursor_row))
                .saturating_sub(1)
                .min(cursor_row)
                .min(max_scroll_row);
            let current_offset =
                base.saturating_sub(cache.prefix_rows.get(self.scroll_row).copied().unwrap_or(0));
            if cursor_row < self.scroll_row || current_offset < min_cursor_row {
                self.scroll_row = top_target;
                return;
            }
            if current_offset <= max_cursor_row {
                return;
            }
            let bottom_target = cache
                .prefix_rows
                .partition_point(|&value| value < base.saturating_sub(max_cursor_row))
                .min(cursor_row)
                .min(max_scroll_row);
            self.scroll_row = bottom_target;
            return;
        }

        if cursor_row < self.scroll_row {
            self.scroll_row = self
                .scroll_row_for_top_margin(
                    cursor_row,
                    cursor_segment_index,
                    min_cursor_row,
                    wrap_cols,
                    indent_size,
                )
                .min(max_scroll_row);
            return;
        }

        let mut row_offset = 0usize;
        let mut row_counts = Vec::with_capacity(distance);
        for line_index in self.scroll_row..cursor_row {
            let row_count = self.line_visual_row_count(line_index, wrap_cols, indent_size);
            row_offset = row_offset.saturating_add(row_count);
            row_counts.push(row_count);
        }
        row_offset = row_offset.saturating_add(cursor_segment_index);
        if row_offset < min_cursor_row {
            self.scroll_row = self
                .scroll_row_for_top_margin(
                    cursor_row,
                    cursor_segment_index,
                    min_cursor_row,
                    wrap_cols,
                    indent_size,
                )
                .min(max_scroll_row);
            return;
        }
        if row_offset <= max_cursor_row {
            return;
        }

        let mut offset = row_offset;
        let mut new_scroll = self.scroll_row;
        for row_count in row_counts {
            if offset <= max_cursor_row || new_scroll >= cursor_row {
                break;
            }
            offset = offset.saturating_sub(row_count);
            new_scroll = new_scroll.saturating_add(1);
        }
        self.scroll_row = new_scroll.min(max_scroll_row);
    }
}
