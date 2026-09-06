#![allow(unused_imports)]
use super::*;

#[test]
fn draw_buffer_text_keeps_cursor_line_as_one_text_run() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "abc";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: 3,
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "abc".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_expands_tabs_to_spaces() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\tcargo";
    let char_map = LineCharMap::with_tab_width(line, 4);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "    cargo".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_omits_variation_selectors_from_scene_text() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "⚛️";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "⚛".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_renders_escape_controls_as_caret_notation() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\u{1b}[31mSet-PSReadLineOption";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "^[[31mSet-PSReadLineOption".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_omits_byte_order_mark_from_scene_text() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\u{feff}<Project";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 0,
            y: 0,
            text: "<Project".to_owned(),
            color: to_render_color(default_color),
        },]
    );
    Ok(())
}

#[test]
fn draw_buffer_text_skips_lines_that_only_contain_byte_order_marks() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let line = "\u{feff}";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: line.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: None,
            default_color,
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.is_empty());
    Ok(())
}

#[test]
fn draw_buffer_text_keeps_git_status_segments_aligned_with_icon_prefix() -> Result<(), String> {
    let line = SectionRenderLine {
        text: format!(
            "{} Head: master f9d8c15 Added some more keybinds",
            editor_icons::symbols::dev::DEV_GIT_BRANCH
        ),
        depth: 1,
        section_id: GIT_SECTION_HEADERS.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);
    let char_map = LineCharMap::new(&formatted);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        BufferTextRun {
            x: 0,
            y: 0,
            line: &formatted,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: formatted.chars().count(),
            },
            char_map: &char_map,
            line_syntax_spans: Some(&spans),
            default_color: Color::RGB(240, 240, 240),
            cell_width: 8,
        },
        None,
    )
    .map_err(|error| error.to_string())?;

    let text_segments = scene
        .into_iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, .. } => Some(text),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        text_segments,
        vec![
            "  ".to_owned(),
            editor_icons::symbols::dev::DEV_GIT_BRANCH.to_owned(),
            " ".to_owned(),
            "Head:".to_owned(),
            " ".to_owned(),
            "master".to_owned(),
            " ".to_owned(),
            "f9d8c15".to_owned(),
            " ".to_owned(),
            "Added some more keybinds".to_owned(),
        ]
    );
    Ok(())
}
