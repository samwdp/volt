#![allow(unused_imports)]
use super::*;

#[test]
fn file_reload_notifications_target_only_matching_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-targeted");
    let active_path = root.join("src").join("main.rs");
    let hidden_path = root.join("src").join("lib.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&active_path, "fn main() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&hidden_path, "pub fn hidden() {}\n").map_err(|error| error.to_string())?;

    let (active_buffer_id, hidden_buffer_id) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, active_buffer_id, &active_path)?;
    configure_file_buffer(&mut state, hidden_buffer_id, &hidden_path)?;

    std::fs::write(
        &hidden_path,
        "pub fn hidden() {\n    println!(\"disk\");\n}\n",
    )
    .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &hidden_path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false
    )?);
    wait_for_file_reload_worker(&mut state, &[hidden_buffer_id])?;
    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, active_buffer_id)?.text.line(1),
        None
    );
    assert_eq!(
        shell_buffer(&state.runtime, hidden_buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"disk\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_reload_hidden_buffers_without_focus_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-hidden");
    let active_path = root.join("src").join("main.rs");
    let hidden_path = root.join("src").join("lib.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&active_path, "fn main() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&hidden_path, "pub fn hidden() {}\n").map_err(|error| error.to_string())?;

    let (active_buffer_id, hidden_buffer_id) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, active_buffer_id, &active_path)?;
    configure_file_buffer(&mut state, hidden_buffer_id, &hidden_path)?;

    std::fs::write(
        &hidden_path,
        "pub fn hidden() {\n    println!(\"background\");\n}\n",
    )
    .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &hidden_path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false,
    )?);
    wait_for_file_reload_worker(&mut state, &[hidden_buffer_id])?;
    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, hidden_buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"background\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_wait_for_dirty_buffers_to_become_clean() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-dirty");
    let path = root.join("src").join("main.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;

    let (buffer_id, _) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, buffer_id, &path)?;

    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
    }
    std::fs::write(&path, "fn main() {\n    println!(\"disk\");\n}\n")
        .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false,
    )?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(0)
            .as_deref(),
        Some("// local")
    );

    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        assert!(buffer.text.undo());
        assert!(!buffer.text.is_dirty());
    }

    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"disk\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn apply_pending_lsp_state_toasts_only_when_notification_revision_moves() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    apply_pending_lsp_state(&mut state.runtime)?;
    let before = shell_ui(&state.runtime)?.notification_revision();

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), before);

    manager.record_show_message("rust-analyzer", "Indexing");
    apply_pending_lsp_state(&mut state.runtime)?;
    let after = shell_ui(&state.runtime)?.notification_revision();
    assert!(after > before);
    let now = Instant::now();
    assert!(
        shell_ui(&state.runtime)?
            .visible_notifications(now)
            .iter()
            .any(|notification| notification.title.contains("rust-analyzer")
                && notification
                    .body_lines
                    .iter()
                    .any(|line| line.contains("Indexing")))
    );

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_ui(&state.runtime)?.notification_revision(), after);
    Ok(())
}

#[test]
fn block_cursor_text_overlay_positions_multibyte_cursor_text() {
    let line = "aéz";
    let char_map = LineCharMap::new(line);
    let overlay = block_cursor_text_overlay(CursorOverlayQuery {
        x: 24,
        line,
        char_map: &char_map,
        segment: LineWrapSegment {
            start_col: 0,
            end_col: 3,
        },
        line_index: 0,
        cursor: TextPoint::new(0, 1),
        color: Some(Color::RGB(1, 2, 3)),
        cell_width: 8,
    })
    .expect("cursor on a multibyte character should produce an overlay");

    assert_eq!(overlay.draw_x, 32);
    assert_eq!(overlay.text, "é");
    assert_eq!(overlay.color, Color::RGB(1, 2, 3));
}

