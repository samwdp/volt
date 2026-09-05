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
fn hover_diagnostic_provider_fragments_preserve_fenced_code_blocks() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("alpha");
        buffer.set_cursor(TextPoint::new(0, 2));
        buffer.set_lsp_diagnostics(vec![LspDiagnostic::new(
            "rust-analyzer",
            "Try this:\n```rust\nfn example() {}\n```",
            LspDiagnosticSeverity::Warning,
            TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
        )]);
    }

    let fragments = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        hover_diagnostic_provider_fragments(buffer, &NullUserLibrary)
    };

    assert_eq!(
        fragments,
        vec![
            HoverProviderFragment::PlainLines(vec![format!(
                "{} rust-analyzer",
                NullUserLibrary.lsp_diagnostic_icon()
            )]),
            HoverProviderFragment::MarkdownText(
                "Try this:\n```rust\nfn example() {}\n```".to_owned()
            ),
        ]
    );
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
fn line_color_segments_prefers_rainbow_paren_token_for_equal_width_spans() {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token("syntax.type", editor_theme::Color::rgb(1, 2, 3))
                .with_token("rainbow.paren.depth.2", editor_theme::Color::rgb(4, 5, 6)),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let spans = vec![
        LineSyntaxSpan {
            start: 0,
            end: 1,
            capture_name: Arc::from("type"),
            theme_token: Arc::from("syntax.type"),
        },
        LineSyntaxSpan {
            start: 0,
            end: 1,
            capture_name: Arc::from("rainbow.paren.open.2"),
            theme_token: Arc::from("rainbow.paren.depth.2"),
        },
    ];

    let segments = line_color_segments(
        "(",
        Some(&spans),
        Some(&registry),
        Color::RGB(0, 0, 0),
        &[0, 1],
        0,
    );

    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].0, "(".to_owned());
    assert_eq!(segments[0].1, Color::RGB(4, 5, 6));
}

#[test]
fn browser_buffer_submit_tracks_requested_navigation() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("example.com/docs");
    }

    submit_input_buffer(&mut state.runtime)?;

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let state = buffer
        .browser_state
        .as_ref()
        .ok_or_else(|| "browser state missing".to_owned())?;
    assert_eq!(state.current_url.as_deref(), None);
    assert_eq!(
        state.requested_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert!(state.is_loading);
    assert_eq!(
        buffer.display_name(),
        "*browser* [loading] https://example.com/docs"
    );
    Ok(())
}

#[test]
fn browser_escape_from_insert_keeps_input_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("https://example.com");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .cursor_char(),
        "https://example.com".chars().count()
    );
    Ok(())
}

#[test]
fn acp_input_field_visual_yank_copies_selected_text() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "alpha beta", None)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.cursor = 0;
    }

    focus_input_normal_mode(&mut state, buffer_id)?;
    start_visual_mode_with_kind(&mut state.runtime, VisualSelectionKind::Character)?;
    apply_motion_command(&mut state.runtime, ShellMotion::Right)?;
    apply_visual_operator(&mut state.runtime, VimOperator::Yank)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        ui.vim().yank,
        Some(YankRegister::Character("al".to_owned()))
    );
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .selection_anchor,
        None
    );
    Ok(())
}

#[test]
fn acp_input_field_dd_deletes_current_line() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta\ngamma")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char("alpha\n".chars().count());
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "alpha\ngamma");
    assert_eq!(ui.vim().yank, Some(YankRegister::Line("beta\n".to_owned())));
    Ok(())
}

#[test]
fn acp_input_field_dw_deletes_motion_range() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha beta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("w")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "beta");
    assert_eq!(
        ui.vim().yank,
        Some(YankRegister::Character("alpha ".to_owned()))
    );
    Ok(())
}

#[test]
fn acp_input_field_cw_enters_insert_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha beta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("w")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(input.text(), "beta");

    state
        .handle_text_input("zeta ")
        .map_err(|error| error.to_string())?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "zeta beta");
    Ok(())
}

#[test]
fn acp_input_field_visual_line_delete_removes_selected_lines() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta\ngamma")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(input.text(), "gamma");
    Ok(())
}

#[test]
fn acp_input_field_o_and_o_open_new_lines() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char("alpha\n".chars().count());
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    state
        .handle_text_input("middle")
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("O")
        .map_err(|error| error.to_string())?;
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    state
        .handle_text_input("above")
        .map_err(|error| error.to_string())?;

    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "alpha\nbeta\nabove\nmiddle");
    Ok(())
}

#[test]
fn acp_input_field_yy_and_p_work_linewise() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_acp_test_buffer(&mut state, "alpha\nbeta")?;
    {
        let input = shell_buffer_mut(&mut state.runtime, buffer_id)?
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_cursor_char(0);
    }
    focus_input_normal_mode(&mut state, buffer_id)?;

    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("p")
        .map_err(|error| error.to_string())?;

    let input = shell_buffer(&state.runtime, buffer_id)?
        .input_field()
        .ok_or_else(|| "ACP input field missing".to_owned())?;
    assert_eq!(input.text(), "alpha\nalpha\nbeta");
    Ok(())
}

