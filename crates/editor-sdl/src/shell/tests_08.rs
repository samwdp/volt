#[test]
fn acp_section_layout_orders_output_input_footer_and_statusline() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(
        &mut state,
        40,
        "",
        Some("chat · gpt-5.4 · shift+tab switch mode"),
    )?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let acp_layout = acp_buffer_layout(buffer, rect, layout, 8, 18)
        .ok_or_else(|| "ACP layout missing".to_owned())?;

    assert!(
        acp_layout.output.rect.y() + acp_layout.output.rect.height() as i32
            <= acp_layout.input.rect.y()
    );
    assert!(
        acp_layout.input.rect.y() + acp_layout.input.rect.height() as i32
            <= acp_layout.footer.rect.y()
    );
    assert!(
        acp_layout.footer.rect.y() + acp_layout.footer.rect.height() as i32 <= layout.pane_bottom
    );
    assert_eq!(
        acp_layout.input.rect.height() as i32,
        18 + input_panel_chrome_height()
    );
    Ok(())
}

#[test]
fn browser_input_layout_uses_symmetric_vertical_padding() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let browser_layout = browser_buffer_layout(buffer, rect, layout, 8, 18)
        .ok_or_else(|| "browser layout missing".to_owned())?;

    assert_eq!(
        browser_layout.input.rect.height() as i32,
        18 + input_panel_chrome_height()
    );
    Ok(())
}

#[test]
fn render_browser_input_cursor_uses_rounded_rect_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let cursor_color = Color::RGB(7, 77, 177);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("volt");
        input.cursor = 2;
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let browser_layout = browser_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "browser layout missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_browser_buffer_body(
        &mut target,
        BrowserBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(55, 71, 99, 255),
            cursor: cursor_color,
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    let cursor_color = to_render_color(cursor_color);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x >= browser_layout.input.rect.x()
                && rect.x < browser_layout.input.rect.x() + browser_layout.input.rect.width() as i32
                && rect.y >= browser_layout.input.rect.y()
                && rect.y < browser_layout.input.rect.y() + browser_layout.input.rect.height() as i32
                && rect.width == 8
                && rect.height == 16
                && *color == cursor_color
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x >= browser_layout.input.rect.x()
                && rect.x < browser_layout.input.rect.x() + browser_layout.input.rect.width() as i32
                && rect.y >= browser_layout.input.rect.y()
                && rect.y < browser_layout.input.rect.y() + browser_layout.input.rect.height() as i32
                && rect.width == 8
                && rect.height == 16
                && *color == cursor_color
    )));
    Ok(())
}

#[test]
fn command_line_footer_layout_reserves_row_below_statusline() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 18, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;

    assert!(layout.statusline_y < commandline_y);
    assert_eq!(commandline_y - layout.statusline_y, 26);
    Ok(())
}

#[test]
fn render_buffer_draws_command_line_row_without_active_overlay() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 16, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;
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
                row_visible: true,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
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
                && rect.y == commandline_y - 6
                && rect.width == 304
                && rect.height == 1
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == commandline_y
                && rect.width == 304
                && rect.height == 16
    )));
    Ok(())
}

#[test]
fn render_buffer_draws_show_paren_match_highlight() -> Result<(), String> {
    let match_color = Color::RGBA(12, 34, 56, 128);
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme").with_token(
                TOKEN_SHOW_PAREN_MATCH,
                editor_theme::Color::rgba(12, 34, 56, 128),
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*show-paren*", vec!["call(foo)".to_owned()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 4));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-theme",
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

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { color, .. }
                if *color == to_render_color(match_color)
        )),
        "expected show-paren match highlight, scene={scene:?}"
    );
    Ok(())
}

#[test]
fn render_buffer_draws_show_paren_html_tag_highlight() -> Result<(), String> {
    let match_color = Color::RGBA(9, 8, 7, 120);
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme").with_token(
                TOKEN_SHOW_PAREN_MATCH,
                editor_theme::Color::rgba(9, 8, 7, 120),
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*show-paren-html*",
        vec!["<div>hi</div>".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("html".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 1));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-theme",
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

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { color, .. }
                if *color == to_render_color(match_color)
        )),
        "expected show-paren HTML tag highlight, scene={scene:?}"
    );
    Ok(())
}

#[test]
fn render_buffer_skips_show_paren_html_tag_highlight_for_csharp() -> Result<(), String> {
    let match_color = Color::RGBA(9, 8, 7, 120);
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme").with_token(
                TOKEN_SHOW_PAREN_MATCH,
                editor_theme::Color::rgba(9, 8, 7, 120),
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*show-paren-csharp*",
        vec!["<div>hi</div>".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("csharp".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 1));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-theme",
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

    assert!(
        scene.iter().all(|command| !matches!(
            command,
            DrawCommand::FillRoundedRect { color, .. }
                if *color == to_render_color(match_color)
        )),
        "C# buffers should not highlight HTML tags, scene={scene:?}"
    );
    Ok(())
}

