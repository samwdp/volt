#![allow(unused_imports)]
use super::*;

#[test]
fn stalled_syntax_request_becomes_due_again() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*stalled-syntax-request*",
        vec!["fn main() {}".to_owned()],
    )?;
    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.set_language_id(Some("rust".to_owned()));
    buffer.force_syntax_refresh();
    buffer.mark_syntax_refresh_requested(buffer.full_syntax_window());
    let requested_at = buffer
        .syntax_requested_at
        .ok_or_else(|| "syntax request timestamp missing".to_owned())?;

    assert!(!buffer.syntax_refresh_due(requested_at));
    assert!(buffer.syntax_refresh_due(
        requested_at + SYNTAX_REFRESH_REQUEST_TIMEOUT + Duration::from_millis(1)
    ));
    Ok(())
}

#[test]
fn disconnected_syntax_worker_restarts_without_stranding_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*disconnected-syntax-worker*",
        vec!["fn main() {}".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.force_syntax_refresh();
    }
    let (request_tx, request_rx) = mpsc::channel();
    drop(request_rx);
    shell_ui_mut(&mut state.runtime)?
        .syntax_refresh_worker
        .request_tx = Some(request_tx);

    refresh_pending_syntax(&mut state.runtime)?;
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .is_configured()
    );
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .has_live_worker()
    );
    Ok(())
}

#[test]
fn syntax_refresh_reuses_shared_worker_across_buffers() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first = install_text_test_buffer(
        &mut state,
        "*shared-syntax-worker-a*",
        vec!["fn main() {}".to_owned()],
    )?;
    let second = install_text_test_buffer(
        &mut state,
        "*shared-syntax-worker-b*",
        vec!["fn other() {}".to_owned()],
    )?;
    for buffer_id in [first, second] {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.force_syntax_refresh();
    }

    refresh_pending_syntax(&mut state.runtime)?;
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .has_live_worker()
    );
    refresh_pending_syntax(&mut state.runtime)?;
    assert!(
        shell_ui(&state.runtime)?
            .syntax_refresh_worker
            .has_live_worker(),
        "second buffer must reuse the same shared worker"
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
fn diagnostic_underlines_expand_to_cover_narrowest_syntax_token() {
    let diagnostics = vec![LspDiagnostic::new(
        "rust-analyzer",
        "warning",
        LspDiagnosticSeverity::Warning,
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 2)),
    )];
    let line_spans = diagnostic_line_spans_for_diagnostics(&diagnostics);
    let syntax_spans = vec![
        LineSyntaxSpan {
            start: 0,
            end: 10,
            capture_name: Arc::from("source_file"),
            theme_token: Arc::from("syntax.source"),
        },
        LineSyntaxSpan {
            start: 0,
            end: 3,
            capture_name: Arc::from("keyword"),
            theme_token: Arc::from("syntax.keyword"),
        },
    ];

    assert_eq!(
        diagnostic_underlines_for_segment(
            line_spans.get(&0).map(Box::as_ref).unwrap_or(&[]),
            Some(syntax_spans.as_slice()),
            10,
            LineWrapSegment {
                start_col: 0,
                end_col: 10,
            },
        ),
        vec![DiagnosticUnderlineSpan {
            start_col: 0,
            end_col: 3,
            severity: LspDiagnosticSeverity::Warning,
        }]
    );
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
fn render_plugin_sections_draw_visual_selection_highlight() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id =
        install_plugin_sections_test_buffer(&mut state, &["alpha beta"], &["gamma delta"])?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    assert!(buffer.plugin_switch_pane());
    buffer.set_cursor(TextPoint::new(0, 5));

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let selection_color = Color::RGBA(55, 71, 99, 255);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_plugin_section_buffer_body(
        &mut target,
        PluginSectionDraw {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            layout,
            visual_selection: Some(VisualSelection::Range(TextRange::new(
                TextPoint::new(0, 0),
                TextPoint::new(0, 5),
            ))),
            yank_flash: None,
            input_mode: InputMode::Visual,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: selection_color,
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}

#[test]
fn render_acp_output_draws_visual_selection_highlight() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_push_system_message("alpha beta");
    buffer.sync_acp_viewport_metrics(640, 360, 8, 16, true);

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let selection_color = Color::RGBA(55, 71, 99, 255);
    let line_index = buffer.line_count().saturating_sub(1);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        AcpBufferDraw {
            buffer,
            rect,
            layout,
            active: true,
            visual_selection: Some(VisualSelection::Range(TextRange::new(
                TextPoint::new(line_index, 0),
                TextPoint::new(line_index, 5),
            ))),
            yank_flash: None,
            input_mode: InputMode::Visual,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: selection_color,
            yank_flash_color: Color::RGBA(112, 196, 255, 120),
            cursor: Color::RGB(110, 170, 255),
            cursor_roundness: 2,
        },
        CellMetrics {
            cell_width: 8,
            line_height: 16,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}

#[test]
fn render_markdown_hover_content_highlights_registered_code_fences() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    register_rust_highlight_test_language(&mut state.runtime)?;

    let rendered = render_markdown_hover_content(
        &mut state.runtime,
        "Example:\n\n```rust\nfn example() {}\n```\n",
    );

    assert_eq!(
        rendered.lines,
        vec![
            "Example:".to_owned(),
            String::new(),
            "```rust".to_owned(),
            "fn example() {}".to_owned(),
            "```".to_owned(),
        ]
    );
    assert!(rendered.syntax_lines.get(&3).is_some_and(|spans| {
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == "syntax.keyword")
    }));
    Ok(())
}

