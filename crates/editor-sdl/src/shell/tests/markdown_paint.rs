#![allow(unused_imports)]
use super::*;

#[test]
fn sync_visible_buffer_layouts_counts_markdown_pretty_image_rows_for_scrolloff()
-> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: Vec::new(),
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let png = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==";
    let mut text = format!("![red](data:image/png;base64,{png})\n");
    for index in 1..80 {
        text.push_str(&format!("line {index}\n"));
    }
    let buffer_id = install_markdown_test_buffer(&mut state, "*pretty-image-scrolloff*", &text)?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(4, 0));

    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, render_width, render_height);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let wrap_cols = wrap_columns_for_width(render_width, cell_width);
    let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
    let pretty_paint = markdown_pretty_paint_plan(
        buffer,
        &*user_library,
        MarkdownPrettyPaintArgs {
            visible_start: 0,
            visible_end: buffer.line_count().max(1),
            visual_selection: None,
            input_mode: InputMode::Normal,
            pane_width_px: text_width_px,
            line_height,
        },
    );
    let image_rows = pretty_paint
        .images
        .get(&0)
        .map(|image| image.rows())
        .ok_or_else(|| "pretty image did not decode for scroll fixture".to_owned())?;
    assert!(
        image_rows > 1,
        "fixture image should occupy multiple visual rows, got {image_rows}"
    );
    let expected_scrolloff = 3usize.min(layout.visible_rows.saturating_sub(1) / 2);
    assert!(expected_scrolloff > 1);
    let cursor_body_row = pretty_cursor_body_row(
        buffer,
        rect,
        &*user_library,
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
    )
    .ok_or_else(|| "cursor went off screen before scrolloff".to_owned())?;
    assert!(
        cursor_body_row >= expected_scrolloff,
        "cursor visual row {cursor_body_row} is above scrolloff {expected_scrolloff}"
    );
    assert!(
        cursor_body_row
            <= layout
                .visible_rows
                .saturating_sub(1)
                .saturating_sub(expected_scrolloff),
        "cursor visual row {cursor_body_row} is below scrolloff in {} visible rows",
        layout.visible_rows
    );
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_reuses_plan_for_same_revision() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-cache-revision*", PRETTY_CACHE_FIXTURE)?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let args = markdown_pretty_paint_args(buffer);
    let first = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let first_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing cached plan after first paint")?;
    let second = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let second_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing cached plan after second paint")?;
    assert!(
        std::sync::Arc::ptr_eq(&first_plan, &second_plan),
        "same revision should reuse MarkdownPrettyPlan"
    );
    assert_eq!(first.text_overrides, second.text_overrides);
    let heading = first
        .text_overrides
        .get(&0)
        .ok_or("heading Pretty override missing")?;
    assert!(
        heading.contains("Title") && !heading.starts_with("# "),
        "heading should conceal markers: {heading:?}"
    );
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_rebuilds_after_edit() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-cache-edit*", PRETTY_CACHE_FIXTURE)?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let before_plan = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let args = markdown_pretty_paint_args(buffer);
        let paint = markdown_pretty_paint_plan(buffer, &*user_library, args);
        let plan = markdown_pretty::last_cached_pretty_plan(buffer)
            .ok_or("missing cached plan before edit")?;
        (paint.text_overrides, plan)
    };
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 7));
        buffer.insert_text("!");
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let args = markdown_pretty_paint_args(buffer);
    let after = markdown_pretty_paint_plan(buffer, &*user_library, args);
    let after_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing cached plan after edit")?;
    assert!(!std::sync::Arc::ptr_eq(&before_plan.1, &after_plan));
    assert_ne!(before_plan.0, after.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_cursor_anti_conceal_uses_source() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*pretty-anti-conceal-cursor*",
        "# Title\n- item\n",
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    assert!(
        !paint.text_overrides.contains_key(&0),
        "cursor line should paint Markdown Raw: {:?}",
        paint.text_overrides
    );
    assert!(
        paint.text_overrides.contains_key(&1),
        "non-cursor Pretty lines should still override: {:?}",
        paint.text_overrides
    );
    let plan = markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing cached plan")?;
    let reused =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let reused_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing reused plan")?;
    assert!(std::sync::Arc::ptr_eq(&plan, &reused_plan));
    assert_eq!(paint.text_overrides, reused.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_visual_anti_conceal_then_restores() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*pretty-anti-conceal-visual*",
        PRETTY_CACHE_FIXTURE,
    )?;
    park_cursor_on_plain_pretty_line(&mut state, buffer_id)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let visual = VisualSelection::Range(TextRange::new(TextPoint::new(0, 0), TextPoint::new(1, 6)));
    let visual_args = MarkdownPrettyPaintArgs {
        visual_selection: Some(visual),
        input_mode: InputMode::Visual,
        ..markdown_pretty_paint_args(buffer)
    };
    let visual_paint = markdown_pretty_paint_plan(buffer, &*user_library, visual_args);
    assert!(
        !visual_paint.text_overrides.contains_key(&0),
        "Visual selection should paint Markdown Raw: {:?}",
        visual_paint.text_overrides
    );
    assert!(
        !visual_paint.text_overrides.contains_key(&1),
        "Visual selection should paint Markdown Raw on selected lines: {:?}",
        visual_paint.text_overrides
    );
    let visual_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing plan during visual")?;
    let normal_paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let normal_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing plan after visual")?;
    assert!(std::sync::Arc::ptr_eq(&visual_plan, &normal_plan));
    assert!(normal_paint.text_overrides.contains_key(&0));
    assert!(normal_paint.text_overrides.contains_key(&1));
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_toggle_off_is_raw() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-toggle-off*", "# Title\n- item\n")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.toggle_markdown_pretty(true);
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let paint =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    assert!(paint.text_overrides.is_empty());
    assert!(paint.images.is_empty());
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_kill_switch_skips() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        markdown_pretty: MarkdownPrettyConfig {
            kill_switch_enabled: true,
            kill_switch_max_lines: 0,
            ..MarkdownPrettyConfig::default()
        },
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_markdown_test_buffer(&mut state, "*pretty-kill-switch*", "# Title\n- item\n")?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let first =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let first_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing kill-switch sentinel")?;
    assert!(first.text_overrides.is_empty());
    assert!(first_plan.skipped_by_kill_switch);
    let second =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let second_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing reused sentinel")?;
    assert!(std::sync::Arc::ptr_eq(&first_plan, &second_plan));
    assert_eq!(first.text_overrides, second.text_overrides);
    Ok(())
}