#[test]
fn context_overlay_cache_reuses_stale_snapshot_while_typing() {
    let cached = Arc::new(BufferContextOverlaySnapshot {
        key: BufferContextOverlayCacheKey {
            buffer_revision: 41,
            buffer_name: "demo.rs".to_owned(),
            language_id: Some("rust".to_owned()),
            viewport_top_line: 10,
            cursor_line: 20,
            cursor_column: 4,
        },
        headerline_lines: vec!["fn demo".to_owned()],
        ghost_text_by_line: BTreeMap::new(),
    });
    let key = BufferContextOverlayCacheKey {
        buffer_revision: 42,
        buffer_name: "demo.rs".to_owned(),
        language_id: Some("rust".to_owned()),
        viewport_top_line: 11,
        cursor_line: 21,
        cursor_column: 5,
    };

    let snapshot =
        cached_context_overlay_snapshot(Some(&cached), &key, true).expect("stale snapshot");

    assert!(Arc::ptr_eq(&snapshot, &cached));
    assert_eq!(snapshot.key.buffer_revision, 41);
    assert_eq!(snapshot.headerline_lines, vec!["fn demo".to_owned()]);
}

#[test]
fn context_overlay_cache_requires_matching_buffer_identity() {
    let cached = Arc::new(BufferContextOverlaySnapshot {
        key: BufferContextOverlayCacheKey {
            buffer_revision: 1,
            buffer_name: "demo.rs".to_owned(),
            language_id: Some("rust".to_owned()),
            viewport_top_line: 0,
            cursor_line: 0,
            cursor_column: 0,
        },
        headerline_lines: vec!["fn demo".to_owned()],
        ghost_text_by_line: BTreeMap::new(),
    });
    let key = BufferContextOverlayCacheKey {
        buffer_revision: 2,
        buffer_name: "demo.py".to_owned(),
        language_id: Some("python".to_owned()),
        viewport_top_line: 0,
        cursor_line: 0,
        cursor_column: 0,
    };

    assert!(cached_context_overlay_snapshot(Some(&cached), &key, false).is_none());
    assert!(cached_context_overlay_snapshot(Some(&cached), &key, true).is_none());
}

#[test]
fn context_overlay_snapshot_reuses_same_arc_when_key_matches() -> Result<(), String> {
    let user_library = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*headerline-arc-reuse*",
        vec!["alpha".to_owned()],
    )?;
    let first = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer.context_overlay_snapshot(&*user_library, false)
    };
    let second = {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        buffer.context_overlay_snapshot(&*user_library, false)
    };
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(user_library.headerline_call_count(), 1);
    Ok(())
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
            workspace_id: None,
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
fn notification_action_at_point_returns_copilot_sign_in_action() -> Result<(), String> {
    let now = Instant::now();
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "copilot.sign-in".to_owned(),
            severity: NotificationSeverity::Error,
            title: "Copilot authentication required".to_owned(),
            body_lines: vec!["Click notification to sign in.".to_owned()],
            progress: None,
            active: true,
            action: Some(NotificationAction::CopilotSignIn {
                root: Some(PathBuf::from(r"P:\volt")),
            }),
            workspace_id: None,
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
        Some(NotificationAction::CopilotSignIn {
            root: Some(PathBuf::from(r"P:\volt")),
        })
    );
    Ok(())
}

#[test]
fn copilot_auth_notification_shows_device_code_and_stays_active() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let key = copilot_status_notification_key(Some(Path::new(r"P:\volt")));
    apply_copilot_auth_notification(
        &mut state.runtime,
        &key,
        NotificationSeverity::Info,
        "Copilot sign-in started",
        vec![
            "Device code: ABCD-EFGH".to_owned(),
            "Code copied to clipboard.".to_owned(),
            "Enter code in GitHub browser flow.".to_owned(),
        ],
        true,
    )?;

    let now = Instant::now();
    let ui = shell_ui(&state.runtime)?;
    let notification = ui
        .visible_notifications(now)
        .into_iter()
        .find(|notification| notification.key == key)
        .ok_or_else(|| "copilot auth notification missing".to_owned())?;

    assert_eq!(notification.body_lines[0], "Device code: ABCD-EFGH");
    assert!(notification.active);
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
fn render_shell_state_draws_fps_overlay_when_enabled() -> Result<(), String> {
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
    let fps_overlay = FpsOverlaySnapshot {
        latest_frame_time: Duration::from_nanos(8_100_000),
        average_frame_time: Duration::from_nanos(8_300_000),
        worst_frame_time: Duration::from_nanos(10_200_000),
    };

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
                width: 640,
                height: 360,
            },
            fps_overlay: Some(&fps_overlay),
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
        DrawCommand::Text { text, .. } if text.contains("FPS")
    )));
    Ok(())
}

