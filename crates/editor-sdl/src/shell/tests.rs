use super::*;
use agent_client_protocol::{
    Plan, PlanEntry, PlanEntryPriority, PlanEntryStatus, ToolCall, ToolCallContent, ToolCallStatus,
    ToolCallUpdate, ToolCallUpdateFields, ToolKind,
};
use editor_lsp::LspLogDirection;
use editor_render::horizontal_pane_rects;
use sdl3::mouse::MouseState;

#[derive(Debug, Default)]
struct CommandLog(Vec<String>);

fn slice_by_columns(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect()
}

fn syntax_span_segments(line: &str, spans: &[LineSyntaxSpan]) -> Vec<(String, String)> {
    spans
        .iter()
        .map(|span| {
            (
                span.theme_token.clone(),
                slice_by_columns(line, span.start, span.end),
            )
        })
        .collect()
}

fn unique_temp_dir(label: &str) -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let dir = std::env::temp_dir().join(format!(
        "volt-shell-tests-{label}-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir_all(&dir)
        .unwrap_or_else(|error| panic!("failed to create temp dir `{}`: {error}", dir.display()));
    dir
}

#[test]
fn keydown_chord_maps_alt_x() {
    assert_eq!(
        keydown_chord(Keycode::X, Mod::LALTMOD).as_deref(),
        Some("Alt+x")
    );
}

#[test]
fn terminal_key_for_event_maps_special_keys() {
    assert_eq!(
        terminal_key_for_event(Keycode::Tab, Mod::LSHIFTMOD),
        Some(TerminalKey::BackTab)
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
fn directory_view_state_uses_user_oil_defaults() {
    let defaults = editor_plugin_host::NullUserLibrary.oil_defaults();
    let state = DirectoryViewState::new(std::path::PathBuf::from("."), Vec::new(), defaults);

    assert_eq!(state.show_hidden, defaults.show_hidden);
    assert_eq!(state.sort_mode, defaults.sort_mode);
    assert_eq!(state.trash_enabled, defaults.trash_enabled);
}

#[test]
fn oil_normal_mode_dd_applies_delete_immediately() -> Result<(), String> {
    let root = unique_temp_dir("oil-normal-delete");
    let file_path = root.join("alpha.txt");
    std::fs::write(&file_path, "alpha\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    assert!(!file_path.exists());
    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.line_count(), 1);

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
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
fn lsp_log_buffer_name_includes_server_name() {
    assert_eq!(lsp_log_buffer_name("csharp-ls"), "*lsp-log csharp-ls*");
}

#[test]
fn lsp_log_buffer_lines_only_include_entries_for_requested_server() {
    let entries = vec![
        LspLogEntry::new(LspLogDirection::Outgoing, "csharp-ls", "{\"id\":1}"),
        LspLogEntry::new(LspLogDirection::Incoming, "rust-analyzer", "{\"id\":2}"),
    ];
    let filtered = lsp_log_entries_for_server(&entries, "csharp-ls");
    let lines = lsp_log_buffer_lines("csharp-ls", &filtered);
    let body = lines.join("\n");

    assert!(body.contains("*lsp-log csharp-ls* captures live JSON-RPC traffic for `csharp-ls`."));
    assert!(body.contains("OUT csharp-ls"));
    assert!(!body.contains("rust-analyzer"));
}

#[test]
fn errors_buffer_updates_stay_in_the_background() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let active_before = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing".to_owned())?;

    assert_ne!(active_before.2, "*errors*");
    record_runtime_error(&mut state.runtime, "test.error", "boom");

    let active_after = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after logging".to_owned())?;
    assert_eq!(active_after.0, active_before.0);
    assert_eq!(active_after.1, active_before.1);
    assert_eq!(active_after.2, active_before.2);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, active_before.1);
    Ok(())
}

#[test]
fn lsp_log_buffers_stay_in_the_background_until_explicitly_focused() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let active_before = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing".to_owned())?;

    let buffer_id = ensure_lsp_log_buffer(&mut state.runtime, workspace_id, "rust-analyzer")?;
    let active_after_creation = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after creating log buffer".to_owned())?;

    assert_eq!(active_after_creation.0, active_before.0);
    assert_eq!(active_after_creation.1, active_before.1);
    assert_eq!(active_after_creation.2, active_before.2);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, active_before.1);

    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)?;

    let active_after_focus = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after focusing log buffer".to_owned())?;
    assert_eq!(active_after_focus.1, buffer_id);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);
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
fn draw_buffer_text_inverts_the_cursor_glyph_color() -> Result<(), String> {
    let default_color = Color::RGB(240, 240, 240);
    let inverted_color = Color::RGB(16, 18, 24);
    let line = "abc";
    let char_map = LineCharMap::new(line);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_buffer_text(
        &mut target,
        0,
        0,
        line,
        LineWrapSegment {
            start_col: 0,
            end_col: 3,
        },
        &char_map,
        None,
        None,
        default_color,
        8,
        Some(TextColorOverride {
            start: 1,
            end: 2,
            color: inverted_color,
        }),
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![
            DrawCommand::Text {
                x: 0,
                y: 0,
                text: "a".to_owned(),
                color: to_render_color(default_color),
            },
            DrawCommand::Text {
                x: 8,
                y: 0,
                text: "b".to_owned(),
                color: to_render_color(inverted_color),
            },
            DrawCommand::Text {
                x: 16,
                y: 0,
                text: "c".to_owned(),
                color: to_render_color(default_color),
            },
        ]
    );
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
        0,
        0,
        &formatted,
        LineWrapSegment {
            start_col: 0,
            end_col: formatted.chars().count(),
        },
        &char_map,
        Some(&spans),
        None,
        Color::RGB(240, 240, 240),
        8,
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
fn acp_wrapped_text_uses_full_width_on_continuation_rows() {
    let line = AcpRenderedTextLine {
        prefix: vec![
            acp_icon_segment(editor_icons::symbols::cod::COD_COMMENT, AcpColorRole::Accent),
            acp_text_segment(" ", AcpColorRole::Default),
        ],
        text: "Excellent! Now let me gather more context about the project to inform the documentation content:".to_owned(),
        text_role: AcpColorRole::Default,
    };

    let segments = acp_rendered_text_segments(&line, 32);

    assert!(segments.len() > 1);
    assert!(segments[1].end_col.saturating_sub(segments[1].start_col) > 8);
}

#[test]
fn block_cursor_text_override_uses_segment_relative_utf8_offsets() {
    let line = "aéz";
    let char_map = LineCharMap::new(line);
    let override_info = block_cursor_text_override(
        &char_map,
        LineWrapSegment {
            start_col: 0,
            end_col: 3,
        },
        0,
        0,
        1,
        Some(Color::RGB(1, 2, 3)),
    )
    .expect("cursor on a multibyte character should produce an override");

    assert_eq!(override_info.start, 1);
    assert_eq!(override_info.end, 3);
}

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
fn diagnostic_underlines_clip_to_wrapped_segment_and_draw_errors_last() {
    let diagnostics = vec![
        LspDiagnostic::new(
            "rust-analyzer",
            "info",
            LspDiagnosticSeverity::Information,
            TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 4)),
        ),
        LspDiagnostic::new(
            "rust-analyzer",
            "error",
            LspDiagnosticSeverity::Error,
            TextRange::new(TextPoint::new(0, 1), TextPoint::new(0, 3)),
        ),
    ];
    let line_spans = diagnostic_line_spans_for_diagnostics(&diagnostics);

    assert_eq!(
        diagnostic_underlines_for_segment(
            line_spans.get(&0).map(Box::as_ref).unwrap_or(&[]),
            None,
            6,
            LineWrapSegment {
                start_col: 0,
                end_col: 4,
            },
        ),
        vec![
            DiagnosticUnderlineSpan {
                start_col: 0,
                end_col: 4,
                severity: LspDiagnosticSeverity::Information,
            },
            DiagnosticUnderlineSpan {
                start_col: 1,
                end_col: 3,
                severity: LspDiagnosticSeverity::Error,
            },
        ]
    );
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
            capture_name: "source_file".to_owned(),
            theme_token: "syntax.source".to_owned(),
        },
        LineSyntaxSpan {
            start: 0,
            end: 3,
            capture_name: "keyword".to_owned(),
            theme_token: "syntax.keyword".to_owned(),
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
fn diagnostic_line_spans_cache_multiline_ranges_by_line() {
    let diagnostics = vec![LspDiagnostic::new(
        "rust-analyzer",
        "warning",
        LspDiagnosticSeverity::Warning,
        TextRange::new(TextPoint::new(1, 3), TextPoint::new(3, 2)),
    )];
    let line_spans = diagnostic_line_spans_for_diagnostics(&diagnostics);

    assert_eq!(
        line_spans.get(&1).map(Box::as_ref),
        Some(
            [DiagnosticLineSpan {
                start_col: Some(3),
                end_col: None,
                severity: LspDiagnosticSeverity::Warning,
            }]
            .as_slice()
        )
    );
    assert_eq!(
        line_spans.get(&2).map(Box::as_ref),
        Some(
            [DiagnosticLineSpan {
                start_col: None,
                end_col: None,
                severity: LspDiagnosticSeverity::Warning,
            }]
            .as_slice()
        )
    );
    assert_eq!(
        line_spans.get(&3).map(Box::as_ref),
        Some(
            [DiagnosticLineSpan {
                start_col: None,
                end_col: Some(2),
                severity: LspDiagnosticSeverity::Warning,
            }]
            .as_slice()
        )
    );
}

#[test]
fn draw_diagnostic_undercurl_emits_single_scene_command() -> Result<(), String> {
    let color = Color::RGB(224, 107, 117);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    draw_diagnostic_undercurl(&mut target, 10, 20, 6, 10, color)
        .map_err(|error| error.to_string())?;

    assert_eq!(
        scene,
        vec![DrawCommand::Undercurl {
            x: 10,
            y: 20,
            width: 6,
            line_height: 10,
            color: to_render_color(color),
        }]
    );
    Ok(())
}

fn install_acp_test_buffer(
    state: &mut ShellState,
    output_lines: usize,
    input_text: &str,
    hint: Option<&str>,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*acp test*",
            BufferKind::Plugin(ACP_BUFFER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP test buffer is missing".to_owned())?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(
        buffer,
        (1..=output_lines)
            .map(|index| format!("line {index}"))
            .collect(),
        &NullUserLibrary,
    );
    if let Some(input) = shell_buffer.input_field_mut() {
        input.set_text(input_text);
        input.set_hint(hint.map(str::to_owned));
    }
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_plugin_sections_test_buffer(
    state: &mut ShellState,
    input_lines: &[&str],
    output_lines: &[&str],
) -> Result<BufferId, String> {
    install_plugin_sections_test_buffer_with_update(
        state,
        input_lines,
        output_lines,
        editor_plugin_api::PluginBufferSectionUpdate::Replace,
    )
}

fn install_plugin_sections_test_buffer_with_update(
    state: &mut ShellState,
    input_lines: &[&str],
    output_lines: &[&str],
    update: editor_plugin_api::PluginBufferSectionUpdate,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*calculator test*",
            BufferKind::Plugin(buffer_kinds::CALCULATOR.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "plugin test buffer is missing".to_owned())?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(
        buffer,
        input_lines.iter().map(|line| (*line).to_owned()).collect(),
        &NullUserLibrary,
    );
    let output = if output_lines.is_empty() {
        vec!["(press Ctrl+c Ctrl+c to evaluate)".to_owned()]
    } else {
        output_lines.iter().map(|line| (*line).to_owned()).collect()
    };
    shell_buffer.plugin_section_state = PluginSectionBufferState::new(
        PluginBufferSections::new(vec![
            editor_plugin_api::PluginBufferSection::new("Input")
                .with_writable(true)
                .with_initial_lines(input_lines.iter().map(|line| (*line).to_owned()).collect()),
            editor_plugin_api::PluginBufferSection::new("Output")
                .with_min_lines(1)
                .with_initial_lines(output)
                .with_update(update),
        ]),
        Some("Output"),
    );
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_scratch_test_buffer(state: &mut ShellState, name: &str) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, name, BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        name,
        BufferKind::Scratch,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)?;
    Ok(buffer_id)
}

fn focus_test_buffer(state: &mut ShellState, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)
}

