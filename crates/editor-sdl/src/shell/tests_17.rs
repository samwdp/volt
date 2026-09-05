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
fn render_buffer_multicursor_draws_one_cursor_per_range() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*multicursor-render*",
        vec!["alpha alpha alpha".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.input = Some(InputField::new(">"));
        buffer.set_cursor(TextPoint::new(0, 6));
    }

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let rect = PixelRectToRect::rect(0, 0, 320, 180);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let cursor_color = to_render_color(Color::RGB(110, 170, 255));
    let multicursor = MulticursorState {
        match_text: "alpha".to_owned(),
        ranges: vec![
            TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
            TextRange::new(TextPoint::new(0, 6), TextPoint::new(0, 11)),
            TextRange::new(TextPoint::new(0, 12), TextPoint::new(0, 17)),
        ],
        primary: 1,
        cursor_offset: 0,
        visual_anchor_offset: None,
    };
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
                input_mode: InputMode::Insert,
                multicursor: Some(&multicursor),
                vim_targets_input: true,
                recording_macro: None,
                typing_active: false,
            },
            command_line: CommandLineSlot {
                input: None,
                row_visible: NullUserLibrary.commandline_enabled(),
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

    let cursor_positions = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if *color == cursor_color && rect.y == layout.body_y =>
            {
                Some(rect.x)
            }
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>();

    let text_x = rect.x() + 12 + 8 + (5 * 8);
    assert_eq!(
        cursor_positions,
        [text_x, text_x + 6 * 8, text_x + 12 * 8]
            .into_iter()
            .collect()
    );
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
fn shell_start_does_not_construct_browser_web_context() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    assert!(
        !state.browser_host.has_live_web_context(),
        "shell start without a browser buffer must not construct WebContext"
    );
    Ok(())
}

#[test]
fn browser_host_open_devtools_event_is_ignored_without_a_live_webview() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    state
        .apply_browser_host_events(&[BrowserHostEvent::OpenDevtoolsRequested { buffer_id }])
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[test]
fn browser_devtools_shortcut_requested_recognizes_f12_and_ctrl_shift_i() {
    assert!(browser_devtools_shortcut_requested(
        Keycode::F12,
        Mod::NOMOD
    ));
    assert!(browser_devtools_shortcut_requested(
        Keycode::F12,
        shift_mod()
    ));
    assert!(browser_devtools_shortcut_requested(
        Keycode::I,
        ctrl_mod() | shift_mod()
    ));
}

#[test]
fn input_field_paste_shortcut_requested_recognizes_ctrl_shift_v_only() {
    assert!(input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod() | shift_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        shift_mod()
    ));
    assert!(!input_field_paste_shortcut_requested(
        Keycode::V,
        ctrl_mod() | shift_mod() | Mod::LALTMOD
    ));
}

#[test]
fn browser_devtools_shortcut_requested_rejects_other_modifiers() {
    assert!(!browser_devtools_shortcut_requested(Keycode::I, ctrl_mod()));
    assert!(!browser_devtools_shortcut_requested(
        Keycode::I,
        ctrl_mod() | shift_mod() | Mod::LALTMOD
    ));
    assert!(!browser_devtools_shortcut_requested(
        Keycode::F11,
        Mod::NOMOD
    ));
}

#[test]
fn workspace_search_provider_extras_copy_ctrl_q_onto_instance() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-search-ctrl-q-extra");
    std::fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "workspace-search-ctrl-q-extra", &root)?;

    let overlay = picker::picker_overlay(&state.runtime, "workspace.search")?;
    assert!(
        overlay.extra_keybinds().iter().any(|binding| {
            binding.chord() == "Ctrl+q" && binding.command_name() == "quickfix.open"
        }),
        "workspace.search provider extras should land on the open picker instance"
    );
    Ok(())
}