#[test]
fn signature_help_markdown_renders_active_parameter_bold_with_syntax_color() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    register_rust_highlight_test_language(&mut state.runtime)?;

    let rendered = {
        let mut rendered = render_markdown_hover_content(
            &mut state.runtime,
            "**Signature 1/2 (active)**\n\n```rust\ncall(alpha, beta)\n```\n",
        );
        apply_signature_active_parameter_emphasis(
            &mut rendered,
            &editor_lsp::LspSignatureActiveParameter {
                signature_index: 0,
                label: "call(alpha, beta)".to_owned(),
                start: 0,
                end: 4,
            },
        );
        rendered
    };

    let spans = rendered
        .syntax_lines
        .get(&3)
        .expect("expected syntax spans on signature line");
    assert!(spans.iter().any(|span| {
        span.theme_token.as_ref() == HOVER_SIGNATURE_ACTIVE_PARAMETER_TOKEN
            && span.start == 0
            && span.end == 4
    }));
    assert!(
        spans
            .iter()
            .any(|span| span.theme_token.as_ref() == "syntax.function")
    );

    let mut theme_registry = editor_theme::ThemeRegistry::new();
    theme_registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token("syntax.function", editor_theme::Color::rgb(10, 20, 30))
                .with_token_style(
                    HOVER_SIGNATURE_ACTIVE_PARAMETER_TOKEN,
                    editor_theme::Color::rgb(240, 240, 240),
                    editor_theme::ThemeStyle::new(true, false),
                ),
        )
        .map_err(|error| error.to_string())?;

    let line = "call(alpha, beta)";
    let char_map = LineCharMap::new(line);
    let byte_offsets = &char_map.bytes[..=char_map.len()];
    let colored = line_color_segments(
        line,
        Some(spans),
        Some(&theme_registry),
        Color::RGB(240, 240, 240),
        byte_offsets,
        0,
    );
    let call_segment = colored
        .iter()
        .find(|(text, _, _)| text == "call")
        .expect("expected colored segment for active parameter");
    assert_eq!(call_segment.1, Color::RGB(10, 20, 30));
    assert_eq!(call_segment.2, TextStyle::new(true, false));
    Ok(())
}

#[test]
fn index_syntax_lines_preserves_capture_names() {
    let text = TextBuffer::from_text("alpha");
    let lines = index_syntax_lines(
        editor_syntax::SyntaxSnapshot {
            language_id: "rust".to_owned(),
            root_kind: "source_file".to_owned(),
            has_errors: false,
            highlight_spans: vec![editor_syntax::HighlightSpan {
                start_byte: 0,
                end_byte: 5,
                start_position: editor_syntax::SyntaxPoint::new(0, 0),
                end_position: editor_syntax::SyntaxPoint::new(0, 5),
                capture_name: Arc::from("function"),
                theme_token: Arc::from("syntax.function"),
            }],
        },
        &text,
    );

    let spans = lines.get(&0).expect("expected indexed syntax line");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].capture_name.as_ref(), "function");
    assert_eq!(spans[0].theme_token.as_ref(), "syntax.function");
}

