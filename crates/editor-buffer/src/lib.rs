#![doc = r#"Rope-backed text storage, editing, cursor movement, and line-oriented access."#]

use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    path::{Path, PathBuf},
};

use ropey::{Rope, RopeBuilder, RopeSlice};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Rope-backed text storage, editing, cursor movement, and line-oriented access.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Line and column position within a text buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct TextPoint {
    /// Zero-based line index.
    pub line: usize,
    /// Zero-based character column within the line.
    pub column: usize,
}

impl TextPoint {
    /// Creates a new text point.
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// Half-open range between two text points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextRange {
    start: TextPoint,
    end: TextPoint,
}

impl TextRange {
    /// Creates a new text range.
    pub const fn new(start: TextPoint, end: TextPoint) -> Self {
        Self { start, end }
    }

    /// Returns the range start.
    pub const fn start(self) -> TextPoint {
        self.start
    }

    /// Returns the range end.
    pub const fn end(self) -> TextPoint {
        self.end
    }

    /// Returns the range with start and end sorted.
    pub fn normalized(self) -> Self {
        if self.start <= self.end {
            self
        } else {
            Self {
                start: self.end,
                end: self.start,
            }
        }
    }
}

/// Selection anchored at one point and extended to another.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Selection {
    anchor: TextPoint,
    head: TextPoint,
}

impl Selection {
    /// Creates a caret selection at a single point.
    pub const fn caret(point: TextPoint) -> Self {
        Self {
            anchor: point,
            head: point,
        }
    }

    /// Creates a selection with an explicit anchor and head.
    pub const fn new(anchor: TextPoint, head: TextPoint) -> Self {
        Self { anchor, head }
    }

    /// Returns the anchor position.
    pub const fn anchor(self) -> TextPoint {
        self.anchor
    }

    /// Returns the active head position.
    pub const fn head(self) -> TextPoint {
        self.head
    }

    /// Reports whether the selection is a caret.
    pub fn is_caret(self) -> bool {
        self.anchor == self.head
    }

    /// Returns the normalized selection range.
    pub fn range(self) -> TextRange {
        TextRange::new(self.anchor, self.head).normalized()
    }
}

/// Preferred newline representation when writing buffers to disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LineEnding {
    /// Unix-style newlines.
    #[default]
    Lf,
    /// Windows-style newlines.
    Crlf,
}

impl LineEnding {
    /// Returns the serialized newline sequence.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::Crlf => "\r\n",
        }
    }
}

/// Lightweight statistics for the current buffer state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferStats {
    /// Total line count in the buffer.
    pub lines: usize,
    /// Total character count in the buffer.
    pub chars: usize,
    /// Total byte count in the buffer.
    pub bytes: usize,
    /// Current logical revision identifier.
    pub revision: u64,
    /// Whether the current state differs from the last saved state.
    pub dirty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditRecord {
    start_char: usize,
    removed_text: String,
    inserted_text: String,
    before_cursor: TextPoint,
    after_cursor: TextPoint,
    before_state_id: u64,
    after_state_id: u64,
}

/// Rope-backed editable document for large-file-friendly text operations.
#[derive(Debug, Clone)]
pub struct TextBuffer {
    rope: Rope,
    cursor: TextPoint,
    path: Option<PathBuf>,
    preferred_line_ending: LineEnding,
    undo_stack: Vec<EditRecord>,
    redo_stack: Vec<EditRecord>,
    state_id: u64,
    saved_state_id: u64,
    next_state_id: u64,
}

impl TextBuffer {
    /// Creates a new empty scratch buffer.
    pub fn new() -> Self {
        Self {
            rope: Rope::new(),
            cursor: TextPoint::default(),
            path: None,
            preferred_line_ending: LineEnding::Lf,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            state_id: 0,
            saved_state_id: 0,
            next_state_id: 1,
        }
    }