fn prepare_quickfix_workspace_search_picker(
    test_name: &str,
) -> Result<(ShellState, PathBuf, PathBuf, PathBuf), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir(test_name);
    let first = root.join("src").join("main.rs");
    let second = root.join("src").join("lib.rs");
    std::fs::create_dir_all(first.parent().ok_or_else(|| "missing src dir".to_owned())?)
        .map_err(|error| error.to_string())?;
    std::fs::write(&first, "fn alpha() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "fn beta() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, test_name, &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Workspace Search",
            vec![
                workspace_search::workspace_search_match_entry(
                    &root,
                    "src/main.rs",
                    1,
                    4,
                    "fn alpha() {}",
                ),
                workspace_search::workspace_search_match_entry(
                    &root,
                    "src/lib.rs",
                    1,
                    4,
                    "fn beta() {}",
                ),
            ],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new("Ctrl+q", "quickfix.open")]),
    );

    Ok((state, root, first, second))
}

#[test]
fn ctrl_q_with_workspace_search_picker_exports_quickfix_instead_of_quitting() -> Result<(), String>
{
    let (mut state, _root, _first, _second) =
        prepare_quickfix_workspace_search_picker("quickfix-ctrl-q-export")?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
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

    assert!(!handled);
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "Ctrl+Q did not open quickfix popup".to_owned())?;
    assert_eq!(
        shell_ui(&state.runtime)?.popup_buffer_id,
        Some(popup.active_buffer)
    );
    assert!(shell_ui(&state.runtime)?.popup_focus);
    let buffer = shell_buffer(&state.runtime, popup.active_buffer)?;
    assert!(buffer_is_quickfix(&buffer.kind));
    let first_line = buffer
        .text
        .line(0)
        .ok_or_else(|| "quickfix first line missing".to_owned())?;
    assert!(first_line.contains("main.rs:1:4 | fn alpha() {}"));
    assert!(first_line.contains("[ ] "));
    let second_line = buffer
        .text
        .line(1)
        .ok_or_else(|| "quickfix second line missing".to_owned())?;
    assert!(second_line.contains("lib.rs:1:4 | fn beta() {}"));
    assert!(second_line.contains("[ ] "));
    Ok(())
}

#[test]
fn ctrl_q_with_non_quickfix_picker_does_not_quit() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Buffers",
        vec![PickerEntry {
            item: PickerItem::new("buffer:alpha", "alpha", "scratch", None::<&str>),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    ));
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
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

    assert!(!handled);
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    Ok(())
}

#[test]
fn ctrl_q_without_quickfix_extra_does_not_export_popup_global() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("quickfix-ctrl-q-no-extra");
    let first = root.join("src").join("main.rs");
    std::fs::create_dir_all(first.parent().ok_or_else(|| "missing src dir".to_owned())?)
        .map_err(|error| error.to_string())?;
    std::fs::write(&first, "fn alpha() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "quickfix-ctrl-q-no-extra", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Search",
        vec![workspace_search::workspace_search_match_entry(
            &root,
            "src/main.rs",
            1,
            4,
            "fn alpha() {}",
        )],
    ));
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Q),
                scancode: None,
                keymod: ctrl_mod(),
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

    assert!(!handled);
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    assert!(active_runtime_popup(&state.runtime)?.is_none());
    Ok(())
}

#[test]
fn picker_extra_keybind_snapshots_context_closes_and_runs_command() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.picker-extra",
            "Consumes picker one-shot context",
            CommandSource::Core,
            |runtime| {
                let context = shell_ui_mut(runtime)?
                    .take_picker_one_shot()
                    .ok_or_else(|| "picker one-shot missing".to_owned())?;
                let selected = context
                    .selected()
                    .ok_or_else(|| "selected row missing".to_owned())?;
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push(format!(
                    "{}|{}|{}",
                    selected.id(),
                    selected.label(),
                    selected.path().unwrap_or("")
                ));
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;

    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Worktrees",
            vec![PickerEntry {
                item: PickerItem::new(
                    r"P:\repo\feature",
                    "feature",
                    "branch",
                    Some(r"P:\repo\feature"),
                ),
                action: PickerAction::NoOp,
                quickfix: None,
            }],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new(
            "Ctrl+d",
            "tests.picker-extra",
        )]),
    );

    let handled = state
        .try_runtime_keybinding(Keycode::D, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(
        shell_ui_mut(&mut state.runtime)?
            .take_picker_one_shot()
            .is_none()
    );
    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec![r"P:\repo\feature|feature|P:\repo\feature".to_string()]
    );
    Ok(())
}