#[test]
fn markdown_pretty_paint_plan_forced_language_caches() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*pretty-forced-language*")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "# Title".to_owned(),
            "- item".to_owned(),
            "plain".to_owned(),
        ]);
        buffer.set_forced_language_id("markdown");
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let first =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let first_plan =
        markdown_pretty::last_cached_pretty_plan(buffer).ok_or("missing Forced Language plan")?;
    let second =
        markdown_pretty_paint_plan(buffer, &*user_library, markdown_pretty_paint_args(buffer));
    let second_plan = markdown_pretty::last_cached_pretty_plan(buffer)
        .ok_or("missing reused Forced Language plan")?;
    assert!(std::sync::Arc::ptr_eq(&first_plan, &second_plan));
    assert_eq!(first.text_overrides, second.text_overrides);
    assert!(
        first
            .text_overrides
            .get(&0)
            .is_some_and(|line| line.contains("Title") && !line.starts_with("# ")),
        "Forced Language markdown should Pretty: {:?}",
        first.text_overrides
    );
    Ok(())
}

#[test]
fn hover_tab_shortcut_beats_markdown_table_navigation_and_allows_scroll() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*hover-markdown-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
    let cursor_before = shell_buffer(&state.runtime, buffer_id)?.cursor_point();
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, false)?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(state.hover_focused().map_err(|error| error.to_string())?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        cursor_before
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 1);
    Ok(())
}

#[test]
fn markdown_table_detection_requires_markdown_and_a_delimiter_row() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let markdown = install_markdown_test_buffer(
        &mut state,
        "*markdown-table*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    let malformed = install_markdown_test_buffer(
        &mut state,
        "*markdown-malformed*",
        "| Header 1 | Header 2 |\n| nope | nope |\n| Some text | Some more text |",
    )?;
    let scratch = install_scratch_test_buffer(&mut state, "*not-markdown*")?;
    shell_buffer_mut(&mut state.runtime, scratch)?.replace_with_lines(vec![
        "| Header 1 | Header 2 |".to_owned(),
        "| --- | --- |".to_owned(),
    ]);

    let table =
        detect_markdown_table(shell_buffer(&state.runtime, markdown)?).ok_or("table missing")?;
    assert_eq!(table.start_line, 0);
    assert_eq!(table.column_count, 2);
    assert_eq!(table.rows.len(), 3);
    assert!(table.rows[1].is_delimiter);
    assert!(detect_markdown_table(shell_buffer(&state.runtime, malformed)?).is_none());
    assert!(detect_markdown_table(shell_buffer(&state.runtime, scratch)?).is_none());
    Ok(())
}

#[test]
fn markdown_table_typing_auto_aligns_and_bootstraps_delimiter_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-align*",
        "| Header 1 | Header 2 |\n| -- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 3));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    state
        .handle_text_input("-")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(0).as_deref(),
        Some("| Header 1  | Header 2       |")
    );
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("| --------- | -------------- |")
    );
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text | Some more text |")
    );
    Ok(())
}

#[test]
fn markdown_table_enter_inserts_a_new_row() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-enter*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(3).as_deref(),
        Some("|           |                |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 2));
    Ok(())
}

#[test]
fn markdown_table_preserves_insert_mode_spaces() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-space*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 11));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input(" ")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text  | Some more text |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 12));
    let _ = buffer;

    state
        .handle_text_input("m")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text m | Some more text |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 13));
    Ok(())
}

#[test]
fn markdown_table_insert_tab_adds_a_column_across_the_table() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 14));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(0).as_deref(),
        Some("| Header 1  | Header 2       |   |")
    );
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("| --------- | -------------- | --- |")
    );
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text | Some more text |   |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 31));
    Ok(())
}

#[test]
fn markdown_table_normal_tab_moves_between_columns() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-normal-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 14)
    );
    Ok(())
}
