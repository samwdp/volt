use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use ropey::{Rope, RopeBuilder, RopeSlice, iter::Chunks as RopeChunks};

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

/// Whether `language_id` should match HTML/XML tags for show-paren and `%`.
pub fn language_matches_markup_tags(language_id: Option<&str>) -> bool {
    matches!(language_id, Some("html" | "xml" | "jsx" | "tsx"))
}

/// Cursor-local delimiter or tag pair used by show-paren highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShowParenMatch {
    /// Delimiter or tag under the cursor.
    pub origin: TextRange,
    /// Matching delimiter or tag, when one exists within the scan limit.
    pub counterpart: Option<TextRange>,
    /// Whether `counterpart` is a true match for `origin`.
    pub matched: bool,
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

/// Distinguishes Vim-style `word` and `WORD` semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordKind {
    /// Alphanumeric and underscore word boundaries.
    Word,
    /// Any non-whitespace run.
    BigWord,
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

/// One logical edit expressed in byte offsets and line/column positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextEdit {
    /// Revision before the edit was applied.
    pub before_revision: u64,
    /// Revision after the edit was applied.
    pub after_revision: u64,
    /// Starting byte offset of the edit.
    pub start_byte: usize,
    /// Exclusive ending byte offset in the old text.
    pub old_end_byte: usize,
    /// Exclusive ending byte offset in the new text.
    pub new_end_byte: usize,
    /// Starting position of the edit in the old text.
    pub start_position: TextPoint,
    /// Exclusive ending position of the replaced range in the old text.
    pub old_end_position: TextPoint,
    /// Exclusive ending position of the inserted range in the new text.
    pub new_end_position: TextPoint,
}

enum TextByteChunkSource<'a> {
    Empty(Option<&'a [u8]>),
    Chunks(RopeChunks<'a>),
}

/// Iterator over UTF-8 byte chunks from a [`TextBuffer`] byte range.
pub struct TextByteChunks<'a> {
    source: TextByteChunkSource<'a>,
}

impl<'a> TextByteChunks<'a> {
    fn empty() -> Self {
        Self {
            source: TextByteChunkSource::Empty(Some(&[])),
        }
    }

    fn from_chunks(chunks: RopeChunks<'a>) -> Self {
        Self {
            source: TextByteChunkSource::Chunks(chunks),
        }
    }
}

impl<'a> Iterator for TextByteChunks<'a> {
    type Item = &'a [u8];

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.source {
            TextByteChunkSource::Empty(chunk) => chunk.take(),
            TextByteChunkSource::Chunks(chunks) => chunks.next().map(str::as_bytes),
        }
    }
}

/// Rope-backed text used by tree-sitter parse and highlight without requiring undo history.
pub trait SyntaxText {
    /// Logical revision identifier for incremental parse sessions.
    fn revision(&self) -> u64;
    /// Total UTF-8 byte count.
    fn byte_count(&self) -> usize;
    /// Number of logical lines.
    fn line_count(&self) -> usize;
    /// One line without its trailing line ending.
    fn line(&self, line_index: usize) -> Option<String>;
    /// Starting byte offset for a line.
    fn line_start_byte(&self, line_index: usize) -> Option<usize>;
    /// UTF-8 chunk containing `byte_index` and that chunk's starting byte offset.
    fn chunk_at_byte(&self, byte_index: usize) -> Option<(&str, usize)>;
    /// UTF-8 chunks covering a byte range.
    fn byte_slice_chunks(&self, byte_range: Range<usize>) -> TextByteChunks<'_>;
    /// Forward edit chain from `revision` to the current state, when contiguous.
    fn edits_since(&self, revision: u64) -> Option<Vec<TextEdit>>;
}

/// Snapshot plus the edit chain needed for incremental highlighting on a worker.
#[derive(Debug, Clone)]
pub struct HighlightDocument {
    snapshot: TextSnapshot,
    revision: u64,
    edits_from: u64,
    edits: Vec<TextEdit>,
}

impl HighlightDocument {
    /// Captures buffer text and, when possible, edits since `from_revision`.
    pub fn from_buffer(buffer: &TextBuffer, from_revision: u64) -> Self {
        let revision = buffer.revision();
        match buffer.edits_since(from_revision) {
            Some(edits) => Self {
                snapshot: buffer.snapshot(),
                revision,
                edits_from: from_revision,
                edits,
            },
            None => Self {
                snapshot: buffer.snapshot(),
                revision,
                edits_from: revision,
                edits: Vec::new(),
            },
        }
    }