#[test]
fn popup_focus_ctrl_n_cycles_popup_buffers_instead_of_marked_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let first = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-a*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    let second = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-b*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![first, second], first)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(first, "*popup-a*", BufferKind::Scratch, &*user_library);
        ui.ensure_popup_buffer(second, "*popup-b*", BufferKind::Scratch, &*user_library);
        ui.set_popup_buffer(first);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    let handled = state
        .try_runtime_keybinding(Keycode::N, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "popup missing after Ctrl+n".to_owned())?;
    assert_eq!(popup.active_buffer, second);
    assert_eq!(shell_ui(&state.runtime)?.popup_buffer_id, Some(second));
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

#[test]
fn popup_buffer_escape_enters_normal_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let popup_buffer = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup-escape*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![popup_buffer], popup_buffer)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(
            popup_buffer,
            "*popup-escape*",
            BufferKind::Scratch,
            &*user_library,
        );
        ui.set_popup_buffer(popup_buffer);
        ui.set_popup_focus(true);
        ui.enter_insert_mode();
    }

    let handled = state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;
    assert!(handled, "Escape must be handled in a focused popup");
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn popup_focus_j_k_do_not_cycle_workspace_dock() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("popup-jk-dock-a");
    let second_root = unique_temp_dir("popup-jk-dock-b");
    let first = open_workspace_from_project(&mut state.runtime, "popup-jk-a", &first_root)?;
    let _second = open_workspace_from_project(&mut state.runtime, "popup-jk-b", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;

    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let popup_buffer = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*popup*", BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Popup", vec![popup_buffer], popup_buffer)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(&state.runtime);
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.ensure_popup_buffer(popup_buffer, "*popup*", BufferKind::Scratch, &*user_library);
        ui.set_popup_buffer(popup_buffer);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    let modes = state
        .overlay_minor_modes()
        .map_err(|error| error.to_string())?;
    assert!(
        modes.contains(&KeymapScope::Popup),
        "popup focus must activate Popup Minor Mode: {modes:?}"
    );
    assert!(
        !modes.contains(&KeymapScope::WorkspaceDock),
        "popup focus must not activate Workspace Dock Minor Mode: {modes:?}"
    );
    for chord in ["j", "k"] {
        let overlay = state
            .runtime
            .keymaps()
            .find_in_scopes(&modes, KeymapVimMode::Normal, chord)
            .map(|binding| binding.command_name().to_owned());
        assert_ne!(
            overlay.as_deref(),
            Some("workspace.dock.next"),
            "popup {chord} must not fire workspace dock cycle"
        );
        assert_ne!(
            overlay.as_deref(),
            Some("workspace.dock.previous"),
            "popup {chord} must not fire workspace dock cycle"
        );
    }

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first,
        "popup j must not cycle the workspace dock"
    );
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first,
        "popup k must not cycle the workspace dock"
    );
    Ok(())
}