    /// Creates a buffer from text, normalizing internal storage to `\n`.
    pub fn from_text(text: impl AsRef<str>) -> Self {
        let text = text.as_ref();
        let mut builder = RopeBuilder::new();
        let normalized = normalize_inline_text(text);
        builder.append(&normalized);
        Self::from_rope(builder.finish(), detect_preferred_line_ending(text), None)
    }

    /// Reads a UTF-8 text buffer from a reader using incremental line loading.
    pub fn from_reader<R: Read>(reader: R) -> io::Result<Self> {
        let mut reader = BufReader::new(reader);
        let mut builder = RopeBuilder::new();
        let mut line = String::new();
        let mut saw_crlf = false;

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line)?;
            if bytes_read == 0 {
                break;
            }

            if line.ends_with("\r\n") {
                saw_crlf = true;
                line.truncate(line.len().saturating_sub(2));
                builder.append(&line);
                builder.append("\n");
                continue;
            }

            let normalized = normalize_inline_text(&line);
            builder.append(&normalized);
        }

        let preferred_line_ending = if saw_crlf {
            LineEnding::Crlf
        } else {
            LineEnding::Lf
        };

        Ok(Self::from_rope(
            builder.finish(),
            preferred_line_ending,
            None,
        ))
    }

    /// Loads a UTF-8 text buffer from a file path.
    pub fn load_from_path(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        let file = File::open(path)?;
        let mut buffer = Self::from_reader(file)?;
        buffer.path = Some(path.to_path_buf());
        Ok(buffer)
    }

    /// Returns the backing file path, if the buffer is file-backed.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the preferred newline representation for serialization.
    pub const fn preferred_line_ending(&self) -> LineEnding {
        self.preferred_line_ending
    }

    /// Sets the preferred newline representation for future writes.
    pub fn set_preferred_line_ending(&mut self, line_ending: LineEnding) {
        self.preferred_line_ending = line_ending;
    }

    /// Returns the current logical revision identifier.
    pub const fn revision(&self) -> u64 {
        self.state_id
    }

    /// Returns whether the buffer differs from the last saved state.
    pub const fn is_dirty(&self) -> bool {
        self.state_id != self.saved_state_id
    }

    /// Returns aggregate statistics for the buffer.
    pub fn stats(&self) -> BufferStats {
        BufferStats {
            lines: self.line_count(),
            chars: self.char_count(),
            bytes: self.byte_count(),
            revision: self.revision(),
            dirty: self.is_dirty(),
        }
    }

    /// Returns the total number of logical lines in the buffer.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Returns the total number of characters in the buffer.
    pub fn char_count(&self) -> usize {
        self.rope.len_chars()
    }

    /// Returns the total number of bytes in the buffer.
    pub fn byte_count(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Returns the current cursor position.
    pub const fn cursor(&self) -> TextPoint {
        self.cursor
    }

    /// Returns the character index for a point after clamping it into the buffer.
    pub fn point_to_char_index(&self, point: TextPoint) -> usize {
        self.point_to_char(point)
    }

    /// Returns the point corresponding to a character index.
    pub fn point_from_char_index(&self, char_index: usize) -> TextPoint {
        self.char_to_point(char_index)
    }

    /// Returns the character at a point when it lies within the visible buffer contents.
    pub fn char_at_point(&self, point: TextPoint) -> Option<char> {
        let char_index = self.point_to_char(point);
        if char_index >= self.char_count() {
            return None;
        }

        self.rope.get_char(char_index)
    }

    /// Returns the point immediately before the given point.
    pub fn point_before(&self, point: TextPoint) -> Option<TextPoint> {
        let char_index = self.point_to_char(point);
        (char_index > 0).then(|| self.char_to_point(char_index - 1))
    }

    /// Returns the point immediately after the given point.
    pub fn point_after(&self, point: TextPoint) -> Option<TextPoint> {
        let char_index = self.point_to_char(point);
        (char_index < self.char_count()).then(|| self.char_to_point(char_index + 1))
    }

    /// Moves the cursor to a clamped valid position.
    pub fn set_cursor(&mut self, point: TextPoint) {
        self.cursor = self.clamp_point(point);
    }

    /// Returns the visible character length of a line.
    pub fn line_len_chars(&self, line_index: usize) -> Option<usize> {
        if line_index >= self.line_count() {
            return None;
        }

        Some(self.line_len_chars_impl(line_index))
    }

    /// Returns a single line without its trailing line ending.
    pub fn line(&self, line_index: usize) -> Option<String> {
        if line_index >= self.line_count() {
            return None;
        }

        Some(trimmed_line(self.rope.line(line_index)))
    }

    /// Returns a window of lines without trailing line endings.
    pub fn lines(&self, start_line: usize, max_lines: usize) -> Vec<String> {
        if max_lines == 0 || start_line >= self.line_count() {
            return Vec::new();
        }

        let end = (start_line + max_lines).min(self.line_count());
        (start_line..end)
            .map(|line_index| trimmed_line(self.rope.line(line_index)))
            .collect()
    }

    /// Returns the full normalized text backing the buffer.
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Returns the current contents of a range.
    pub fn slice(&self, range: TextRange) -> String {
        let range = range.normalized();
        let start_char = self.point_to_char(range.start());
        let end_char = self.point_to_char(range.end());
        self.rope.slice(start_char..end_char).to_string()
    }

    /// Returns the first non-blank point on a line.
    pub fn first_non_blank_in_line(&self, line_index: usize) -> Option<TextPoint> {
        let line = self.line(line_index)?;
        let column = line
            .chars()
            .position(|character| !character.is_whitespace())
            .unwrap_or(0);
        Some(TextPoint::new(line_index, column))
    }

    /// Returns the full range covering a logical line, including its trailing newline when present.
    pub fn line_range(&self, line_index: usize) -> Option<TextRange> {
        if line_index >= self.line_count() {
            return None;
        }

        let start_char = self.rope.line_to_char(line_index);
        let end_char = if line_index + 1 < self.line_count() {
            self.rope.line_to_char(line_index + 1)
        } else {
            self.char_count()
        };
        Some(TextRange::new(
            self.char_to_point(start_char),
            self.char_to_point(end_char),
        ))
    }

    /// Replaces a range with new text and records the edit for undo/redo.
    pub fn replace(&mut self, range: TextRange, text: &str) {
        let range = range.normalized();
        let start_char = self.point_to_char(range.start());
        let end_char = self.point_to_char(range.end());
        let inserted_text = normalize_inline_text(text);
        let removed_text = self.rope.slice(start_char..end_char).to_string();
        let before_cursor = self.cursor;
        let before_state_id = self.state_id;
        let after_state_id = self.next_state_id;

        self.next_state_id += 1;
        self.apply_char_edit(start_char, end_char, &inserted_text);
        self.cursor = self.char_to_point(start_char + inserted_text.chars().count());
        self.state_id = after_state_id;
        self.redo_stack.clear();
        self.undo_stack.push(EditRecord {
            start_char,
            removed_text,
            inserted_text,
            before_cursor,
            after_cursor: self.cursor,
            before_state_id,
            after_state_id,
        });
    }

    /// Deletes a range.
    pub fn delete(&mut self, range: TextRange) {
        self.replace(range, "");
    }

    /// Inserts text at the current cursor position.
    pub fn insert_text(&mut self, text: &str) {
        let cursor = self.cursor;
        self.replace(TextRange::new(cursor, cursor), text);
    }

    /// Inserts a normalized newline at the current cursor position.
    pub fn insert_newline(&mut self) {
        self.insert_text("\n");
    }

    /// Deletes the character immediately before the cursor.
    pub fn backspace(&mut self) -> bool {
        let current_char = self.point_to_char(self.cursor);
        if current_char == 0 {
            return false;
        }

        let previous = self.char_to_point(current_char - 1);
        self.replace(TextRange::new(previous, self.cursor), "");
        true
    }

    /// Deletes the character immediately after the cursor.
    pub fn delete_forward(&mut self) -> bool {
        let current_char = self.point_to_char(self.cursor);
        if current_char >= self.char_count() {
            return false;
        }

        let next = self.char_to_point(current_char + 1);
        self.replace(TextRange::new(self.cursor, next), "");
        true
    }

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
        if self.char_count() == 0 {
            return false;
        }

        let original = self.point_to_char(self.cursor);
        if original >= self.char_count() {
            return false;
        }

        let mut char_index = original;
        if is_word_char(self.rope.char(char_index)) {
            while char_index < self.char_count() && is_word_char(self.rope.char(char_index)) {
                char_index += 1;
            }
        }

        while char_index < self.char_count() && !is_word_char(self.rope.char(char_index)) {
            char_index += 1;
        }

        if char_index == original {
            return false;
        }

        self.cursor = self.char_to_point(char_index);
        true
    }

    /// Moves the cursor to the start of the previous word.
    pub fn move_word_backward(&mut self) -> bool {
        if self.char_count() == 0 {
            return false;
        }

        let original = self.point_to_char(self.cursor);
        if original == 0 {
            return false;
        }

        let mut char_index = original.saturating_sub(1);
        while char_index > 0 && !is_word_char(self.rope.char(char_index)) {
            char_index -= 1;
        }
        if !is_word_char(self.rope.char(char_index)) {
            return false;
        }
        while char_index > 0 && is_word_char(self.rope.char(char_index - 1)) {
            char_index -= 1;
        }

        self.cursor = self.char_to_point(char_index);
        true
    }

    /// Moves the cursor to the end of the current or next word.
    pub fn move_word_end_forward(&mut self) -> bool {
        if self.char_count() == 0 {
            return false;
        }

        let original = self.point_to_char(self.cursor);
        if original >= self.char_count() {
            return false;
        }

        let mut char_index = original;
        while char_index < self.char_count() && !is_word_char(self.rope.char(char_index)) {
            char_index += 1;
        }
        if char_index >= self.char_count() {
            return false;
        }
        while char_index + 1 < self.char_count() && is_word_char(self.rope.char(char_index + 1)) {
            char_index += 1;
        }

        self.cursor = self.char_to_point(char_index);
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

    /// Restores the previous edit, if one exists.
    pub fn undo(&mut self) -> bool {
        let Some(record) = self.undo_stack.pop() else {
            return false;
        };

        let inserted_len = record.inserted_text.chars().count();
        self.apply_char_edit(
            record.start_char,
            record.start_char + inserted_len,
            &record.removed_text,
        );
        self.cursor = self.clamp_point(record.before_cursor);
        self.state_id = record.before_state_id;
        self.redo_stack.push(record);
        true
    }

    /// Reapplies the next redo edit, if one exists.
    pub fn redo(&mut self) -> bool {
        let Some(record) = self.redo_stack.pop() else {
            return false;
        };

        let removed_len = record.removed_text.chars().count();
        self.apply_char_edit(
            record.start_char,
            record.start_char + removed_len,
            &record.inserted_text,
        );
        self.cursor = self.clamp_point(record.after_cursor);
        self.state_id = record.after_state_id;
        self.undo_stack.push(record);
        true
    }

    /// Writes the buffer to an arbitrary writer using the preferred line ending.
    pub fn write_to<W: Write>(&self, writer: W) -> io::Result<()> {
        let mut writer = BufWriter::new(writer);

        match self.preferred_line_ending {
            LineEnding::Lf => self.rope.write_to(&mut writer)?,
            LineEnding::Crlf => self.write_crlf(&mut writer)?,
        }

        writer.flush()
    }

    /// Saves the buffer to its existing backing path.
    pub fn save(&mut self) -> io::Result<()> {
        let Some(path) = self.path.clone() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "text buffer has no backing path",
            ));
        };

        self.save_to_path(path)
    }

    /// Saves the buffer to a path and adopts it as the new backing path.
    pub fn save_to_path(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        let file = File::create(path)?;
        self.write_to(file)?;
        self.path = Some(path.to_path_buf());
        self.saved_state_id = self.state_id;
        Ok(())
    }

    fn from_rope(rope: Rope, preferred_line_ending: LineEnding, path: Option<PathBuf>) -> Self {
        Self {
            rope,
            cursor: TextPoint::default(),
            path,
            preferred_line_ending,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            state_id: 0,
            saved_state_id: 0,
            next_state_id: 1,
        }
    }

    fn line_len_chars_impl(&self, line_index: usize) -> usize {
        visible_line_len(self.rope.line(line_index))
    }

    fn clamp_point(&self, point: TextPoint) -> TextPoint {
        let max_line = self.line_count().saturating_sub(1);
        let line = point.line.min(max_line);
        let column = point.column.min(self.line_len_chars_impl(line));
        TextPoint { line, column }
    }

    fn point_to_char(&self, point: TextPoint) -> usize {
        let point = self.clamp_point(point);
        self.rope.line_to_char(point.line) + point.column
    }

    fn char_to_point(&self, char_index: usize) -> TextPoint {
        if self.char_count() == 0 {
            return TextPoint::default();
        }

        let char_index = char_index.min(self.char_count());
        if char_index == self.char_count() {
            let line = self.line_count().saturating_sub(1);
            return TextPoint {
                line,
                column: self.line_len_chars_impl(line),
            };
        }

        let line = self.rope.char_to_line(char_index);
        let column = char_index
            .saturating_sub(self.rope.line_to_char(line))
            .min(self.line_len_chars_impl(line));
        TextPoint { line, column }
    }

    fn apply_char_edit(&mut self, start_char: usize, end_char: usize, text: &str) {
        if start_char < end_char {
            self.rope.remove(start_char..end_char);
        }
        if !text.is_empty() {
            self.rope.insert(start_char, text);
        }
    }

    fn write_crlf<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        for line_index in 0..self.line_count() {
            let mut line = self.rope.line(line_index).to_string();
            let had_newline = line.ends_with('\n');
            if had_newline {
                line.pop();
            }
            writer.write_all(line.as_bytes())?;
            if had_newline {
                writer.write_all(LineEnding::Crlf.as_str().as_bytes())?;
            }
        }

        Ok(())
    }
}

