#![allow(unused_imports)]
use super::*;

#[test]
fn terminal_key_for_event_maps_special_keys() {
    assert_eq!(
        terminal_key_for_event(Keycode::Tab, Mod::LSHIFTMOD),
        Some(TerminalKey::BackTab)
    );
    assert_eq!(
        terminal_key_for_event(Keycode::Return2, Mod::NOMOD),
        Some(TerminalKey::Enter)
    );
    assert_eq!(
        terminal_key_for_event(Keycode::C, ctrl_mod()),
        Some(TerminalKey::CtrlC)
    );
    assert_eq!(
        terminal_key_for_event(Keycode::PageDown, Mod::NOMOD),
        Some(TerminalKey::PageDown)
    );
}

#[test]
fn terminal_buffers_are_read_only_without_prompt_input() {
    let (read_only, input) = buffer_interaction(&BufferKind::Terminal, &NullUserLibrary);
    assert!(read_only);
    assert!(input.is_none());
}

#[test]
fn terminal_placeholder_lines_describe_shell_launch_not_vertical_slice() {
    let lines = placeholder_lines("*terminal*", &BufferKind::Terminal, &NullUserLibrary);
    let body = lines.join("\n");

    assert!(body.contains("*terminal* is launching the configured shell."));
    assert!(body.contains("Press i to enter terminal input mode"));
    assert!(!body.contains("vertical slice"));
    assert!(!body.contains("compiled terminal package"));
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
fn terminal_scroll_for_motion_maps_terminal_viewport_navigation() {
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::Down, None),
        Some(TerminalViewportScroll::LineDelta(-1))
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::Up, Some(3)),
        Some(TerminalViewportScroll::LineDelta(3))
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::FirstLine, Some(42)),
        Some(TerminalViewportScroll::Top)
    );
    assert_eq!(
        terminal_scroll_for_motion(ShellMotion::LastLine, None),
        Some(TerminalViewportScroll::Bottom)
    );
    assert_eq!(terminal_scroll_for_motion(ShellMotion::Left, None), None);
}

#[test]
fn terminal_mode_insert_hook_allows_reentering_insert_for_terminals() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_INSERT, HookEvent::new())
        .map_err(|error| error.to_string())?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn terminal_mode_normal_hook_uses_live_terminal_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_viewport_lines(2);
        buffer.replace_with_lines_follow_output(vec![
            "zero".to_owned(),
            "one".to_owned(),
            "two".to_owned(),
            "three456".to_owned(),
        ]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            8,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        3,
                        "two",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        8,
                        "three456",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                1,
                5,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 5));
    Ok(())
}

#[test]
fn terminal_popup_mode_normal_hook_uses_live_terminal_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_popup_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal popup test buffer missing".to_owned())?;
        buffer.set_viewport_lines(2);
        buffer.replace_with_lines_follow_output(vec![
            "zero".to_owned(),
            "one".to_owned(),
            "two".to_owned(),
            "three456".to_owned(),
        ]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            8,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        3,
                        "two",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        8,
                        "three456",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
            Some(editor_terminal::TerminalCursorSnapshot::new(
                1,
                4,
                1,
                editor_terminal::TerminalCursorShape::Beam,
                "e",
            )),
            None,
        ));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 4));
    Ok(())
}

#[test]
fn popup_terminal_event_context_prefers_popup_buffer_when_focused() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let pane_buffer = install_scratch_test_buffer(&mut state, "*popup-pane*")?;
    let popup_buffer = install_terminal_popup_test_buffer(&mut state)?;

    let context = active_buffer_event_context(&state.runtime)?;
    assert_eq!(context.buffer_id, popup_buffer);
    assert!(context.is_terminal);
    assert_ne!(context.buffer_id, pane_buffer);

    shell_ui_mut(&mut state.runtime)?.set_popup_focus(false);

    let context = active_buffer_event_context(&state.runtime)?;
    assert_eq!(context.buffer_id, pane_buffer);
    assert!(!context.is_terminal);
    Ok(())
}

