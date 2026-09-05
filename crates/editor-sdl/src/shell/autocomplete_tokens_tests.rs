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