#[test]
fn render_shell_state_uses_opaque_overlay_chrome_for_docked_runtime_popup_surface()
-> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let (scene, popup_rect) = render_shell_state_scene_with_docked_runtime_popup(Some(&registry))?;
    let popup_surface_fills = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.x == popup_rect.x()
                    && rect.y == popup_rect.y()
                    && rect.width == popup_rect.width()
                    && rect.height == popup_rect.height() =>
            {
                Some(*color)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        popup_surface_fills,
        vec![to_render_color(Color::RGBA(15, 16, 20, 255))]
    );
    Ok(())
}

#[test]
fn render_shell_state_uses_opaque_overlay_chrome_for_notification_surface() -> Result<(), String> {
    let base_background = Color::RGB(15, 16, 20);
    let expected_background = adjust_color(base_background, 18);
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let (scene, notification_rect) =
        render_shell_state_scene_with_notification_overlay(Some(&registry))?;
    let body_x = notification_rect.x + 1 + OVERLAY_ACCENT_BAR_WIDTH as i32;
    let notification_surface_fills = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::FillRoundedRect { rect, color, .. }
                if rect.x == body_x
                    && rect.y == notification_rect.y + 1
                    && rect.width
                        == notification_rect
                            .width()
                            .saturating_sub(2)
                            .saturating_sub(OVERLAY_ACCENT_BAR_WIDTH)
                    && rect.height == notification_rect.height().saturating_sub(2) =>
            {
                Some(*color)
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        notification_surface_fills,
        vec![to_render_color(Color::RGBA(
            expected_background.r,
            expected_background.g,
            expected_background.b,
            255,
        ))]
    );
    Ok(())
}

#[test]
fn render_picker_overlay_uses_opaque_overlay_chrome() -> Result<(), String> {
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
    let picker = PickerOverlay::from_entries(
        "Projects",
        vec![PickerEntry {
            item: PickerItem::new(
                ".config",
                ".config",
                "git",
                Some("C:\\Users\\sam\\.config".to_owned()),
            ),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    picker::render_picker_overlay(
        &mut target,
        &fonts,
        PickerOverlayDraw {
            picker: &picker,
            size: WindowSize {
                width: 320,
                height: 180,
            },
            line_height: 16,
            theme_registry: Some(&registry),
            picker_layout: editor_plugin_api::PickerLayout::default(),
            truncate_strategy: editor_plugin_api::PickerTruncateStrategy::Auto,
        },
    )
    .map_err(|error| error.to_string())?;

    let popup_rect = picker_card_rect(320, 180, editor_plugin_api::PickerLayout::default());
    let inner_x = popup_rect.x + 1;
    let inner_y = popup_rect.y + 1;
    let inner_height = popup_rect.height.saturating_sub(2);
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { rect, color, .. }
            if rect.x == inner_x + OVERLAY_ACCENT_BAR_WIDTH as i32
                && rect.y == inner_y
                && rect.width
                    == popup_rect
                        .width
                        .saturating_sub(2)
                        .saturating_sub(OVERLAY_ACCENT_BAR_WIDTH)
                && rect.height == inner_height
                && *color == to_render_color(Color::RGBA(15, 16, 20, 255))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::Text { color, .. } if color.a == 255
    )));
    Ok(())
}