#[test]
fn index_syntax_lines_converts_byte_columns_after_variation_selector() {
    let line = "- ⚛️ Built";
    let text = TextBuffer::from_text(line);
    let start_byte = line.find("Built").expect("Built should be present");
    let end_byte = start_byte + "Built".len();
    let lines = index_syntax_lines(
        editor_syntax::SyntaxSnapshot {
            language_id: "markdown".to_owned(),
            root_kind: "document".to_owned(),
            has_errors: false,
            highlight_spans: vec![editor_syntax::HighlightSpan {
                start_byte,
                end_byte,
                start_position: editor_syntax::SyntaxPoint::new(0, start_byte),
                end_position: editor_syntax::SyntaxPoint::new(0, end_byte),
                capture_name: Arc::from("text.literal"),
                theme_token: Arc::from("syntax.string"),
            }],
        },
        &text,
    );

    assert_eq!(
        syntax_span_segments(line, lines.get(&0).expect("expected line spans")),
        vec![("syntax.string".to_owned(), "Built".to_owned())]
    );
}

#[test]
fn line_color_segments_colors_opening_brace_from_rust_highlight_pipeline() {
    let line = "use crate::{";
    let text = editor_buffer::TextBuffer::from_text(line);
    let mut registry = editor_syntax::SyntaxRegistry::new();
    registry
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-rainbow-render-test",
                ["__rainbow_render_test__"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "punctuation.bracket",
                    "syntax.punctuation.bracket",
                )],
            )
            .with_extra_highlight_query(
                r#"
[
  "(" ")" "[" "]" "{" "}"
] @punctuation.bracket
"#,
            ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));

    let mut snapshot = registry
        .highlight_buffer_for_extension("__rainbow_render_test__", &text)
        .unwrap_or_else(|error| panic!("unexpected error: {error:?}"));
    editor_syntax::apply_rainbow_delimiter_spans(&mut snapshot, line, true);
    let syntax_lines = index_syntax_lines(snapshot, &text);
    let spans = syntax_lines
        .get(&0)
        .unwrap_or_else(|| panic!("expected syntax spans for line 0: {syntax_lines:?}"));
    let brace_col = line
        .char_indices()
        .find_map(|(byte, character)| (character == '{').then_some(byte))
        .map(|byte| line[..byte].chars().count())
        .expect("opening brace column");
    let overlapping: Vec<_> = spans
        .iter()
        .filter(|span| brace_col >= span.start && brace_col < span.end)
        .collect();
    assert!(
        overlapping
            .iter()
            .any(|span| span.theme_token.starts_with("rainbow.paren.")),
        "expected rainbow span at opening brace column {brace_col}, spans={overlapping:?}, all={spans:?}"
    );
    assert!(
        overlapping
            .iter()
            .all(|span| span.theme_token.as_ref() == "rainbow.paren.depth.1"),
        "opening brace captures should all share depth 1, got {overlapping:?}"
    );

    let mut theme_registry = editor_theme::ThemeRegistry::new();
    theme_registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    "syntax.punctuation.bracket",
                    editor_theme::Color::rgb(1, 2, 3),
                )
                .with_token("rainbow.paren.depth.1", editor_theme::Color::rgb(4, 5, 6)),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let char_map = LineCharMap::new(line);
    let byte_offsets = &char_map.bytes[..=char_map.len()];
    let colored = line_color_segments(
        line,
        Some(spans),
        Some(&theme_registry),
        Color::RGB(240, 240, 240),
        byte_offsets,
        0,
    );
    let brace_segment = colored
        .iter()
        .find(|(text, _, _)| text == "{")
        .unwrap_or_else(|| panic!("expected colored segment for '{{', got {colored:?}"));
    assert_eq!(
        brace_segment.1,
        Color::RGB(4, 5, 6),
        "opening brace should use rainbow token color, got {colored:?}"
    );
}

