#[test]
fn apply_pending_lsp_state_toasts_only_when_notification_revision_moves() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    apply_pending_lsp_state(&mut state.runtime)?;
    let before = shell_ui(&state.runtime)?.notification_revision();

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);

    manager.record_show_message("rust-analyzer", "Indexing");
    apply_pending_lsp_state(&mut state.runtime)?;
    let after = shell_ui(&state.runtime)?.notification_revision();
    assert!(after > before);
    let now = Instant::now();
    assert!(
        shell_ui(&state.runtime)?
            .visible_notifications(now)
            .iter()
            .any(|notification| notification.title.contains("rust-analyzer")
                && notification
                    .body_lines
                    .iter()
                    .any(|line| line.contains("Indexing")))
    );

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), after);
    Ok(())
}

#[test]
fn apply_pending_lsp_state_refreshes_attached_server_label_when_session_set_changes()
-> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer", "biome"])?;
    let rust_path = PathBuf::from("src").join("main.rs");
    let biome_path = PathBuf::from("src").join("lib.rs");
    let rust_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-label-rust*",
        &rust_path,
        vec!["fn main() {}".to_owned()],
    )?;
    let biome_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-label-biome*",
        &biome_path,
        vec!["pub fn lib() {}".to_owned()],
    )?;
    manager
        .attach_memory_session("rust-analyzer", &rust_path, Vec::new())
        .map_err(|error| error.to_string())?;
    manager
        .attach_memory_session("biome", &biome_path, Vec::new())
        .map_err(|error| error.to_string())?;

    shell_ui_mut(&mut state.runtime)?.focus_buffer(rust_id);
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_ui(&state.runtime)?.attached_lsp_server(),
        Some("rust-analyzer")
    );

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_ui(&state.runtime)?.attached_lsp_server(),
        Some("rust-analyzer")
    );

    shell_ui_mut(&mut state.runtime)?.focus_buffer(biome_id);
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_ui(&state.runtime)?.attached_lsp_server(),
        Some("biome")
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_does_nothing_without_lsp_enabled_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let path = PathBuf::from("src").join("main.rs");
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        for buffer in &mut ui.buffers {
            buffer.set_lsp_enabled(false);
        }
    }

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(manager.diagnostics_for_path_lookups(), 0);
    Ok(())
}

#[test]
fn saved_theme_selection_round_trips() {
    let dir = unique_temp_dir("theme-save");
    let path = dir.join("active-theme.txt");
    write_saved_theme_selection(&path, "volt-dark")
        .unwrap_or_else(|error| panic!("unexpected save error: {error}"));

    assert_eq!(
        read_saved_theme_selection(&path)
            .unwrap_or_else(|error| panic!("unexpected read error: {error}")),
        Some("volt-dark".to_owned())
    );

    std::fs::remove_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to remove temp dir `{}`: {error}", dir.display()));
}

#[test]
fn restore_saved_theme_selection_activates_saved_theme() {
    let dir = unique_temp_dir("theme-restore");
    let path = dir.join("active-theme.txt");
    write_saved_theme_selection(&path, "amber")
        .unwrap_or_else(|error| panic!("unexpected save error: {error}"));

    let mut registry = ThemeRegistry::new();
    registry
        .register(editor_theme::Theme::new("volt-dark", "Volt Dark"))
        .unwrap_or_else(|error| panic!("unexpected register error: {error}"));
    registry
        .register(editor_theme::Theme::new("amber", "Amber"))
        .unwrap_or_else(|error| panic!("unexpected register error: {error}"));

    restore_saved_theme_selection(&mut registry, &path)
        .unwrap_or_else(|error| panic!("unexpected restore error: {error}"));

    assert_eq!(
        registry.active_theme().map(|theme| theme.id()),
        Some("amber")
    );

    std::fs::remove_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to remove temp dir `{}`: {error}", dir.display()));
}

#[test]
fn restore_saved_theme_selection_clears_unknown_theme() {
    let dir = unique_temp_dir("theme-stale");
    let path = dir.join("active-theme.txt");
    write_saved_theme_selection(&path, "missing-theme")
        .unwrap_or_else(|error| panic!("unexpected save error: {error}"));

    let mut registry = ThemeRegistry::new();
    registry
        .register(editor_theme::Theme::new("gruvbox-dark", "Gruvbox Dark"))
        .unwrap_or_else(|error| panic!("unexpected register error: {error}"));

    let error = restore_saved_theme_selection(&mut registry, &path)
        .expect_err("unknown saved theme should surface an error");
    assert!(error.contains("missing-theme"));
    assert!(!path.exists());
    assert_eq!(
        registry.active_theme().map(|theme| theme.id()),
        Some("gruvbox-dark")
    );

    std::fs::remove_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to remove temp dir `{}`: {error}", dir.display()));
}

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
fn line_char_map_treats_variation_selectors_as_zero_width() {
    let line = "⚛️x";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 3);
    assert_eq!(char_map.display_text_for_range(line, 0, 2), "⚛");
    assert_eq!(
        char_map.display_text_for_range(line, 0, line.chars().count()),
        "⚛x"
    );
}

