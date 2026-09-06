#![allow(unused_imports)]
use super::*;

#[test]
fn draw_line_ghost_text_for_segment_skips_non_terminal_wrap_segments() -> Result<(), String> {
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let char_map = LineCharMap::new("alpha beta");

    draw_line_ghost_text_for_segment(
        &mut target,
        GhostTextSegmentDraw {
            x: 0,
            y: 0,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: 10,
            },
            char_map: &char_map,
            line_len: 24,
            ghost_text: Some("hidden"),
            color: Color::RGB(140, 144, 152),
            cell_width: 8,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.is_empty());
    Ok(())
}

#[test]
fn ensure_visible_builds_wrap_cache_for_large_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let lines = (0..(LARGE_BUFFER_WRAP_CACHE_LINE_THRESHOLD + 2))
        .map(|index| {
            if index % 7 == 0 {
                "abcdef".to_owned()
            } else {
                "abcde".to_owned()
            }
        })
        .collect();
    let buffer_id = install_text_test_buffer(&mut state, "*large-wrap-cache*", lines)?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_viewport_lines(20);
    buffer.set_cursor(TextPoint::new(10, 0));
    buffer.ensure_visible(20, 5, 4, 0, 0);

    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was not built for large buffer".to_owned())?;
    assert_eq!(
        cache.max_scroll_row(20),
        buffer.max_scroll_row_for_wrapped_rows(20, 5, 4)
    );
    Ok(())
}

#[test]
fn single_line_insert_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*single-line-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    let cache = WrapRowCache::build(buffer, 5, 4);
    buffer.wrap_cache = Some(cache);
    buffer.set_cursor(TextPoint::new(0, 5));

    buffer.insert_text("f");

    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after single-line insert".to_owned())?;
    assert_eq!(cache.prefix_rows, vec![0, 2, 3]);
    Ok(())
}

#[test]
fn insert_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*newline-wrap-edit*",
        vec![
            "abcde".to_owned(),
            "    wrappedtail".to_owned(),
            "end".to_owned(),
        ],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 3));

    buffer.insert_text("\n");

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)?;
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after newline insert".to_owned())?;
    assert_eq!(cache.prefix_rows.len(), 5);
    Ok(())
}

#[test]
fn join_lines_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*join-wrap-edit*",
        vec!["abcde".to_owned(), "fghij".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(1, 0));

    buffer.backspace();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)?;
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was cleared after join".to_owned())?;
    assert_eq!(cache.prefix_rows.len(), 3);
    Ok(())
}

#[test]
fn delete_forward_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*delete-newline-wrap-edit*",
        vec!["abcde".to_owned(), "fghij".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 5));

    buffer.delete_forward();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn newline_insert_does_not_create_wrap_cache() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*newline-no-wrap-cache*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = None;
    buffer.set_cursor(TextPoint::new(0, 2));

    buffer.insert_text("\n");

    assert!(
        buffer.wrap_cache.is_none(),
        "newline must not create a wrap cache by itself"
    );
    Ok(())
}

#[test]
fn replace_mode_newline_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*replace-newline-wrap-edit*",
        vec!["hello".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 2));

    buffer.replace_mode_text("\n");

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn open_line_below_updates_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*open-line-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 1));

    buffer.open_line_below();

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn same_line_replace_keeps_wrap_cache_prefix_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*replace-range-wrap-edit*",
        vec!["hello".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));

    buffer.replace_range(
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 0)),
        "    ",
    );

    assert_wrap_cache_matches_cold_build(buffer, 8, 4)
}

#[test]
fn undo_newline_wrap_cache_matches_cold_rebuild() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*undo-newline-wrap-edit*",
        vec!["abcde".to_owned(), "tail".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.wrap_cache = Some(WrapRowCache::build(buffer, 8, 4));
    buffer.set_cursor(TextPoint::new(0, 3));
    buffer.insert_text("\n");
    buffer.record_undo_snapshot();
    buffer.undo();

    assert_eq!(buffer.line_count(), 2);
    match buffer.wrap_cache.as_ref() {
        None => {}
        Some(_) => assert_wrap_cache_matches_cold_build(buffer, 8, 4)?,
    }
    Ok(())
}

#[test]
fn wrap_line_segments_keeps_unbroken_words_together() {
    let segments = wrap_line_segments(&LineCharMap::new("alpha betagamma delta"), 10, 10);

    assert_eq!(
        segments
            .into_iter()
            .map(|segment| (segment.start_col, segment.end_col))
            .collect::<Vec<_>>(),
        vec![(0, 6), (6, 16), (16, 21)]
    );
}

#[test]
fn input_field_wrap_keeps_words_intact() {
    let mut input = InputField::new("> ");
    input.set_text("prefix text Please see the screenshot of this input");
    let rows = input.wrapped_visual_rows(28);

    assert!(
        !rows.iter().any(|row| row == "Pl" || row == "ease"),
        "rows: {rows:?}"
    );
    assert!(
        rows.windows(2)
            .all(|pair| { !(pair[0].ends_with("Pl") && pair[1].starts_with("ease")) }),
        "rows: {rows:?}"
    );
}

#[test]
fn wrap_columns_shrink_when_debug_fringe_widens() {
    let idle = wrap_columns_for_width_with_fringe(320, 8, 1);
    let live = wrap_columns_for_width_with_fringe(320, 8, 2);
    assert!(live < idle);
}