impl Default for TextBuffer {
    fn default() -> Self {
        Self::new()
    }
}

fn detect_preferred_line_ending(text: &str) -> LineEnding {
    if text.contains("\r\n") {
        LineEnding::Crlf
    } else {
        LineEnding::Lf
    }
}

fn normalize_inline_text(text: &str) -> String {
    if !text.contains('\r') {
        return text.to_owned();
    }

    let mut normalized = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(character) = chars.next() {
        if character == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            normalized.push('\n');
        } else {
            normalized.push(character);
        }
    }

    normalized
}

fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn visible_line_len(slice: RopeSlice<'_>) -> usize {
    let len = slice.len_chars();
    if len == 0 {
        return 0;
    }

    match slice.get_char(len - 1) {
        Some('\n') => len - 1,
        _ => len,
    }
}

fn trimmed_line(slice: RopeSlice<'_>) -> String {
    let mut line = slice.to_string();
    if line.ends_with('\n') {
        line.pop();
    }
    line
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write as _, io::Cursor};

    use super::{LineEnding, TextBuffer, TextPoint, TextRange};

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    #[test]
    fn from_reader_normalizes_crlf_and_tracks_line_endings() {
        let input = Cursor::new("alpha\r\nbeta");
        let buffer = must(TextBuffer::from_reader(input));

        assert_eq!(buffer.preferred_line_ending(), LineEnding::Crlf);
        assert_eq!(buffer.text(), "alpha\nbeta");
        assert_eq!(buffer.line_count(), 2);
        assert_eq!(buffer.line(0).as_deref(), Some("alpha"));
        assert_eq!(buffer.line(1).as_deref(), Some("beta"));
    }

    #[test]
    fn replace_insert_and_backspace_update_cursor_and_content() {
        let mut buffer = TextBuffer::from_text("alpha\nbeta");
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_newline();
        buffer.insert_text("z");

        assert_eq!(buffer.text(), "alpha\nz\nbeta");
        assert_eq!(buffer.cursor(), TextPoint::new(1, 1));

        assert!(buffer.backspace());
        assert_eq!(buffer.text(), "alpha\n\nbeta");
        assert_eq!(buffer.cursor(), TextPoint::new(1, 0));

        buffer.replace(
            TextRange::new(TextPoint::new(1, 0), TextPoint::new(2, 0)),
            "inserted\n",
        );
        assert_eq!(buffer.text(), "alpha\ninserted\nbeta");
    }

    #[test]
    fn undo_and_redo_restore_previous_states() {
        let mut buffer = TextBuffer::from_text("hello");
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text(" world");
        assert_eq!(buffer.text(), "hello world");
        assert!(buffer.is_dirty());

        assert!(buffer.undo());
        assert_eq!(buffer.text(), "hello");
        assert!(!buffer.is_dirty());

        assert!(buffer.redo());
        assert_eq!(buffer.text(), "hello world");
        assert!(buffer.is_dirty());
    }

    #[test]
    fn write_to_uses_the_selected_line_ending() {
        let mut buffer = TextBuffer::from_text("alpha\r\nbeta");
        let mut crlf = Vec::new();
        must(buffer.write_to(&mut crlf));
        let crlf = match String::from_utf8(crlf) {
            Ok(text) => text,
            Err(error) => panic!("unexpected utf8 error: {error:?}"),
        };
        assert_eq!(crlf, "alpha\r\nbeta");

        buffer.set_preferred_line_ending(LineEnding::Lf);
        let mut lf = Vec::new();
        must(buffer.write_to(&mut lf));
        let lf = match String::from_utf8(lf) {
            Ok(text) => text,
            Err(error) => panic!("unexpected utf8 error: {error:?}"),
        };
        assert_eq!(lf, "alpha\nbeta");
    }

    #[test]
    fn large_buffers_expose_line_windows_without_full_materialization() {
        let mut source = String::new();
        for index in 0..200_001 {
            if index > 0 {
                source.push('\n');
            }
            let _ = write!(&mut source, "line {index}");
        }

        let buffer = TextBuffer::from_text(&source);
        assert_eq!(buffer.line_count(), 200_001);
        assert_eq!(buffer.line(0).as_deref(), Some("line 0"));
        assert_eq!(buffer.line(200_000).as_deref(), Some("line 200000"));

        let window = buffer.lines(199_998, 3);
        assert_eq!(window, vec!["line 199998", "line 199999", "line 200000"]);
    }

    #[test]
    fn move_word_forward_advances_to_the_next_word() {
        let mut buffer = TextBuffer::from_text("alpha beta\ngamma");

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 6));

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(1, 0));

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(1, 5));

        assert!(!buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(1, 5));
    }

    #[test]
    fn move_word_backward_and_end_cover_word_navigation() {
        let mut buffer = TextBuffer::from_text("alpha beta gamma");
        buffer.set_cursor(TextPoint::new(0, 11));

        assert!(buffer.move_word_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 6));

        assert!(buffer.move_word_end_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 9));
    }

    #[test]
    fn line_ranges_and_char_searches_resolve_expected_points() {
        let buffer = TextBuffer::from_text("alpha beta\ngamma");

        assert_eq!(
            buffer.first_non_blank_in_line(0),
            Some(TextPoint::new(0, 0))
        );
        assert_eq!(
            buffer.line_range(0),
            Some(TextRange::new(TextPoint::new(0, 0), TextPoint::new(1, 0)))
        );
        assert_eq!(
            buffer.find_forward_in_line(TextPoint::new(0, 0), 'b'),
            Some(TextPoint::new(0, 6))
        );
        assert_eq!(
            buffer.find_backward_in_line(TextPoint::new(0, 9), 'b'),
            Some(TextPoint::new(0, 6))
        );
    }
}
