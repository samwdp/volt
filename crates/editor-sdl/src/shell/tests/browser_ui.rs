#![allow(unused_imports)]
use super::*;

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
fn render_browser_selected_section_applies_window_opacity() -> Result<(), String> {
    let _guard = crate::window_effects::force_surface_window_opacity_for_tests();
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "browser shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?;
        input.set_text("volt");
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
            theme_registry: Some(&registry),
            base_background: Color::RGB(15, 16, 20),
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
            yank_flash_color: Color::RGBA(55, 71, 99, 255),
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
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == browser_layout.input.rect.x()
                && rect.y == browser_layout.input.rect.y()
                && rect.width == browser_layout.input.rect.width()
                && rect.height == browser_layout.input.rect.height()
                && color.a == 128
    )));
    Ok(())
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
fn browser_normal_mode_i_binding_focuses_input_without_inserting_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.enter_normal_mode();
        ui.set_active_vim_target(VimTarget::Buffer);
    }

    state
        .handle_text_input("I")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_id));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        ""
    );
    Ok(())
}

#[test]
fn browser_insert_mode_enter_binding_submits_current_url() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .set_text("example.com/docs");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

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

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    Ok(())
}

#[test]
fn browser_insert_mode_ctrl_enter_binding_submits_current_url() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .set_text("example.com/docs");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
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

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
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
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        18,
        8,
        state.user_library.commandline_enabled(),
    );
    let viewport = browser_viewport_rect(
        buffer,
        rect,
        8,
        18,
        state.user_library.commandline_enabled(),
    )
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
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 480,
                height: 180,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
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
fn browser_sync_plan_excludes_pdf_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("pdf-browser-plan");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["page one"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 400,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        plan.buffers
            .iter()
            .all(|buffer| buffer.buffer_id != buffer_id)
    );
    assert!(
        plan.visible_surfaces
            .iter()
            .all(|surface| surface.buffer_id != buffer_id)
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
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
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 400,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
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
fn browser_url_command_opens_split_browser_with_detected_url() -> Result<(), String> {
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

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "browser split buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    Ok(())
}

#[test]
fn browser_open_buffer_command_opens_split_with_file_url() -> Result<(), String> {
    let root = unique_temp_dir("browser-open-buffer");
    let html_path = root.join("page.html");
    std::fs::write(&html_path, "<html><body>preview</body></html>")
        .map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("<html><body>preview</body></html>");
        buffer.text.set_path(html_path.clone());
    }

    open_active_buffer_in_browser_split(&mut state.runtime)?;

    assert!(active_runtime_popup(&state.runtime)?.is_none());
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(buffer_id)
        .ok_or_else(|| "browser split buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    let expected_url = path_to_file_url(&html_path);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some(expected_url.as_str())
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn browser_open_buffer_command_uses_existing_split_pane() -> Result<(), String> {
    let root = unique_temp_dir("browser-open-buffer-split");
    let html_path = root.join("preview.html");
    std::fs::write(&html_path, "<html></html>").map_err(|error| error.to_string())?;

    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let source_buffer_id = active_shell_buffer_id(&state.runtime)?;
    {
        let buffer = state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?;
        buffer.text = TextBuffer::from_text("<html></html>");
        buffer.text.set_path(html_path.clone());
    }
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 2);
    focus_test_buffer(&mut state, source_buffer_id)?;

    open_active_buffer_in_browser_split(&mut state.runtime)?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.pane_count(), 2);
    let browser_buffer_id = active_shell_buffer_id(&state.runtime)?;
    let buffer = ui
        .buffer(browser_buffer_id)
        .ok_or_else(|| "browser buffer missing".to_owned())?;
    assert!(buffer_is_browser(&buffer.kind));
    assert!(
        ui.panes()
            .is_some_and(|panes| panes.iter().any(|pane| pane.buffer_id == source_buffer_id)),
        "source file buffer should remain open in the other pane"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
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
    state
        .handle_text_input("example.com")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_id));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        "example.com"
    );
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
fn browser_host_new_window_event_routes_into_browser_popup() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_browser_test_buffer(&mut state)?;

    state
        .apply_browser_host_events(&[BrowserHostEvent::NewWindowRequested {
            buffer_id,
            url: "https://example.com/oauth/callback?code=test".to_owned(),
        }])
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened from new-window event".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    let popup_buffer = ui
        .buffer(popup.active_buffer)
        .ok_or_else(|| "popup browser buffer missing".to_owned())?;
    assert!(ui.popup_focus);
    assert!(matches!(
        popup_buffer.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
    assert_eq!(
        popup_buffer
            .browser_state
            .as_ref()
            .and_then(|browser| browser.requested_url.as_deref()),
        Some("https://example.com/oauth/callback?code=test")
    );
    Ok(())
}

#[test]
fn browser_popup_command_focuses_the_popup_surface() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let pane_buffer = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("browser.open-popup")
        .map_err(|error| error.to_string())?;

    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "browser popup was not opened".to_owned())?;
    let ui = shell_ui(&state.runtime)?;
    assert!(ui.popup_focus);
    assert_eq!(ui.popup_buffer_id, Some(popup.active_buffer));
    assert_eq!(active_shell_buffer_id(&state.runtime)?, popup.active_buffer);
    assert_ne!(popup.active_buffer, pane_buffer);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(matches!(
        shell_buffer(&state.runtime, popup.active_buffer)?.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
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