#[test]
fn render_autocomplete_overlay_uses_opaque_overlay_chrome() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*autocomplete-overlay*",
        vec!["alpha".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 5));

    let overlay = AutocompleteOverlay {
        buffer_id,
        buffer_revision: 0,
        query: AutocompleteQuery {
            prefix: String::new(),
            token: "alpha".to_owned(),
            replace_range: TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
        },
        entries: vec![AutocompleteEntry {
            provider_id: "manual".to_owned(),
            provider_label: "Manual".to_owned(),
            provider_icon: "M".to_owned(),
            item_icon: "•".to_owned(),
            label: "alpha".to_owned(),
            replacement: "alpha".to_owned(),
            replace_range: None,
            detail: Some("detail".to_owned()),
            documentation: Some("documentation".to_owned()),
        }],
        selected_index: 0,
        loading: false,
    };
    let base_background = theme_color(Some(&registry), "ui.background", Color::RGB(15, 16, 20));
    let is_dark = is_dark_color(base_background);
    let accent = theme_color(
        Some(&registry),
        "ui.selection",
        adjust_color(base_background, if is_dark { 48 } else { -48 }),
    );
    let panel_background = theme_color(
        Some(&registry),
        "ui.autocomplete.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let selected_background = theme_color(
        Some(&registry),
        "ui.autocomplete.selection",
        blend_color(accent, panel_background, 0.72),
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_autocomplete_overlay(
        &mut target,
        shell_ui(&state.runtime)?,
        &overlay,
        OverlayAnchorContext {
            pane_rect: PixelRectToRect::rect(0, 0, 640, 360),
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 16,
            },
            typing_active: false,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    panel_background.r,
                    panel_background.g,
                    panel_background.b,
                    255,
                ))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    selected_background.r,
                    selected_background.g,
                    selected_background.b,
                    255,
                ))
    )));
    Ok(())
}

#[test]
fn render_hover_overlay_uses_opaque_overlay_chrome() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.5),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_text_test_buffer(&mut state, "*hover-overlay*", vec!["hover".to_owned()])?;
    install_hover_test_overlay(&mut state, false)?;
    let hover = shell_ui(&state.runtime)?
        .hover()
        .cloned()
        .ok_or_else(|| "hover overlay missing".to_owned())?;

    let base_background = theme_color(Some(&registry), "ui.background", Color::RGB(15, 16, 20));
    let base_foreground = theme_color(
        Some(&registry),
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let accent = theme_color(
        Some(&registry),
        "ui.selection",
        adjust_color(base_background, if is_dark { 48 } else { -48 }),
    );
    let background = theme_color(
        Some(&registry),
        "ui.hover.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let header_background = theme_color(
        Some(&registry),
        "ui.hover.header.background",
        adjust_color(background, if is_dark { 6 } else { -6 }),
    );
    let selected_tab = theme_color(
        Some(&registry),
        "ui.hover.selection",
        blend_color(accent, header_background, 0.68),
    );
    let _muted = theme_color(
        Some(&registry),
        "ui.hover.muted",
        blend_color(base_foreground, background, 0.46),
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_hover_overlay(
        &mut target,
        shell_ui(&state.runtime)?,
        &hover,
        OverlayAnchorContext {
            pane_rect: PixelRectToRect::rect(0, 0, 640, 360),
            user_library: &NullUserLibrary,
            theme_registry: Some(&registry),
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 16,
            },
            typing_active: false,
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    background.r,
                    background.g,
                    background.b,
                    255,
                ))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    header_background.r,
                    header_background.g,
                    header_background.b,
                    255,
                ))
    )));
    assert!(scene.iter().any(|command| matches!(
        command,
        DrawCommand::FillRoundedRect { color, .. }
            if *color
                == to_render_color(Color::RGBA(
                    selected_tab.r,
                    selected_tab.g,
                    selected_tab.b,
                    255,
                ))
    )));
    Ok(())
}

#[test]
fn hover_overlay_width_tracks_content_and_clamps_to_pane() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_text_test_buffer(&mut state, "*hover-width*", vec!["hover".to_owned()])?;
    install_hover_test_overlay(&mut state, false)?;
    let mut hover = shell_ui(&state.runtime)?
        .hover()
        .cloned()
        .ok_or_else(|| "hover overlay missing".to_owned())?;
    let cell_width = 8;

    let short_provider = hover
        .current_provider()
        .cloned()
        .ok_or_else(|| "hover provider missing".to_owned())?;
    assert_eq!(
        hover_overlay_width(&hover, &short_provider, 4000, cell_width),
        44 * cell_width as u32
    );

    hover.providers[0].lines = vec!["x".repeat(100)];
    let long_provider = hover.providers[0].clone();
    assert_eq!(
        hover_overlay_width(&hover, &long_provider, 4000, cell_width),
        100 * cell_width as u32 + 28
    );

    assert_eq!(
        hover_overlay_width(&hover, &long_provider, 400, cell_width),
        384
    );
    Ok(())
}