fn install_browser_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        BROWSER_BUFFER_NAME,
        BufferKind::Plugin(BROWSER_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_terminal_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, "*terminal*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*terminal*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn install_terminal_popup_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_popup_buffer(workspace_id, "*terminal-popup*", BufferKind::Terminal, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .open_popup(workspace_id, "Terminal", vec![buffer_id], buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_popup_buffer(
        buffer_id,
        "*terminal-popup*",
        BufferKind::Terminal,
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.set_popup_focus(true);
    Ok(buffer_id)
}

fn install_git_status_test_buffer(state: &mut ShellState) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*git-status*",
            BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.ensure_buffer(
        buffer_id,
        "*git-status*",
        BufferKind::Plugin(GIT_STATUS_KIND.to_owned()),
        &NullUserLibrary,
    );
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn run_git_in_dir(root: &std::path::Path, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| format!("failed to run git {:?}: {error}", args))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if stderr.is_empty() {
            format!("git {:?} failed with status {}", args, output.status)
        } else {
            format!("git {:?} failed: {stderr}", args)
        };
        return Err(message);
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn init_git_repo_with_commit(label: &str) -> Result<std::path::PathBuf, String> {
    let repo = unique_temp_dir(label);
    run_git_in_dir(&repo, &["init", "-q"])?;
    run_git_in_dir(&repo, &["config", "user.email", "volt-tests@example.com"])?;
    run_git_in_dir(&repo, &["config", "user.name", "Volt Tests"])?;
    run_git_in_dir(&repo, &["config", "commit.gpgsign", "false"])?;
    std::fs::write(repo.join("README.md"), "seed\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "README.md"])?;
    run_git_in_dir(&repo, &["commit", "-qm", "initial"])?;
    Ok(repo)
}

fn open_repo_git_status_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "git-test", root)?;
    let buffer_id = install_git_status_test_buffer(state)?;
    refresh_git_status_buffer(&mut state.runtime, buffer_id)?;
    Ok(buffer_id)
}

fn open_oil_test_buffer(
    state: &mut ShellState,
    root: &std::path::Path,
) -> Result<BufferId, String> {
    open_workspace_from_project(&mut state.runtime, "oil-test", root)?;
    open_oil_directory(&mut state.runtime, root.to_path_buf())?;
    active_shell_buffer_id(&state.runtime)
}

fn install_text_test_buffer(
    state: &mut ShellState,
    name: &str,
    lines: Vec<String>,
) -> Result<BufferId, String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(workspace_id, name, BufferKind::Scratch, None)
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "text test buffer is missing".to_owned())?;
    let shell_buffer = ShellBuffer::from_runtime_buffer(buffer, lines, &NullUserLibrary);
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn screen_point_for_buffer_point(
    state: &mut ShellState,
    buffer_id: BufferId,
    point: TextPoint,
    render_width: u32,
    render_height: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<(f32, f32), String> {
    let original_cursor = shell_buffer(&state.runtime, buffer_id)?.cursor_point();
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(point);
    let anchor = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer_cursor_screen_anchor(
            buffer,
            PixelRectToRect::rect(0, 0, render_width, render_height),
            state.runtime.services().get::<ThemeRegistry>(),
            cell_width,
            line_height,
        )
        .ok_or_else(|| "buffer cursor screen anchor was missing".to_owned())?
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(original_cursor);
    Ok((
        (anchor.x + (cell_width / 2).max(1)) as f32,
        (anchor.y + (line_height / 2).max(1)) as f32,
    ))
}

fn git_status_line_for_action_detail(
    state: &ShellState,
    buffer_id: BufferId,
    action_id: &str,
    detail: &str,
) -> Result<usize, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    (0..buffer.line_count())
        .find(|line_index| {
            git_action_detail(buffer.section_line_meta(*line_index), action_id).as_deref()
                == Some(detail)
        })
        .ok_or_else(|| format!("git status line for `{detail}` and `{action_id}` was not found"))
}

fn set_git_status_visual_line_selection(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    let (start_line, end_line) = if start_line <= end_line {
        (start_line, end_line)
    } else {
        (end_line, start_line)
    };
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(start_line, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(end_line, 0));
    Ok(())
}

fn set_git_status_visual_block_selection_with_ctrl_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    assert!(
        state
            .try_runtime_keybinding(Keycode::V, ctrl_mod())
            .map_err(|error| error.to_string())?
    );

    state
        .handle_text_input("v")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Block
    );
    Ok(())
}

fn set_git_status_visual_line_selection_with_shift_v(
    state: &mut ShellState,
    buffer_id: BufferId,
    start_line: usize,
    end_line: usize,
) -> Result<(), String> {
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(start_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;

    let motion = if end_line >= start_line { "j" } else { "k" };
    for _ in 0..start_line.abs_diff(end_line) {
        state
            .handle_text_input(motion)
            .map_err(|error| error.to_string())?;
    }

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Visual);
    assert_eq!(
        shell_ui(&state.runtime)?.vim().visual_kind,
        VisualSelectionKind::Line
    );
    Ok(())
}

type GitSnapshotPaths = (BTreeSet<String>, BTreeSet<String>, BTreeSet<String>);