#[test]
fn terminal_put_shortcuts_paste_yanks_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
    {
        let vim = shell_ui_mut(&mut state.runtime)?.vim_mut();
        vim.active_register = Some('a');
        vim.registers.insert(
            'a',
            YankRegister::Character("volt terminal paste".to_owned()),
        );
    }

    assert!(handle_terminal_vim_edit(
        &mut state.runtime,
        VimEditAction::PutAfter
    )?);
    assert!(terminal_buffer_state(&state.runtime)?.contains(buffer_id));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert_eq!(shell_ui(&state.runtime)?.vim().pending, None);
    Ok(())
}

#[test]
fn terminal_popup_bootstraps_session_and_enters_insert_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_popup_test_buffer(&mut state)?;

    let popup = state
        .runtime_popup()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;

    assert_eq!(popup.active_buffer, buffer_id);
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    assert!(terminal_buffer_state(&state.runtime)?.contains(buffer_id));
    Ok(())
}

#[test]
fn terminal_popup_command_focuses_the_popup_surface() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let pane_buffer = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("terminal.popup")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, Some(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, popup.active_buffer);
    assert_ne!(popup.active_buffer, pane_buffer);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(terminal_buffer_state(&state.runtime)?.contains(popup.active_buffer));
    Ok(())
}

#[test]
fn dismissed_popup_toggle_restores_terminal_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    state
        .runtime
        .execute_command("terminal.popup")
        .map_err(|error| error.to_string())?;

    let first_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;
    let terminal_buffer = first_popup.active_buffer;
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));

    state
        .runtime
        .execute_command("picker.toggle-popup-window")
        .map_err(|error| error.to_string())?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    assert_eq!(shell_ui(&state.runtime)?.popup_buffer_id, None);
    assert!(shell_buffer(&state.runtime, terminal_buffer).is_ok());
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));

    state
        .runtime
        .execute_command("picker.toggle-popup-window")
        .map_err(|error| error.to_string())?;

    let restored_popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "terminal popup was not restored".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(restored_popup.active_buffer, terminal_buffer);
    assert_eq!(ui.popup_buffer_id, Some(terminal_buffer));
    assert!(ui.popup_focus);
    assert!(terminal_buffer_state(&state.runtime)?.contains(terminal_buffer));
    Ok(())
}

#[test]
fn render_terminal_buffer_prefers_terminal_render_snapshot() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            12,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
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
                ]),
                editor_terminal::TerminalRenderLine::new(vec![]),
            ],
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
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    let rendered_text = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(rendered_text.contains(&"echo hello"));
    assert!(
        !rendered_text
            .iter()
            .any(|text| text.contains("launching the configured shell"))
    );
    assert!(
        scene
            .iter()
            .any(|command| matches!(command, DrawCommand::FillRoundedRect { .. }))
    );
    Ok(())
}

#[test]
fn terminal_box_drawing_chars_render_as_strokes() -> Result<(), String> {
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    let color = Color::RGB(200, 205, 210);

    draw_terminal_text_run(&mut target, 10, 20, "a│b", color, 8, 16)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        scene
            .iter()
            .filter_map(|command| match command {
                DrawCommand::Text { text, .. } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec!["a", "b"]
    );
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 21
                && rect.y == 20
                && rect.height == 16
                && *color == to_render_color(Color::RGB(200, 205, 210))
    )));
    Ok(())
}

#[test]
fn render_terminal_buffer_keeps_terminal_content_opaque_with_window_opacity() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let terminal_background = editor_terminal::TerminalRgb {
        r: 24,
        g: 36,
        b: 48,
    };
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec!["echo hello".to_owned()]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            1,
            12,
            vec![editor_terminal::TerminalRenderLine::new(vec![
                editor_terminal::TerminalRenderRun::new(
                    0,
                    10,
                    "echo hello",
                    editor_terminal::TerminalRgb {
                        r: 215,
                        g: 221,
                        b: 232,
                    },
                    Some(terminal_background),
                    None,
                ),
            ])],
            None,
            None,
        ));
    }

    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Insert,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: Some(&registry),
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == 12
                && rect.y == layout.body_y
                && rect.width == 80
                && rect.height == 16
                && *color == to_render_color(Color::RGB(24, 36, 48))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.y == layout.statusline_y - 6
                && rect.height == 1
                && color.a == 255
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { text, color, .. }
            if text == "echo hello" && color.a == 255
    )));
    Ok(())
}

