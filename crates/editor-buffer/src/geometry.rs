use std::ops::Range;

use ropey::{Rope, iter::Chunks as RopeChunks};

use crate::buffer::TextBuffer;

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
    pub(crate) start: TextPoint,
    pub(crate) end: TextPoint,
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
    pub(crate) anchor: TextPoint,
    pub(crate) head: TextPoint,
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

pub(crate) enum TextByteChunkSource<'a> {
    Empty(Option<&'a [u8]>),
    Chunks(RopeChunks<'a>),
}

/// Iterator over UTF-8 byte chunks from a [`TextBuffer`] byte range.
pub struct TextByteChunks<'a> {
    pub(crate) source: TextByteChunkSource<'a>,
}

impl<'a> TextByteChunks<'a> {
    pub(crate) fn empty() -> Self {
        Self {
            source: TextByteChunkSource::Empty(Some(&[])),
        }
    }

    pub(crate) fn from_chunks(chunks: RopeChunks<'a>) -> Self {
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
    pub(crate) snapshot: TextSnapshot,
    pub(crate) revision: u64,
    pub(crate) edits_from: u64,
    pub(crate) edits: Vec<TextEdit>,
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
    pub(crate) rope: Rope,
    pub(crate) cursor: TextPoint,
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

    pub(crate) fn line_len_chars_impl(&self, line_index: usize) -> usize {
        visible_line_len(self.rope.line(line_index))
    }

    pub(crate) fn clamp_point(&self, point: TextPoint) -> TextPoint {
        let max_line = self.line_count().saturating_sub(1);
        let line = point.line.min(max_line);
        let column = point.column.min(self.line_len_chars_impl(line));
        TextPoint { line, column }
    }

    pub(crate) fn point_to_char(&self, point: TextPoint) -> usize {
        let point = self.clamp_point(point);
        self.rope.line_to_char(point.line) + point.column
    }

    pub(crate) fn char_to_point(&self, char_index: usize) -> TextPoint {
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

pub(crate) fn advance_point_by_text(mut point: TextPoint, text: &str) -> TextPoint {
    for character in text.chars() {
        if character == '\n' {
            point.line += 1;
            point.column = 0;
        } else {
            point.column += 1;
        }
    }
    point
}
