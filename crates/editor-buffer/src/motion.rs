use crate::buffer::*;
use crate::geometry::*;
use crate::objects::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WordMotionClass {
    Whitespace,
    Word,
    Punctuation,
}

impl TextBuffer {
    /// Moves the cursor left by one visible character.
    pub fn move_left(&mut self) -> bool {
        if self.cursor.column > 0 {
            self.cursor.column -= 1;
            return true;
        }

        if self.cursor.line == 0 {
            return false;
        }

        self.cursor.line -= 1;
        self.cursor.column = self.line_len_chars_impl(self.cursor.line);
        true
    }

    /// Moves the cursor right by one visible character.
    pub fn move_right(&mut self) -> bool {
        let line_len = self.line_len_chars_impl(self.cursor.line);
        if self.cursor.column < line_len {
            self.cursor.column += 1;
            return true;
        }

        if self.cursor.line + 1 >= self.line_count() {
            return false;
        }

        self.cursor.line += 1;
        self.cursor.column = 0;
        true
    }

    /// Moves the cursor up one line, clamping the target column.
    pub fn move_up(&mut self) -> bool {
        if self.cursor.line == 0 {
            return false;
        }

        self.cursor.line -= 1;
        self.cursor.column = self
            .cursor
            .column
            .min(self.line_len_chars_impl(self.cursor.line));
        true
    }

    /// Moves the cursor down one line, clamping the target column.
    pub fn move_down(&mut self) -> bool {
        if self.cursor.line + 1 >= self.line_count() {
            return false;
        }

        self.cursor.line += 1;
        self.cursor.column = self
            .cursor
            .column
            .min(self.line_len_chars_impl(self.cursor.line));
        true
    }

    /// Moves the cursor to the start of the next word.
    pub fn move_word_forward(&mut self) -> bool {
        self.move_object_forward(WordKind::Word)
    }

    /// Moves the cursor to the start of the next Vim `WORD`.
    pub fn move_big_word_forward(&mut self) -> bool {
        self.move_object_forward(WordKind::BigWord)
    }

    /// Moves the cursor to the start of the previous word.
    pub fn move_word_backward(&mut self) -> bool {
        self.move_object_backward(WordKind::Word)
    }

    /// Moves the cursor to the start of the previous Vim `WORD`.
    pub fn move_big_word_backward(&mut self) -> bool {
        self.move_object_backward(WordKind::BigWord)
    }

    /// Moves the cursor to the end of the current or next word.
    pub fn move_word_end_forward(&mut self) -> bool {
        self.move_object_end_forward(WordKind::Word)
    }

    /// Moves the cursor to the end of the current or next Vim `WORD`.
    pub fn move_big_word_end_forward(&mut self) -> bool {
        self.move_object_end_forward(WordKind::BigWord)
    }

    /// Moves the cursor backward to the end of the previous word.
    pub fn move_word_end_backward(&mut self) -> bool {
        self.move_object_end_backward(WordKind::Word)
    }

    /// Moves the cursor backward to the end of the previous Vim `WORD`.
    pub fn move_big_word_end_backward(&mut self) -> bool {
        self.move_object_end_backward(WordKind::BigWord)
    }

    /// Moves the cursor to the matching paired delimiter or HTML/XML tag.
    pub fn move_matching_delimiter(&mut self, language_id: Option<&str>) -> bool {
        let markup_tags = language_matches_markup_tags(language_id);
        let Some(origin) = self.match_pair_origin_char(markup_tags) else {
            return false;
        };
        let Some(pair) = self.pair_at_char(origin, markup_tags) else {
            return false;
        };
        let Some(destination) = pair.counterpart.filter(|_| pair.matched) else {
            return false;
        };
        let next = destination.start();
        if next == self.cursor {
            return false;
        }
        self.cursor = next;
        true
    }

    /// Moves the cursor to the start of the next sentence.
    pub fn move_sentence_forward(&mut self) -> bool {
        let ranges = self.collect_sentence_ranges();
        let Some(index) = self.range_index_at(self.cursor, &ranges) else {
            return false;
        };
        let target_index = index.saturating_add(1);
        let Some((target, _)) = ranges.get(target_index).copied() else {
            return false;
        };
        self.cursor = self.char_to_point(target);
        true
    }

    /// Moves the cursor to the start of the current or previous sentence.
    pub fn move_sentence_backward(&mut self) -> bool {
        let ranges = self.collect_sentence_ranges();
        let Some(index) = self.range_index_at(self.cursor, &ranges) else {
            return false;
        };
        let current_start = ranges[index].0;
        let current_point = self.point_to_char(self.cursor);
        let target_index = if current_point > current_start {
            index
        } else {
            index.saturating_sub(1)
        };
        let Some((target, _)) = ranges.get(target_index).copied() else {
            return false;
        };
        self.cursor = self.char_to_point(target);
        true
    }

    /// Moves the cursor to the start of the next paragraph.
    pub fn move_paragraph_forward(&mut self) -> bool {
        if let Some(target) = self.next_paragraph_boundary_after(self.cursor.line) {
            self.cursor = TextPoint::new(target, 0);
            return true;
        }

        let last_line = (0..self.line_count())
            .rfind(|line| !self.line_is_blank(*line))
            .unwrap_or_else(|| self.line_count().saturating_sub(1));
        let target = self
            .first_non_blank_in_line(last_line)
            .unwrap_or(TextPoint::new(last_line, 0));
        if target == self.cursor {
            return false;
        }
        self.cursor = target;
        true
    }