#[test]
fn render_terminal_buffer_uses_buffer_cursor_in_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let cursor_color = Color::RGB(110, 170, 255);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec![
            "echo hello".to_owned(),
            "second line".to_owned(),
        ]);
        buffer.set_cursor(TextPoint::new(1, 2));
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            2,
            12,
            vec![
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        10,
                        "echo hello",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
                editor_terminal::TerminalRenderLine::new(vec![
                    editor_terminal::TerminalRenderRun::new(
                        0,
                        11,
                        "second line",
                        editor_terminal::TerminalRgb {
                            r: 215,
                            g: 221,
                            b: 232,
                        },
                        None,
                        None,
                    ),
                ]),
            ],
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
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let text_x = rect.x() + 12;
    let expected_x = text_x + 2 * 8;
    let expected_y = layout.body_y + 16;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Normal,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: cursor_color,
            cursor_roundness: 2,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == expected_x
                && rect.y == expected_y
                && *color == to_render_color(cursor_color)
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == text_x
                && rect.y == layout.body_y
                && *color == to_render_color(cursor_color)
    )));
    Ok(())
}

#[test]
fn render_terminal_buffer_uses_editor_insert_cursor_style() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let cursor_color = Color::RGB(110, 170, 255);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec!["echo hello".to_owned()]);
        buffer.set_terminal_render(editor_terminal::TerminalRenderSnapshot::new(
            1,
            12,
            vec![editor_terminal::TerminalRenderLine::new(vec![
                editor_terminal::TerminalRenderRun::new(
                    0,
                    10,
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
                3,
                1,
                editor_terminal::TerminalCursorShape::Block,
                "o",
            )),
            None,
        ));
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "terminal test buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let text_x = rect.x() + 12;
    let expected_x = text_x + 3 * 8;
    let expected_y = layout.body_y;
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_terminal_buffer(
        &mut target,
        TerminalBufferDraw {
            buffer,
            terminal_render: buffer
                .terminal_render()
                .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
            rect,
            layout,
            active: true,
            input_mode: InputMode::Insert,
            visual_selection: None,
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: cursor_color,
            cursor_roundness: 4,
        },
        TerminalStatusline {
            text: "status".to_owned(),
            active: Color::RGB(110, 170, 255),
            inactive: Color::RGB(140, 144, 152),
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, radius }
            if rect.x == expected_x
                && rect.y == expected_y
                && rect.width == 2
                && rect.height == 16
                && *radius == 4
                && *color == to_render_color(cursor_color)
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { rect, color }
            if rect.x == expected_x
                && rect.y == expected_y
                && rect.width == 2
                && rect.height == 16
                && *color == to_render_color(cursor_color)
    )));
    assert!(!scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == expected_x
                && rect.y == expected_y
                && rect.width == 8
                && rect.height == 16
                && *color == to_render_color(cursor_color)
    )));
    Ok(())
}

#[test]
fn popup_terminal_escape_enters_normal_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    install_terminal_popup_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    // Escape resolves to `picker.cancel` through the Popup Minor Mode; without
    // an open picker it must fall through to Normal mode instead of being
    // swallowed.
    let handled = state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    assert!(handled, "Escape must be handled in a focused popup");
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn popup_terminal_enter_falls_through_to_terminal_input() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    install_terminal_popup_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    // Enter resolves to `picker.submit` through the Popup Minor Mode. Without an
    // open picker that binding must not claim the key, or the terminal never
    // receives Enter to run a command.
    let handled = state
        .try_runtime_keybinding(Keycode::Return, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    assert!(
        !handled,
        "Enter must fall through when a non-picker popup is focused"
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn popup_terminal_enter_writes_to_live_session() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state
        .runtime
        .execute_command("terminal.popup")
        .map_err(|error| error.to_string())?;
    let buffer_id = active_runtime_popup(&state.runtime)?
        .map(|popup| popup.active_buffer)
        .ok_or_else(|| "terminal popup was not opened".to_owned())?;
    assert!(terminal_buffer_state(&state.runtime)?.contains(buffer_id));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);

    let handled = state
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
            640,
            240,
            8,
            16,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled, "Enter must not quit the shell");
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    assert!(
        terminal_buffer_state(&state.runtime)?.contains(buffer_id),
        "Enter must reach the live terminal session instead of failing picker.submit"
    );
    Ok(())
}
