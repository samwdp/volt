#[test]
fn browser_page_load_event_accepts_redirect_after_location_sync() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let user_library = shell_user_library(&state.runtime);
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        request_browser_buffer_navigation(
            buffer,
            "https://example.com/start",
            false,
            &*user_library,
        );
    }

    apply_browser_location_updates(
        &mut state.runtime,
        &[BrowserLocationUpdate {
            buffer_id,
            current_url: "https://example.com/redirected#section".to_owned(),
        }],
    )?;

    state
        .apply_browser_host_events(&[BrowserHostEvent::PageLoadStateChanged {
            buffer_id,
            current_url: "https://example.com/redirected#section".to_owned(),
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
        Some("https://example.com/redirected#section")
    );
    assert_eq!(
        browser.requested_url.as_deref(),
        Some("https://example.com/redirected#section")
    );
    assert!(!browser.is_loading);
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
fn hover_tab_shortcut_beats_markdown_table_navigation_and_allows_scroll() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*hover-markdown-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
    let cursor_before = shell_buffer(&state.runtime, buffer_id)?.cursor_point();
    let _buffer_id = install_scrollable_hover_test_overlay(&mut state, false)?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
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

    assert!(state.hover_focused().map_err(|error| error.to_string())?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        cursor_before
    );

    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    assert_eq!(hover_scroll_offset(&state)?, 1);
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
fn markdown_table_detection_requires_markdown_and_a_delimiter_row() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let markdown = install_markdown_test_buffer(
        &mut state,
        "*markdown-table*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    let malformed = install_markdown_test_buffer(
        &mut state,
        "*markdown-malformed*",
        "| Header 1 | Header 2 |\n| nope | nope |\n| Some text | Some more text |",
    )?;
    let scratch = install_scratch_test_buffer(&mut state, "*not-markdown*")?;
    shell_buffer_mut(&mut state.runtime, scratch)?.replace_with_lines(vec![
        "| Header 1 | Header 2 |".to_owned(),
        "| --- | --- |".to_owned(),
    ]);

    let table =
        detect_markdown_table(shell_buffer(&state.runtime, markdown)?).ok_or("table missing")?;
    assert_eq!(table.start_line, 0);
    assert_eq!(table.column_count, 2);
    assert_eq!(table.rows.len(), 3);
    assert!(table.rows[1].is_delimiter);
    assert!(detect_markdown_table(shell_buffer(&state.runtime, malformed)?).is_none());
    assert!(detect_markdown_table(shell_buffer(&state.runtime, scratch)?).is_none());
    Ok(())
}

#[test]
fn markdown_table_typing_auto_aligns_and_bootstraps_delimiter_rows() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-align*",
        "| Header 1 | Header 2 |\n| -- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 3));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    state
        .handle_text_input("-")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(0).as_deref(),
        Some("| Header 1  | Header 2       |")
    );
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("| --------- | -------------- |")
    );
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text | Some more text |")
    );
    Ok(())
}

#[test]
fn markdown_table_enter_inserts_a_new_row() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-enter*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
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
    assert_eq!(
        buffer.text.line(3).as_deref(),
        Some("|           |                |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 2));
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
fn insert_mode_enter_splits_brace_pair_into_indented_line() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*brace-pair-enter*",
        vec!["if true {}".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 9));
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
    assert_eq!(buffer.text.line(0).as_deref(), Some("if true {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    "));
    assert_eq!(buffer.text.line(2).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 4));
    Ok(())
}

#[test]
fn insert_mode_enter_splits_bracket_pair_into_indented_line() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*bracket-pair-enter*",
        vec!["let items = [];".to_owned()],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 13));
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
    assert_eq!(buffer.text.line(0).as_deref(), Some("let items = ["));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    "));
    assert_eq!(buffer.text.line(2).as_deref(), Some("];"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(1, 4));
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
fn vim_open_line_below_before_typescript_closing_object_dedents_to_sibling_indent()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("typescript")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typescript-vim-open-line*",
        vec![
            ";".to_owned(),
            "export const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({"
                .to_owned(),
            "  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({".to_owned(),
            "    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,".to_owned(),
            "    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),".to_owned(),
            "    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,".to_owned(),
            "    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],".to_owned(),
            "    keepUnusedDataFor: 300".to_owned(),
            "  }),".to_owned(),
            "});".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("typescript".to_owned()));
        buffer.set_cursor(TextPoint::new(8, 2));
    }

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("open-line-below"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(9).as_deref(), Some("  "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(9, 2));
    Ok(())
}