fn git_status_snapshot_paths(
    state: &ShellState,
    buffer_id: BufferId,
) -> Result<GitSnapshotPaths, String> {
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let snapshot = buffer
        .git_snapshot()
        .ok_or_else(|| "git snapshot missing".to_owned())?;
    let staged = snapshot
        .staged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let unstaged = snapshot
        .unstaged()
        .iter()
        .map(|entry| entry.path().to_owned())
        .collect();
    let untracked = snapshot.untracked().iter().cloned().collect();
    Ok((staged, unstaged, untracked))
}

fn install_hover_test_overlay(state: &mut ShellState, focused: bool) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![
            HoverProviderContent {
                provider_label: "Alpha".to_owned(),
                provider_icon: "A".to_owned(),
                lines: vec!["first".to_owned()],
            },
            HoverProviderContent {
                provider_label: "Beta".to_owned(),
                provider_icon: "B".to_owned(),
                lines: vec!["second".to_owned()],
            },
            HoverProviderContent {
                provider_label: "Gamma".to_owned(),
                provider_icon: "G".to_owned(),
                lines: vec!["third".to_owned()],
            },
        ],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 8,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

fn install_scrollable_hover_test_overlay(
    state: &mut ShellState,
    focused: bool,
) -> Result<BufferId, String> {
    let buffer_id = shell_ui(&state.runtime)?
        .active_buffer_id()
        .ok_or_else(|| "active buffer missing".to_owned())?;
    let anchor = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .cursor_point();
    let lines = (1..=12)
        .map(|line| format!("Line {line}"))
        .collect::<Vec<_>>();
    shell_ui_mut(&mut state.runtime)?.set_hover(HoverOverlay {
        buffer_id,
        anchor,
        token: "hover".to_owned(),
        providers: vec![HoverProviderContent {
            provider_label: "Scrollable".to_owned(),
            provider_icon: "S".to_owned(),
            lines,
        }],
        provider_index: 0,
        scroll_offset: 0,
        focused,
        line_limit: 4,
        pending_g_prefix: false,
        count: None,
    });
    Ok(buffer_id)
}

fn hover_scroll_offset(state: &ShellState) -> Result<usize, String> {
    shell_ui(&state.runtime)?
        .hover()
        .map(|hover| hover.scroll_offset)
        .ok_or_else(|| "hover overlay missing".to_owned())
}

fn test_notification_update(
    key: &str,
    severity: NotificationSeverity,
    title: &str,
    body_lines: &[&str],
    progress: Option<u8>,
    active: bool,
) -> NotificationUpdate {
    NotificationUpdate {
        key: key.to_owned(),
        severity,
        title: title.to_owned(),
        body_lines: body_lines.iter().map(|line| (*line).to_owned()).collect(),
        progress: progress.map(|percentage| NotificationProgress {
            percentage: Some(percentage),
        }),
        active,
        action: None,
    }
}

#[test]
fn parse_rg_workspace_search_line_extracts_location() {
    let parsed = parse_rg_workspace_search_line(r"src\main.rs:12:7:let answer = compute();")
        .expect("rg output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\main.rs");
    assert_eq!(parsed.1, 12);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "let answer = compute();");
}

#[test]
fn parse_grep_workspace_search_line_finds_case_insensitive_column() {
    let parsed = parse_grep_workspace_search_line(r"src\lib.rs:3:Hello Workspace", "workspace")
        .expect("grep output should parse into a workspace search match");
    assert_eq!(parsed.0, r"src\lib.rs");
    assert_eq!(parsed.1, 3);
    assert_eq!(parsed.2, 7);
    assert_eq!(parsed.3, "Hello Workspace");
}

#[test]
fn workspace_search_char_column_handles_utf8_offsets() {
    assert_eq!(workspace_search_char_column("aébc", 0), 0);
    assert_eq!(workspace_search_char_column("aébc", 1), 1);
    assert_eq!(workspace_search_char_column("aébc", 3), 2);
}

#[test]
fn frame_pacing_remaining_clamps_to_120fps_budget() {
    let now = Instant::now();
    let remaining = frame_pacing_remaining(now - Duration::from_millis(2), now);
    assert!(remaining >= Duration::from_micros(6_000));
    assert_eq!(
        frame_pacing_remaining(now - Duration::from_millis(10), now),
        Duration::from_secs(0)
    );
}

#[test]
fn git_refresh_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(git_refresh_deferred_for_typing(Some(now), now));
    assert!(git_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!git_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!git_refresh_deferred_for_typing(None, now));
}

#[test]
fn frame_pacing_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(frame_pacing_deferred_for_typing(Some(now), now));
    assert!(frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(None, now));
}

#[test]
fn typing_event_batches_yield_once_budget_is_exhausted() {
    let now = Instant::now();
    assert!(!should_yield_after_typing_batch(
        0,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(!should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT - 1,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        1,
        now - TYPING_EVENT_BATCH_TIME_BUDGET
    ));
}

#[test]
fn truncate_text_to_width_uses_cell_budget() {
    assert_eq!(truncate_text_to_width("abcdef", 24, 4), "abcdef");
    assert_eq!(truncate_text_to_width("abcdef", 20, 4), "ab...");
    assert_eq!(truncate_text_to_width("abcdef", 8, 4), "...");
}

#[test]
fn git_status_header_spans_skip_leading_icons() {
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

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(),
                editor_icons::symbols::dev::DEV_GIT_BRANCH.to_owned(),
            ),
            (TOKEN_GIT_STATUS_HEADER_LABEL.to_owned(), "Head:".to_owned()),
            (
                TOKEN_GIT_STATUS_HEADER_VALUE.to_owned(),
                "master".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_HASH.to_owned(),
                "f9d8c15".to_owned()
            ),
            (
                TOKEN_GIT_STATUS_HEADER_SUMMARY.to_owned(),
                "Added some more keybinds".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_entry_spans_skip_leading_icons() {
    let line = SectionRenderLine {
        text: format!(
            "{} crates/editor-sdl/src/shell.rs",
            editor_icons::symbols::cod::COD_DIFF_MODIFIED
        ),
        depth: 1,
        section_id: GIT_SECTION_UNSTAGED.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_ENTRY_MODIFIED.to_owned(),
                editor_icons::symbols::cod::COD_DIFF_MODIFIED.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_ENTRY_PATH.to_owned(),
                "crates/editor-sdl/src/shell.rs".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_stash_spans_handle_compact_stash_names() {
    let line = SectionRenderLine {
        text: format!(
            "{} stash[0] WIP on master: overnight todo",
            editor_icons::symbols::cod::COD_HISTORY
        ),
        depth: 1,
        section_id: GIT_SECTION_STASHES.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![
            (
                TOKEN_GIT_STATUS_STASH_NAME.to_owned(),
                editor_icons::symbols::cod::COD_HISTORY.to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_STASH_NAME.to_owned(),
                "stash[0]".to_owned(),
            ),
            (
                TOKEN_GIT_STATUS_STASH_SUMMARY.to_owned(),
                "WIP on master: overnight todo".to_owned(),
            ),
        ]
    );
}

#[test]
fn git_status_uppercase_f_starts_pull_prefix() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_git_status_test_buffer(&mut state)?;

    assert!(handle_git_status_chord(&mut state.runtime, "F")?);
    assert_eq!(take_git_prefix(&mut state.runtime)?, Some(GitPrefix::Pull));
    Ok(())
}

#[test]
fn git_status_sequence_commands_are_registered() -> Result<(), String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;

    for &(name, _, _) in GIT_STATUS_COMMANDS {
        assert!(
            state.runtime.commands().contains(name),
            "missing command `{name}`"
        );
    }
    for name in ["git.diff", "git.log", "git.stash-list"] {
        assert!(
            state.runtime.commands().contains(name),
            "missing command `{name}`"
        );
    }

    Ok(())
}

#[test]
fn git_status_command_name_maps_sequences_to_picker_commands() {
    assert_eq!(
        git_status_command_name(&NullUserLibrary, None, "S"),
        Some("git.status.stage-all")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Pull), "u"),
        Some("git.status.pull-upstream")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Branch), "b"),
        Some("git.status.branches")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Diff), "w"),
        Some("git.diff")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Log), "l"),
        Some("git.log")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Stash), "l"),
        Some("git.stash-list")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Rebase), "f"),
        Some("git.status.rebase-autosquash")
    );
    assert_eq!(
        git_status_command_name(&NullUserLibrary, Some(GitPrefix::Reset), "f"),
        Some("git.status.checkout-file")
    );
}