    /// Moves the cursor to the start of the current or previous paragraph.
    pub fn move_paragraph_backward(&mut self) -> bool {
        let search_line = if self.line_is_blank(self.cursor.line) {
            let mut run_start = self.cursor.line;
            while run_start > 0 && self.line_is_blank(run_start - 1) {
                run_start -= 1;
            }
            run_start
        } else {
            self.cursor.line
        };
        let Some(target) = self.previous_paragraph_boundary_before(search_line) else {
            let target = TextPoint::new(0, 0);
            if target == self.cursor {
                return false;
            }
            self.cursor = target;
            return true;
        };
        self.cursor = TextPoint::new(target, 0);
        true
    }

    /// Finds the next occurrence of a character on the current line.
    pub fn find_forward_in_line(&self, from: TextPoint, target: char) -> Option<TextPoint> {
        let from = self.clamp_point(from);
        let line = self.line(from.line)?;
        let start_column = from.column.saturating_add(1);
        let byte_index = line
            .char_indices()
            .nth(start_column)
            .map(|(index, _)| index)
            .unwrap_or(line.len());
        let suffix = line.get(byte_index..)?;
        let column_offset = suffix.chars().position(|character| character == target)?;
        Some(TextPoint::new(from.line, start_column + column_offset))
    }

    /// Finds the previous occurrence of a character on the current line.
    pub fn find_backward_in_line(&self, from: TextPoint, target: char) -> Option<TextPoint> {
        let from = self.clamp_point(from);
        let line = self.line(from.line)?;
        if from.column == 0 {
            return None;
        }

        let characters = line.chars().collect::<Vec<_>>();
        (0..from.column.min(characters.len()))
            .rev()
            .find_map(|column| {
                (characters.get(column) == Some(&target)).then(|| TextPoint::new(from.line, column))
            })
    }

    pub(crate) fn move_object_forward(&mut self, kind: WordKind) -> bool {
        if self.char_count() == 0 {
            return false;
        }

        let original = self.point_to_char(self.cursor);
        if original >= self.char_count() {
            return false;
        }

        let mut char_index = original;
        let current_class = word_motion_class(self.rope.char(char_index), kind);
        if current_class != WordMotionClass::Whitespace {
            while char_index < self.char_count()
                && word_motion_class(self.rope.char(char_index), kind) == current_class
            {
                char_index += 1;
            }
        }

        while char_index < self.char_count()
            && word_motion_class(self.rope.char(char_index), kind) == WordMotionClass::Whitespace
        {
            char_index += 1;
        }

        if char_index == original {
            return false;
        }

        self.cursor = self.char_to_point(char_index);
        true
    }

    pub(crate) fn move_object_backward(&mut self, kind: WordKind) -> bool {
        if self.char_count() == 0 {
            return false;
        }

        let original = self.point_to_char(self.cursor);
        if original == 0 {
            return false;
        }

        let mut char_index = original.saturating_sub(1);
        while char_index > 0
            && word_motion_class(self.rope.char(char_index), kind) == WordMotionClass::Whitespace
        {
            char_index -= 1;
        }
        let current_class = word_motion_class(self.rope.char(char_index), kind);
        if current_class == WordMotionClass::Whitespace {
            return false;
        }
        while char_index > 0
            && word_motion_class(self.rope.char(char_index - 1), kind) == current_class
        {
            char_index -= 1;
        }

        self.cursor = self.char_to_point(char_index);
        true
    }

    pub(crate) fn move_object_end_forward(&mut self, kind: WordKind) -> bool {
        if self.char_count() == 0 {
            return false;
        }

        let original = self.point_to_char(self.cursor);
        if original >= self.char_count() {
            return false;
        }

        let mut char_index = original;
        if matches_word_kind(self.rope.char(char_index), kind) {
            while char_index + 1 < self.char_count()
                && matches_word_kind(self.rope.char(char_index + 1), kind)
            {
                char_index += 1;
            }
            if char_index == original {
                char_index += 1;
            }
        }
        while char_index < self.char_count()
            && is_object_separator(self.rope.char(char_index), kind)
        {
            char_index += 1;
        }
        if char_index >= self.char_count() {
            return false;
        }
        while char_index + 1 < self.char_count()
            && matches_word_kind(self.rope.char(char_index + 1), kind)
        {
            char_index += 1;
        }

        self.cursor = self.char_to_point(char_index);
        true
    }

    pub(crate) fn move_object_end_backward(&mut self, kind: WordKind) -> bool {
        if self.char_count() == 0 {
            return false;
        }

        let original = self
            .point_to_char(self.cursor)
            .min(self.char_count().saturating_sub(1));
        let mut char_index = original;
        if matches_word_kind(self.rope.char(char_index), kind) {
            while char_index > 0 && matches_word_kind(self.rope.char(char_index - 1), kind) {
                char_index -= 1;
            }
            if char_index == 0 {
                return false;
            }
            char_index -= 1;
        } else if char_index == 0 {
            return false;
        } else {
            char_index -= 1;
        }

        while char_index > 0 && !matches_word_kind(self.rope.char(char_index), kind) {
            char_index -= 1;
        }
        if !matches_word_kind(self.rope.char(char_index), kind) {
            return false;
        }
        while char_index + 1 < self.char_count()
            && matches_word_kind(self.rope.char(char_index + 1), kind)
        {
            char_index += 1;
        }

        self.cursor = self.char_to_point(char_index);
        true
    }
}
