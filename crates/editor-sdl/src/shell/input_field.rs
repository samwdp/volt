#[derive(Debug, Clone)]
struct InputField {
    prompt: String,
    text: String,
    placeholder: Option<String>,
    hint: Option<String>,
    cursor: usize,
    selection_anchor: Option<usize>,
}

impl InputField {
    fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            text: String::new(),
            placeholder: None,
            hint: None,
            cursor: 0,
            selection_anchor: None,
        }
    }

    fn prompt(&self) -> &str {
        &self.prompt
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn placeholder(&self) -> Option<&str> {
        self.placeholder.as_deref()
    }

    fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }

    fn set_placeholder(&mut self, placeholder: Option<String>) {
        self.placeholder = placeholder;
    }

    fn set_hint(&mut self, hint: Option<String>) {
        self.hint = hint;
    }

    fn text_line_count(&self) -> usize {
        self.line_starts().len().max(1)
    }

    fn input_wrap_cols(&self, available_cols: usize) -> (usize, usize) {
        let prompt_len = self.prompt.chars().count();
        let cols = available_cols.saturating_sub(prompt_len).max(1);
        (cols, cols)
    }

    fn wrapped_visual_rows(&self, available_cols: usize) -> Vec<String> {
        let (first_cols, continuation_cols) = self.input_wrap_cols(available_cols);
        let mut rows = Vec::new();
        for line in self.text.split('\n') {
            if line.is_empty() {
                rows.push(String::new());
                continue;
            }
            let char_map = LineCharMap::new(line);
            let segments = wrap_line_segments(&char_map, first_cols, continuation_cols);
            for segment in segments {
                rows.push(char_map.display_text_for_range(
                    line,
                    segment.start_col,
                    segment.end_col,
                ));
            }
        }
        if rows.is_empty() {
            rows.push(String::new());
        }
        rows
    }

    fn visible_wrapped_visual_rows(
        &self,
        available_cols: usize,
        max_rows: usize,
    ) -> (Vec<String>, usize) {
        let rows = self.wrapped_visual_rows(available_cols);
        if max_rows == 0 || rows.len() <= max_rows {
            return (rows, 0);
        }
        let (cursor_row, _) = self.cursor_visual_row_col(available_cols);
        let max_start = rows.len().saturating_sub(max_rows);
        let start_row = cursor_row
            .saturating_add(1)
            .saturating_sub(max_rows)
            .min(max_start);
        let visible_rows = rows.into_iter().skip(start_row).take(max_rows).collect();
        (visible_rows, start_row)
    }

    fn visual_line_count(&self, available_cols: usize) -> usize {
        self.wrapped_visual_rows(available_cols).len().max(1)
    }

    fn cursor_visual_row_col(&self, available_cols: usize) -> (usize, usize) {
        self.visual_row_col_for_cursor(self.cursor_char(), available_cols)
    }

    fn visual_row_col_for_cursor(
        &self,
        cursor_char: usize,
        available_cols: usize,
    ) -> (usize, usize) {
        let (first_cols, continuation_cols) = self.input_wrap_cols(available_cols);
        let (logical_line, col_in_logical_line) = self.line_col_for_char(cursor_char);
        let mut visual_row = 0usize;
        for (idx, line) in self.text.split('\n').enumerate() {
            if idx == logical_line {
                if line.is_empty() {
                    return (visual_row, 0);
                }
                let char_map = LineCharMap::new(line);
                let segments = wrap_line_segments(&char_map, first_cols, continuation_cols);
                let segment_index = segment_index_for_column(&segments, col_in_logical_line);
                let segment = segments
                    .get(segment_index)
                    .unwrap_or_else(|| segments.first().expect("line has wrap segments"));
                let col_in_wrap_row = char_map.display_cols_between(
                    segment.start_col,
                    col_in_logical_line.min(segment.end_col),
                );
                return (visual_row.saturating_add(segment_index), col_in_wrap_row);
            }
            if line.is_empty() {
                visual_row = visual_row.saturating_add(1);
            } else {
                let char_map = LineCharMap::new(line);
                visual_row = visual_row.saturating_add(
                    wrap_line_segments(&char_map, first_cols, continuation_cols).len(),
                );
            }
        }
        (visual_row, 0)
    }

    fn line_col_for_char(&self, cursor_char: usize) -> (usize, usize) {
        let mut consumed = 0usize;
        for (line_index, line) in self.text.split('\n').enumerate() {
            let line_len = line.chars().count();
            if cursor_char <= consumed + line_len {
                return (line_index, cursor_char.saturating_sub(consumed));
            }
            consumed = consumed.saturating_add(line_len + 1);
        }
        self.cursor_line_col()
    }

    fn append_text(&mut self, text: &str) {
        self.insert_text(text);
    }

    fn set_text(&mut self, text: &str) {
        let filtered: String = text
            .chars()
            .filter(|character| *character != '\r')
            .collect();
        self.text.clear();
        self.text.push_str(&filtered);
        self.cursor = self.text.chars().count();
        self.selection_anchor = None;
    }

    fn backspace(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor == 0 {
            return false;
        }
        let start = self.cursor.saturating_sub(1);
        let end = self.cursor;
        self.delete_range(start, end);
        self.cursor = start;
        true
    }

    fn delete_forward(&mut self) -> bool {
        if self.delete_selection() {
            return true;
        }
        if self.cursor >= self.text.chars().count() {
            return false;
        }
        let end = self.cursor.saturating_add(1);
        self.delete_range(self.cursor, end);
        true
    }

    fn move_left(&mut self) -> bool {
        self.selection_anchor = None;
        if self.cursor == 0 {
            return false;
        }
        self.cursor = self.cursor.saturating_sub(1);
        true
    }

    fn move_right(&mut self) -> bool {
        self.selection_anchor = None;
        let total = self.text.chars().count();
        if self.cursor >= total {
            return false;
        }
        self.cursor = (self.cursor + 1).min(total);
        true
    }

    fn move_up(&mut self) -> bool {
        self.selection_anchor = None;
        let starts = self.line_starts();
        let total = self.text.chars().count();
        let (line, col) = self.cursor_line_col_with_starts(&starts);
        if line == 0 {
            return false;
        }
        let prev_line = line.saturating_sub(1);
        let prev_len = Self::line_len_for(&starts, total, prev_line);
        let new_col = col.min(prev_len);
        self.cursor = starts[prev_line] + new_col;
        true
    }

    fn move_down(&mut self) -> bool {
        self.selection_anchor = None;
        let starts = self.line_starts();
        let total = self.text.chars().count();
        let (line, col) = self.cursor_line_col_with_starts(&starts);
        let next_line = line.saturating_add(1);
        if next_line >= starts.len() {
            return false;
        }
        let next_len = Self::line_len_for(&starts, total, next_line);
        let new_col = col.min(next_len);
        self.cursor = starts[next_line] + new_col;
        true
    }

    fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.selection_anchor = None;
    }

    fn insert_text(&mut self, text: &str) {
        let filtered: String = text
            .chars()
            .filter(|character| *character != '\r')
            .collect();
        if filtered.is_empty() {
            return;
        }
        let _ = self.delete_selection();
        let insert_at = self.byte_index_for_char(self.cursor);
        self.text.insert_str(insert_at, &filtered);
        self.cursor = self.cursor.saturating_add(filtered.chars().count());
    }

    fn char_count(&self) -> usize {
        self.text.chars().count()
    }

    fn cursor_char(&self) -> usize {
        self.cursor.min(self.char_count())
    }

    fn cursor_point(&self) -> TextPoint {
        let buffer = TextBuffer::from_text(&self.text);
        buffer.point_from_char_index(self.cursor_char())
    }

    fn move_line_start(&mut self) -> bool {
        self.selection_anchor = None;
        let before = self.cursor_char();
        let (line, _) = self.cursor_line_col();
        let starts = self.line_starts();
        self.cursor = starts.get(line).copied().unwrap_or(0);
        self.cursor != before
    }

    fn move_line_end(&mut self) -> bool {
        self.selection_anchor = None;
        let before = self.cursor_char();
        let starts = self.line_starts();
        let total = self.char_count();
        let (line, _) = self.cursor_line_col_with_starts(&starts);
        self.cursor = starts[line] + Self::line_len_for(&starts, total, line);
        self.cursor != before
    }

    fn start_selection(&mut self) {
        self.selection_anchor = Some(self.cursor_char());
    }

    fn clear_selection(&mut self) {
        self.selection_anchor = None;
    }

    fn selected_char_range(&self, kind: VisualSelectionKind) -> Option<(usize, usize)> {
        let anchor = self.selection_anchor?;
        let total = self.char_count();
        if total == 0 {
            return None;
        }
        match kind {
            VisualSelectionKind::Character => {
                let head = self.cursor_char().min(total.saturating_sub(1));
                if head >= anchor {
                    Some((anchor, (head + 1).min(total)))
                } else {
                    Some((head, (anchor + 1).min(total)))
                }
            }
            VisualSelectionKind::Line => {
                let anchor_point = {
                    let buffer = TextBuffer::from_text(&self.text);
                    buffer.point_from_char_index(anchor.min(total))
                };
                let head_point = {
                    let buffer = TextBuffer::from_text(&self.text);
                    buffer.point_from_char_index(self.cursor_char().min(total))
                };
                let starts = self.line_starts();
                let start_line = anchor_point.line.min(head_point.line);
                let end_line = anchor_point.line.max(head_point.line);
                let start = starts.get(start_line).copied().unwrap_or(0);
                let end = if end_line + 1 < starts.len() {
                    starts[end_line + 1]
                } else {
                    total
                };
                Some((start, end))
            }
            VisualSelectionKind::Block => None,
        }
    }

    fn selected_text(&self, kind: VisualSelectionKind) -> Option<String> {
        let (start, end) = self.selected_char_range(kind)?;
        let start_byte = self.byte_index_for_char(start);
        let end_byte = self.byte_index_for_char(end);
        (start_byte < end_byte).then(|| self.text[start_byte..end_byte].to_owned())
    }

    fn delete_selection(&mut self) -> bool {
        self.delete_selection_kind(VisualSelectionKind::Character)
    }

    fn delete_selection_kind(&mut self, kind: VisualSelectionKind) -> bool {
        let Some((start, end)) = self.selected_char_range(kind) else {
            return false;
        };
        self.delete_char_range(start, end)
    }

    fn set_cursor_char(&mut self, cursor: usize) {
        self.cursor = cursor.min(self.char_count());
        self.selection_anchor = None;
    }

    fn slice_char_range(&self, start: usize, end: usize) -> String {
        let start = start.min(self.char_count());
        let end = end.min(self.char_count());
        let start_byte = self.byte_index_for_char(start);
        let end_byte = self.byte_index_for_char(end);
        if start_byte >= end_byte {
            return String::new();
        }
        self.text[start_byte..end_byte].to_owned()
    }

    fn delete_char_range(&mut self, start: usize, end: usize) -> bool {
        let start = start.min(self.char_count());
        let end = end.min(self.char_count());
        if start >= end {
            return false;
        }
        self.delete_range(start, end);
        self.cursor = start.min(self.char_count());
        self.selection_anchor = None;
        true
    }

    fn replace_char_range(&mut self, start: usize, end: usize, text: &str) -> bool {
        let filtered: String = text
            .chars()
            .filter(|character| *character != '\r')
            .collect();
        let start = start.min(self.char_count());
        let end = end.min(self.char_count());
        if start > end {
            return false;
        }
        self.delete_range(start, end);
        let insert_at = self.byte_index_for_char(start);
        self.text.insert_str(insert_at, &filtered);
        self.cursor = start.saturating_add(filtered.chars().count());
        self.selection_anchor = None;
        true
    }

    fn text_buffer(&self) -> TextBuffer {
        let mut buffer = TextBuffer::from_text(&self.text);
        buffer.set_cursor(self.cursor_point());
        buffer
    }

    fn line_range_chars(&self, line_index: usize) -> Option<(usize, usize)> {
        let buffer = self.text_buffer();
        let range = buffer.line_range(line_index)?;
        Some((
            buffer.point_to_char_index(range.start()),
            buffer.point_to_char_index(range.end()),
        ))
    }

    fn line_span_range_chars(
        &self,
        start_line: usize,
        line_count: usize,
    ) -> Option<(usize, usize)> {
        let buffer = self.text_buffer();
        if buffer.line_count() == 0 {
            return None;
        }
        let end_line = start_line
            .saturating_add(line_count.max(1).saturating_sub(1))
            .min(buffer.line_count().saturating_sub(1));
        let start = buffer.line_range(start_line)?.start();
        let end = buffer.line_range(end_line)?.end();
        Some((
            buffer.point_to_char_index(start),
            buffer.point_to_char_index(end),
        ))
    }

    fn text_object_range_chars(
        &self,
        kind: VimTextObjectKind,
        around: bool,
        count: usize,
    ) -> Option<(usize, usize)> {
        let buffer = self.text_buffer();
        let range = match kind {
            VimTextObjectKind::Word => buffer.word_range_at(buffer.cursor(), around, count),
            VimTextObjectKind::BigWord => {
                buffer.word_range_at_kind(buffer.cursor(), WordKind::BigWord, around, count)
            }
            VimTextObjectKind::Sentence => buffer.sentence_range_at(buffer.cursor(), around, count),
            VimTextObjectKind::Paragraph => {
                buffer.paragraph_range_at(buffer.cursor(), around, count)
            }
            VimTextObjectKind::Delimited { open, close } => {
                buffer.delimited_range_at(buffer.cursor(), open, close, around)
            }
            VimTextObjectKind::Tag => buffer.tag_range_at(buffer.cursor(), around, None),
        }?;
        Some((
            buffer.point_to_char_index(range.start()),
            buffer.point_to_char_index(range.end()),
        ))
    }

    fn open_line_below(&mut self) -> bool {
        let buffer = self.text_buffer();
        let line = buffer.cursor().line;
        let insertion = buffer
            .line_range(line)
            .map(|range| buffer.point_to_char_index(range.end()))
            .unwrap_or_else(|| self.char_count());
        self.replace_char_range(insertion, insertion, "\n");
        self.cursor = insertion.saturating_add(1).min(self.char_count());
        self.selection_anchor = None;
        true
    }

    fn open_line_above(&mut self) -> bool {
        let buffer = self.text_buffer();
        let line = buffer.cursor().line;
        let insertion = buffer
            .line_range(line)
            .map(|range| buffer.point_to_char_index(range.start()))
            .unwrap_or(0);
        self.replace_char_range(insertion, insertion, "\n");
        self.cursor = insertion.min(self.char_count());
        self.selection_anchor = None;
        true
    }

    fn replace_chars_at_cursor(&mut self, character: char, count: usize) -> bool {
        let start = self.cursor_char();
        let mut end = start;
        for _ in 0..count.max(1) {
            let Some(current) = self.text.chars().nth(end) else {
                break;
            };
            if current == '\n' {
                break;
            }
            end = end.saturating_add(1);
        }
        if start == end {
            return false;
        }
        let replaced = self.replace_char_range(start, end, &character.to_string());
        self.cursor = start.min(self.char_count());
        replaced
    }

    fn selection_visual_ranges(
        &self,
        kind: VisualSelectionKind,
        available_cols: usize,
    ) -> Vec<(usize, usize, usize)> {
        let Some((start, end)) = self.selected_char_range(kind) else {
            return Vec::new();
        };
        let (first_cols, continuation_cols) = self.input_wrap_cols(available_cols);
        let starts = self.line_starts();
        let total = self.char_count();
        let mut visual_row_offsets = Vec::with_capacity(starts.len());
        let mut visual_row = 0usize;
        for line in self.text.split('\n') {
            visual_row_offsets.push(visual_row);
            if line.is_empty() {
                visual_row = visual_row.saturating_add(1);
            } else {
                let char_map = LineCharMap::new(line);
                visual_row = visual_row.saturating_add(
                    wrap_line_segments(&char_map, first_cols, continuation_cols).len(),
                );
            }
        }
        let mut ranges = Vec::new();
        for (line_index, line_start) in starts.iter().copied().enumerate() {
            let line_len = Self::line_len_for(&starts, total, line_index);
            let line_end = line_start + line_len;
            let line_selection_start = start.max(line_start);
            let line_selection_end = end.min(line_end);
            if line_selection_start >= line_selection_end {
                continue;
            }
            let start_col = line_selection_start - line_start;
            let end_col = line_selection_end - line_start;
            let line_text = self
                .text
                .chars()
                .skip(line_start)
                .take(line_len)
                .collect::<String>();
            let char_map = LineCharMap::new(&line_text);
            let segments = wrap_line_segments(&char_map, first_cols, continuation_cols);
            for (segment_index, segment) in segments.iter().enumerate() {
                let selection_start = start_col.max(segment.start_col);
                let selection_end = end_col.min(segment.end_col);
                if selection_start < selection_end {
                    ranges.push((
                        visual_row_offsets[line_index] + segment_index,
                        char_map.display_cols_between(segment.start_col, selection_start),
                        char_map.display_cols_between(segment.start_col, selection_end),
                    ));
                }
            }
        }
        ranges
    }

    fn cursor_line_col(&self) -> (usize, usize) {
        let starts = self.line_starts();
        self.cursor_line_col_with_starts(&starts)
    }

    fn cursor_line_col_with_starts(&self, starts: &[usize]) -> (usize, usize) {
        let line = starts
            .iter()
            .rposition(|start| *start <= self.cursor)
            .unwrap_or(0);
        let col = self.cursor.saturating_sub(starts[line]);
        (line, col)
    }

    fn line_starts(&self) -> Vec<usize> {
        let mut starts = vec![0];
        for (index, character) in self.text.chars().enumerate() {
            if character == '\n' {
                starts.push(index.saturating_add(1));
            }
        }
        starts
    }

    fn line_len_for(starts: &[usize], total: usize, line: usize) -> usize {
        let start = starts.get(line).copied().unwrap_or(0);
        let end = starts
            .get(line.saturating_add(1))
            .copied()
            .map(|next| next.saturating_sub(1))
            .unwrap_or(total);
        end.saturating_sub(start)
    }

    fn byte_index_for_char(&self, char_index: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_index)
            .map(|(index, _)| index)
            .unwrap_or(self.text.len())
    }

    fn delete_range(&mut self, start: usize, end: usize) {
        let start_byte = self.byte_index_for_char(start);
        let end_byte = self.byte_index_for_char(end);
        if start_byte < end_byte {
            self.text.replace_range(start_byte..end_byte, "");
        }
    }
}

#[derive(Debug, Clone)]
struct SectionLineMeta {
    section_id: String,
    kind: SectionRenderLineKind,
    action: Option<SectionAction>,
}

#[derive(Debug, Clone, Default)]
struct SectionedBufferState {
    collapsed: SectionCollapseState,
    lines: Vec<SectionLineMeta>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BufferContextOverlayCacheKey {
    buffer_revision: u64,
    buffer_name: String,
    language_id: Option<String>,
    viewport_top_line: usize,
    cursor_line: usize,
    cursor_column: usize,
}

#[derive(Debug)]
struct BufferContextOverlaySnapshot {
    key: BufferContextOverlayCacheKey,
    headerline_lines: Vec<String>,
    ghost_text_by_line: BTreeMap<usize, String>,
}

#[derive(Debug, Clone)]
struct InlineCompletionState {
    item: LspInlineCompletionItem,
    buffer_revision: u64,
    cursor: TextPoint,
    shown: bool,
}