#[test]
fn git_status_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-visual-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection(&mut state, buffer_id, alpha, beta)?;

    assert!(handle_git_status_chord(&mut state.runtime, "s")?);

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-visual-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection(&mut state, buffer_id, alpha, beta)?;

    assert!(handle_git_status_chord(&mut state.runtime, "u")?);

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("s")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("u")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_ctrl_v_visual_x_deletes_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-ctrl-v-delete")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_block_selection_with_ctrl_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(staged.is_empty());
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(!repo.join("alpha.txt").exists());
    assert!(!repo.join("beta.txt").exists());
    assert!(repo.join("gamma.txt").exists());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_s_stages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-stage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("s")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(
        staged,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_u_unstages_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-unstage")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;
    run_git_in_dir(&repo, &["add", "--", "alpha.txt", "beta.txt", "gamma.txt"])?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_UNSTAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("u")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert_eq!(staged, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(unstaged.is_empty());
    assert_eq!(
        untracked,
        BTreeSet::from(["alpha.txt".to_owned(), "beta.txt".to_owned()])
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_status_shift_v_visual_x_deletes_selected_items() -> Result<(), String> {
    let repo = init_git_repo_with_commit("git-status-shift-v-delete")?;
    std::fs::write(repo.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("beta.txt"), "beta\n").map_err(|error| error.to_string())?;
    std::fs::write(repo.join("gamma.txt"), "gamma\n").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = open_repo_git_status_buffer(&mut state, &repo)?;
    let alpha =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "alpha.txt")?;
    let beta =
        git_status_line_for_action_detail(&state, buffer_id, GIT_ACTION_STAGE_FILE, "beta.txt")?;
    set_git_status_visual_line_selection_with_shift_v(&mut state, buffer_id, alpha, beta)?;

    assert_eq!(
        git_status_selected_lines(&state.runtime, buffer_id)?,
        ((alpha..=beta).collect(), true)
    );

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let (staged, unstaged, untracked) = git_status_snapshot_paths(&state, buffer_id)?;
    assert!(staged.is_empty());
    assert!(unstaged.is_empty());
    assert_eq!(untracked, BTreeSet::from(["gamma.txt".to_owned()]));
    assert!(!repo.join("alpha.txt").exists());
    assert!(!repo.join("beta.txt").exists());
    assert!(repo.join("gamma.txt").exists());
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    std::fs::remove_dir_all(&repo).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn git_line_is_untracked_uses_section_metadata() {
    let meta = SectionLineMeta {
        section_id: GIT_SECTION_UNTRACKED.to_owned(),
        kind: SectionRenderLineKind::Item,
        action: None,
    };
    let staged_meta = SectionLineMeta {
        section_id: GIT_SECTION_UNSTAGED.to_owned(),
        kind: SectionRenderLineKind::Item,
        action: None,
    };

    assert!(git_line_is_untracked(Some(&meta)));
    assert!(!git_line_is_untracked(Some(&staged_meta)));
    assert!(!git_line_is_untracked(None));
}

#[test]
fn git_status_commit_message_spans_use_command_token_with_icon_prefix() {
    let line = SectionRenderLine {
        text: format!(
            "{} Press c to commit staged changes.",
            editor_icons::symbols::cod::COD_GIT_COMMIT
        ),
        depth: 1,
        section_id: GIT_SECTION_COMMIT.to_owned(),
        action: None,
        kind: SectionRenderLineKind::Item,
    };
    let formatted = format_section_line(&line);
    let spans = git_status_line_spans(&line, &formatted);

    assert_eq!(
        syntax_span_segments(&formatted, &spans),
        vec![(
            TOKEN_GIT_STATUS_COMMAND.to_owned(),
            format!(
                "{} Press c to commit staged changes.",
                editor_icons::symbols::cod::COD_GIT_COMMIT
            ),
        )]
    );
}

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
fn statusline_icon_segments_split_acp_and_lsp_icons() {
    let user_library = editor_plugin_host::NullUserLibrary;
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
    let user_library = editor_plugin_host::NullUserLibrary;
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
fn notification_center_updates_entries_and_expires_completed_toasts() {
    let now = Instant::now();
    let mut center = NotificationCenter::default();
    assert!(center.apply(
        test_notification_update(
            "progress",
            NotificationSeverity::Info,
            "LSP · rust-analyzer",
            &["Indexing", "Scanning workspace"],
            Some(24),
            true,
        ),
        now,
    ));
    assert_eq!(center.visible(now).len(), 1);
    assert!(center.visible(now)[0].active);

    assert!(center.apply(
        test_notification_update(
            "progress",
            NotificationSeverity::Success,
            "LSP · rust-analyzer",
            &["Indexed workspace"],
            Some(100),
            false,
        ),
        now + Duration::from_millis(25),
    ));
    let visible = center.visible(now + Duration::from_millis(25));
    assert_eq!(visible.len(), 1);
    assert!(!visible[0].active);
    assert_eq!(visible[0].severity, NotificationSeverity::Success);

    assert!(!center.prune_expired(now + NOTIFICATION_AUTO_DISMISS - Duration::from_millis(1)));
    assert!(center.prune_expired(now + NOTIFICATION_AUTO_DISMISS + Duration::from_millis(50)));
    assert!(
        center
            .visible(now + NOTIFICATION_AUTO_DISMISS + Duration::from_millis(50))
            .is_empty()
    );
}

#[test]
fn notification_center_prioritizes_active_toasts_with_visible_limit() {
    let now = Instant::now();
    let mut center = NotificationCenter::default();
    assert!(center.apply(
        test_notification_update(
            "old-complete",
            NotificationSeverity::Success,
            "Done",
            &["Completed task"],
            None,
            false,
        ),
        now,
    ));
    assert!(center.apply(
        test_notification_update(
            "active-a",
            NotificationSeverity::Info,
            "Active A",
            &["Working"],
            Some(10),
            true,
        ),
        now + Duration::from_millis(10),
    ));
    assert!(center.apply(
        test_notification_update(
            "active-b",
            NotificationSeverity::Info,
            "Active B",
            &["Working"],
            Some(40),
            true,
        ),
        now + Duration::from_millis(20),
    ));
    assert!(center.apply(
        test_notification_update(
            "active-c",
            NotificationSeverity::Warning,
            "Active C",
            &["Working"],
            None,
            true,
        ),
        now + Duration::from_millis(30),
    ));
    assert!(center.apply(
        test_notification_update(
            "new-complete",
            NotificationSeverity::Success,
            "Done",
            &["Completed task"],
            None,
            false,
        ),
        now + Duration::from_millis(40),
    ));

    let visible = center.visible(now + Duration::from_millis(40));
    assert_eq!(visible.len(), NOTIFICATION_VISIBLE_LIMIT);
    assert!(visible.iter().all(|notification| notification.active));
    assert_eq!(visible[0].key, "active-c");
    assert_eq!(visible[1].key, "active-b");
    assert_eq!(visible[2].key, "active-a");
}

#[test]
fn notification_action_at_point_returns_acp_permission_action() -> Result<(), String> {
    let now = Instant::now();
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "acp.permission.42".to_owned(),
            severity: NotificationSeverity::Warning,
            title: "project Read file is requesting permission".to_owned(),
            body_lines: vec!["Allow once".to_owned(), "Reject once".to_owned()],
            progress: None,
            active: true,
            action: Some(NotificationAction::OpenAcpPermissionPicker { request_id: 42 }),
        },
        now,
    );

    let ui = shell_ui(&state.runtime)?;
    let layouts = notification_overlay_layouts(
        &ui.visible_notifications(now),
        render_width,
        render_height,
        cell_width,
        line_height,
    );
    let rect = layouts
        .first()
        .map(|layout| layout.rect)
        .ok_or_else(|| "notification layout missing".to_owned())?;
    let action = notification_action_at_point(
        ui,
        render_width,
        render_height,
        cell_width,
        line_height,
        now,
        (rect.x() + 4, rect.y() + 4),
    );

    assert_eq!(
        action,
        Some(NotificationAction::OpenAcpPermissionPicker { request_id: 42 })
    );
    Ok(())
}

#[test]
fn acp_footer_layout_orders_output_input_hint_and_statusline() -> Result<(), String> {
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
    let output_bottom = layout.body_y + layout.visible_rows as i32 * 18;
    let hint_y = layout.input_y + layout.input_box_height + layout.input_hint_gap;

    assert!(output_bottom <= layout.input_y);
    assert!(layout.input_y < hint_y);
    assert!(hint_y < layout.statusline_y);
    Ok(())
}

#[test]
fn plugin_sections_layout_keeps_output_pane_at_bottom_with_single_row_start() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(
        &mut state,
        &["a = 1", "b = 2", "sqrt(a + b)"],
        &["(press Ctrl+c Ctrl+c to evaluate)"],
    )?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let panes = plugin_section_buffer_layout(buffer, rect, layout, 8, 18)
        .ok_or_else(|| "plugin section layout missing".to_owned())?;

    assert_eq!(panes.panes[1].visible_rows, 1);
    assert!(panes.panes[0].rect.y() >= layout.body_y);
    assert!(
        panes.panes[0].rect.y() + panes.panes[0].rect.height() as i32 <= panes.panes[1].rect.y()
    );
    assert!(panes.panes[1].rect.y() + panes.panes[1].rect.height() as i32 <= layout.pane_bottom);
    Ok(())
}

#[test]
fn plugin_sections_switching_output_pane_changes_focus_and_read_only_state() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(&mut state, &["a = 1"], &["1"])?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;

    assert_eq!(buffer.plugin_active_section_index(), Some(0));
    assert!(!buffer.is_read_only());

    assert!(buffer.plugin_switch_pane());
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());

    assert!(buffer.plugin_switch_pane());
    assert_eq!(buffer.plugin_active_section_index(), Some(0));
    assert!(!buffer.is_read_only());
    Ok(())
}

