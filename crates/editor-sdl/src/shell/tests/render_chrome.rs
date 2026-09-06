#![allow(unused_imports)]
use super::*;

#[test]
fn statusline_lsp_diagnostics_counts_errors_and_warnings() {
    let diagnostics = vec![
        LspDiagnostic::new(
            "rust-analyzer",
            "error",
            LspDiagnosticSeverity::Error,
            TextRange::new(TextPoint::new(0, 1), TextPoint::new(0, 2)),
        ),
        LspDiagnostic::new(
            "rust-analyzer",
            "warning",
            LspDiagnosticSeverity::Warning,
            TextRange::new(TextPoint::new(1, 3), TextPoint::new(1, 5)),
        ),
        LspDiagnostic::new(
            "rust-analyzer",
            "info",
            LspDiagnosticSeverity::Information,
            TextRange::new(TextPoint::new(2, 0), TextPoint::new(2, 1)),
        ),
    ];

    assert_eq!(
        statusline_lsp_diagnostics(&diagnostics),
        Some(editor_plugin_api::LspDiagnosticsInfo {
            errors: 1,
            warnings: 1,
        })
    );
}

#[test]
fn statusline_icon_segments_split_acp_and_lsp_icons() {
    let user_library = user::UserLibraryImpl;
    let acp_icon = editor_icons::symbols::fa::FA_CONNECTDEVELOP;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let statusline = format!("NORMAL | {acp_icon} | Ln 3, Col 9 | {lsp_icon} rust-analyzer");
    assert_eq!(
        statusline_icon_segments(&statusline, &[acp_icon, lsp_icon]),
        vec![
            ("NORMAL | ", false),
            (acp_icon, true),
            (" | Ln 3, Col 9 | ", false),
            (lsp_icon, true),
            (" rust-analyzer", false),
        ]
    );
}

#[test]
fn statusline_icon_segments_split_diagnostic_icons() {
    let user_library = user::UserLibraryImpl;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let error_icon = user_library.statusline_lsp_error_icon();
    let warning_icon = user_library.statusline_lsp_warning_icon();
    let prefix = format!("NORMAL | {lsp_icon} rust-analyzer ");
    let statusline = format!("NORMAL | {lsp_icon} rust-analyzer {error_icon} 2 {warning_icon} 4");
    assert_eq!(
        statusline_icon_segments(&statusline, &[error_icon, warning_icon]),
        vec![
            (prefix.as_str(), false),
            (error_icon, true),
            (" 2 ", false),
            (warning_icon, true),
            (" 4", false),
        ]
    );
}

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