#[test]
fn picker_extra_keybind_falls_through_for_shared_popup_navigation() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries(
            "Buffers",
            vec![
                PickerEntry {
                    item: PickerItem::new("a", "alpha", "scratch", None::<&str>),
                    action: PickerAction::NoOp,
                    quickfix: None,
                },
                PickerEntry {
                    item: PickerItem::new("b", "beta", "scratch", None::<&str>),
                    action: PickerAction::NoOp,
                    quickfix: None,
                },
            ],
        )
        .with_extra_keybinds(vec![PickerExtraKeybind::new(
            "Ctrl+d",
            "tests.picker-extra",
        )]),
    );

    let handled = state
        .try_runtime_keybinding(Keycode::N, ctrl_mod())
        .map_err(|error| error.to_string())?;
    assert!(handled);
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    let selected = shell_ui(&state.runtime)?
        .picker()
        .and_then(|picker| picker.session().selected())
        .map(|matched| matched.item().id().to_owned());
    assert_eq!(selected.as_deref(), Some("b"));
    Ok(())
}

#[test]
#[ignore = "enable once quickfix picker export command lands"]
fn quickfix_picker_export_opens_popup_and_renders_workspace_search_results() -> Result<(), String> {
    let (state, root, first, second) =
        prepare_quickfix_workspace_search_picker("quickfix-export-popup")?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "picker missing before quickfix export".to_owned())?;
    assert_eq!(picker.session().matches().len(), 2);
    let _ = (first, second);
    let _ = active_runtime_popup(&state.runtime)?;
    let _ = shell_ui(&state.runtime)?.popup_buffer_id;
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = root;
    unimplemented!("invoke quickfix export and assert popup buffer renders exported rows");
}

#[test]
#[ignore = "enable once quickfix enter handler lands"]
fn quickfix_enter_opens_target_and_moves_focus_back_to_workspace() -> Result<(), String> {
    let (state, root, first, _) = prepare_quickfix_workspace_search_picker("quickfix-enter-focus")?;

    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let _ = (root, first);
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = original_buffer_id;
    unimplemented!(
        "export picker to quickfix, press Enter on quickfix row, assert workspace focus"
    );
}

#[test]
#[ignore = "enable once quickfix next and previous commands land"]
fn quickfix_next_previous_wraparound_tracks_current_list() -> Result<(), String> {
    let (state, root, first, second) =
        prepare_quickfix_workspace_search_picker("quickfix-wraparound")?;

    let _ = active_shell_buffer_id(&state.runtime)?;
    let _ = shell_ui(&state.runtime)?.popup_focus;
    let _ = (root, first, second);
    unimplemented!("export picker, drive quickfix.next/previous, assert wraparound navigation");
}

#[test]
#[ignore = "enable once quickfix export command lands"]
fn quickfix_export_from_unsupported_picker_is_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Buffers",
        vec![PickerEntry {
            item: PickerItem::new("buffer:alpha", "alpha", "scratch", None::<&str>),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    ));

    let _ = original_buffer_id;
    let _ = active_runtime_popup(&state.runtime)?;
    unimplemented!("invoke quickfix export and assert picker closes or no-ops without popup");
}

// ---------------------------------------------------------------------------
// LeaveOpen exit action
// ---------------------------------------------------------------------------