#[test]
fn render_picker_overlay_uses_picker_text_tokens() -> Result<(), String> {
    let mut registry = ThemeRegistry::new();
    let picker_foreground = Color::RGB(220, 224, 230);
    let picker_muted = Color::RGB(176, 182, 191);
    let picker_subtle = Color::RGB(138, 144, 154);
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_token(
                    TOKEN_PICKER_FOREGROUND,
                    editor_theme::Color::rgb(
                        picker_foreground.r,
                        picker_foreground.g,
                        picker_foreground.b,
                    ),
                )
                .with_token(
                    TOKEN_PICKER_MUTED,
                    editor_theme::Color::rgb(picker_muted.r, picker_muted.g, picker_muted.b),
                )
                .with_token(
                    TOKEN_PICKER_SUBTLE,
                    editor_theme::Color::rgb(picker_subtle.r, picker_subtle.g, picker_subtle.b),
                ),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));
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
    let picker = PickerOverlay::from_entries(
        "Projects",
        vec![
            PickerEntry {
                item: PickerItem::new("alpha", "alpha", "one", None::<String>),
                action: PickerAction::NoOp,
                quickfix: None,
            },
            PickerEntry {
                item: PickerItem::new("beta", "beta", "two", None::<String>),
                action: PickerAction::NoOp,
                quickfix: None,
            },
        ],
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    picker::render_picker_overlay(
        &mut target,
        &fonts,
        PickerOverlayDraw {
            picker: &picker,
            size: WindowSize {
                width: 640,
                height: 360,
            },
            line_height: 16,
            theme_registry: Some(&registry),
            picker_layout: editor_plugin_api::PickerLayout::default(),
            truncate_strategy: editor_plugin_api::PickerTruncateStrategy::Auto,
        },
    )
    .map_err(|error| error.to_string())?;

    let expected_unselected_label = blend_color(picker_foreground, Color::RGB(15, 16, 20), 0.12);
    let text_commands = scene
        .iter()
        .filter_map(|command| match command {
            DrawCommand::Text { text, color, .. } => Some((text.clone(), *color)),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert!(
        text_commands.iter().any(|(text, color)| {
            text == "Projects" && *color == to_render_color(picker_foreground)
        }),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands
            .iter()
            .any(|(text, color)| text == "alpha" && *color == to_render_color(picker_foreground)),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands.iter().any(|(text, color)| {
            text == "beta" && *color == to_render_color(expected_unselected_label)
        }),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands
            .iter()
            .any(|(text, color)| text == "filter" && *color == to_render_color(picker_muted)),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands
            .iter()
            .any(|(text, color)| text == "two" && *color == to_render_color(picker_muted)),
        "unexpected picker text colors: {text_commands:?}"
    );
    assert!(
        text_commands.iter().any(|(text, color)| {
            text == "2 / 2 results" && *color == to_render_color(picker_subtle)
        }),
        "unexpected picker text colors: {text_commands:?}"
    );
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
fn browser_sync_plan_avoids_notification_overlays() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    let now = Instant::now();
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .apply_notification(
            test_notification_update(
                "progress",
                NotificationSeverity::Info,
                "LSP · rust-analyzer",
                &[
                    "Indexing workspace",
                    "Scanning project files",
                    "Resolving dependencies",
                    "Refreshing diagnostics",
                    "Updating symbol cache",
                    "Preparing semantic tokens",
                ],
                Some(32),
                true,
            ),
            now,
        );

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 260,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now,
        },
    )
    .map_err(|error| error.to_string())?;
    let notifications = state
        .ui()
        .map_err(|error| error.to_string())?
        .visible_notifications(now);
    let notification_rects = notification_overlay_layouts(&notifications, 800, 260, 8, 18)
        .into_iter()
        .map(|layout| layout.rect)
        .collect::<Vec<_>>();

    assert_eq!(plan.buffers.len(), 1);
    assert!(!notification_rects.is_empty());
    assert!(plan.visible_surfaces.iter().all(|surface| {
        notification_rects
            .iter()
            .all(|overlay| !rects_intersect(browser_viewport_rect_rect(surface.rect), *overlay))
    }));
    Ok(())
}

