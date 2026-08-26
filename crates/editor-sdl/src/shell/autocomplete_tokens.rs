//! Revision-keyed buffer-token frequency maps for autocomplete.

use std::collections::BTreeMap;
use std::ops::Range;

use editor_buffer::{TextEdit, TextSnapshot};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum AutocompleteTokenScanKind {
    Reused,
    Incremental,
    Rebuilt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AutocompleteTokenScan {
    pub kind: AutocompleteTokenScanKind,
    pub scanned_bytes: usize,
}

#[derive(Debug, Default)]
pub(super) struct AutocompleteTokenCache {
    buffer_id: Option<u64>,
    revision: u64,
    snapshot: Option<TextSnapshot>,
    counts: BTreeMap<String, usize>,
    #[cfg(test)]
    last_scan: Option<AutocompleteTokenScan>,
}

impl AutocompleteTokenCache {
    pub(super) fn counts(&self) -> &BTreeMap<String, usize> {
        &self.counts
    }

    #[cfg(test)]
    pub(super) fn last_scan(&self) -> Option<AutocompleteTokenScan> {
        self.last_scan
    }

    pub(super) fn key(&self) -> Option<(u64, u64)> {
        self.buffer_id.map(|buffer_id| (buffer_id, self.revision))
    }

    pub(super) fn refresh(
        &mut self,
        buffer_id: u64,
        revision: u64,
        snapshot: &TextSnapshot,
        edits_from: Option<u64>,
        edits: Option<&[TextEdit]>,
    ) -> AutocompleteTokenScan {
        if self.buffer_id == Some(buffer_id) && self.revision == revision && self.snapshot.is_some()
        {
            let scan = AutocompleteTokenScan {
                kind: AutocompleteTokenScanKind::Reused,
                scanned_bytes: 0,
            };
            #[cfg(test)]
            {
                self.last_scan = Some(scan);
            }
            return scan;
        }

        let can_increment = self.buffer_id == Some(buffer_id)
            && edits_from == Some(self.revision)
            && self.snapshot.is_some();
        if can_increment
            && let Some(edits) = edits
            && let Some(old_snapshot) = self.snapshot.as_ref()
            && edits_chain_matches(edits, self.revision, revision)
            && let Some(scanned_bytes) =
                apply_edits_to_counts(&mut self.counts, old_snapshot, snapshot, edits)
        {
            self.revision = revision;
            self.snapshot = Some(snapshot.clone());
            let scan = AutocompleteTokenScan {
                kind: AutocompleteTokenScanKind::Incremental,
                scanned_bytes,
            };
            #[cfg(test)]
            {
                self.last_scan = Some(scan);
            }
            return scan;
        }

        self.rebuild(buffer_id, revision, snapshot)
    }

    fn rebuild(
        &mut self,
        buffer_id: u64,
        revision: u64,
        snapshot: &TextSnapshot,
    ) -> AutocompleteTokenScan {
        self.counts = collect_autocomplete_token_counts(snapshot);
        self.buffer_id = Some(buffer_id);
        self.revision = revision;
        self.snapshot = Some(snapshot.clone());
        let scan = AutocompleteTokenScan {
            kind: AutocompleteTokenScanKind::Rebuilt,
            scanned_bytes: snapshot.byte_count(),
        };
        #[cfg(test)]
        {
            self.last_scan = Some(scan);
        }
        scan
    }
}

pub(super) fn collect_autocomplete_token_counts(
    snapshot: &TextSnapshot,
) -> BTreeMap<String, usize> {
    collect_tokens_in_range(snapshot, 0, snapshot.byte_count()).unwrap_or_default()
}

pub(super) fn is_completion_word_char(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

fn edits_chain_matches(edits: &[TextEdit], from_revision: u64, to_revision: u64) -> bool {
    if from_revision == to_revision {
        return edits.is_empty();
    }
    let Some(first) = edits.first() else {
        return false;
    };
    let Some(last) = edits.last() else {
        return false;
    };
    first.before_revision == from_revision && last.after_revision == to_revision
}

fn apply_edits_to_counts(
    counts: &mut BTreeMap<String, usize>,
    old_snapshot: &TextSnapshot,
    new_snapshot: &TextSnapshot,
    edits: &[TextEdit],
) -> Option<usize> {
    if edits.is_empty() {
        return Some(0);
    }

    let mut old_ranges = Vec::new();
    let mut new_ranges = Vec::new();
    for index in 0..edits.len() {
        old_ranges.push(original_byte_range(edits, index));
        new_ranges.push(final_byte_range(edits, index));
    }

    let old_ranges = expand_and_merge_ranges(old_snapshot, old_ranges)?;
    let new_ranges = expand_and_merge_ranges(new_snapshot, new_ranges)?;

    let scanned_bytes = old_ranges
        .iter()
        .chain(new_ranges.iter())
        .map(range_len)
        .sum();

    for range in old_ranges {
        let tokens = collect_tokens_in_range(old_snapshot, range.start, range.end)?;
        subtract_counts(counts, tokens)?;
    }
    for range in new_ranges {
        let tokens = collect_tokens_in_range(new_snapshot, range.start, range.end)?;
        add_counts(counts, tokens);
    }
    Some(scanned_bytes)
}

fn original_byte_range(edits: &[TextEdit], index: usize) -> Range<usize> {
    let mut range = edits[index].start_byte..edits[index].old_end_byte;
    for prior in edits[..index].iter().rev() {
        range = map_range_backward(range, prior);
    }
    range
}

fn final_byte_range(edits: &[TextEdit], index: usize) -> Range<usize> {
    let mut range = edits[index].start_byte..edits[index].new_end_byte;
    for later in &edits[index + 1..] {
        range = map_range_forward(range, later);
    }
    range
}

fn map_range_backward(range: Range<usize>, edit: &TextEdit) -> Range<usize> {
    let start = edit.start_byte;
    let old_end = edit.old_end_byte;
    let new_end = edit.new_end_byte;
    if range.end <= start {
        return range;
    }
    if range.start >= new_end {
        let start_byte = map_byte_backward(range.start, edit);
        let end_byte = map_byte_backward(range.end, edit);
        return start_byte..end_byte;
    }
    let mapped_start = range.start.min(start);
    let mapped_end = if range.end <= new_end {
        old_end
    } else {
        map_byte_backward(range.end, edit).max(old_end)
    };
    mapped_start..mapped_end.max(old_end)
}

fn map_range_forward(range: Range<usize>, edit: &TextEdit) -> Range<usize> {
    let start = edit.start_byte;
    let old_end = edit.old_end_byte;
    let new_end = edit.new_end_byte;
    if range.end <= start {
        return range;
    }
    if range.start >= old_end {
        let start_byte = map_byte_forward(range.start, edit);
        let end_byte = map_byte_forward(range.end, edit);
        return start_byte..end_byte;
    }
    let mapped_start = range.start.min(start);
    let mapped_end = if range.end <= old_end {
        new_end
    } else {
        map_byte_forward(range.end, edit).max(new_end)
    };
    mapped_start..mapped_end.max(new_end)
}

fn map_byte_backward(byte: usize, edit: &TextEdit) -> usize {
    if byte <= edit.start_byte {
        byte
    } else if byte >= edit.new_end_byte {
        add_delta(
            byte,
            edit.old_end_byte as isize - edit.new_end_byte as isize,
        )
    } else {
        edit.start_byte
    }
}

fn map_byte_forward(byte: usize, edit: &TextEdit) -> usize {
    if byte <= edit.start_byte {
        byte
    } else if byte >= edit.old_end_byte {
        add_delta(
            byte,
            edit.new_end_byte as isize - edit.old_end_byte as isize,
        )
    } else {
        edit.start_byte
    }
}

fn add_delta(byte: usize, delta: isize) -> usize {
    if delta >= 0 {
        byte.saturating_add(delta as usize)
    } else {
        byte.saturating_sub(delta.unsigned_abs())
    }
}

fn expand_and_merge_ranges(
    snapshot: &TextSnapshot,
    ranges: Vec<Range<usize>>,
) -> Option<Vec<Range<usize>>> {
    let mut expanded = Vec::new();
    for range in ranges {
        expanded.push(expand_token_bounds(snapshot, range)?);
    }
    Some(merge_ranges(expanded))
}

fn expand_token_bounds(snapshot: &TextSnapshot, range: Range<usize>) -> Option<Range<usize>> {
    let byte_count = snapshot.byte_count();
    if range.start > byte_count || range.end > byte_count || range.start > range.end {
        return None;
    }
    let mut start = range.start;
    let mut end = range.end;
    while let Some(character) = char_before_byte(snapshot, start) {
        if !is_completion_word_char(character) {
            break;
        }
        start = start.saturating_sub(character.len_utf8());
    }
    while let Some(character) = char_at_byte(snapshot, end) {
        if !is_completion_word_char(character) {
            break;
        }
        end = end.saturating_add(character.len_utf8());
        if end > byte_count {
            return None;
        }
    }
    Some(start..end)
}

fn merge_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    if ranges.len() <= 1 {
        return ranges;
    }
    ranges.sort_by_key(|range| range.start);
    let mut iter = ranges.into_iter();
    let Some(mut current) = iter.next() else {
        return Vec::new();
    };
    let mut merged = Vec::new();
    for range in iter {
        if range.start <= current.end {
            current.end = current.end.max(range.end);
        } else {
            merged.push(std::mem::replace(&mut current, range));
        }
    }
    merged.push(current);
    merged
}

fn range_len(range: &Range<usize>) -> usize {
    range.end.saturating_sub(range.start)
}

fn collect_tokens_in_range(
    snapshot: &TextSnapshot,
    start: usize,
    end: usize,
) -> Option<BTreeMap<String, usize>> {
    let byte_count = snapshot.byte_count();
    if start > end || end > byte_count {
        return None;
    }
    let mut counts = BTreeMap::new();
    if start == end {
        return Some(counts);
    }
    let mut token = String::new();
    for chunk in snapshot.byte_slice_chunks(start..end) {
        let text = match std::str::from_utf8(chunk) {
            Ok(text) => text,
            Err(_) => {
                finish_token(&mut counts, &mut token);
                continue;
            }
        };
        for character in text.chars() {
            if is_completion_word_char(character) {
                token.push(character);
                continue;
            }
            finish_token(&mut counts, &mut token);
        }
    }
    finish_token(&mut counts, &mut token);
    Some(counts)
}

fn finish_token(counts: &mut BTreeMap<String, usize>, token: &mut String) {
    if token.is_empty() {
        return;
    }
    *counts.entry(std::mem::take(token)).or_insert(0) += 1;
}

fn add_counts(dest: &mut BTreeMap<String, usize>, src: BTreeMap<String, usize>) {
    for (token, frequency) in src {
        *dest.entry(token).or_insert(0) += frequency;
    }
}

fn subtract_counts(dest: &mut BTreeMap<String, usize>, src: BTreeMap<String, usize>) -> Option<()> {
    for (token, frequency) in src {
        let count = dest.get_mut(&token)?;
        if *count < frequency {
            return None;
        }
        if *count == frequency {
            dest.remove(&token);
        } else {
            *count -= frequency;
        }
    }
    Some(())
}

fn char_at_byte(snapshot: &TextSnapshot, byte: usize) -> Option<char> {
    if byte >= snapshot.byte_count() {
        return None;
    }
    let (chunk, chunk_start) = snapshot.chunk_at_byte(byte)?;
    let offset = byte.saturating_sub(chunk_start);
    if let Some(rest) = chunk.get(offset..) {
        return rest.chars().next();
    }
    None
}

fn char_before_byte(snapshot: &TextSnapshot, byte: usize) -> Option<char> {
    if byte == 0 {
        return None;
    }
    let index = byte - 1;
    let (chunk, chunk_start) = snapshot.chunk_at_byte(index)?;
    let offset = index.saturating_sub(chunk_start) + 1;
    if offset > 0 && offset <= chunk.len() {
        return chunk.get(..offset)?.chars().next_back();
    }
    chunk.chars().next_back()
}

#[cfg(test)]
mod tests {
    use super::*;
    use editor_buffer::{TextBuffer, TextPoint, TextRange};

    fn assert_counts_match_rescan(cache: &AutocompleteTokenCache, buffer: &TextBuffer) {
        assert_eq!(
            cache.counts(),
            &collect_autocomplete_token_counts(&buffer.snapshot())
        );
    }

    fn large_source_fixture() -> String {
        let mut text = String::with_capacity(32 * 1024);
        for index in 0..2000 {
            text.push_str("ident");
            let _ = std::fmt::Write::write_fmt(&mut text, format_args!("{index:04} "));
            if index % 10 == 9 {
                text.push('\n');
            }
        }
        text.push_str("cursor");
        text
    }

    fn refresh_rebuild(cache: &mut AutocompleteTokenCache, buffer: &TextBuffer, buffer_id: u64) {
        cache.refresh(buffer_id, buffer.revision(), &buffer.snapshot(), None, None);
    }

    fn refresh_from_edits(
        cache: &mut AutocompleteTokenCache,
        buffer: &TextBuffer,
        buffer_id: u64,
        from_revision: u64,
    ) -> AutocompleteTokenScan {
        let edits = buffer.edits_since(from_revision);
        cache.refresh(
            buffer_id,
            buffer.revision(),
            &buffer.snapshot(),
            Some(from_revision),
            edits.as_deref(),
        )
    }

    #[test]
    fn incremental_token_counts_match_rescan_after_one_insert() {
        let mut buffer = TextBuffer::from_text(large_source_fixture());
        let mut cache = AutocompleteTokenCache::default();
        refresh_rebuild(&mut cache, &buffer, 1);
        assert_eq!(
            cache.last_scan().map(|scan| scan.kind),
            Some(AutocompleteTokenScanKind::Rebuilt)
        );

        let from_revision = buffer.revision();
        let last_line = buffer.line_count().saturating_sub(1);
        let last_column = buffer
            .line(last_line)
            .map(|line| line.chars().count())
            .unwrap_or(0);
        buffer.set_cursor(TextPoint::new(last_line, last_column));
        buffer.insert_text(" fresh");

        let scan = refresh_from_edits(&mut cache, &buffer, 1, from_revision);
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Incremental);
        assert!(
            scan.scanned_bytes < buffer.byte_count() / 20,
            "one-character-adjacent insert should not recount unrelated tokens (scanned {} of {})",
            scan.scanned_bytes,
            buffer.byte_count()
        );
        assert_eq!(
            cache.counts(),
            &collect_autocomplete_token_counts(&buffer.snapshot())
        );
        assert_eq!(cache.counts().get("fresh"), Some(&1));
        assert_eq!(cache.counts().get("ident0000"), Some(&1));
    }

    #[test]
    fn incremental_token_counts_drop_deleted_identifier() {
        let mut buffer = TextBuffer::from_text("alpha beta alpha");
        let mut cache = AutocompleteTokenCache::default();
        refresh_rebuild(&mut cache, &buffer, 7);
        assert_eq!(cache.counts().get("alpha"), Some(&2));
        assert_eq!(cache.counts().get("beta"), Some(&1));

        let from_revision = buffer.revision();
        buffer.replace(
            TextRange::new(TextPoint::new(0, 11), TextPoint::new(0, 16)),
            "",
        );
        let scan = refresh_from_edits(&mut cache, &buffer, 7, from_revision);
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Incremental);
        assert_eq!(cache.counts().get("alpha"), Some(&1));
        assert_counts_match_rescan(&cache, &buffer);

        let from_revision = buffer.revision();
        buffer.replace(
            TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
            "",
        );
        let scan = refresh_from_edits(&mut cache, &buffer, 7, from_revision);
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Incremental);
        assert!(!cache.counts().contains_key("alpha"));
        assert_eq!(cache.counts().get("beta"), Some(&1));
        assert_counts_match_rescan(&cache, &buffer);
    }

    #[test]
    fn token_counts_reuse_map_for_same_revision() {
        let buffer = TextBuffer::from_text("alpha beta");
        let mut cache = AutocompleteTokenCache::default();
        refresh_rebuild(&mut cache, &buffer, 3);
        let rebuilt_bytes = cache.last_scan().map(|scan| scan.scanned_bytes);
        let scan = cache.refresh(
            3,
            buffer.revision(),
            &buffer.snapshot(),
            Some(buffer.revision()),
            Some(&[]),
        );
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Reused);
        assert_eq!(scan.scanned_bytes, 0);
        assert_ne!(rebuilt_bytes, Some(0));
        assert_counts_match_rescan(&cache, &buffer);
    }

    #[test]
    fn token_counts_rebuild_when_edit_chain_missing() {
        let mut buffer = TextBuffer::from_text("alpha beta");
        let mut cache = AutocompleteTokenCache::default();
        refresh_rebuild(&mut cache, &buffer, 4);
        buffer.set_cursor(TextPoint::new(0, 10));
        buffer.insert_text(" gamma");
        let scan = cache.refresh(4, buffer.revision(), &buffer.snapshot(), Some(0), None);
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Rebuilt);
        assert_eq!(scan.scanned_bytes, buffer.byte_count());
        assert_eq!(cache.counts().get("gamma"), Some(&1));
        assert_counts_match_rescan(&cache, &buffer);
    }

    #[test]
    fn token_counts_match_rescan_after_reload() {
        let mut buffer = TextBuffer::from_text("alpha beta alpha");
        let mut cache = AutocompleteTokenCache::default();
        refresh_rebuild(&mut cache, &buffer, 5);
        let from_revision = buffer.revision();
        buffer.reload_from_buffer(TextBuffer::from_text("gamma gamma"));
        assert!(buffer.edits_since(from_revision).is_none());
        let scan = refresh_from_edits(&mut cache, &buffer, 5, from_revision);
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Rebuilt);
        assert_eq!(cache.counts().get("gamma"), Some(&2));
        assert!(!cache.counts().contains_key("alpha"));
        assert_counts_match_rescan(&cache, &buffer);
    }

    #[test]
    fn incremental_token_counts_apply_burst_of_inserts() {
        let mut buffer = TextBuffer::from_text("hello world");
        let mut cache = AutocompleteTokenCache::default();
        refresh_rebuild(&mut cache, &buffer, 6);
        let from_revision = buffer.revision();
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text("a");
        buffer.insert_text("b");
        buffer.insert_text("c");
        let scan = refresh_from_edits(&mut cache, &buffer, 6, from_revision);
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Incremental);
        assert!(!cache.counts().contains_key("hello"));
        assert_eq!(cache.counts().get("helloabc"), Some(&1));
        assert_eq!(cache.counts().get("world"), Some(&1));
        assert_counts_match_rescan(&cache, &buffer);
    }

    #[test]
    fn tiny_file_incremental_counts_match_full_rescan() {
        let mut buffer = TextBuffer::from_text("a b a");
        let mut cache = AutocompleteTokenCache::default();
        refresh_rebuild(&mut cache, &buffer, 8);
        let from_revision = buffer.revision();
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text(" c");
        let scan = refresh_from_edits(&mut cache, &buffer, 8, from_revision);
        assert_eq!(scan.kind, AutocompleteTokenScanKind::Incremental);
        assert_eq!(cache.counts().get("a"), Some(&2));
        assert_eq!(cache.counts().get("b"), Some(&1));
        assert_eq!(cache.counts().get("c"), Some(&1));
        assert_counts_match_rescan(&cache, &buffer);
    }
}