#[test]
fn plugin_sections_replace_output_lines_in_place() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_plugin_sections_test_buffer(&mut state, &["a = 1"], &["old", "lines"])?;

    shell_buffer_mut(&mut state.runtime, buffer_id)?
        .set_plugin_output_lines(vec!["2".to_owned(), "3".to_owned()]);

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let state = buffer
        .plugin_sections()
        .ok_or_else(|| "plugin section state missing".to_owned())?;
    let output = state
        .attached_section(1)
        .ok_or_else(|| "output section missing".to_owned())?;
    let lines = (0..output.line_count())
        .map(|index| output.text.line(index).unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(lines, vec!["2", "3"]);
    Ok(())
}

#[test]
fn plugin_sections_can_append_output_lines() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_plugin_sections_test_buffer_with_update(
        &mut state,
        &["a = 1"],
        &["old"],
        editor_plugin_api::PluginBufferSectionUpdate::Append,
    )?;

    shell_buffer_mut(&mut state.runtime, buffer_id)?
        .set_plugin_output_lines(vec!["new".to_owned()]);

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let state = buffer
        .plugin_sections()
        .ok_or_else(|| "plugin section state missing".to_owned())?;
    let output = state
        .attached_section(1)
        .ok_or_else(|| "output section missing".to_owned())?;
    let lines = (0..output.line_count())
        .map(|index| output.text.line(index).unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(lines, vec!["old", "new"]);
    Ok(())
}

#[test]
fn sync_active_viewport_matches_acp_footer_visible_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(
        &mut state,
        40,
        "first line\nsecond line",
        Some("chat · gpt-5.4 · shift+tab switch mode"),
    )?;

    state
        .sync_active_viewport(400, 18)
        .map_err(|error| error.to_string())?;

    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let layout = buffer_footer_layout(buffer, PixelRectToRect::rect(0, 0, 800, 400), 18, 8);
    assert_eq!(buffer.viewport_lines(), layout.visible_rows);

    buffer.scroll_output_to_end();
    buffer.append_output_lines(&["tail".to_owned()]);

    assert!(
        buffer.line_at_viewport_offset(buffer.viewport_lines().saturating_sub(1)) + 1
            >= buffer.line_count()
    );
    Ok(())
}

#[test]
fn acp_plan_entries_populate_static_plan_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(vec![
        PlanEntry::new(
            "Render the ACP plan pane",
            PlanEntryPriority::High,
            PlanEntryStatus::Pending,
        ),
        PlanEntry::new(
            "Stream tool output into cards",
            PlanEntryPriority::Medium,
            PlanEntryStatus::InProgress,
        ),
    ]));

    let acp = buffer.acp_state.as_ref().expect("ACP state missing");
    assert_eq!(acp.plan_entries.len(), 2);
    match &acp.plan_pane.render_lines[0] {
        AcpRenderedLine::Text(line) => {
            assert_eq!(line.text, "Render the ACP plan pane");
            assert_eq!(line.prefix[0].role, AcpColorRole::PriorityHigh);
        }
        other => panic!("expected text line, got {other:?}"),
    }
    match &acp.plan_pane.render_lines[1] {
        AcpRenderedLine::Text(line) => {
            assert_eq!(line.text, "Stream tool output into cards");
            assert!(line.prefix[0].animate);
        }
        other => panic!("expected text line, got {other:?}"),
    }
    Ok(())
}

#[test]
fn acp_tool_call_updates_replace_existing_output_item() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_upsert_tool_call(
        ToolCall::new("tool-1", "Read file")
            .kind(ToolKind::Read)
            .status(ToolCallStatus::Pending),
    );
    buffer.acp_update_tool_call(ToolCallUpdate::new(
        "tool-1",
        ToolCallUpdateFields::new()
            .title("Read src\\main.rs")
            .status(ToolCallStatus::Completed)
            .content(vec![ToolCallContent::from("Loaded 42 lines")]),
    ));

    let acp = buffer.acp_state.as_ref().expect("ACP state missing");
    let tool_calls = acp
        .output_items
        .iter()
        .filter_map(|item| match item {
            AcpOutputItem::ToolCall(tool_call) => Some(tool_call),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].title, "Read src\\main.rs");
    assert_eq!(tool_calls[0].status, ToolCallStatus::Completed);
    assert_eq!(tool_calls[0].content.len(), 1);
    assert_eq!(acp.tool_item_indices.len(), 1);
    Ok(())
}

#[test]
fn acp_plan_height_caps_wrapped_content_at_ten_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(
        (0..4)
            .map(|index| {
                PlanEntry::new(
                    format!(
                        "ACP plan item {index} should wrap several visual rows in a narrow pane so the plan height clamp is exercised"
                    ),
                    PlanEntryPriority::Medium,
                    PlanEntryStatus::Pending,
                )
            })
            .collect(),
    ));

    buffer.sync_acp_viewport_metrics(220, 420, 8, 16);

    let acp = buffer.acp_state.as_ref().expect("ACP state missing");
    assert_eq!(acp.plan_pane.visible_rows(), 10);
    assert!(acp.output_pane.visible_rows() >= 1);
    Ok(())
}

#[test]
fn acp_scroll_output_to_end_reaches_last_rendered_line() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_set_plan(Plan::new(vec![PlanEntry::new(
        "Keep the plan compact",
        PlanEntryPriority::Medium,
        PlanEntryStatus::InProgress,
    )]));
    for index in 0..48 {
        buffer.acp_push_system_message(format!("output line {index}"));
    }

    buffer.sync_acp_viewport_metrics(800, 400, 8, 16);
    buffer.scroll_output_to_end();

    assert!(
        buffer.line_at_viewport_offset(buffer.viewport_lines().saturating_sub(1)) + 1
            >= buffer.line_count()
    );
    Ok(())
}

#[test]
fn acp_visual_selection_uses_output_text_without_prefix() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");
    buffer.acp_push_system_message("alpha beta");
    let line_index = buffer.line_count().saturating_sub(1);
    buffer.set_cursor(TextPoint::new(line_index, 4));

    let selection = visual_selection(
        buffer,
        TextPoint::new(line_index, 0),
        VisualSelectionKind::Character,
    )
    .ok_or_else(|| "visual selection should not be empty".to_owned())?;
    let VisualSelection::Range(range) = selection else {
        return Err("expected a range selection".to_owned());
    };

    assert_eq!(buffer.slice(range), "alpha");
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

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let selection_color = Color::RGBA(55, 71, 99, 255);
    let line_index = buffer.line_count().saturating_sub(1);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        buffer,
        rect,
        layout,
        true,
        Some(VisualSelection::Range(TextRange::new(
            TextPoint::new(line_index, 0),
            TextPoint::new(line_index, 5),
        ))),
        None,
        InputMode::Visual,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(215, 221, 232),
        Color::RGB(140, 144, 152),
        Color::RGB(40, 44, 52),
        selection_color,
        Color::RGBA(112, 196, 255, 120),
        Color::RGB(110, 170, 255),
        2,
        8,
        16,
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRect { color, .. } if *color == to_render_color(selection_color)
    )));
    Ok(())
}

#[test]
fn render_acp_headers_use_rounded_caps() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_acp_test_buffer(&mut state, 0, "", None)?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    buffer.init_acp_view("GitHub Copilot");

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let acp_layout = acp_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "missing ACP layout".to_owned())?;
    let header_height = (16 + 10) as u32;
    let header_radius = 9.min(header_height / 2);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_acp_buffer_body(
        &mut target,
        buffer,
        rect,
        layout,
        true,
        None,
        None,
        InputMode::Normal,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(215, 221, 232),
        Color::RGB(140, 144, 152),
        Color::RGB(40, 44, 52),
        Color::RGBA(55, 71, 99, 255),
        Color::RGBA(112, 196, 255, 120),
        Color::RGB(110, 170, 255),
        2,
        8,
        16,
    )
    .map_err(|error| error.to_string())?;

    for pane in [acp_layout.plan, acp_layout.output] {
        assert!(scene.iter().any(|command| matches!(
            command,
            DrawCommand::FillRoundedRect { rect, radius, .. }
                if rect.x == pane.rect.x() + 1
                    && rect.y == pane.rect.y() + 1
                    && rect.width == pane.rect.width().saturating_sub(2)
                    && rect.height == header_height
                    && *radius == header_radius
        )));
    }
    Ok(())
}