#[test]
fn acp_escape_from_insert_keeps_input_cursor_position() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "prompt", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .runtime
        .emit_hook(HOOK_MODE_NORMAL, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .cursor_char(),
        "prompt".chars().count()
    );
    Ok(())
}

#[test]
fn acp_second_escape_returns_hjkl_and_visual_mode_to_output_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, "*acp*", user::acp::ACP_BUFFER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.init_acp_view("GitHub Copilot");
        let acp = buffer
            .acp_state
            .as_mut()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        acp.output_pane.replace_render_lines(
            vec![
                AcpRenderedLine::Text(AcpRenderedTextLine {
                    prefix: Vec::new(),
                    text: "alpha".to_owned(),
                    text_role: AcpColorRole::Default,
                    syntax_spans: Vec::new(),
                    row_fill: None,
                    gutter: false,
                    align: AcpChatAlign::Full,
                    bubble: false,
                    bubble_group: 0,
                }),
                AcpRenderedLine::Text(AcpRenderedTextLine {
                    prefix: Vec::new(),
                    text: "beta".to_owned(),
                    text_role: AcpColorRole::Default,
                    syntax_spans: Vec::new(),
                    row_fill: None,
                    gutter: false,
                    align: AcpChatAlign::Full,
                    bubble: false,
                    bubble_group: 0,
                }),
            ],
            false,
            4,
        );
        if let Some(input) = buffer.input_field_mut() {
            input.set_text("prompt");
            input.cursor = input.text().len();
        }
    }

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.acp_active_pane(),
        Some(AcpPane::Output)
    );

    assert!(
        state
            .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert!(shell_ui(&state.runtime)?.vim().target == VimTarget::Input);

    assert!(
        state
            .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    assert!(shell_ui(&state.runtime)?.vim().target == VimTarget::Buffer);

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert_eq!(acp.output_pane.cursor(), TextPoint::new(1, 0));
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .cursor_char(),
        "prompt".chars().count()
    );

    state
        .handle_text_input("v")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("h")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(1, 0)));
    assert_eq!(ui.vim().target, VimTarget::Buffer);
    Ok(())
}

#[test]
fn paste_text_into_active_input_buffer_updates_acp_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "alpha", None)?;

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        " beta"
    )?);

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "alpha beta"
    );
    Ok(())
}

#[test]
fn paste_text_into_active_input_buffer_closes_acp_picker_for_multiline_text() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/fix", None)?;
    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "ACP Slash Commands",
        Vec::new(),
    ));

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        "\nmore context"
    )?);

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "/fix\nmore context"
    );
    Ok(())
}

#[test]
fn acp_nonleading_double_slash_does_not_open_slash_picker() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "i have this code ", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .handle_text_input("//")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "i have this code //"
    );
    Ok(())
}

#[test]
fn acp_slash_picker_text_input_updates_acp_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
            .with_kind(PickerKind::AcpSlash { buffer_id }),
    );

    state
        .handle_text_input("fix")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "/fix"
    );
    Ok(())
}

#[test]
fn acp_slash_picker_backspace_can_delete_leading_slash() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    shell_ui_mut(&mut state.runtime)?.set_picker(
        PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
            .with_kind(PickerKind::AcpSlash { buffer_id }),
    );
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Backspace),
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

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        ""
    );
    Ok(())
}

#[test]
fn acp_paste_code_with_inline_double_slash_comments_closes_slash_picker() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "/", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
        ui.set_picker(
            PickerOverlay::from_entries("ACP Slash Commands", Vec::new())
                .with_kind(PickerKind::AcpSlash { buffer_id }),
        );
    }

    let pasted = "        Unknown = 0,\n        Vehicle=1,\n        Other,\n        SmartPhone,\n        Person,\n        Trailer,\n        Train,\n        Aircraft,\n        Luggage,\n        Skip,\n        IoTDevice=10,\n        Building,\n        Robot,\n        Parcel,\n        Animal,\n        CommercialWasteBin,\n        Keg,\n        Crane,\n        Generator,\n        RetailCage,\n        GolfBuggy=20,\n        RoadSweeper,\n        BarCodeScanner,\n        Printer,\n        Computer,\n        Gritter,\n        AirStarterUnit,\n        AircraftEngineeringServicingEquipment, // ASE\n        AircraftTowBar, // ACTB\n        AircraftTug, // TUGS - 30\n        BaggageDollie, // BAGD\n        BaggagePOD, // POD\n        BaggageTug, // EBT\n        BeltLoader, // BELT\n        Car,\n        Van, // CAR\n        CateringVehicle, // CATV\n        Coach, // COAC\n        DeIcingVehicle, // DEIC\n        GroundPowerUnit, // GPU\n        HighLoader, // HILO - 40\n        Lorry,\n        LowLoader, // LOLO\n        Minibus, // MBUS\n        MotorisedStep, // MSTP\n        NonMotorisedStep, // STPN\n        PassengerBoardingRamp, // PBR\n        PassengerMobility, // LIFT - Ambulift\n        FuelBowser,\n        WaterBowser,\n";

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        pasted
    )?);

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        format!("/{pasted}")
    );
    Ok(())
}