#[test]
fn render_terminal_buffer_path_draws_command_line_separator_without_footer_fill()
-> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            1,
            12,
            vec![editor_terminal::TerminalRenderLine::new(vec![
                editor_terminal::TerminalRenderRun::new(
                    0,
                    11,
                    "echo hello",
                    editor_terminal::TerminalRgb {
                        r: 215,
                        g: 221,
                        b: 232,
                    },
                    None,
                    None,
                ),
            ])],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                0,
                0,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 16, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;
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
                row_visible: true,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
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
                && rect.y == commandline_y - 6
                && rect.width == 304
                && rect.height == 1
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, .. }
            if rect.x == 8
                && rect.y == commandline_y
                && rect.width == 304
                && rect.height == 16
    )));
    Ok(())
}

#[test]
fn render_buffer_uses_theme_commandline_background_token() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme").with_token(
                TOKEN_COMMANDLINE_BACKGROUND,
                editor_theme::Color::rgba(10, 20, 30, 144),
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout_with_command_line(buffer, rect, 16, 8, true);
    let commandline_y = layout
        .commandline_y
        .ok_or_else(|| "command line row is missing".to_owned())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    let mut command_line = CommandLineOverlay::new();
    command_line.append_text("w");
    let command_line_input = command_line.input();
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
                input: Some(command_line_input),
                row_visible: true,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-theme",
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
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == 8
                && rect.y == commandline_y
                && rect.width == 304
                && rect.height == 16
                && *color == to_render_color(Color::RGBA(10, 20, 30, 144))
    )));
    Ok(())
}

#[test]
fn render_buffer_falls_back_to_statusline_theme_tokens_for_text() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    let active_text = Color::RGB(10, 20, 30);
    let inactive_text = Color::RGB(40, 50, 60);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    TOKEN_STATUSLINE_ACTIVE,
                    editor_theme::Color::rgb(active_text.r, active_text.g, active_text.b),
                )
                .with_token(
                    TOKEN_STATUSLINE_INACTIVE,
                    editor_theme::Color::rgb(inactive_text.r, inactive_text.g, inactive_text.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let render_user_library = HeaderlineTestUserLibrary::default();
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);

    let mut active_scene = Vec::new();
    let mut active_target = DrawTarget::Scene(&mut active_scene);
    render_buffer(
        &mut active_target,
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
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

    assert!(active_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(active_text)
    )));

    let mut inactive_scene = Vec::new();
    let mut inactive_target = DrawTarget::Scene(&mut inactive_scene);
    render_buffer(
        &mut inactive_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot {
                rect,
                active: false,
            },
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
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

    assert!(inactive_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(inactive_text)
    )));
    Ok(())
}

#[test]
fn render_buffer_paints_modeline_mode_chip_and_right_aligned_segment() -> Result<(), String> {
    struct ModelineChipTestUserLibrary;

    impl UserLibrary for ModelineChipTestUserLibrary {
        fn modeline_segments(
            &self,
            context: &StatuslineContext<'_>,
        ) -> Vec<editor_plugin_api::ModelineSegment> {
            use editor_plugin_api::{ModelinePart, ModelineSegment};
            vec![
                ModelineSegment::left(vec![ModelinePart::new(
                    format!(" {} ", context.vim_mode),
                    "ui.modeline.mode.normal.foreground",
                    Some("ui.modeline.mode.normal.background".into()),
                )]),
                ModelineSegment::left(vec![ModelinePart::fg(
                    format!("{up} 2", up = editor_icons::symbols::cod::COD_ARROW_UP),
                    "ui.modeline.git.added",
                )]),
                ModelineSegment::right(vec![ModelinePart::fg("RHS", "ui.modeline.muted")]),
            ]
        }
    }

    let mut registry = ThemeRegistry::new();
    let mode_fg = Color::RGB(10, 10, 10);
    let mode_bg = Color::RGB(90, 160, 255);
    let git_added = Color::RGB(50, 200, 80);
    let muted = Color::RGB(120, 120, 130);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(TOKEN_STATUSLINE_ACTIVE, editor_theme::Color::rgb(1, 2, 3))
                .with_token(
                    TOKEN_STATUSLINE_FOREGROUND,
                    editor_theme::Color::rgb(200, 200, 200),
                )
                .with_token(
                    "ui.modeline.mode.normal.foreground",
                    editor_theme::Color::rgb(mode_fg.r, mode_fg.g, mode_fg.b),
                )
                .with_token(
                    "ui.modeline.mode.normal.background",
                    editor_theme::Color::rgb(mode_bg.r, mode_bg.g, mode_bg.b),
                )
                .with_token(
                    "ui.modeline.git.added",
                    editor_theme::Color::rgb(git_added.r, git_added.g, git_added.b),
                )
                .with_token(
                    "ui.modeline.muted",
                    editor_theme::Color::rgb(muted.r, muted.g, muted.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let render_user_library = ModelineChipTestUserLibrary;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let statusline_x = rect.x() + 12;
    let max_width = rect.width().saturating_sub(24);
    let rhs_width = monospace_text_width("RHS", 8);
    let expected_rhs_x = statusline_x + max_width.saturating_sub(rhs_width) as i32;

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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
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

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.y == layout.statusline_y
                    && *color == to_render_color(mode_bg)
        )),
        "expected mode chip background fill"
    );
    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { y, color, text, .. }
                if *y == layout.statusline_y
                    && *color == to_render_color(mode_fg)
                    && text.contains("NORMAL")
        )),
        "expected mode chip foreground text"
    );
    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { y, color, .. }
                if *y == layout.statusline_y && *color == to_render_color(git_added)
        )),
        "expected git added color"
    );
    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { x, y, color, text, .. }
                if *x == expected_rhs_x
                    && *y == layout.statusline_y
                    && *color == to_render_color(muted)
                    && text == "RHS"
        )),
        "expected right-aligned RHS segment"
    );
    Ok(())
}