#[test]
fn sync_active_viewport_uses_active_pane_height_for_horizontal_splits() -> Result<(), String> {
    let render_width = 640;
    let render_height = 320;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_text_test_buffer(
        &mut state,
        "*split-viewport*",
        (0..120).map(|index| format!("line {index}")).collect(),
    )?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Horizontal)?;

    state
        .sync_active_viewport_for_render_size(render_width, render_height, line_height)
        .map_err(|error| error.to_string())?;

    let pane_rect = horizontal_pane_rects(render_width, render_height, 2)
        .into_iter()
        .next()
        .ok_or_else(|| "horizontal split did not produce a pane rect".to_owned())?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;
    let pane_layout = buffer_footer_layout(
        buffer,
        PixelRectToRect::rect(pane_rect.x, pane_rect.y, pane_rect.width, pane_rect.height),
        line_height,
        8,
    );
    let full_layout = buffer_footer_layout(
        buffer,
        PixelRectToRect::rect(0, 0, render_width, render_height),
        line_height,
        8,
    );

    assert_eq!(buffer.viewport_lines(), pane_layout.visible_rows);
    assert!(pane_layout.visible_rows < full_layout.visible_rows);
    Ok(())
}

#[test]
fn sync_visible_buffer_layouts_use_split_width_for_vertical_splits() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let line = format!(
        "const wrapped_line = {};",
        "abcdefghijklmnopqrstuvwxyz".repeat(8)
    );
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*split-wrap*",
        (0..120).map(|_| line.clone()).collect(),
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(80, 80));

    state
        .sync_active_viewport_for_render_size(render_width, render_height, line_height)
        .map_err(|error| error.to_string())?;
    {
        let visible_rows = shell_buffer(&state.runtime, buffer_id)?.viewport_lines();
        let indent_size = theme_lang_indent(
            state.runtime.services().get::<ThemeRegistry>(),
            shell_buffer(&state.runtime, buffer_id)?.language_id(),
        );
        shell_buffer_mut(&mut state.runtime, buffer_id)?.ensure_visible(
            visible_rows,
            wrap_columns_for_width(render_width, cell_width),
            indent_size,
        );
    }
    shell_ui_mut(&mut state.runtime)?
        .workspace_view_mut()
        .ok_or_else(|| "workspace view is missing".to_owned())?
        .split_buffer_id = buffer_id;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    install_acp_test_buffer(
        &mut state,
        40,
        "",
        Some("chat · gpt-5.4 · shift+tab switch mode"),
    )?;

    let pane_rect = vertical_pane_rects(render_width, render_height, 2)
        .into_iter()
        .nth(1)
        .ok_or_else(|| "vertical split did not produce a right pane rect".to_owned())?;
    let before_sync = buffer_cursor_screen_anchor(
        shell_buffer(&state.runtime, buffer_id)?,
        PixelRectToRect::rect(pane_rect.x, pane_rect.y, pane_rect.width, pane_rect.height),
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
    );

    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let after_sync = buffer_cursor_screen_anchor(
        buffer,
        PixelRectToRect::rect(pane_rect.x, pane_rect.y, pane_rect.width, pane_rect.height),
        state.runtime.services().get::<ThemeRegistry>(),
        cell_width,
        line_height,
    );
    assert!(before_sync.is_none());
    assert!(after_sync.is_some());
    Ok(())
}

#[test]
fn material_icons_rasterize_from_nfm_with_fontdue() -> Result<(), String> {
    let font_path = resolve_bundled_icon_font_dir()
        .map_err(|error| error.to_string())?
        .join("NFM.ttf");
    let bytes = fs::read(&font_path).map_err(|error| error.to_string())?;
    let font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())?;
    let material_icon = editor_icons::symbols::md::MD_FORMAT_BOLD
        .chars()
        .next()
        .ok_or_else(|| "material icon glyph missing".to_owned())?;
    let (metrics, bitmap) = font.rasterize(material_icon, 48.0);
    let occupied_rows = bitmap
        .chunks(metrics.width)
        .map(|row| row.iter().filter(|alpha| **alpha > 32).count())
        .filter(|count| *count > 0)
        .collect::<Vec<_>>();
    let unique_row_widths = occupied_rows.iter().copied().collect::<BTreeSet<_>>();

    assert!(material_icon as u32 > 0xFFFF);
    assert!(metrics.width > 0);
    assert!(metrics.height > 0);
    assert!(!occupied_rows.is_empty());
    assert!(unique_row_widths.len() > 4);
    Ok(())
}

#[test]
fn codicon_glyphs_fit_inside_one_editor_cell() -> Result<(), String> {
    let font_path = resolve_bundled_icon_font_dir()
        .map_err(|error| error.to_string())?
        .join("NFM.ttf");
    let bytes = fs::read(&font_path).map_err(|error| error.to_string())?;
    let font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())?;
    let codicon = editor_icons::symbols::cod::COD_DIFF_ADDED
        .chars()
        .next()
        .ok_or_else(|| "codicon glyph missing".to_owned())?;
    let requested_pixel_size = 18.0;
    let (raw_metrics, _) = font.rasterize(codicon, requested_pixel_size);
    let cell_width = raw_metrics.width.saturating_sub(1).max(1) as i32;
    let (fitted_metrics, _) =
        rasterize_icon_glyph_for_cell(&font, codicon, requested_pixel_size, cell_width);
    let layout = icon_glyph_cell_layout(&fitted_metrics, cell_width);

    assert!(raw_metrics.width > cell_width as usize);
    assert!(fitted_metrics.width as i32 <= cell_width);
    assert_eq!(layout.advance, cell_width);
    assert!(layout.draw_offset_x >= 0);
    assert!(layout.draw_offset_x + fitted_metrics.width as i32 <= cell_width);
    Ok(())
}

#[test]
fn font_role_prefers_icon_font_for_private_use_glyphs_without_symbol_hint() -> Result<(), String> {
    let branch = editor_icons::symbols::ple::PL_BRANCH
        .chars()
        .next()
        .ok_or_else(|| "powerline branch glyph missing".to_owned())?;

    assert!(is_private_use_character(branch));
    assert_eq!(
        resolve_font_role_for_char(Some(0), true, false, branch),
        FontRole::Icon(0)
    );
    Ok(())
}

#[test]
fn font_role_prefers_icon_font_for_symbol_like_prompt_glyphs() -> Result<(), String> {
    let prompt = '\u{276F}';

    assert!(is_symbol_like_character(prompt));
    assert!(!is_private_use_character(prompt));
    assert_eq!(
        resolve_font_role_for_char(Some(0), true, false, prompt),
        FontRole::Icon(0)
    );
    Ok(())
}

#[test]
fn autocomplete_or_group_uses_first_provider_with_results() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("alpha alphabet\nalp");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(1, 3));

    let (buffer_id, buffer_revision, text, cursor, query) = {
        let ui = state.ui().map_err(|error| error.to_string())?;
        let buffer_id = ui
            .active_buffer_id()
            .ok_or_else(|| "active buffer missing".to_owned())?;
        let buffer = ui
            .buffer(buffer_id)
            .ok_or_else(|| "shell buffer missing".to_owned())?;
        let text = buffer.text.snapshot();
        let query = autocomplete_query(&text, true)
            .ok_or_else(|| "autocomplete query missing".to_owned())?;
        (
            buffer_id,
            buffer.text.revision(),
            text,
            buffer.cursor_point(),
            query,
        )
    };
    let request = AutocompleteWorkerRequest {
        request_id: 1,
        buffer_id,
        buffer_revision,
        text,
        plugin_kind: None,
        path: None,
        root: None,
        cursor,
        query,
        providers: vec![
            AutocompleteProviderSpec {
                id: "primary".to_owned(),
                label: "Primary".to_owned(),
                icon: "P".to_owned(),
                item_icon: "1".to_owned(),
                or_group: Some("source".to_owned()),
                buffer_kind: None,
                items: Vec::new(),
                kind: AutocompleteProviderKind::Buffer,
            },
            AutocompleteProviderSpec {
                id: "fallback".to_owned(),
                label: "Fallback".to_owned(),
                icon: "F".to_owned(),
                item_icon: "2".to_owned(),
                or_group: Some("source".to_owned()),
                buffer_kind: None,
                items: Vec::new(),
                kind: AutocompleteProviderKind::Buffer,
            },
        ],
        result_limit: 8,
        lsp_client: None,
    };

    let entries = autocomplete_entries(&request);
    assert!(!entries.is_empty());
    assert!(entries.iter().all(|entry| entry.provider_id == "primary"));
    Ok(())
}