#[test]
fn vim_open_line_below_after_typescript_outer_object_opener_uses_sibling_indent()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    if !syntax_registry_mut(&mut state.runtime)?
        .is_installed("typescript")
        .map_err(|error| error.to_string())?
    {
        return Ok(());
    }
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typescript-vim-open-top-level-entry*",
        vec![
            ";".to_owned(),
            "export const Endpoints = (builder: EndpointBuilder<any, any, any>) => ({"
                .to_owned(),
            "  getOutdoorTrackingHistoryByCustomer: builder.query<DashboardTrackingHistory[], TrackingHistoryAttributes, DashboardTrackingHistoryDto[]>({".to_owned(),
            "    query: (args: TrackingHistoryAttributes) => `outdoordashboard/trackingactivity/${args.customerId}?days=${args.days}`,".to_owned(),
            "    transformResponse: (response: DashboardTrackingHistoryDto[]) => toDashboardTrackingHistorySummaries(response),".to_owned(),
            "    transformErrorResponse: (response: { status: string | number }, _meta, _arg) => response.status,".to_owned(),
            "    providesTags: [{ type: HOURLY_TAG, id: 'LIST' }],".to_owned(),
            "    keepUnusedDataFor: 300".to_owned(),
            "  }),".to_owned(),
            "});".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("typescript".to_owned()));
        buffer.set_cursor(TextPoint::new(1, 0));
    }

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(2).as_deref(), Some("  "));
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 2));
    Ok(())
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
fn markdown_table_preserves_insert_mode_spaces() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-space*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 11));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input(" ")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text  | Some more text |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 12));
    let _ = buffer;

    state
        .handle_text_input("m")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text m | Some more text |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 13));
    Ok(())
}

#[test]
fn insert_mode_tab_inserts_spaces_using_language_theme_options() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(&mut state, "*rust-insert-tab*", vec![String::new()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
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

    let theme_registry = state.runtime.services().get::<ThemeRegistry>();
    assert!(!theme_lang_use_tabs(theme_registry, Some("rust")));
    let expected = tab_insert_string(theme_registry, Some("rust"));
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some(expected.as_str()));
    assert_eq!(
        buffer.cursor_point(),
        TextPoint::new(0, expected.chars().count())
    );
    Ok(())
}

#[test]
fn replace_mode_tab_inserts_make_tabs_using_language_theme_options() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id =
        install_text_test_buffer(&mut state, "*make-replace-tab*", vec!["recipe".to_owned()])?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("make".to_owned()));
        buffer.set_cursor(TextPoint::new(0, 0));
    }
    shell_ui_mut(&mut state.runtime)?.enter_replace_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
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

    let theme_registry = state.runtime.services().get::<ThemeRegistry>();
    assert!(theme_lang_use_tabs(theme_registry, Some("make")));
    let expected = tab_insert_string(theme_registry, Some("make"));
    let expected_line = format!("{expected}ecipe");
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some(expected_line.as_str()));
    assert_eq!(
        buffer.cursor_point(),
        TextPoint::new(0, expected.chars().count())
    );
    Ok(())
}

#[test]
fn markdown_table_insert_tab_adds_a_column_across_the_table() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 14));
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
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
    assert_eq!(
        buffer.text.line(0).as_deref(),
        Some("| Header 1  | Header 2       |   |")
    );
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("| --------- | -------------- | --- |")
    );
    assert_eq!(
        buffer.text.line(2).as_deref(),
        Some("| Some text | Some more text |   |")
    );
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 31));
    Ok(())
}

#[test]
fn markdown_table_normal_tab_moves_between_columns() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_markdown_test_buffer(
        &mut state,
        "*markdown-normal-tab*",
        "| Header 1 | Header 2 |\n| --- | --- |\n| Some text | Some more text |",
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(2, 2));
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    assert!(
        state
            .try_runtime_keybinding(Keycode::Tab, Mod::NOMOD)
            .map_err(|error| error.to_string())?
    );

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 14)
    );
    Ok(())
}

#[test]
fn non_table_normal_tab_still_cycles_panes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_a = install_scratch_test_buffer(&mut state, "*pane-a*")?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    cycle_runtime_pane(&mut state.runtime)?;
    let buffer_b = install_scratch_test_buffer(&mut state, "*pane-b*")?;
    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();

    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Tab),
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
    assert_eq!(ui.active_buffer_id(), Some(buffer_a));
    assert_ne!(ui.active_buffer_id(), Some(buffer_b));
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
fn vim_repeat_search_preserves_forward_and_backward_bindings() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*vim-search-repeat*",
        vec![
            "alpha".to_owned(),
            "beta".to_owned(),
            "alpha".to_owned(),
            "beta".to_owned(),
            "alpha".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));

    run_vim_search(&mut state.runtime, VimSearchDirection::Forward, "alpha")?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 0)
    );

    repeat_vim_search(&mut state.runtime, true)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(0, 0)
    );
    assert!(matches!(
        shell_ui(&state.runtime)?.vim().last_search,
        Some(LastSearch {
            direction: VimSearchDirection::Forward,
            ..
        })
    ));

    repeat_vim_search(&mut state.runtime, false)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.cursor_point(),
        TextPoint::new(2, 0)
    );
    assert!(matches!(
        shell_ui(&state.runtime)?.vim().last_search,
        Some(LastSearch {
            direction: VimSearchDirection::Forward,
            ..
        })
    ));
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