#[test]
fn render_buffer_paints_opaque_modeline_band_when_window_is_transparent() -> Result<(), String> {
    let _guard = crate::window_effects::force_surface_window_opacity_for_tests();
    let mut registry = ThemeRegistry::new();
    let base_background = Color::RGB(15, 16, 20);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.25)
                .with_token(
                    "ui.background",
                    editor_theme::Color::rgb(
                        base_background.r,
                        base_background.g,
                        base_background.b,
                    ),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
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

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRect { rect: fill, color }
                if fill.x == rect.x()
                    && fill.y == layout.statusline_y
                    && fill.width == rect.width()
                    && fill.height == 16
                    && *color
                        == to_render_color(Color::RGBA(
                            base_background.r,
                            base_background.g,
                            base_background.b,
                            255,
                        ))
        )),
        "modeline band must stay fully opaque when window.opacity < 1"
    );
    Ok(())
}

#[test]
fn render_buffer_uses_statusline_foreground_tokens() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    let active_text = Color::RGB(212, 218, 226);
    let inactive_text = Color::RGB(148, 154, 164);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    TOKEN_STATUSLINE_ACTIVE,
                    editor_theme::Color::rgb(10, 20, 30),
                )
                .with_token(
                    TOKEN_STATUSLINE_INACTIVE,
                    editor_theme::Color::rgb(40, 50, 60),
                )
                .with_token(
                    TOKEN_STATUSLINE_FOREGROUND,
                    editor_theme::Color::rgb(active_text.r, active_text.g, active_text.b),
                )
                .with_token(
                    TOKEN_STATUSLINE_INACTIVE_FOREGROUND,
                    editor_theme::Color::rgb(inactive_text.r, inactive_text.g, inactive_text.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let render_user_library = HeaderlineTestUserLibrary::default();
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);

    let mut active_scene = Vec::new();
    let mut active_target = DrawTarget::Scene(&mut active_scene);
    render_buffer(
        &mut active_target,
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
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

    assert!(active_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(active_text)
    )));

    let mut inactive_scene = Vec::new();
    let mut inactive_target = DrawTarget::Scene(&mut inactive_scene);
    render_buffer(
        &mut inactive_target,
        BufferDrawRequest {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot {
                rect,
                active: false,
            },
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
                row_visible: false,
            },
        },
        BufferChrome {
            user_library: &render_user_library,
            theme_registry: Some(&registry),
            workspace_name: "test-workspace",
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

    assert!(inactive_scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { y, color, .. }
            if *y == layout.statusline_y && *color == to_render_color(inactive_text)
    )));
    Ok(())
}

#[test]
fn render_shell_state_uses_theme_background_for_active_pane() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let base_background = Color::RGB(15, 16, 20);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry: None,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 320,
                height: 180,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 0
                && rect.y == 0
                && rect.width == 320
                && rect.height == 180
                && *color == to_render_color(base_background)
    )));
    Ok(())
}

#[test]
fn render_shell_state_applies_window_opacity_only_to_backgrounds() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    let ui = shell_ui(&state.runtime)?;
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 16,
            emoji_font_size: 16,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        None,
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize {
                width: 320,
                height: 180,
            },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width: 8,
                line_height: 16,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Clear { color } if color.a == 0
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 0
                && rect.y == 0
                && rect.width == 320
                && rect.height == 180
                && color.a == 128
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { color, .. } if color.a == 255
    )));
    Ok(())
}