#[test]
fn completion_token_at_cursor_supports_trailing_token_edge() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("alpha beta");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 5));

    let (range, token) = completion_token_at_cursor(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "completion token missing at cursor edge".to_owned())?;

    assert_eq!(token, "alpha");
    assert_eq!(range.start(), TextPoint::new(0, 0));
    assert_eq!(range.end(), TextPoint::new(0, 5));
    Ok(())
}

#[test]
fn manual_autocomplete_entries_only_apply_to_matching_plugin_buffers() {
    let provider = AutocompleteProviderSpec {
        id: "calculator".to_owned(),
        label: "Calculator".to_owned(),
        icon: "C".to_owned(),
        item_icon: "ƒ".to_owned(),
        or_group: None,
        buffer_kind: Some("calculator".to_owned()),
        items: vec![editor_plugin_api::AutocompleteProviderItem {
            label: "sqrt(x)".to_owned(),
            replacement: "sqrt".to_owned(),
            detail: Some("Square root".to_owned()),
            documentation: Some("Returns the square root of x.".to_owned()),
        }],
        kind: AutocompleteProviderKind::Manual,
    };
    let query = AutocompleteQuery {
        prefix: "sq".to_owned(),
        token: "sq".to_owned(),
        replace_range: TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 2)),
    };

    let matching = manual_autocomplete_entries(&Some("calculator".to_owned()), &query, &provider);
    assert_eq!(matching.len(), 1);
    assert_eq!(matching[0].0.replacement, "sqrt");

    let non_matching =
        manual_autocomplete_entries(&Some("git-status".to_owned()), &query, &provider);
    assert!(non_matching.is_empty());
}

#[test]
fn hover_manual_provider_lines_match_current_plugin_token() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.kind = BufferKind::Plugin("calculator".to_owned());
        buffer.text = TextBuffer::from_text("sqrt");
        buffer.set_cursor(TextPoint::new(0, 2));
    }
    let provider = HoverProviderSpec {
        label: "Calculator".to_owned(),
        icon: "C".to_owned(),
        buffer_kind: Some("calculator".to_owned()),
        topics: vec![editor_plugin_api::HoverProviderTopic {
            token: "sqrt".to_owned(),
            lines: vec!["sqrt(x)".to_owned(), "Square root".to_owned()],
        }],
        kind: HoverProviderKind::Manual,
    };

    let lines = hover_manual_provider_lines(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
        &provider,
    );
    assert_eq!(lines, vec!["sqrt(x)".to_owned(), "Square root".to_owned()]);
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
                capture_name: "function".to_owned(),
                theme_token: "syntax.function".to_owned(),
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
fn index_syntax_lines_preserves_capture_names() {
    let lines = index_syntax_lines(editor_syntax::SyntaxSnapshot {
        language_id: "rust".to_owned(),
        root_kind: "source_file".to_owned(),
        has_errors: false,
        highlight_spans: vec![editor_syntax::HighlightSpan {
            start_byte: 0,
            end_byte: 5,
            start_position: editor_syntax::SyntaxPoint::new(0, 0),
            end_position: editor_syntax::SyntaxPoint::new(0, 5),
            capture_name: "function".to_owned(),
            theme_token: "syntax.function".to_owned(),
        }],
    });

    let spans = lines.get(&0).expect("expected indexed syntax line");
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].capture_name, "function");
    assert_eq!(spans[0].theme_token, "syntax.function");
}

#[test]
fn browser_buffer_submit_tracks_current_url() -> Result<(), String> {
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
    assert_eq!(
        state.current_url.as_deref(),
        Some("https://example.com/docs")
    );
    assert_eq!(buffer.display_name(), "*browser* https://example.com/docs");
    assert!(buffer.text.text().contains("https://example.com/docs"));
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
    assert!(buffer.text.text().contains("https://docs.rs/volt"));
    Ok(())
}

#[test]
fn hover_next_command_cycles_open_overlay_without_focus() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Alpha".to_owned())
    );

    cycle_hover_provider(&mut state.runtime, true)?;

    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Beta".to_owned())
    );
    assert!(!state.hover_focused().map_err(|error| error.to_string())?);
    Ok(())
}

#[test]
fn hover_previous_command_wraps_open_overlay_without_focus() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Alpha".to_owned())
    );

    cycle_hover_provider(&mut state.runtime, false)?;

    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Gamma".to_owned())
    );
    Ok(())
}

#[cfg(target_os = "windows")]
#[test]
fn system_symbol_fallback_font_covers_starship_prompt_glyphs() -> Result<(), String> {
    let fallback = resolve_system_icon_font_paths()
        .into_iter()
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("seguisym.ttf"))
        })
        .ok_or_else(|| "Segoe UI Symbol fallback font was not found".to_owned())?;
    let bytes = fs::read(&fallback).map_err(|error| error.to_string())?;
    let font = RasterFont::from_bytes(bytes, fontdue::FontSettings::default())
        .map_err(|error| error.to_string())?;

    for glyph in ['◎', '⎪', '▴', '●', '◦', '◃', '◈', '⎥', '⎈', '◨', '⊃'] {
        let (metrics, _) = font.rasterize(glyph, 48.0);
        assert!(
            metrics.width > 0 && metrics.height > 0,
            "fallback font did not cover `{glyph}` (U+{:04X})",
            glyph as u32
        );
    }
    Ok(())
}

#[test]
fn hover_tab_shortcut_focuses_open_overlay() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert!(state.hover_visible().map_err(|error| error.to_string())?);
    assert!(!state.hover_focused().map_err(|error| error.to_string())?);

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::empty())
            .map_err(|error| error.to_string())?
    );

    assert!(state.hover_focused().map_err(|error| error.to_string())?);
    Ok(())
}

#[test]
fn hover_ctrl_n_shortcut_prefers_hover_overlay_over_popup_cycle() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_hover_test_overlay(&mut state, false)?;
    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Alpha".to_owned())
    );

    assert!(
        state
            .try_runtime_keybinding(Keycode::N, ctrl_mod())
            .map_err(|error| error.to_string())?
    );

    assert_eq!(
        state
            .hover_provider_label()
            .map_err(|error| error.to_string())?,
        Some("Beta".to_owned())
    );
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
fn vim_g_prefix_executes_workspace_keybinding() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.g-prefix-exact",
            "Test exact g-prefix binding",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push("exact".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .register_key_binding_for_mode(
            "g z",
            "tests.g-prefix-exact",
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        state.ui().map_err(|error| error.to_string())?.vim().pending,
        Some(VimPending::GPrefix {
            operator: None,
            line_target: None,
        })
    );

    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec!["exact".to_owned()]
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);
    Ok(())
}

#[test]
fn vim_g_prefix_preserves_longer_workspace_sequence() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.g-prefix-sequence",
            "Test longer g-prefix binding",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push("sequence".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .register_key_binding_for_mode(
            "g z z",
            "tests.g-prefix-sequence",
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        Vec::<String>::new()
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(
        ui.vim().pending,
        Some(VimPending::GPrefix {
            operator: None,
            line_target: None,
        })
    );
    assert_eq!(
        ui.vim().pending_change_prefix,
        Some(VimRecordedInput::Chord("g z".to_owned()))
    );

    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec!["sequence".to_owned()]
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);
    Ok(())
}

#[test]
fn browser_viewport_rect_stays_above_prompt_footer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout(buffer, rect, 18, 8);
    let viewport = browser_viewport_rect(buffer, rect, 18)
        .ok_or_else(|| "browser viewport missing".to_owned())?;
    let viewport_bottom = viewport.y + viewport.height as i32;

    assert!(viewport.width > 0);
    assert!(viewport.height > 0);
    assert!(viewport.y >= layout.body_y - 2);
    assert!(viewport_bottom <= layout.input_y);
    Ok(())
}

#[test]
fn browser_surface_hit_testing_excludes_prompt_footer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        None,
        800,
        400,
        8,
        18,
        Instant::now(),
    )
    .map_err(|error| error.to_string())?;
    let surface = plan
        .visible_surfaces
        .iter()
        .find(|surface| surface.buffer_id == buffer_id)
        .ok_or_else(|| "browser surface missing".to_owned())?;

    assert_eq!(
        browser_surface_buffer_at_point(&plan, surface.rect.x + 4, surface.rect.y + 4),
        Some(buffer_id)
    );
    assert_eq!(
        browser_surface_buffer_at_point(
            &plan,
            surface.rect.x + 4,
            surface.rect.y + surface.rect.height as i32 + 4
        ),
        None
    );
    Ok(())
}