#[test]
fn line_char_map_treats_byte_order_marks_as_zero_width() {
    let line = "\u{feff}<Project";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 8);
    assert_eq!(
        char_map.display_text_for_range(line, 0, line.chars().count()),
        "<Project"
    );
}

#[test]
fn line_char_map_renders_escape_as_caret_notation() {
    let line = "\u{1b}[31m";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 6);
    assert_eq!(
        char_map.display_text_for_range(line, 0, line.chars().count()),
        "^[[31m"
    );
}

#[test]
fn line_char_map_cursor_anchor_skips_variation_selectors() {
    let line = "⚛️x";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.cursor_anchor_col(0), 0);
    assert_eq!(char_map.cursor_anchor_col(1), 0);
    assert_eq!(char_map.cursor_anchor_col(2), 2);
}

#[test]
fn line_char_map_treats_emoji_as_double_width() {
    let line = "🙂x";
    let char_map = LineCharMap::new(line);

    assert_eq!(char_map.display_cols_between(0, 1), 2);
    assert_eq!(char_map.display_cols_between(0, line.chars().count()), 3);
    assert_eq!(char_map.char_col_for_display_col(0), 0);
    assert_eq!(char_map.char_col_for_display_col(1), 0);
    assert_eq!(char_map.char_col_for_display_col(2), 1);
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
fn monospace_text_width_ignores_variation_selectors() {
    assert_eq!(monospace_text_width("⚛️", 8), 8);
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

#[test]
fn draw_line_ghost_text_for_segment_draws_after_the_last_visible_column() -> Result<(), String> {
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let char_map = LineCharMap::new("a");

    draw_line_ghost_text_for_segment(
        &mut target,
        GhostTextSegmentDraw {
            x: 24,
            y: 8,
            segment: LineWrapSegment {
                start_col: 0,
                end_col: 1,
            },
            char_map: &char_map,
            line_len: 1,
            ghost_text: Some(" render(value: usize)"),
            color: Color::RGB(140, 144, 152),
            cell_width: 8,
        },
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Text {
            x: 40,
            y: 8,
            text: " render(value: usize)".to_owned(),
            color: to_render_color(Color::RGB(140, 144, 152)),
        }]
    );
    Ok(())
}

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
fn visible_headerline_lines_keeps_innermost_contexts_when_space_is_limited() {
    let lines = [
        "module app".to_owned(),
        "impl Demo".to_owned(),
        "render(value: usize)".to_owned(),
    ];
    assert_eq!(
        visible_headerline_lines(&lines, 3),
        vec!["impl Demo", "render(value: usize)"]
    );
}

#[test]
fn visible_headerline_lines_reserves_at_least_one_buffer_row() {
    let lines = ["render()".to_owned()];
    assert!(visible_headerline_lines(&lines, 1).is_empty());
    assert_eq!(visible_headerline_row_count(&lines, 1), 0);
}

#[test]
fn render_buffer_headerline_reserves_rows_above_buffer_body() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary::default();
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline-scrolloff*",
        vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.scroll_row = 1;
        buffer.set_cursor(TextPoint::new(1, 1));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: state.runtime.services().get::<ThemeRegistry>(),
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. } if *y == layout.body_y + 16 && text == "beta"
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. } if *y == layout.body_y && text == "beta"
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_keeps_cursor_below_sticky_row() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary::default();
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline*",
        vec!["abcdefghijklmnopqrstuvwxyz".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 25));

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let cursor_color = to_render_color(Color::RGB(110, 170, 255));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y + 16 && text == "abcdefghijklmnopqrstuvwxyz"
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text == "fn render(value: usize)"
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.y == layout.body_y + 16 && *color == cursor_color
    )));
    let headerline_index = scene
        .iter()
        .position(|command| {
            matches!(
                command,
                DrawCommand::Text { y, text, .. }
                    if *y == layout.body_y && text == "fn render(value: usize)"
            )
        })
        .ok_or_else(|| "missing headerline draw".to_owned())?;
    let cursor_index = scene
        .iter()
        .position(|command| {
            matches!(
                command,
                DrawCommand::FillRoundedRect { rect, color, .. }
                    if rect.y == layout.body_y + 16 && *color == cursor_color
            )
        })
        .ok_or_else(|| "missing cursor draw".to_owned())?;
    assert!(cursor_index > headerline_index);
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.y == layout.body_y && *color == cursor_color
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_truncates_preserving_tail_scope() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec!["abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz".to_owned()],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec!["abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz".to_owned()],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*headerline-gap*", vec!["alpha".to_owned()])?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let cursor_color = to_render_color(Color::RGB(110, 170, 255));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text.starts_with("...")
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.y == layout.body_y + 16 && *color == cursor_color
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_divider_sits_below_last_headerline_row() -> Result<(), String> {
    let render_user_library = HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec![
            "module app".to_owned(),
            "fn render(value: usize)".to_owned(),
        ],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 1.0,
        headerline_lines: vec![
            "module app".to_owned(),
            "fn render(value: usize)".to_owned(),
        ],
        headerline_requires_scrolled_viewport: false,
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*headerline-divider*", vec!["alpha".to_owned()])?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_buffer(
        &mut target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == layout.body_y + (2 * 16) - 1
                && rect.width == 304
                && rect.height == 1
    )));
    Ok(())
}