    /// Returns the captured snapshot.
    pub const fn snapshot(&self) -> &TextSnapshot {
        &self.snapshot
    }
}

impl SyntaxText for HighlightDocument {
    fn revision(&self) -> u64 {
        self.revision
    }

    fn byte_count(&self) -> usize {
        self.snapshot.byte_count()
    }

    fn line_count(&self) -> usize {
        self.snapshot.line_count()
    }

    fn line(&self, line_index: usize) -> Option<String> {
        self.snapshot.line(line_index)
    }

    fn line_start_byte(&self, line_index: usize) -> Option<usize> {
        self.snapshot.line_start_byte(line_index)
    }

    fn chunk_at_byte(&self, byte_index: usize) -> Option<(&str, usize)> {
        self.snapshot.chunk_at_byte(byte_index)
    }

    fn byte_slice_chunks(&self, byte_range: Range<usize>) -> TextByteChunks<'_> {
        self.snapshot.byte_slice_chunks(byte_range)
    }

    fn edits_since(&self, revision: u64) -> Option<Vec<TextEdit>> {
        if revision == self.revision {
            return Some(Vec::new());
        }
        if revision == self.edits_from {
            return Some(self.edits.clone());
        }
        None
    }
}

/// Lightweight read-only snapshot of a [`TextBuffer`] for background work.
#[derive(Debug, Clone)]
pub struct TextSnapshot {
    rope: Rope,
    cursor: TextPoint,
}

impl TextSnapshot {
    /// Returns the total number of logical lines in the snapshot.
    pub fn line_count(&self) -> usize {
        self.rope.len_lines()
    }

    /// Returns the total number of characters in the snapshot.
    pub fn char_count(&self) -> usize {
        self.rope.len_chars()
    }

    /// Returns the cursor position captured in the snapshot.
    pub const fn cursor(&self) -> TextPoint {
        self.cursor
    }

