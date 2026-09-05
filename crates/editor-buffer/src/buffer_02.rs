impl SyntaxText for TextBuffer {
    fn revision(&self) -> u64 {
        TextBuffer::revision(self)
    }

    fn byte_count(&self) -> usize {
        TextBuffer::byte_count(self)
    }

    fn line_count(&self) -> usize {
        TextBuffer::line_count(self)
    }

    fn line(&self, line_index: usize) -> Option<String> {
        TextBuffer::line(self, line_index)
    }

    fn line_start_byte(&self, line_index: usize) -> Option<usize> {
        TextBuffer::line_start_byte(self, line_index)
    }

    fn chunk_at_byte(&self, byte_index: usize) -> Option<(&str, usize)> {
        TextBuffer::chunk_at_byte(self, byte_index)
    }

    fn byte_slice_chunks(&self, byte_range: Range<usize>) -> TextByteChunks<'_> {
        TextBuffer::byte_slice_chunks(self, byte_range)
    }

    fn edits_since(&self, revision: u64) -> Option<Vec<TextEdit>> {
        TextBuffer::edits_since(self, revision)
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

fn is_inline_whitespace(character: char) -> bool {
    character.is_whitespace() && !matches!(character, '\n' | '\r')
}

fn is_punctuation_char(character: char) -> bool {
    !character.is_whitespace() && !is_word_char(character)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WordMotionClass {
    Whitespace,
    Word,
    Punctuation,
}

fn word_motion_class(character: char, kind: WordKind) -> WordMotionClass {
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

fn matches_word_kind(character: char, kind: WordKind) -> bool {
    match kind {
        WordKind::Word => is_word_char(character),
        WordKind::BigWord => !character.is_whitespace(),
    }
}

fn is_object_separator(character: char, kind: WordKind) -> bool {
    match kind {
        WordKind::Word => character.is_whitespace() || is_punctuation_char(character),
        WordKind::BigWord => character.is_whitespace(),
    }
}

fn is_sentence_closer(character: char) -> bool {
    matches!(character, '"' | '\'' | ')' | ']' | '}')
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TagToken {
    name: String,
    start: usize,
    end_exclusive: usize,
    is_closing: bool,
    self_closing: bool,
}

fn parse_tag_token(chars: &[char], start: usize) -> Option<TagToken> {
    parse_tag_token_at(start, chars.len(), |index| chars.get(index).copied())
}

fn parse_tag_token_at(
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

fn find_matching_close_tag(chars: &[char], open_tag: &TagToken) -> Option<TagToken> {
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

fn is_tag_name_start_char(character: char) -> bool {
    character.is_alphabetic() || matches!(character, '_' | ':')
}

fn is_tag_name_char(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '-' | '_' | ':')
}

fn is_plausible_unquoted_tag_body_char(character: char) -> bool {
    character.is_whitespace()
        || character.is_alphanumeric()
        || "-_:.=/#@*?![](),".contains(character)
}

const SHOW_PAREN_SCAN_LIMIT: usize = 102_400;
const SHOW_PAREN_TAG_LOOKBACK: usize = 4_096;
const SHOW_PAREN_TAG_MAX_LEN: usize = 2_048;

fn delimiter_partner(character: char) -> Option<(char, char, bool)> {
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

fn advance_point_by_text(mut point: TextPoint, text: &str) -> TextPoint {
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

#[cfg(test)]
mod tests;
