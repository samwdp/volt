    use std::{
        fmt::Write as _,
        fs,
        io::Cursor,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        HighlightDocument, LineEnding, SyntaxText, TextBuffer, TextEdit, TextPoint, TextRange,
        WordKind, language_matches_markup_tags,
    };

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn create(name: &str, contents: &str) -> std::io::Result<Self> {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default();
            let path = std::env::temp_dir().join(format!("volt-buffer-{name}-{unique}.txt"));
            fs::write(&path, contents)?;
            Ok(Self { path })
        }

        fn overwrite(&self, contents: &str) -> std::io::Result<()> {
            fs::write(&self.path, contents)
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
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
    fn edits_since_returns_contiguous_forward_edits() {
        let mut buffer = TextBuffer::from_text("alpha");
        let base_revision = buffer.revision();
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text(" beta");
        buffer.insert_text("\ngamma");

        let edits = must(buffer.edits_since(base_revision).ok_or("missing edits"));
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[0].before_revision, base_revision);
        assert_eq!(edits[0].after_revision + 1, edits[1].after_revision);
        assert_eq!(edits[0].start_position, TextPoint::new(0, 5));
        assert_eq!(edits[1].new_end_position, TextPoint::new(1, 5));
    }

    #[test]
    fn highlight_document_captures_edits_without_undo_history() {
        let mut buffer = TextBuffer::from_text("alpha");
        let base_revision = buffer.revision();
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text(" beta");
        let document = HighlightDocument::from_buffer(&buffer, base_revision);
        let edits = document
            .edits_since(base_revision)
            .expect("contiguous edits");
        assert_eq!(edits.len(), 1);
        assert_eq!(document.revision(), buffer.revision());
        assert_eq!(document.snapshot().text(), buffer.text());
        let unknown_revision = if base_revision == 0 {
            u64::MAX
        } else {
            base_revision - 1
        };
        assert!(document.edits_since(unknown_revision).is_none());
    }

    #[test]
    fn highlight_document_falls_back_to_full_parse_without_contiguous_edits() {
        let mut buffer = TextBuffer::from_text("alpha");
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text(" beta");
        let current = buffer.revision();
        let document = HighlightDocument::from_buffer(&buffer, current.saturating_add(10));
        assert_eq!(document.revision(), current);
        assert_eq!(
            document.edits_since(current).expect("same revision"),
            Vec::<TextEdit>::new()
        );
        assert!(document.edits_since(0).is_none());
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
    fn reload_from_path_updates_content_preserves_cursor_and_marks_clean() {
        let file = must(TempFile::create("reload", "alpha\nbeta\n"));
        let mut buffer = must(TextBuffer::load_from_path(file.path()));
        buffer.set_cursor(TextPoint::new(1, 2));

        must(file.overwrite("alpha\nbravo\ncharlie\r\n"));

        assert!(must(buffer.reload_from_path()));
        assert_eq!(buffer.text(), "alpha\nbravo\ncharlie\n");
        assert_eq!(buffer.cursor(), TextPoint::new(1, 2));
        assert_eq!(buffer.preferred_line_ending(), LineEnding::Crlf);
        assert_eq!(buffer.revision(), 1);
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn reload_from_path_returns_false_when_disk_state_is_unchanged() {
        let file = must(TempFile::create("reload-same", "alpha\nbeta\n"));
        let mut buffer = must(TextBuffer::load_from_path(file.path()));

        assert!(!must(buffer.reload_from_path()));
        assert_eq!(buffer.text(), "alpha\nbeta\n");
        assert_eq!(buffer.revision(), 0);
        assert!(!buffer.is_dirty());
    }

    #[test]
    fn reload_from_path_requires_a_backing_file() {
        let mut buffer = TextBuffer::from_text("scratch");
        let error = match buffer.reload_from_path() {
            Ok(changed) => panic!("expected reload error, got changed={changed}"),
            Err(error) => error,
        };

        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
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
    fn text_snapshot_preserves_pre_edit_content_and_cursor() {
        let mut buffer = TextBuffer::from_text("alpha\nbeta");
        buffer.set_cursor(TextPoint::new(1, 2));
        let snapshot = buffer.snapshot();

        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text("!");

        assert_eq!(snapshot.cursor(), TextPoint::new(1, 2));
        assert_eq!(snapshot.text(), "alpha\nbeta");
        assert_eq!(snapshot.line(1).as_deref(), Some("beta"));
        assert_eq!(snapshot.point_to_char_index(TextPoint::new(1, 0)), 6);
        assert_eq!(
            snapshot.point_after(TextPoint::new(0, 4)),
            Some(TextPoint::new(0, 5))
        );
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

        assert!(buffer.move_word_end_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 15));
    }

    #[test]
    fn big_word_backward_end_and_match_pair_cover_quickref_motion_slice() {
        let mut buffer = TextBuffer::from_text("alpha-beta gamma");
        buffer.set_cursor(TextPoint::new(0, 0));

        assert!(buffer.move_big_word_end_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 9));

        assert!(buffer.move_big_word_end_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 15));

        buffer.set_cursor(TextPoint::new(0, 0));

        assert!(buffer.move_big_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 11));

        assert!(buffer.move_big_word_end_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 15));

        assert!(buffer.move_big_word_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 11));

        assert!(buffer.move_big_word_end_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 9));

        let mut buffer = TextBuffer::from_text("call(foo[bar])");
        buffer.set_cursor(TextPoint::new(0, 4));
        assert!(buffer.move_matching_delimiter(Some("rust")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 13));

        assert!(buffer.move_matching_delimiter(Some("rust")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 4));
    }

    #[test]
    fn word_motions_treat_punctuation_runs_as_words() {
        let mut buffer = TextBuffer::from_text("alpha... beta");

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 5));

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 9));

        assert!(buffer.move_word_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 5));

        assert!(buffer.move_word_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 0));
    }

    #[test]
    fn word_motions_stop_on_punctuation_before_crossing_lines() {
        let mut buffer = TextBuffer::from_text("PluginKeymapScope::Workspace,\n),\nnormal_binding");
        buffer.set_cursor(TextPoint::new(0, 19));

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 28));

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(1, 0));

        assert!(buffer.move_word_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(2, 0));

        assert!(buffer.move_word_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(1, 0));

        assert!(buffer.move_word_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 28));

        assert!(buffer.move_word_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 19));
    }

    #[test]
    fn sentence_and_paragraph_motions_cover_structure_navigation() {
        let mut buffer = TextBuffer::from_text("Alpha. Bravo! Charlie?\n\nDelta\nEcho\n\nFoxtrot");
        buffer.set_cursor(TextPoint::new(0, 2));
        assert!(buffer.move_sentence_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 7));

        assert!(buffer.move_sentence_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 0));

        buffer.set_cursor(TextPoint::new(2, 1));
        assert!(buffer.move_paragraph_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(1, 0));

        assert!(buffer.move_paragraph_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(0, 0));

        buffer.set_cursor(TextPoint::new(2, 1));
        assert!(buffer.move_paragraph_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(4, 0));

        assert!(buffer.move_paragraph_backward());
        assert_eq!(buffer.cursor(), TextPoint::new(1, 0));

        buffer.set_cursor(TextPoint::new(5, 1));
        assert!(buffer.move_paragraph_forward());
        assert_eq!(buffer.cursor(), TextPoint::new(5, 0));
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

    #[test]
    fn word_ranges_cover_inner_and_around_text_objects() {
        let buffer = TextBuffer::from_text("alpha beta  gamma");

        let inner = buffer
            .word_range_at(TextPoint::new(0, 7), false, 1)
            .expect("inner word range");
        assert_eq!(buffer.slice(inner), "beta");

        let around = buffer
            .word_range_at(TextPoint::new(0, 7), true, 1)
            .expect("around word range");
        assert_eq!(buffer.slice(around), "beta  ");

        let counted = buffer
            .word_range_at(TextPoint::new(0, 7), false, 2)
            .expect("counted word range");
        assert_eq!(buffer.slice(counted), "beta  gamma");
    }

    #[test]
    fn word_kind_ranges_cover_big_word_objects() {
        let buffer = TextBuffer::from_text("alpha-beta gamma");

        let word = buffer
            .word_range_at_kind(TextPoint::new(0, 7), WordKind::Word, false, 1)
            .expect("word range");
        assert_eq!(buffer.slice(word), "beta");

        let big_word = buffer
            .word_range_at_kind(TextPoint::new(0, 7), WordKind::BigWord, false, 1)
            .expect("big word range");
        assert_eq!(buffer.slice(big_word), "alpha-beta");

        let around_big_word = buffer
            .word_range_at_kind(TextPoint::new(0, 7), WordKind::BigWord, true, 1)
            .expect("around big word range");
        assert_eq!(buffer.slice(around_big_word), "alpha-beta ");
    }

    #[test]
    fn around_word_ranges_at_line_end_exclude_newline() {
        let buffer = TextBuffer::from_text("alpha beta\ngamma");

        let around = buffer
            .word_range_at(TextPoint::new(0, 7), true, 1)
            .expect("around word range");
        assert_eq!(buffer.slice(around), " beta");
    }

    #[test]
    fn sentence_ranges_cover_inner_and_around_text_objects() {
        let buffer = TextBuffer::from_text("Alpha beta.  Gamma delta!  Last bit?");

        let inner = buffer
            .sentence_range_at(TextPoint::new(0, 15), false, 1)
            .expect("inner sentence range");
        assert_eq!(buffer.slice(inner), "Gamma delta!");

        let around = buffer
            .sentence_range_at(TextPoint::new(0, 15), true, 1)
            .expect("around sentence range");
        assert_eq!(buffer.slice(around), "Gamma delta!  ");

        let counted = buffer
            .sentence_range_at(TextPoint::new(0, 15), false, 2)
            .expect("counted sentence range");
        assert_eq!(buffer.slice(counted), "Gamma delta!  Last bit?");
    }

    #[test]
    fn paragraph_ranges_cover_inner_and_around_text_objects() {
        let buffer = TextBuffer::from_text("one\n\nalpha\nbeta\n\ntwo\n");

        let inner = buffer
            .paragraph_range_at(TextPoint::new(2, 1), false, 1)
            .expect("inner paragraph range");
        assert_eq!(buffer.slice(inner), "alpha\nbeta\n");

        let around = buffer
            .paragraph_range_at(TextPoint::new(2, 1), true, 1)
            .expect("around paragraph range");
        assert_eq!(buffer.slice(around), "alpha\nbeta\n\n");
    }

    #[test]
    fn delimited_ranges_cover_quotes_and_brackets() {
        let buffer = TextBuffer::from_text("call(foo[bar], \"baz\")");

        let inner_parens = buffer
            .delimited_range_at(TextPoint::new(0, 6), '(', ')', false)
            .expect("inner paren range");
        assert_eq!(buffer.slice(inner_parens), "foo[bar], \"baz\"");

        let around_brackets = buffer
            .delimited_range_at(TextPoint::new(0, 9), '[', ']', true)
            .expect("around bracket range");
        assert_eq!(buffer.slice(around_brackets), "[bar]");

        let inner_quotes = buffer
            .delimited_range_at(TextPoint::new(0, 17), '"', '"', false)
            .expect("inner quote range");
        assert_eq!(buffer.slice(inner_quotes), "baz");
    }

    #[test]
    fn delimited_and_tag_ranges_cover_quickref_objects() {
        let buffer = TextBuffer::from_text("foo <bar> baz <div>hello</div>");

        let inner_angle = buffer
            .delimited_range_at(TextPoint::new(0, 5), '<', '>', false)
            .expect("inner angle range");
        assert_eq!(buffer.slice(inner_angle), "bar");

        let around_tag = buffer
            .tag_range_at(TextPoint::new(0, 20), true, Some("html"))
            .expect("around tag range");
        assert_eq!(buffer.slice(around_tag), "<div>hello</div>");

        let inner_tag = buffer
            .tag_range_at(TextPoint::new(0, 20), false, Some("html"))
            .expect("inner tag range");
        assert_eq!(buffer.slice(inner_tag), "hello");

        assert_eq!(
            buffer.tag_range_at(TextPoint::new(0, 20), true, Some("csharp")),
            None
        );
    }

    #[test]
    fn show_paren_at_opening_paren_finds_closing_paren() {
        let buffer = TextBuffer::from_text("call(foo)");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 4), Some("rust"))
            .expect("opening paren match");
        assert_eq!(
            found.origin,
            TextRange::new(TextPoint::new(0, 4), TextPoint::new(0, 5))
        );
        assert_eq!(
            found.counterpart,
            Some(TextRange::new(TextPoint::new(0, 8), TextPoint::new(0, 9)))
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_closing_paren_finds_opening_paren() {
        let buffer = TextBuffer::from_text("call(foo)");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 8), Some("rust"))
            .expect("closing paren match");
        assert_eq!(
            found.origin,
            TextRange::new(TextPoint::new(0, 8), TextPoint::new(0, 9))
        );
        assert_eq!(
            found.counterpart,
            Some(TextRange::new(TextPoint::new(0, 4), TextPoint::new(0, 5)))
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_nested_paren_matches_inner_pair() {
        let buffer = TextBuffer::from_text("call(foo[bar])");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 8), Some("rust"))
            .expect("inner bracket match");
        assert_eq!(buffer.slice(found.origin), "[");
        assert_eq!(
            found.counterpart.map(|range| buffer.slice(range)),
            Some("]".to_owned())
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_unmatched_paren_marks_mismatch() {
        let buffer = TextBuffer::from_text("call(foo");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 4), Some("rust"))
            .expect("unmatched paren");
        assert_eq!(buffer.slice(found.origin), "(");
        assert_eq!(found.counterpart, None);
        assert!(!found.matched);
    }

    #[test]
    fn show_paren_at_ignores_cursor_off_delimiter() {
        let buffer = TextBuffer::from_text("call(foo)");
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 0), Some("rust")),
            None
        );
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 5), Some("rust")),
            None
        );
    }

    #[test]
    fn show_paren_at_less_than_zero_is_not_an_html_tag() {
        let buffer = TextBuffer::from_text(
            "if (x < 0)\n{\n    return;\n}\nprivate async Task Send<T>(T x);",
        );
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 6), Some("csharp")),
            None
        );
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(2, 4), Some("csharp")),
            None
        );
    }

    #[test]
    fn show_paren_at_less_than_identifier_is_not_an_html_tag() {
        let buffer = TextBuffer::from_text("if (x < Foo)\nList<T> y;");
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 6), Some("csharp")),
            None
        );
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(1, 0), Some("csharp")),
            None
        );
    }

    #[test]
    fn show_paren_at_comparison_with_greater_on_same_line_is_not_html_tag() {
        let buffer = TextBuffer::from_text("if (a < b && c > d) {}");
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 6), Some("csharp")),
            None
        );
    }

    #[test]
    fn show_paren_at_html_tag_with_attributes_finds_closing_tag() {
        let buffer = TextBuffer::from_text("<div class=\"x\">hi</div>");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 1), Some("html"))
            .expect("opening tag match");
        assert_eq!(buffer.slice(found.origin), "<div class=\"x\">");
        assert_eq!(
            found.counterpart.map(|range| buffer.slice(range)),
            Some("</div>".to_owned())
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_jsx_tag_with_and_expression_finds_closing_tag() {
        let buffer = TextBuffer::from_text("<div hidden={a && b}>hi</div>");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 1), Some("jsx"))
            .expect("opening tag match");
        assert_eq!(buffer.slice(found.origin), "<div hidden={a && b}>");
        assert_eq!(
            found.counterpart.map(|range| buffer.slice(range)),
            Some("</div>".to_owned())
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_multiline_html_tag_finds_closing_tag() {
        let buffer = TextBuffer::from_text("<div\n  class=\"x\">hi</div>");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 1), Some("html"))
            .expect("opening tag match");
        assert_eq!(buffer.slice(found.origin), "<div\n  class=\"x\">");
        assert_eq!(
            found.counterpart.map(|range| buffer.slice(range)),
            Some("</div>".to_owned())
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_opening_html_tag_finds_closing_tag() {
        let buffer = TextBuffer::from_text("<div>hi</div>");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 1), Some("html"))
            .expect("opening tag match");
        assert_eq!(buffer.slice(found.origin), "<div>");
        assert_eq!(
            found.counterpart.map(|range| buffer.slice(range)),
            Some("</div>".to_owned())
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_closing_html_tag_finds_opening_tag() {
        let buffer = TextBuffer::from_text("<div>hi</div>");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 8), Some("html"))
            .expect("closing tag match");
        assert_eq!(buffer.slice(found.origin), "</div>");
        assert_eq!(
            found.counterpart.map(|range| buffer.slice(range)),
            Some("<div>".to_owned())
        );
        assert!(found.matched);
    }

    #[test]
    fn show_paren_at_html_tag_ignores_inner_content() {
        let buffer = TextBuffer::from_text("<div>hi</div>");
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 5), Some("html")),
            None
        );
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 6), Some("html")),
            None
        );
    }

    #[test]
    fn show_paren_at_nested_html_tags_match_same_name() {
        let buffer = TextBuffer::from_text("<div><span>x</span></div>");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 0), Some("html"))
            .expect("outer open tag");
        assert_eq!(buffer.slice(found.origin), "<div>");
        assert_eq!(
            found.counterpart.map(|range| buffer.slice(range)),
            Some("</div>".to_owned())
        );
        let inner = buffer
            .show_paren_at(TextPoint::new(0, 6), Some("html"))
            .expect("inner open tag");
        assert_eq!(buffer.slice(inner.origin), "<span>");
        assert_eq!(
            inner.counterpart.map(|range| buffer.slice(range)),
            Some("</span>".to_owned())
        );
    }

    #[test]
    fn show_paren_at_self_closing_html_tag_has_no_pair() {
        let buffer = TextBuffer::from_text("<img src=\"x\" />");
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 1), Some("html")),
            None
        );
    }

    #[test]
    fn show_paren_at_unmatched_html_tag_marks_mismatch() {
        let buffer = TextBuffer::from_text("<div>hi");
        let found = buffer
            .show_paren_at(TextPoint::new(0, 0), Some("html"))
            .expect("unmatched tag");
        assert_eq!(buffer.slice(found.origin), "<div>");
        assert_eq!(found.counterpart, None);
        assert!(!found.matched);
    }

    #[test]
    fn move_matching_delimiter_jumps_between_html_tags() {
        let mut buffer = TextBuffer::from_text("<div>hi</div>");
        buffer.set_cursor(TextPoint::new(0, 1));
        assert!(buffer.move_matching_delimiter(Some("html")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 7));
        assert!(buffer.move_matching_delimiter(Some("html")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 0));
    }

    #[test]
    fn move_matching_delimiter_scans_forward_to_paren_on_the_line() {
        let mut buffer = TextBuffer::from_text("call(foo)");
        buffer.set_cursor(TextPoint::new(0, 0));
        assert!(buffer.move_matching_delimiter(Some("rust")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 8));
    }

    #[test]
    fn move_matching_delimiter_scans_forward_to_html_tag_on_the_line() {
        let mut buffer = TextBuffer::from_text("item <div>hi</div>");
        buffer.set_cursor(TextPoint::new(0, 0));
        assert!(buffer.move_matching_delimiter(Some("html")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 12));
    }

    #[test]
    fn move_matching_delimiter_jumps_nested_html_tags() {
        let mut buffer = TextBuffer::from_text("<div><span>x</span></div>");
        buffer.set_cursor(TextPoint::new(0, 0));
        assert!(buffer.move_matching_delimiter(Some("html")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 19));
        buffer.set_cursor(TextPoint::new(0, 6));
        assert!(buffer.move_matching_delimiter(Some("html")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 12));
    }

    #[test]
    fn language_matches_markup_tags_only_html_xml_jsx_tsx() {
        assert!(language_matches_markup_tags(Some("html")));
        assert!(language_matches_markup_tags(Some("xml")));
        assert!(language_matches_markup_tags(Some("jsx")));
        assert!(language_matches_markup_tags(Some("tsx")));
        assert!(!language_matches_markup_tags(Some("csharp")));
        assert!(!language_matches_markup_tags(Some("rust")));
        assert!(!language_matches_markup_tags(None));
    }

    #[test]
    fn show_paren_at_html_tags_ignored_outside_markup_languages() {
        let buffer = TextBuffer::from_text("<div>hi</div>");
        assert_eq!(
            buffer.show_paren_at(TextPoint::new(0, 1), Some("csharp")),
            None
        );
        assert_eq!(buffer.show_paren_at(TextPoint::new(0, 1), None), None);
        let found = buffer
            .show_paren_at(TextPoint::new(0, 1), Some("xml"))
            .expect("xml tags match");
        assert_eq!(buffer.slice(found.origin), "<div>");
        let tsx = buffer
            .show_paren_at(TextPoint::new(0, 1), Some("tsx"))
            .expect("tsx tags match");
        assert_eq!(buffer.slice(tsx.origin), "<div>");
    }

    #[test]
    fn move_matching_delimiter_ignores_html_tags_outside_markup_languages() {
        let mut buffer = TextBuffer::from_text("item <div>hi</div>");
        buffer.set_cursor(TextPoint::new(0, 0));
        assert!(!buffer.move_matching_delimiter(Some("csharp")));
        assert_eq!(buffer.cursor(), TextPoint::new(0, 0));
    }