#[test]
fn input_prompt_overlay_confirm_delivers_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-confirm");
    open_workspace_from_project(&mut state.runtime, "input-prompt-confirm", &root)?;
    let marker = "volt-input-prompt-confirm";
    let popup_buffer = start_workspace_compile(&mut state, &shell_echo_command(marker))?;
    wait_for_streamed_command_output_line(&mut state, popup_buffer, marker)?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close after Enter with text"
    );
    assert!(
        shell_ui(&state.runtime)?
            .buffer(popup_buffer)
            .is_some_and(|buffer| buffer.text.text().contains(marker)),
        "confirmed prompt text should reach streamed compile command"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_escape_cancels() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-escape");
    open_workspace_from_project(&mut state.runtime, "input-prompt-escape", &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;

    state
        .try_runtime_keybinding(Keycode::Escape, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        !shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt should close on Escape"
    );
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "Escape should discard the compile prompt without opening a popup"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_enter_with_empty_text_is_noop() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("input-prompt-empty");
    open_workspace_from_project(&mut state.runtime, "input-prompt-empty", &root)?;
    execute_shell_command(&mut state, "workspace.compile")?;

    state
        .try_runtime_keybinding(Keycode::Return, Mod::empty())
        .map_err(|e| e.to_string())?;

    assert!(
        shell_ui(&state.runtime)?.input_prompt_visible(),
        "prompt must stay open when Enter pressed with empty text"
    );
    assert!(
        active_runtime_popup(&state.runtime)?.is_none(),
        "empty Enter should not open the compile popup"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn input_prompt_overlay_prefill_appears_in_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let overlay = InputPromptOverlay::new("test.prompt", "Build: ", "cargo build");
    shell_ui_mut(&mut state.runtime)?.open_input_prompt(overlay);
    assert_eq!(
        shell_ui(&state.runtime)?.input_prompt().map(|p| p.text()),
        Some("cargo build")
    );
    Ok(())
}

#[test]
fn render_shell_state_draws_input_prompt_overlay_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let overlay = InputPromptOverlay::new(COMPILE_PROMPT_ID, "Build command: ", "cargo build");
    shell_ui_mut(&mut state.runtime)?.open_input_prompt(overlay);

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
                width: 640,
                height: 360,
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

    assert!(
        scene.iter().any(|command| matches!(
            command,
            DrawCommand::Text { text, .. } if text.contains("Build command: cargo build")
        )),
        "InputPromptOverlay must draw into the command-line footer row"
    );
    Ok(())
}

// ─── workspace.compile prompt tests ──────────────────────────────────────────

#[test]
fn workspace_compile_opens_input_prompt_overlay() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("compile-prompt-opens");
    open_workspace_from_project(&mut state.runtime, "compile-prompt-opens", &root)?;

    execute_shell_command(&mut state, "workspace.compile")?;

    let prompt = shell_ui(&state.runtime)?.input_prompt();
    assert!(prompt.is_some(), "InputPromptOverlay should be open");
    assert_eq!(
        prompt.map(|p| p.id.as_str()),
        Some(COMPILE_PROMPT_ID),
        "overlay id must be COMPILE_PROMPT_ID"
    );
    std::fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn workspace_dock_unread_badge_tracks_other_workspace_notifications() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-dock-unread-a");
    let second_root = unique_temp_dir("workspace-dock-unread-b");
    let first = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let second = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    switch_runtime_workspace(&mut state.runtime, first)?;
    let now = Instant::now();
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "other-ws".to_owned(),
            severity: NotificationSeverity::Info,
            title: "Agent finished".to_owned(),
            body_lines: vec!["done".to_owned()],
            progress: None,
            active: true,
            action: None,
            workspace_id: Some(second),
        },
        now,
    );
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == second)
        .ok_or_else(|| "second workspace missing from dock".to_owned())?;
    assert!(second_entry.unread >= 1);
    let first_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == first)
        .ok_or_else(|| "first workspace missing from dock".to_owned())?;
    assert_eq!(first_entry.unread, 0);
    switch_runtime_workspace(&mut state.runtime, second)?;
    let entries = collect_workspace_dock_entries(&state.runtime)?;
    let second_entry = entries
        .iter()
        .find(|entry| entry.workspace_id == second)
        .ok_or_else(|| "second workspace missing after switch".to_owned())?;
    assert_eq!(second_entry.unread, 0);
    Ok(())
}