#[test]
fn recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let install_root = TempTestDir::new("treesitter-recompile-empty");
    state
        .runtime
        .services_mut()
        .insert(editor_syntax::SyntaxRegistry::with_install_root(
            install_root.path(),
        ));

    recompile_installed_tree_sitter_languages(&mut state.runtime)?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let notifications = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
    let notification = notifications
        .into_iter()
        .find(|notification| notification.key == "treesitter.recompile-installed")
        .ok_or_else(|| "tree-sitter recompile notification was not shown".to_owned())?;
    assert_eq!(notification.title, "Tree-sitter recompile complete");
    assert_eq!(
        notification.body_lines,
        vec!["No installed Tree-sitter grammars found.".to_owned()]
    );
    Ok(())
}

#[test]
fn format_current_line_indent_uses_syntax_queries_for_blank_lines() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    syntax_registry_mut(&mut state.runtime)?
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-test-indent",
                ["rs"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "keyword",
                    "syntax.keyword",
                )],
            )
            .with_extra_indent_query(include_str!(concat!(
                core::env!("CARGO_MANIFEST_DIR"),
                "/../volt/assets/grammars/queries/rust/indents.scm"
            ))),
        )
        .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*rust-indent*")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "fn main() {".to_owned(),
            "    if true {".to_owned(),
            String::new(),
            "    }".to_owned(),
            "}".to_owned(),
        ]);
        buffer.set_language_id(Some("rust-test-indent".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 0));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 4, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(2)
            .as_deref(),
        Some("        ")
    );
    Ok(())
}

#[test]
fn format_current_line_indent_uses_syntax_queries_for_closing_braces() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    syntax_registry_mut(&mut state.runtime)?
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-test-dedent",
                ["rs"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "keyword",
                    "syntax.keyword",
                )],
            )
            .with_extra_indent_query(include_str!(concat!(
                core::env!("CARGO_MANIFEST_DIR"),
                "/../volt/assets/grammars/queries/rust/indents.scm"
            ))),
        )
        .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*rust-dedent*")?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(vec![
            "fn main() {".to_owned(),
            "    if true {".to_owned(),
            "        }".to_owned(),
            "}".to_owned(),
        ]);
        buffer.set_language_id(Some("rust-test-dedent".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 8));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 4, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(2)
            .as_deref(),
        Some("    }")
    );
    Ok(())
}

#[test]
fn format_current_line_indent_skips_cold_syntax_parse_for_large_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    syntax_registry_mut(&mut state.runtime)?
        .register(
            editor_syntax::LanguageConfiguration::new(
                "rust-test-large-indent",
                ["rs"],
                rust_test_language,
                tree_sitter_rust::HIGHLIGHTS_QUERY,
                [editor_syntax::CaptureThemeMapping::new(
                    "keyword",
                    "syntax.keyword",
                )],
            )
            .with_extra_indent_query(include_str!(concat!(
                core::env!("CARGO_MANIFEST_DIR"),
                "/../volt/assets/grammars/queries/rust/indents.scm"
            ))),
        )
        .map_err(|error| error.to_string())?;
    let buffer_id = install_scratch_test_buffer(&mut state, "*rust-large-indent*")?;
    let mut lines = vec![String::new(); LARGE_BUFFER_SYNC_INDENT_LINE_THRESHOLD + 4];
    lines[0] = "fn main() {".to_owned();
    lines[1] = "    if true {".to_owned();
    lines[2] = String::new();
    lines[3] = "    }".to_owned();
    lines[4] = "}".to_owned();
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.replace_with_lines(lines);
        buffer.set_language_id(Some("rust-test-large-indent".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 0));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 4, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(2)
            .as_deref(),
        Some("        ")
    );
    assert!(shell_ui(&state.runtime)?.indent_parse_sessions.is_empty());
    Ok(())
}

#[test]
fn comment_toggle_styles_cover_all_shipped_syntax_languages() {
    let missing = user::syntax_languages()
        .into_iter()
        .filter_map(|language| {
            comment_style_for_language_path(
                Some(language.id()),
                language.file_extensions().first().map(String::as_str),
                language.file_names().first().map(String::as_str),
            )
            .is_none()
            .then(|| language.id().to_owned())
        })
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing comment styles for: {}",
        missing.join(", ")
    );
}