fn wait_for_streamed_command_worker_done(
    state: &mut ShellState,
    buffer_id: BufferId,
) -> Result<(), String> {
    for _ in 0..500 {
        refresh_pending_streamed_commands(&mut state.runtime)?;
        let tracked = shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id);
        if !tracked {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    Err(format!(
        "streamed command worker for buffer `{buffer_id}` did not finish in time"
    ))
}

#[test]
fn leave_open_keeps_popup_buffer_after_process_exits() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = open_streamed_command_popup(
        &mut state.runtime,
        StreamedCommandSpec {
            popup_title: "Leave Open Test".to_owned(),
            buffer_name: "*leave-open-test*".to_owned(),
            command_label: "true".to_owned(),
            #[cfg(unix)]
            program: "true".to_owned(),
            #[cfg(windows)]
            program: "cmd".to_owned(),
            #[cfg(unix)]
            args: vec![],
            #[cfg(windows)]
            args: vec!["/C".to_owned(), "exit 0".to_owned()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
            on_exit: StreamedCommandExitAction::LeaveOpen,
            notify_on_success: false,
            notify_on_failure: false,
        },
    )?;

    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;

    let ui = shell_ui(&state.runtime)?;
    assert!(
        ui.buffer(buffer_id).is_some(),
        "LeaveOpen: popup buffer must remain open after process exits"
    );
    assert!(
        !ui.streamed_command_worker.contains(buffer_id),
        "LeaveOpen: worker should be done"
    );
    Ok(())
}

// ---------------------------------------------------------------------------
// Cancel flag: closing a popup buffer kills its worker
// ---------------------------------------------------------------------------

#[test]
fn closing_streamed_command_popup_kills_worker() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = open_streamed_command_popup(
        &mut state.runtime,
        StreamedCommandSpec {
            popup_title: "Cancel Test".to_owned(),
            buffer_name: "*cancel-test*".to_owned(),
            command_label: "sleep".to_owned(),
            #[cfg(unix)]
            program: "sleep".to_owned(),
            #[cfg(windows)]
            program: "cmd".to_owned(),
            #[cfg(unix)]
            args: vec!["60".to_owned()],
            #[cfg(windows)]
            args: vec!["/C".to_owned(), "timeout /T 60 /NOBREAK".to_owned()],
            env: Vec::new(),
            cwd: std::env::temp_dir(),
            on_exit: StreamedCommandExitAction::LeaveOpen,
            notify_on_success: false,
            notify_on_failure: false,
        },
    )?;

    assert!(
        shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should be active before close"
    );

    close_buffer_immediate(&mut state.runtime, buffer_id).map_err(|error| error.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?
            .streamed_command_worker
            .contains(buffer_id),
        "worker should be removed immediately after buffer close"
    );

    wait_for_streamed_command_worker_done(&mut state, buffer_id)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// InputPromptOverlay
// ---------------------------------------------------------------------------

fn shell_echo_command(marker: &str) -> String {
    if cfg!(windows) {
        format!("Write-Output {marker}")
    } else {
        format!("printf '{marker}\\n'")
    }
}

fn shell_sleep_then_echo_command(seconds: u64, marker: &str) -> String {
    if cfg!(windows) {
        format!("Start-Sleep -Seconds {seconds}; Write-Output {marker}")
    } else {
        format!("sleep {seconds}; printf '{marker}\\n'")
    }
}

fn execute_shell_command(state: &mut ShellState, command: &str) -> Result<(), String> {
    state
        .runtime
        .execute_command(command)
        .map_err(|error| error.to_string())
}

fn active_input_prompt_text(state: &ShellState) -> Result<Option<String>, String> {
    Ok(shell_ui(&state.runtime)?
        .input_prompt()
        .map(|prompt| prompt.text().to_owned()))
}

fn confirm_input_prompt(state: &mut ShellState, text: &str) -> Result<(), String> {
    if !text.is_empty() {
        state
            .handle_text_input(text)
            .map_err(|error| error.to_string())?;
    }
    state
        .try_runtime_keybinding(Keycode::Return, Mod::empty())
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn start_workspace_compile(state: &mut ShellState, command: &str) -> Result<BufferId, String> {
    execute_shell_command(state, "workspace.compile")?;
    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "workspace.compile should open InputPromptOverlay"
    );
    confirm_input_prompt(state, command)?;
    active_runtime_popup(&state.runtime)?
        .map(|popup| popup.active_buffer)
        .ok_or_else(|| "compile confirmation did not open streamed popup".to_owned())
}

fn prompt_prefill_for_marker(
    tag: &str,
    marker_name: &str,
    marker_contents: &str,
) -> Result<String, String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir(tag);
    if !marker_name.is_empty() {
        std::fs::write(root.join(marker_name), marker_contents)
            .map_err(|error| error.to_string())?;
    }
    open_workspace_from_project(&mut state.runtime, tag, &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;
    let text = active_input_prompt_text(&state)?.unwrap_or_default();
    std::fs::remove_dir_all(&root).ok();
    Ok(text)
}