    /// Returns the character index for a point after clamping it into the snapshot.
    pub fn point_to_char_index(&self, point: TextPoint) -> usize {
        self.point_to_char(point)
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

    /// Returns a single line without its trailing line ending.
    pub fn line(&self, line_index: usize) -> Option<String> {
        if line_index >= self.line_count() {
            return None;
        }

        Some(trimmed_line(self.rope.line(line_index)))
    }

    /// Returns the full normalized text backing the snapshot.
    pub fn text(&self) -> String {
        self.rope.to_string()
    }

    /// Returns the total UTF-8 byte count.
    pub fn byte_count(&self) -> usize {
        self.rope.len_bytes()
    }

    /// Returns the starting byte offset for a line.
    pub fn line_start_byte(&self, line_index: usize) -> Option<usize> {
        (line_index < self.line_count()).then(|| self.rope.line_to_byte(line_index))
    }

    /// Returns the UTF-8 chunk containing a byte index and the chunk's starting byte offset.
    pub fn chunk_at_byte(&self, byte_index: usize) -> Option<(&str, usize)> {
        (byte_index <= self.byte_count()).then(|| {
            let (chunk, chunk_start_byte, _, _) = self.rope.chunk_at_byte(byte_index);
            (chunk, chunk_start_byte)
        })
    }

    /// Returns an iterator over UTF-8 chunks for a byte range.
    pub fn byte_slice_chunks(&self, byte_range: Range<usize>) -> TextByteChunks<'_> {
        debug_assert!(byte_range.start <= byte_range.end);
        debug_assert!(byte_range.end <= self.byte_count());
        if byte_range.start == byte_range.end {
            return TextByteChunks::empty();
        }
        TextByteChunks::from_chunks(self.rope.byte_slice(byte_range).chunks())
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct EditRecord {
    start_char: usize,
    removed_text: String,
    inserted_text: String,
    edit: TextEdit,
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

    /// Attaches a backing file path without changing the current text contents.
    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.path = Some(path.into());
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

    /// Marks the current buffer state as clean.
    pub fn mark_clean(&mut self) {
        self.saved_state_id = self.state_id;
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

    /// Returns a lightweight read-only snapshot suitable for background work.
    pub fn snapshot(&self) -> TextSnapshot {
        TextSnapshot {
            rope: self.rope.clone(),
            cursor: self.cursor,
        }
    }

    /// Returns the starting byte offset for a line.
    pub fn line_start_byte(&self, line_index: usize) -> Option<usize> {
        (line_index < self.line_count()).then(|| self.rope.line_to_byte(line_index))
    }

    /// Returns the UTF-8 chunk containing a byte index and the chunk's starting byte offset.
    pub fn chunk_at_byte(&self, byte_index: usize) -> Option<(&str, usize)> {
        (byte_index <= self.byte_count()).then(|| {
            let (chunk, chunk_start_byte, _, _) = self.rope.chunk_at_byte(byte_index);
            (chunk, chunk_start_byte)
        })
    }

    /// Returns an iterator over UTF-8 chunks for a byte range.
    pub fn byte_slice_chunks(&self, byte_range: Range<usize>) -> TextByteChunks<'_> {
        assert!(byte_range.start <= byte_range.end);
        assert!(byte_range.end <= self.byte_count());
        if byte_range.start == byte_range.end {
            return TextByteChunks::empty();
        }
        TextByteChunks::from_chunks(self.rope.byte_slice(byte_range).chunks())
    }

    /// Returns the applied edit chain needed to move from `revision` to the current state.
    ///
    /// Returns `None` when the current undo history cannot describe a contiguous forward path.
    pub fn edits_since(&self, revision: u64) -> Option<Vec<TextEdit>> {
        if revision > self.state_id {
            return None;
        }
        if revision == self.state_id {
            return Some(Vec::new());
        }
        let start_index = self
            .undo_stack
            .iter()
            .position(|record| record.before_state_id == revision)?;
        let records = &self.undo_stack[start_index..];
        if records.is_empty()
            || records.first()?.before_state_id != revision
            || records.last()?.after_state_id != self.state_id
            || records
                .windows(2)
                .any(|pair| pair[0].after_state_id != pair[1].before_state_id)
        {
            return None;
        }
        Some(records.iter().map(|record| record.edit).collect())
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

    /// Returns the current word text object range at a point.
    pub fn word_range_at(&self, point: TextPoint, around: bool, count: usize) -> Option<TextRange> {
        self.word_range_at_kind(point, WordKind::Word, around, count)
    }

    /// Returns the current Vim `word` or `WORD` text object range at a point.
    pub fn word_range_at_kind(
        &self,
        point: TextPoint,
        kind: WordKind,
        around: bool,
        count: usize,
    ) -> Option<TextRange> {
        self.object_range_at(point, around, count, |character| {
            matches_word_kind(character, kind)
        })
    }

    /// Returns the delimited text object range around a point.
    pub fn delimited_range_at(
        &self,
        point: TextPoint,
        open: char,
        close: char,
        around: bool,
    ) -> Option<TextRange> {
        if self.char_count() == 0 {
            return None;
        }

        let mut char_index = self.point_to_char(point);
        if char_index >= self.char_count() {
            char_index = self.char_count().saturating_sub(1);
        }

        let (start_char, end_char) = if open == close {
            self.quoted_range_chars(char_index, open)?
        } else {
            self.delimited_range_chars(char_index, open, close)?
        };

        let range = if around {
            TextRange::new(
                self.char_to_point(start_char),
                self.char_to_point(end_char + 1),
            )
        } else {
            TextRange::new(
                self.char_to_point(start_char + 1),
                self.char_to_point(end_char),
            )
        };
        (range.start() <= range.end()).then_some(range)
    }

    /// Returns the current sentence text object range at a point.
    pub fn sentence_range_at(
        &self,
        point: TextPoint,
        around: bool,
        count: usize,
    ) -> Option<TextRange> {
        let sentences = self.collect_sentence_ranges();
        let index = self.range_index_at(point, &sentences)?;
        let start_char = sentences[index].0;
        let end_index = (index + count.max(1).saturating_sub(1)).min(sentences.len() - 1);
        let mut end_char = sentences[end_index].1;
        let mut adjusted_start = start_char;

        if around {
            let mut trailing = end_char;
            while trailing < self.char_count() && self.rope.char(trailing).is_whitespace() {
                trailing += 1;
            }
            if trailing > end_char {
                end_char = trailing;
            } else {
                while adjusted_start > 0 && self.rope.char(adjusted_start - 1).is_whitespace() {
                    adjusted_start -= 1;
                }
            }
        }

        Some(TextRange::new(
            self.char_to_point(adjusted_start),
            self.char_to_point(end_char),
        ))
    }

    /// Returns the current paragraph text object range at a point.
    pub fn paragraph_range_at(
        &self,
        point: TextPoint,
        around: bool,
        count: usize,
    ) -> Option<TextRange> {
        if self.line_count() == 0 || count == 0 {
            return None;
        }

        let mut line_index = point.line.min(self.line_count().saturating_sub(1));
        if self.line_is_blank(line_index) {
            let mut next = line_index;
            while next < self.line_count() && self.line_is_blank(next) {
                next += 1;
            }
            if next < self.line_count() {
                line_index = next;
            } else {
                if line_index == 0 {
                    return None;
                }
                let mut previous = line_index.saturating_sub(1);
                while previous > 0 && self.line_is_blank(previous) {
                    previous -= 1;
                }
                if self.line_is_blank(previous) {
                    return None;
                }
                line_index = previous;
            }
        }

        let mut start_line = line_index;
        while start_line > 0 && !self.line_is_blank(start_line - 1) {
            start_line -= 1;
        }

        let mut end_line = line_index;
        while end_line + 1 < self.line_count() && !self.line_is_blank(end_line + 1) {
            end_line += 1;
        }

        for _ in 1..count {
            let mut next_line = end_line + 1;
            while next_line < self.line_count() && self.line_is_blank(next_line) {
                next_line += 1;
            }
            if next_line >= self.line_count() {
                break;
            }
            end_line = next_line;
            while end_line + 1 < self.line_count() && !self.line_is_blank(end_line + 1) {
                end_line += 1;
            }
        }

        if around {
            let mut included_trailing_blank = false;
            let mut trailing = end_line + 1;
            while trailing < self.line_count() && self.line_is_blank(trailing) {
                end_line = trailing;
                included_trailing_blank = true;
                trailing += 1;
            }
            if !included_trailing_blank {
                while start_line > 0 && self.line_is_blank(start_line - 1) {
                    start_line -= 1;
                }
            }
        }

        Some(TextRange::new(
            self.line_range(start_line)?.start(),
            self.line_range(end_line)?.end(),
        ))
    }

    /// Returns the current HTML/XML tag text object range at a point.
    pub fn tag_range_at(
        &self,
        point: TextPoint,
        around: bool,
        language_id: Option<&str>,
    ) -> Option<TextRange> {
        if !language_matches_markup_tags(language_id) || self.char_count() == 0 {
            return None;
        }

        let chars = self.rope.chars().collect::<Vec<_>>();
        let mut char_index = self.point_to_char(point).min(chars.len().saturating_sub(1));
        if char_index >= chars.len() {
            char_index = chars.len().saturating_sub(1);
        }

        for start in (0..=char_index).rev() {
            let Some(open_tag) = parse_tag_token(&chars, start) else {
                continue;
            };
            if open_tag.is_closing || open_tag.self_closing {
                continue;
            }
            let Some(close_tag) = find_matching_close_tag(&chars, &open_tag) else {
                continue;
            };
            if !(open_tag.start <= char_index && char_index < close_tag.end_exclusive) {
                continue;
            }

            let range = if around {
                TextRange::new(
                    self.char_to_point(open_tag.start),
                    self.char_to_point(close_tag.end_exclusive),
                )
            } else {
                TextRange::new(
                    self.char_to_point(open_tag.end_exclusive),
                    self.char_to_point(close_tag.start),
                )
            };
            return (range.start() <= range.end()).then_some(range);
        }

        None
    }

    /// Replaces a range with new text and records the edit for undo/redo.
    pub fn replace(&mut self, range: TextRange, text: &str) {
        let range = range.normalized();
        let start_char = self.point_to_char(range.start());
        let end_char = self.point_to_char(range.end());
        let start_position = self.char_to_point(start_char);
        let start_byte = self.rope.char_to_byte(start_char);
        let inserted_text = normalize_inline_text(text);
        let removed_text = self.rope.slice(start_char..end_char).to_string();
        let old_end_byte = start_byte + removed_text.len();
        let new_end_byte = start_byte + inserted_text.len();
        let old_end_position = advance_point_by_text(start_position, &removed_text);
        let new_end_position = advance_point_by_text(start_position, &inserted_text);
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
            edit: TextEdit {
                before_revision: before_state_id,
                after_revision: after_state_id,
                start_byte,
                old_end_byte,
                new_end_byte,
                start_position,
                old_end_position,
                new_end_position,
            },
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

    /// Returns the show-paren pair when `point` sits on a delimiter or HTML/XML tag.
    pub fn show_paren_at(
        &self,
        point: TextPoint,
        language_id: Option<&str>,
    ) -> Option<ShowParenMatch> {
        if self.char_count() == 0 {
            return None;
        }
        let line_len = self.line_len_chars(point.line)?;
        if point.column >= line_len {
            return None;
        }

        let char_index = self.point_to_char(point);
        if char_index >= self.char_count() {
            return None;
        }
        self.pair_at_char(char_index, language_matches_markup_tags(language_id))
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

    /// Reloads the buffer from its backing path when the on-disk content changed.
    pub fn reload_from_path(&mut self) -> io::Result<bool> {
        let Some(path) = self.path.clone() else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "text buffer has no backing path",
            ));
        };

        let reloaded = Self::load_from_path(&path)?;
        Ok(self.reload_from_buffer(reloaded))
    }

    /// Applies file-backed contents that were loaded outside the UI thread.
    pub fn reload_from_buffer(&mut self, reloaded: Self) -> bool {
        let content_changed = self.text() != reloaded.text();
        let line_ending_changed = self.preferred_line_ending != reloaded.preferred_line_ending;
        if !content_changed && !line_ending_changed {
            return false;
        }

        let cursor = self.cursor;
        let state_id = self.next_state_id;

        self.rope = reloaded.rope;
        self.path = reloaded.path;
        self.preferred_line_ending = reloaded.preferred_line_ending;
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.state_id = state_id;
        self.saved_state_id = state_id;
        self.next_state_id = self.next_state_id.saturating_add(1);
        self.cursor = self.clamp_point(cursor);

        true
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

    fn object_range_at(
        &self,
        point: TextPoint,
        around: bool,
        count: usize,
        predicate: impl Fn(char) -> bool,
    ) -> Option<TextRange> {
        if self.char_count() == 0 || count == 0 {
            return None;
        }

        let mut char_index = self
            .point_to_char(point)
            .min(self.char_count().saturating_sub(1));
        if !predicate(self.rope.char(char_index)) {
            while char_index < self.char_count() && !predicate(self.rope.char(char_index)) {
                char_index += 1;
            }
            if char_index >= self.char_count() {
                return None;
            }
        }

        let mut start_char = self.object_start_char(char_index, &predicate);
        let mut end_char = self.object_end_char(char_index, &predicate);
        for _ in 1..count {
            let mut next_object = end_char;
            while next_object < self.char_count() && !predicate(self.rope.char(next_object)) {
                next_object += 1;
            }
            if next_object >= self.char_count() {
                break;
            }
            end_char = self.object_end_char(next_object, &predicate);
        }

        if around {
            let mut trailing = end_char;
            while trailing < self.char_count() && is_inline_whitespace(self.rope.char(trailing)) {
                trailing += 1;
            }
            if trailing > end_char {
                end_char = trailing;
            } else {
                while start_char > 0 && is_inline_whitespace(self.rope.char(start_char - 1)) {
                    start_char -= 1;
                }
            }
        }

        Some(TextRange::new(
            self.char_to_point(start_char),
            self.char_to_point(end_char),
        ))
    }

    fn move_object_forward(&mut self, kind: WordKind) -> bool {
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

    fn move_object_backward(&mut self, kind: WordKind) -> bool {
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

    fn move_object_end_forward(&mut self, kind: WordKind) -> bool {
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

    fn move_object_end_backward(&mut self, kind: WordKind) -> bool {
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

    fn range_index_at(&self, point: TextPoint, ranges: &[(usize, usize)]) -> Option<usize> {
        if ranges.is_empty() {
            return None;
        }

        let char_index = self
            .point_to_char(point)
            .min(self.char_count().saturating_sub(1));
        ranges
            .iter()
            .position(|(start, end)| *start <= char_index && char_index < *end)
            .or_else(|| ranges.iter().position(|(start, _)| *start > char_index))
            .or(Some(ranges.len().saturating_sub(1)))
    }

    fn line_is_blank(&self, line_index: usize) -> bool {
        self.line(line_index)
            .map(|line| line.trim().is_empty())
            .unwrap_or(true)
    }

    fn collect_sentence_ranges(&self) -> Vec<(usize, usize)> {
        let mut ranges = Vec::new();
        let mut start = 0usize;
        let len = self.char_count();

        while start < len {
            while start < len && self.rope.char(start).is_whitespace() {
                start += 1;
            }
            if start >= len {
                break;
            }

            let mut cursor = start;
            let mut end = len;
            while cursor < len {
                if self.is_blank_line_gap(cursor) {
                    end = cursor;
                    break;
                }

                if self.is_sentence_terminator_at(cursor) {
                    end = self.sentence_end_char(cursor);
                    break;
                }

                cursor += 1;
            }

            if start < end {
                ranges.push((start, end));
            }
            start = end.max(start + 1);
        }

        ranges
    }

    fn next_paragraph_boundary_after(&self, line_index: usize) -> Option<usize> {
        let line_count = self.line_count();
        if line_count == 0 {
            return None;
        }
        let mut line = line_index.saturating_add(1);
        while line < line_count {
            if !self.line_is_blank(line) {
                line += 1;
                continue;
            }

            let run_start = line;
            while line + 1 < line_count && self.line_is_blank(line + 1) {
                line += 1;
            }
            let run_end = line;
            let separated_blocks = run_start > 0
                && run_end + 1 < line_count
                && !self.line_is_blank(run_start - 1)
                && !self.line_is_blank(run_end + 1);
            if separated_blocks {
                return Some(run_start);
            }
            line = line.saturating_add(1);
        }
        None
    }

    fn previous_paragraph_boundary_before(&self, line_index: usize) -> Option<usize> {
        let line_count = self.line_count();
        if line_index == 0 || line_count == 0 {
            return None;
        }
        let mut line = line_index.saturating_sub(1);
        loop {
            if self.line_is_blank(line) {
                let run_end = line;
                let mut run_start = run_end;
                while run_start > 0 && self.line_is_blank(run_start - 1) {
                    run_start -= 1;
                }
                let separated_blocks = run_start > 0
                    && run_end + 1 < line_count
                    && !self.line_is_blank(run_start - 1)
                    && !self.line_is_blank(run_end + 1);
                if separated_blocks {
                    return Some(run_start);
                }
                if run_start == 0 {
                    break;
                }
                line = run_start.saturating_sub(1);
                continue;
            }
            if line == 0 {
                break;
            }
            line = line.saturating_sub(1);
        }
        None
    }

    fn is_blank_line_gap(&self, char_index: usize) -> bool {
        self.rope.char(char_index) == '\n'
            && char_index + 1 < self.char_count()
            && self.rope.char(char_index + 1) == '\n'
    }

    fn is_sentence_terminator_at(&self, char_index: usize) -> bool {
        let character = self.rope.char(char_index);
        if !matches!(character, '.' | '!' | '?') {
            return false;
        }

        let mut next = char_index + 1;
        while next < self.char_count() && is_sentence_closer(self.rope.char(next)) {
            next += 1;
        }

        next >= self.char_count() || self.rope.char(next).is_whitespace()
    }

    fn sentence_end_char(&self, char_index: usize) -> usize {
        let mut end = char_index + 1;
        while end < self.char_count() && is_sentence_closer(self.rope.char(end)) {
            end += 1;
        }
        end
    }

    fn object_start_char(&self, mut char_index: usize, predicate: impl Fn(char) -> bool) -> usize {
        while char_index > 0 && predicate(self.rope.char(char_index - 1)) {
            char_index -= 1;
        }
        char_index
    }

    fn delimited_range_chars(
        &self,
        char_index: usize,
        open: char,
        close: char,
    ) -> Option<(usize, usize)> {
        let start_char = self.find_enclosing_open(char_index, open, close)?;
        let end_char = self.find_matching_close(start_char, open, close)?;
        Some((start_char, end_char))
    }

    fn quoted_range_chars(&self, char_index: usize, quote: char) -> Option<(usize, usize)> {
        let (line_start, line_end) = self.line_char_bounds(self.char_to_point(char_index).line)?;
        let quotes = (line_start..line_end)
            .filter(|index| self.rope.char(*index) == quote && !self.char_is_escaped(*index))
            .collect::<Vec<_>>();
        quotes.chunks(2).find_map(|pair| {
            (pair.len() == 2 && pair[0] <= char_index && char_index <= pair[1])
                .then_some((pair[0], pair[1]))
        })
    }

    fn find_enclosing_open(&self, char_index: usize, open: char, close: char) -> Option<usize> {
        let mut depth = 0usize;
        for index in (0..=char_index).rev() {
            let character = self.rope.char(index);
            if character == open {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
            } else if character == close {
                depth += 1;
            }
        }
        None
    }

    fn find_matching_close(&self, start_char: usize, open: char, close: char) -> Option<usize> {
        self.find_matching_close_limited(start_char, open, close, self.char_count())
    }

    fn find_matching_close_limited(
        &self,
        start_char: usize,
        open: char,
        close: char,
        limit: usize,
    ) -> Option<usize> {
        let mut depth = 0usize;
        let end = (start_char + 1)
            .saturating_add(limit)
            .min(self.char_count());
        for index in (start_char + 1)..end {
            let character = self.rope.char(index);
            if character == open {
                depth += 1;
            } else if character == close {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
            }
        }
        None
    }

    fn find_matching_open_limited(
        &self,
        start_char: usize,
        open: char,
        close: char,
        limit: usize,
    ) -> Option<usize> {
        let mut depth = 0usize;
        let search_start = start_char.saturating_sub(limit);
        for index in (search_start..start_char).rev() {
            let character = self.rope.char(index);
            if character == close {
                depth += 1;
            } else if character == open {
                if depth == 0 {
                    return Some(index);
                }
                depth -= 1;
            }
        }
        None
    }

    fn match_pair_origin_char(&self, markup_tags: bool) -> Option<usize> {
        if self.char_count() == 0 {
            return None;
        }
        let original = self
            .point_to_char(self.cursor)
            .min(self.char_count().saturating_sub(1));
        if self
            .pair_at_char(original, markup_tags)
            .is_some_and(|pair| pair.matched)
        {
            return Some(original);
        }

        let line = self.cursor.line.min(self.line_count().saturating_sub(1));
        let (line_start, line_end) = self.line_char_bounds(line)?;
        (original..line_end)
            .find(|&index| {
                self.is_match_pair_scan_char(index, markup_tags)
                    && self
                        .pair_at_char(index, markup_tags)
                        .is_some_and(|pair| pair.matched)
            })
            .or_else(|| {
                (line_start..original).rev().find(|&index| {
                    self.is_match_pair_scan_char(index, markup_tags)
                        && self
                            .pair_at_char(index, markup_tags)
                            .is_some_and(|pair| pair.matched)
                })
            })
    }

    fn is_match_pair_scan_char(&self, index: usize, markup_tags: bool) -> bool {
        let Some(character) = self.rope.get_char(index) else {
            return false;
        };
        delimiter_partner(character).is_some() || (markup_tags && character == '<')
    }

    fn pair_at_char(&self, char_index: usize, markup_tags: bool) -> Option<ShowParenMatch> {
        let character = self.rope.get_char(char_index)?;
        if let Some((open, close, is_open)) = delimiter_partner(character) {
            let origin = self.char_range(char_index, char_index.saturating_add(1))?;
            let destination = if is_open {
                self.find_matching_close_limited(char_index, open, close, SHOW_PAREN_SCAN_LIMIT)
            } else {
                self.find_matching_open_limited(char_index, open, close, SHOW_PAREN_SCAN_LIMIT)
            };
            return Some(ShowParenMatch {
                origin,
                counterpart: destination
                    .and_then(|index| self.char_range(index, index.saturating_add(1))),
                matched: destination.is_some(),
            });
        }

        if markup_tags {
            self.show_paren_tag_at(char_index)
        } else {
            None
        }
    }

    fn char_range(&self, start_char: usize, end_char: usize) -> Option<TextRange> {
        if start_char >= end_char || end_char > self.char_count() {
            return None;
        }
        Some(TextRange::new(
            self.char_to_point(start_char),
            self.char_to_point(end_char),
        ))
    }

    fn show_paren_tag_at(&self, char_index: usize) -> Option<ShowParenMatch> {
        let tag = self.tag_containing(char_index)?;
        if tag.self_closing {
            return None;
        }

        let origin = self.char_range(tag.start, tag.end_exclusive)?;
        if tag.is_closing {
            let open_tag = self.find_matching_open_tag(&tag, SHOW_PAREN_SCAN_LIMIT);
            return Some(ShowParenMatch {
                origin,
                counterpart: open_tag
                    .as_ref()
                    .and_then(|open| self.char_range(open.start, open.end_exclusive)),
                matched: open_tag.is_some(),
            });
        }

        let close_tag = self.find_matching_close_tag_from(&tag, SHOW_PAREN_SCAN_LIMIT);
        Some(ShowParenMatch {
            origin,
            counterpart: close_tag
                .as_ref()
                .and_then(|close| self.char_range(close.start, close.end_exclusive)),
            matched: close_tag.is_some(),
        })
    }

    fn tag_containing(&self, char_index: usize) -> Option<TagToken> {
        let search_start = char_index.saturating_sub(SHOW_PAREN_TAG_LOOKBACK);
        for start in (search_start..=char_index).rev() {
            if self.rope.get_char(start) != Some('<') {
                continue;
            }
            let Some(tag) =
                parse_tag_token_at(start, self.char_count(), |index| self.rope.get_char(index))
            else {
                continue;
            };
            if tag.start <= char_index && char_index < tag.end_exclusive {
                return Some(tag);
            }
            if tag.end_exclusive <= char_index {
                return None;
            }
        }
        None
    }

    fn find_matching_close_tag_from(&self, open_tag: &TagToken, limit: usize) -> Option<TagToken> {
        let end = open_tag
            .end_exclusive
            .saturating_add(limit)
            .min(self.char_count());
        let mut cursor = open_tag.end_exclusive;
        let mut depth = 0usize;
        while cursor < end {
            if self.rope.get_char(cursor) != Some('<') {
                cursor += 1;
                continue;
            }
            let Some(tag) =
                parse_tag_token_at(cursor, self.char_count(), |index| self.rope.get_char(index))
            else {
                cursor += 1;
                continue;
            };
            if tag.name == open_tag.name {
                if tag.is_closing {
                    if depth == 0 {
                        return Some(tag);
                    }
                    depth -= 1;
                } else if !tag.self_closing {
                    depth += 1;
                }
            }
            cursor = tag.end_exclusive;
        }
        None
    }

    fn find_matching_open_tag(&self, close_tag: &TagToken, limit: usize) -> Option<TagToken> {
        let search_start = close_tag.start.saturating_sub(limit);
        let mut depth = 0usize;
        let mut cursor = close_tag.start;
        while cursor > search_start {
            cursor -= 1;
            if self.rope.get_char(cursor) != Some('<') {
                continue;
            }
            let Some(tag) =
                parse_tag_token_at(cursor, self.char_count(), |index| self.rope.get_char(index))
            else {
                continue;
            };
            if tag.name != close_tag.name {
                continue;
            }
            if tag.is_closing {
                depth += 1;
            } else if !tag.self_closing {
                if depth == 0 {
                    return Some(tag);
                }
                depth -= 1;
            }
        }
        None
    }

    fn line_char_bounds(&self, line_index: usize) -> Option<(usize, usize)> {
        if line_index >= self.line_count() {
            return None;
        }
        let start_char = self.rope.line_to_char(line_index);
        let end_char = if line_index + 1 < self.line_count() {
            self.rope.line_to_char(line_index + 1)
        } else {
            self.char_count()
        };
        Some((start_char, end_char))
    }

    fn char_is_escaped(&self, char_index: usize) -> bool {
        let mut backslashes = 0usize;
        let mut index = char_index;
        while index > 0 && self.rope.char(index - 1) == '\\' {
            backslashes += 1;
            index -= 1;
        }
        backslashes % 2 == 1
    }

    fn object_end_char(&self, mut char_index: usize, predicate: impl Fn(char) -> bool) -> usize {
        while char_index < self.char_count() && predicate(self.rope.char(char_index)) {
            char_index += 1;
        }
        char_index
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