#[test]
fn browser_sync_plan_hides_surfaces_while_picker_is_visible() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .set_picker(PickerOverlay::from_entries("Buffers", Vec::new()));

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        None,
        800,
        400,
        8,
        18,
        Instant::now(),
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(plan.buffers.len(), 1);
    assert!(plan.visible_surfaces.is_empty());
    Ok(())
}

#[test]
fn browser_sync_plan_hides_surfaces_when_notifications_overlap() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .apply_notification(
            test_notification_update(
                "progress",
                NotificationSeverity::Info,
                "LSP · rust-analyzer",
                &["Indexing workspace", "Scanning project"],
                Some(32),
                true,
            ),
            Instant::now(),
        );

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        None,
        800,
        400,
        8,
        18,
        Instant::now(),
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(plan.buffers.len(), 1);
    assert!(plan.visible_surfaces.is_empty());
    Ok(())
}

#[test]
fn detect_browser_url_uses_cursor_hit_or_single_line_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("See https://example.com/docs.");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 10));
    let cursor_hit = detect_browser_url(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "browser URL missing under cursor".to_owned())?;
    assert_eq!(cursor_hit, "https://example.com/docs");

    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 0));
    let single_url = detect_browser_url(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "browser URL missing from single-url line".to_owned())?;
    assert_eq!(single_url, "https://example.com/docs");
    Ok(())
}

#[test]
fn browser_url_command_opens_popup_browser_with_detected_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("Docs: https://example.com/docs.");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 8));

    open_detected_browser_url(&mut state.runtime)?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened".to_owned())?;
    let buffer = shell_ui(&state.runtime)?
        .buffer(popup.active_buffer)
        .ok_or_else(|| "browser popup buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert!(buffer.text.text().contains("https://example.com/docs"));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn sync_active_browser_buffer_enters_insert_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;

    sync_active_buffer(&mut state.runtime)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_id));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn browser_host_focus_parent_event_returns_to_normal_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .enter_insert_mode();

    state
        .apply_browser_host_events(&[BrowserHostEvent::FocusParentRequested { buffer_id }])
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state.ui().map_err(|error| error.to_string())?.input_mode(),
        InputMode::Normal
    );
    Ok(())
}

#[test]
fn insert_mode_is_buffer_local_across_buffer_switches() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    let buffer_b = install_scratch_test_buffer(&mut state, "*vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);

    focus_test_buffer(&mut state, buffer_a)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    Ok(())
}

#[test]
fn insert_mode_is_buffer_local_across_split_focus_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*split-vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;

    let buffer_b = install_scratch_test_buffer(&mut state, "*split-vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);

    cycle_runtime_pane(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Insert);

    cycle_runtime_pane(&mut state.runtime)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn inactive_split_render_reads_saved_buffer_input_mode() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*render-vim-a*")?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;

    let buffer_b = install_scratch_test_buffer(&mut state, "*render-vim-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode_for_buffer(buffer_b, true), InputMode::Normal);
    assert_eq!(ui.input_mode_for_buffer(buffer_a, false), InputMode::Insert);
    Ok(())
}

#[test]
fn visual_mode_is_buffer_local_across_buffer_switches() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*visual-a*")?;
    let anchor = TextPoint::new(0, 0);
    shell_ui_mut(&mut state.runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);

    let buffer_b = install_scratch_test_buffer(&mut state, "*visual-b*")?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_b));
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(ui.vim().visual_anchor, None);

    focus_test_buffer(&mut state, buffer_a)?;
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(anchor));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Character);
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
fn mouse_wheel_scrolls_the_buffer_under_the_pointer() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-scroll*",
        (0..20).map(|index| format!("line {index}")).collect(),
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;

    let handled = state
        .handle_event(
            Event::MouseWheel {
                timestamp: 0,
                window_id: 0,
                which: 0,
                x: 0.0,
                y: -1.0,
                direction: MouseWheelDirection::Normal,
                mouse_x: 24.0,
                mouse_y: 24.0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.scroll_row, MOUSE_WHEEL_SCROLL_LINES as usize);
    assert_eq!(buffer.cursor_row(), MOUSE_WHEEL_SCROLL_LINES as usize);
    Ok(())
}

#[test]
fn mouse_drag_creates_a_character_visual_selection() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-drag*",
        vec!["alpha beta".to_owned(), "gamma delta".to_owned()],
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;
    let start = TextPoint::new(0, 1);
    let end = TextPoint::new(1, 3);
    let (start_x, start_y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        start,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;
    let (end_x, end_y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        end,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: start_x,
                y: start_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseMotion {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mousestate: MouseState::from_sdl_state(0),
                x: end_x,
                y: end_y,
                xrel: end_x - start_x,
                yrel: end_y - start_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 1,
                x: end_x,
                y: end_y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(start));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Character);
    assert_eq!(buffer.cursor_point(), end);
    assert_eq!(
        visual_selection(buffer, start, VisualSelectionKind::Character),
        Some(VisualSelection::Range(TextRange::new(
            start,
            buffer.point_after(end).unwrap_or(end)
        )))
    );
    assert!(state.mouse_drag.is_none());
    Ok(())
}

#[test]
fn mouse_double_click_selects_the_whole_line() -> Result<(), String> {
    let render_width = 640;
    let render_height = 240;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*mouse-double-click*",
        vec!["alpha beta".to_owned(), "gamma delta".to_owned()],
    )?;
    state
        .sync_active_viewport(render_height, line_height)
        .map_err(|error| error.to_string())?;
    let point = TextPoint::new(1, 2);
    let (x, y) = screen_point_for_buffer_point(
        &mut state,
        buffer_id,
        point,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;

    state
        .handle_event(
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 2,
                x,
                y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_event(
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: MouseButton::Left,
                clicks: 2,
                x,
                y,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_anchor, Some(point));
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(buffer.cursor_point(), point);
    assert_eq!(
        visual_selection(buffer, point, VisualSelectionKind::Line),
        buffer.line_span_range(1, 1).map(VisualSelection::Range)
    );
    assert!(state.mouse_drag.is_none());
    Ok(())
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
fn terminal_vim_edit_shortcuts_enter_insert_mode_instead_of_read_only_errors() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_test_buffer(&mut state)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("substitute-char"),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Insert);
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
fn pane_close_hook_closes_the_focused_split() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let initial_pane_id = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .active_pane_id()
        .ok_or_else(|| "initial pane is missing".to_owned())?;

    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Horizontal)?;
    cycle_runtime_pane(&mut state.runtime)?;
    state
        .runtime
        .emit_hook(HOOK_PANE_CLOSE, HookEvent::new())
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?
            .pane_count(),
        1
    );
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    assert_eq!(
        state
            .runtime
            .model()
            .workspace(workspace_id)
            .map_err(|error| error.to_string())?
            .active_pane_id(),
        Some(initial_pane_id)
    );
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(initial_pane_id)
    );
    Ok(())
}

#[test]
fn switch_split_hook_reverses_pane_order_and_preserves_the_active_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;

    let (active_pane_id, before) = {
        let ui = shell_ui(&state.runtime)?;
        assert_eq!(ui.active_pane_index(), 0);
        let active_pane_id = ui
            .active_pane_id()
            .ok_or_else(|| "active pane is missing".to_owned())?;
        let before = ui
            .panes()
            .ok_or_else(|| "pane list is missing".to_owned())?
            .iter()
            .map(|pane| pane.buffer_id)
            .collect::<Vec<_>>();
        (active_pane_id, before)
    };

    state
        .runtime
        .emit_hook(HOOK_PANE_SWITCH_SPLIT, HookEvent::new())
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let after = ui
        .panes()
        .ok_or_else(|| "pane list is missing after switch".to_owned())?
        .iter()
        .map(|pane| pane.buffer_id)
        .collect::<Vec<_>>();

    assert_eq!(after, before.into_iter().rev().collect::<Vec<_>>());
    assert_eq!(ui.active_pane_id(), Some(active_pane_id));
    assert_eq!(ui.active_pane_index(), 1);
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
        buffer,
        buffer
            .terminal_render()
            .ok_or_else(|| "terminal render snapshot missing".to_owned())?,
        rect,
        layout,
        true,
        InputMode::Normal,
        None,
        Color::RGB(15, 16, 20),
        Color::RGB(110, 170, 255),
        Color::RGB(215, 221, 232),
        Color::RGB(40, 44, 52),
        "status".to_owned(),
        Color::RGB(110, 170, 255),
        Color::RGB(140, 144, 152),
        8,
        16,
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