#[test]
fn render_buffer_headerline_only_activates_once_scope_header_leaves_viewport() -> Result<(), String>
{
    let render_user_library = HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: vec!["STICKY HEADER".to_owned()],
        headerline_requires_scrolled_viewport: true,
        ..HeaderlineTestUserLibrary::default()
    };
    let user_library: Arc<dyn UserLibrary> = Arc::new(HeaderlineTestUserLibrary {
        scrolloff: 3.0,
        headerline_lines: vec!["STICKY HEADER".to_owned()],
        headerline_requires_scrolled_viewport: true,
        ..HeaderlineTestUserLibrary::default()
    });
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline-activation*",
        vec![
            "scope header".to_owned(),
            "body line".to_owned(),
            "return 'a'".to_owned(),
        ],
    )?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.scroll_row = 0;
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    let mut hidden_scope_scene = Vec::new();
    let mut hidden_scope_target = DrawTarget::Scene(&mut hidden_scope_scene);
    render_buffer(
        &mut hidden_scope_target,
        BufferDrawRequest {
            buffer: shell_buffer(&state.runtime, buffer_id)?,
            view_state: (shell_buffer(&state.runtime, buffer_id)?).view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;
    assert!(!hidden_scope_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, .. } if text == "STICKY HEADER"
    )));

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.scroll_row = 1;
    }
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut sticky_scene = Vec::new();
    let mut sticky_target = DrawTarget::Scene(&mut sticky_scene);
    render_buffer(
        &mut sticky_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            decorations: BufferDecorations {
                visual_selection: None,
                yank_flash: None,
                input_mode: InputMode::Normal,
                multicursor: None,
                vim_targets_input: false,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: render_user_library.commandline_enabled(),
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
            git_summary: None,
        },
        TextMetrics {
            cell_width: 8,
            line_height: 16,
            ascent: 12,
        },
    )
    .map_err(|error| error.to_string())?;
    assert!(sticky_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, text, .. }
            if *y == layout.body_y && text == "STICKY HEADER"
    )));
    Ok(())
}

#[test]
fn ensure_visible_scrolloff_keeps_cursor_off_bottom_edge() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*scrolloff-bottom*",
        (0..30).map(|index| format!("line {index}")).collect(),
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_viewport_lines(10);
        buffer.set_cursor(TextPoint::new(8, 0));
        buffer.ensure_visible(10, 80, 4, 0, 3);
    }

    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.scroll_row, 2);
    Ok(())
}

#[test]
fn ensure_visible_scrolloff_keeps_cursor_off_top_edge() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*scrolloff-top*",
        (0..30).map(|index| format!("line {index}")).collect(),
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_viewport_lines(10);
        buffer.scroll_row = 5;
        buffer.set_cursor(TextPoint::new(6, 0));
        buffer.ensure_visible(10, 80, 4, 0, 3);
    }

    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.scroll_row, 3);
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
fn worker_syntax_window_matches_visible_window() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*worker-syntax-window*",
        (0..600).map(|index| format!("line {index}")).collect(),
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_language_id(Some("rust".to_owned()));
    buffer.set_viewport_lines(12);
    buffer.scroll_row = 80;

    let desired = buffer
        .desired_syntax_window()
        .ok_or_else(|| "visible syntax window should exist".to_owned())?;
    let worker = buffer
        .worker_syntax_window()
        .ok_or_else(|| "worker syntax window should exist".to_owned())?;
    assert_eq!(worker, desired);
    Ok(())
}

#[test]
fn one_line_scroll_marks_visible_syntax_window_dirty() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*scroll-syntax-window*",
        (0..600).map(|index| format!("line {index}")).collect(),
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_language_id(Some("rust".to_owned()));
    buffer.set_viewport_lines(40);
    buffer.scroll_row = 200;
    let window = buffer
        .desired_syntax_window()
        .ok_or_else(|| "visible syntax window should exist".to_owned())?;
    buffer.set_indexed_syntax_lines(Some(BTreeMap::new()), Some(window));
    buffer.ensure_visible_syntax_window();
    assert!(
        !buffer.syntax_dirty,
        "applied window should cover the current visible window"
    );

    buffer.scroll_row = 201;
    buffer.ensure_visible_syntax_window();
    assert!(
        buffer.syntax_dirty,
        "one-line j/k scroll should request a new syntax window"
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

fn assert_wrap_cache_matches_cold_build(
    buffer: &ShellBuffer,
    wrap_cols: usize,
    indent_size: usize,
) -> Result<(), String> {
    let cache = buffer
        .wrap_cache
        .as_ref()
        .ok_or_else(|| "wrap cache was missing".to_owned())?;
    let cold = WrapRowCache::build(buffer, wrap_cols, indent_size);
    assert_eq!(cache.line_count, cold.line_count);
    assert_eq!(cache.prefix_rows, cold.prefix_rows);
    assert_eq!(cache.wrap_cols, wrap_cols);
    assert_eq!(cache.indent_size, indent_size);
    Ok(())
}
