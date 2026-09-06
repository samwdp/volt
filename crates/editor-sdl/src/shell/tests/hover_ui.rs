#![allow(unused_imports)]
use super::*;

#[test]
fn hover_registry_includes_signature_help_provider() {
    let user_library = editor_plugin_host::NullUserLibrary;
    let registry = HoverRegistry::from_user_config(&user_library);
    assert!(matches!(registry.providers[0].kind, HoverProviderKind::Lsp));
    assert!(matches!(
        registry.providers[1].kind,
        HoverProviderKind::SignatureHelp
    ));
    assert_eq!(registry.providers[1].label, "Signature");
    assert_eq!(
        registry.providers[1].icon,
        user_library.hover_signature_icon()
    );
    assert!(matches!(
        registry.providers[2].kind,
        HoverProviderKind::Diagnostics
    ));
}

#[test]
fn hover_signature_request_point_prefers_callee_over_enclosing_macro() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let text = "let commands = vec![hook_command(\"alpha\", \"beta\", \"gamma\", \"delta\")];";
    let cursor_column = text
        .find("hook_command")
        .ok_or_else(|| "hook_command missing".to_owned())?
        + 4;
    let expected_column = text
        .find("(\"alpha\"")
        .ok_or_else(|| "hook_command call missing".to_owned())?
        + 1;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text(text);
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, cursor_column));

    let point = hover_signature_request_point(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    );

    assert_eq!(point, TextPoint::new(0, expected_column));
    Ok(())
}

#[test]
fn hover_signature_request_point_preserves_argument_cursor_context() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let text = "hook_command(name, description, hook_name, detail)";
    let cursor_column = text
        .find("description")
        .ok_or_else(|| "description missing".to_owned())?
        + 3;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text(text);
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, cursor_column));

    let point = hover_signature_request_point(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    );

    assert_eq!(point, TextPoint::new(0, cursor_column));
    Ok(())
}

#[test]
fn hover_test_provider_lines_include_theme_and_treesitter_tokens() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("alpha");
        buffer.set_cursor(TextPoint::new(0, 2));
        buffer.syntax_lines.insert(
            0,
            vec![LineSyntaxSpan {
                start: 0,
                end: 5,
                capture_name: Arc::from("function"),
                theme_token: Arc::from("syntax.function"),
            }],
        );
    }

    let lines = {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        let token_info = completion_token_at_cursor(buffer);
        hover_test_provider_lines(buffer, token_info.as_ref())
    };

    assert!(
        lines
            .iter()
            .any(|line| line == "Theme color: syntax.function")
    );
    assert!(
        lines
            .iter()
            .any(|line| line == "Tree-sitter token: @function")
    );
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
fn insert_mode_closing_brace_does_not_reindent_inline_block() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*inline-closing-brace*",
        vec!["fn main() {".to_owned(), "    ".to_owned(), "}".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(1, 4));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input("if true {")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("}")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if true {}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 14));
    Ok(())
}

#[test]
fn insert_mode_enter_in_tsx_uses_two_space_indent_query() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("tsx")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id =
        install_text_test_buffer(&mut state, "*tsx-enter*", vec!["<div></div>".to_owned()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("tsx".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 5));
    }
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
    assert_eq!(buffer.text.line(0).as_deref(), Some("<div>"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("  "));
    assert_eq!(buffer.text.line(2).as_deref(), Some("</div>"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 2));
    Ok(())
}

#[test]
fn format_current_line_indent_uses_inherited_tsx_queries() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("tsx")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*tsx-indent-query*",
        vec!["<div>".to_owned(), String::new(), "</div>".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("tsx".to_owned()));
        buffer.set_cursor(TextPoint::new(1, 0));
    }

    format_current_line_indent(&mut state.runtime, buffer_id, 2, false)?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("  ")
    );
    Ok(())
}

#[test]
fn vim_open_line_below_in_tsx_uses_inherited_indent_queries() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("tsx")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*tsx-vim-open-line*",
        vec![
            "export default function Dashboard() {".to_owned(),
            "  return (".to_owned(),
            "    <div className=\"flex flex-1 flex-col gap-4 p-4\">".to_owned(),
            "      <div className=\"flex items-center justify-between\">".to_owned(),
            "      </div>".to_owned(),
            "    </div>".to_owned(),
            "  );".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("tsx".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 4));
    }

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(3).as_deref(), Some("      "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 6));
    Ok(())
}

#[test]
fn focused_hover_text_motions_scroll_without_moving_buffer_cursor() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, true)?;
    let cursor_before = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();

    state
        .handle_text_input("3")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 3);

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 4);

    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 3);
    assert_eq!(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?
            .cursor_point(),
        cursor_before
    );
    Ok(())
}

#[test]
fn focused_hover_gg_and_g_scroll_to_expected_bounds() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, true)?;
    let cursor_before = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();

    state
        .handle_text_input("G")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 8);

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 0);

    state
        .handle_text_input("5")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 4);

    state
        .handle_text_input("2")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("0")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("G")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 8);
    assert_eq!(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?
            .cursor_point(),
        cursor_before
    );
    Ok(())
}

#[test]
fn focused_hover_ctrl_scroll_motions_are_bounded() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, true)?;
    let cursor_before = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::D, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 2);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::F, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 6);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::E, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 7);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::Y, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 6);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::B, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 2);

    assert!(
        state
            .handle_focused_hover_keydown(Keycode::U, ctrl_mod())
            .map_err(|error| error.to_string())?
    );
    assert_eq!(hover_scroll_offset(&state)?, 0);
    assert_eq!(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?
            .cursor_point(),
        cursor_before
    );
    Ok(())
}

#[test]
fn visual_indent_shifts_selected_lines_right() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-indent*",
        vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));

    state
        .runtime
        .emit_hook(HOOK_VIM_EDIT, HookEvent::new().with_detail("visual-indent"))
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("    alpha"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    beta"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 4));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}
