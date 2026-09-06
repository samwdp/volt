#![allow(unused_imports)]
use std::{
    fs::File,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    ops::Range,
    path::{Path, PathBuf},
};

use ropey::{Rope, RopeBuilder, RopeSlice, iter::Chunks as RopeChunks};

#[allow(unused_imports)]
use crate::buffer::*;
#[allow(unused_imports)]
use crate::geometry::*;
#[allow(unused_imports)]
use crate::motion::*;

pub(crate) const SHOW_PAREN_SCAN_LIMIT: usize = 102_400;

pub(crate) const SHOW_PAREN_TAG_LOOKBACK: usize = 4_096;

pub(crate) const SHOW_PAREN_TAG_MAX_LEN: usize = 2_048;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TagToken {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end_exclusive: usize,
    pub(crate) is_closing: bool,
    pub(crate) self_closing: bool,
}

impl TextBuffer {
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

    pub(crate) fn object_range_at(
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

    pub(crate) fn delimited_range_chars(
        &self,
        char_index: usize,
        open: char,
        close: char,
    ) -> Option<(usize, usize)> {
        let start_char = self.find_enclosing_open(char_index, open, close)?;
        let end_char = self.find_matching_close(start_char, open, close)?;
        Some((start_char, end_char))
    }

    pub(crate) fn pair_at_char(
        &self,
        char_index: usize,
        markup_tags: bool,
    ) -> Option<ShowParenMatch> {
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

    pub(crate) fn show_paren_tag_at(&self, char_index: usize) -> Option<ShowParenMatch> {
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
}

pub(crate) fn detect_preferred_line_ending(text: &str) -> LineEnding {
    if text.contains("\r\n") {
        LineEnding::Crlf
    } else {
        LineEnding::Lf
    }
}

pub(crate) fn normalize_inline_text(text: &str) -> String {
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

pub(crate) fn is_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

pub(crate) fn is_inline_whitespace(character: char) -> bool {
    character.is_whitespace() && !matches!(character, '\n' | '\r')
}

pub(crate) fn is_punctuation_char(character: char) -> bool {
    !character.is_whitespace() && !is_word_char(character)
}

pub(crate) fn word_motion_class(character: char, kind: WordKind) -> WordMotionClass {
    match kind {
        WordKind::Word => {
            if character.is_whitespace() {
                WordMotionClass::Whitespace
            } else if is_word_char(character) {
                WordMotionClass::Word
            } else {
                WordMotionClass::Punctuation
            }
        }
        WordKind::BigWord => {
            if character.is_whitespace() {
                WordMotionClass::Whitespace
            } else {
                WordMotionClass::Word
            }
        }
    }
}

pub(crate) fn matches_word_kind(character: char, kind: WordKind) -> bool {
    match kind {
        WordKind::Word => is_word_char(character),
        WordKind::BigWord => !character.is_whitespace(),
    }
}

pub(crate) fn is_object_separator(character: char, kind: WordKind) -> bool {
    match kind {
        WordKind::Word => character.is_whitespace() || is_punctuation_char(character),
        WordKind::BigWord => character.is_whitespace(),
    }
}

pub(crate) fn is_sentence_closer(character: char) -> bool {
    matches!(character, '"' | '\'' | ')' | ']' | '}')
}

pub(crate) fn parse_tag_token(chars: &[char], start: usize) -> Option<TagToken> {
    parse_tag_token_at(start, chars.len(), |index| chars.get(index).copied())
}

pub(crate) fn parse_tag_token_at(
    start: usize,
    len: usize,
    char_at: impl Fn(usize) -> Option<char>,
) -> Option<TagToken> {
    if char_at(start) != Some('<') {
        return None;
    }

    let mut cursor = start + 1;
    match char_at(cursor)? {
        '!' | '?' => return None,
        _ => {}
    }

    let is_closing = if char_at(cursor) == Some('/') {
        cursor += 1;
        true
    } else {
        false
    };

    while char_at(cursor).is_some_and(|character| character.is_whitespace()) {
        cursor += 1;
    }

    let name_start = cursor;
    let first = char_at(cursor)?;
    if !is_tag_name_start_char(first) {
        return None;
    }
    cursor += 1;
    while char_at(cursor).is_some_and(is_tag_name_char) {
        cursor += 1;
    }

    let after_name = char_at(cursor)?;
    if after_name != '>' && after_name != '/' && !after_name.is_whitespace() {
        return None;
    }

    let mut end = cursor;
    let mut quote = None;
    let mut brace_depth = 0usize;
    while end < len {
        if end.saturating_sub(start) > SHOW_PAREN_TAG_MAX_LEN {
            return None;
        }
        let ch = char_at(end)?;
        if let Some(q) = quote {
            if ch == q {
                quote = None;
            }
            end += 1;
            continue;
        }
        if brace_depth > 0 {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth = brace_depth.saturating_sub(1),
                _ => {}
            }
            end += 1;
            continue;
        }
        match ch {
            '>' => break,
            '"' | '\'' => {
                quote = Some(ch);
                end += 1;
            }
            '{' => {
                brace_depth = 1;
                end += 1;
            }
            ch if is_plausible_unquoted_tag_body_char(ch) => {
                end += 1;
            }
            _ => return None,
        }
    }
    if end >= len || char_at(end) != Some('>') {
        return None;
    }

    let name = (name_start..cursor)
        .filter_map(&char_at)
        .collect::<String>();
    let mut tail = end;
    while tail > cursor && char_at(tail - 1).is_some_and(char::is_whitespace) {
        tail -= 1;
    }

    Some(TagToken {
        name,
        start,
        end_exclusive: end + 1,
        is_closing,
        self_closing: !is_closing && tail > cursor && char_at(tail - 1) == Some('/'),
    })
}

pub(crate) fn find_matching_close_tag(chars: &[char], open_tag: &TagToken) -> Option<TagToken> {
    let mut cursor = open_tag.end_exclusive;
    let mut depth = 0usize;
    while cursor < chars.len() {
        if chars[cursor] != '<' {
            cursor += 1;
            continue;
        }

        let Some(tag) = parse_tag_token(chars, cursor) else {
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

pub(crate) fn is_tag_name_start_char(character: char) -> bool {
    character.is_alphabetic() || matches!(character, '_' | ':')
}

pub(crate) fn is_tag_name_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | ':')
}

pub(crate) fn is_plausible_unquoted_tag_body_char(character: char) -> bool {
    character.is_whitespace()
        || character.is_alphanumeric()
        || "-_:.=/#@*?![](),".contains(character)
}

pub(crate) fn delimiter_partner(character: char) -> Option<(char, char, bool)> {
    match character {
        '(' => Some(('(', ')', true)),
        ')' => Some(('(', ')', false)),
        '[' => Some(('[', ']', true)),
        ']' => Some(('[', ']', false)),
        '{' => Some(('{', '}', true)),
        '}' => Some(('{', '}', false)),
        _ => None,
    }
}

pub(crate) fn visible_line_len(slice: RopeSlice<'_>) -> usize {
    let len = slice.len_chars();
    if len == 0 {
        return 0;
    }

    match slice.get_char(len - 1) {
        Some('\n') => len - 1,
        _ => len,
    }
}

pub(crate) fn trimmed_line(slice: RopeSlice<'_>) -> String {
    let mut line = slice.to_string();
    if line.ends_with('\n') {
        line.pop();
    }
    line
}