#[test]
fn acp_at_symbol_opens_git_file_picker_and_return_inserts_mention() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = init_git_repo("acp-files")?;
    fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n")
        .map_err(|error| error.to_string())?;
    run_git_in_dir(&root, &["add", "src/main.rs"])?;
    open_workspace_from_project(&mut state.runtime, "acp-files", &root)
        .map_err(|error| error.to_string())?;

    let buffer_id = install_acp_test_buffer(&mut state, 0, "look at ", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    state
        .handle_text_input("@")
        .map_err(|error| error.to_string())?;

    {
        let ui = shell_ui(&state.runtime)?;
        let picker = ui
            .picker()
            .ok_or_else(|| "ACP file picker should open for @".to_owned())?;
        assert_eq!(picker.session().title(), "ACP Files");
        assert!(
            picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().label() == "src/main.rs"),
            "git file picker should list src/main.rs"
        );
        assert_eq!(ui.picker_kind(), Some(PickerKind::AcpFile { buffer_id }));
    }

    state
        .handle_text_input("main.rs")
        .map_err(|error| error.to_string())?;
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

    let ui = shell_ui(&state.runtime)?;
    assert!(!ui.picker_visible());
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "look at @src/main.rs "
    );
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn acp_paste_image_inserts_mention_token_and_stores_bytes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_acp_test_buffer(&mut state, 0, "see", None)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_acp_input();
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let image = normalize_clipboard_image(TINY_PNG.to_vec(), Some("image/png"), "Image")
        .ok_or_else(|| "png should normalize".to_owned())?;
    assert!(paste_image_into_active_input_buffer(
        &mut state.runtime,
        image
    )?);

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "ACP input field missing".to_owned())?
            .text(),
        "see ![Image](acp-image:1) "
    );
    let images = buffer.acp_pasted_images();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].id, 1);
    assert_eq!(images[0].mime_type, "image/png");
    assert!(!images[0].data.is_empty());
    Ok(())
}

#[test]
fn paste_text_into_active_input_buffer_updates_browser_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    assert!(paste_text_into_active_input_buffer(
        &mut state.runtime,
        "example.com/docs"
    )?);

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    assert_eq!(
        buffer
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        "example.com/docs"
    );
    Ok(())
}

#[test]
fn browser_location_updates_rename_buffer_with_current_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    apply_browser_location_updates(
        &mut state.runtime,
        &[BrowserLocationUpdate {
            buffer_id,
            current_url: "https://docs.rs/volt".to_owned(),
        }],
    )?;

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    assert_eq!(buffer.display_name(), "*browser* https://docs.rs/volt");
    assert_eq!(
        buffer
            .browser_state
            .as_ref()
            .and_then(|browser| browser.current_url.as_deref()),
        Some("https://docs.rs/volt")
    );
    Ok(())
}

#[test]
fn browser_page_load_event_commits_current_url_and_clears_loading() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let user_library = shell_user_library(&state.runtime);
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        request_browser_buffer_navigation(
            buffer,
            "https://example.com/docs",
            false,
            &*user_library,
        );
    }

    state
        .apply_browser_host_events(&[BrowserHostEvent::PageLoadStateChanged {
            buffer_id,
            current_url: "https://example.com/docs".to_owned(),
            is_loading: false,
        }])
        .map_err(|error| error.to_string())?;

    let browser = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?
        .browser_state
        .as_ref()
        .ok_or_else(|| "browser state missing".to_owned())?;
    assert_eq!(
        browser.current_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert_eq!(
        browser.requested_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert!(!browser.is_loading);
    Ok(())
}

#[test]
fn browser_page_load_event_does_not_clobber_a_newer_requested_navigation() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let user_library = shell_user_library(&state.runtime);
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        request_browser_buffer_navigation(buffer, "https://example.com/old", false, &*user_library);
        request_browser_buffer_navigation(buffer, "https://example.com/new", false, &*user_library);
    }

    state
        .apply_browser_host_events(&[BrowserHostEvent::PageLoadStateChanged {
            buffer_id,
            current_url: "https://example.com/old".to_owned(),
            is_loading: false,
        }])
        .map_err(|error| error.to_string())?;

    let browser = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?
        .browser_state
        .as_ref()
        .ok_or_else(|| "browser state missing".to_owned())?;
    assert_eq!(browser.current_url.as_deref(), None);
    assert_eq!(
        browser.requested_url.as_deref(),
        Some("https://example.com/new")
    );
    assert!(browser.is_loading);
    Ok(())
}