#[test]
fn db_query_buffer_receives_sql_highlighting_without_blocking() -> Result<(), String> {
    let state_dir = TempTestDir::new("db-query-syntax-refresh");
    fs::create_dir_all(state_dir.path()).map_err(|error| error.to_string())?;
    let db_path = state_dir.path().join("query.sqlite3");
    let mut state = state_with_user_library()?;
    let connection_string = format!("sqlite://{}", db_path.display());
    db_service_mut(&mut state.runtime)?
        .connect_raw(&connection_string, Some("query"))
        .map_err(|error| error.to_string())?;

    open_db_query_buffer(&mut state.runtime)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert!(buffer_is_db_query(&buffer.kind));
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(3).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.theme_token.starts_with("syntax.keyword"))
        }),
        "DB query starter SQL should receive keyword highlighting"
    );
    Ok(())
}

#[test]
fn opened_sql_file_survives_layout_and_syntax_refresh() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-sql-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("query.sql");
    fs::write(&path, "SELECT *\nFROM widgets\nWHERE id = 1;\n")
        .map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;
    sync_active_buffer_layout_for_test(&mut state)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.language_id(), Some("sql"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(0).is_some_and(|spans| {
            spans
                .iter()
                .any(|span| span.theme_token.starts_with("syntax.keyword"))
        }),
        "opened SQL file should receive keyword highlight spans"
    );
    Ok(())
}

#[test]
fn opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-toml-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("volt.toml");
    fs::write(&path, "title = \"Volt\"\n[editor]\nmode = \"vim\"\n")
        .map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;
    sync_active_buffer_layout_for_test(&mut state)?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.language_id(), Some("toml"));
    assert!(buffer.syntax_error.is_none());
    assert!(
        buffer.line_syntax_spans(0).is_some(),
        "opened TOML file should receive syntax spans"
    );
    Ok(())
}

#[test]
fn opened_file_receives_tree_sitter_highlighting() -> Result<(), String> {
    let root = TempTestDir::new("file-tree-sitter-highlighting");
    fs::create_dir_all(root.path()).map_err(|error| error.to_string())?;
    let path = root.path().join("main.rs");
    fs::write(&path, "fn main() {\n    let value = 1;\n}\n").map_err(|error| error.to_string())?;
    let mut state = state_with_user_library()?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;

    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .line_syntax_spans(0)
            .is_some_and(|spans| {
                spans
                    .iter()
                    .any(|span| span.theme_token.starts_with("syntax.keyword"))
            }),
        "opened file should receive syntax highlight spans"
    );
    Ok(())
}

#[test]
fn render_terminal_buffer_draws_visual_selection_highlight() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_terminal_test_buffer(&mut state)?;
    let selection_color = Color::RGBA(55, 71, 99, 255);
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "terminal test buffer missing".to_owned())?;
        buffer.replace_with_lines_follow_output(vec!["echo hello".to_owned(), String::new()]);
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
                editor_terminal::TerminalRenderLine::new(vec![]),
            ],
            None,
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
            input_mode: InputMode::Visual,
            visual_selection: Some(VisualSelection::Range(TextRange::new(
                TextPoint::new(0, 0),
                TextPoint::new(0, 4),
            ))),
            yank_flash: None,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(215, 221, 232),
            border_color: Color::RGB(40, 44, 52),
            selection: selection_color,
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
        DrawCommand::FillRoundedRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}

#[test]
fn workspace_dock_highlight_tracks_active_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-highlight-a");
    let second_root = unique_temp_dir("workspace-dock-highlight-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == first)
            .is_some_and(|entry| entry.active)
    );
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == second)
            .is_some_and(|entry| !entry.active)
    );

    state
        .runtime
        .execute_command("workspace.next")
        .map_err(|error| error.to_string())?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.active_workspace(), second);
    assert!(
        entries
            .iter()
            .find(|entry| entry.workspace_id == second)
            .is_some_and(|entry| entry.active)
    );
    Ok(())
}
