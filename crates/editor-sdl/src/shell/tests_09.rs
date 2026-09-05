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

fn render_shell_state_scene_with_docked_runtime_popup(
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(Vec<DrawCommand>, Rect), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    install_terminal_popup_test_buffer(&mut state)?;
    let popup = active_runtime_popup(&state.runtime)?
        .ok_or_else(|| "runtime popup was not opened".to_owned())?;
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
    let width = 320;
    let height = 180;
    let cell_width = 8;
    let line_height = 16;
    let popup_height = popup_window_height(height, line_height);
    let popup_rect = PixelRectToRect::rect(
        0,
        height.saturating_sub(popup_height) as i32,
        width,
        popup_height,
    );
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);

    render_shell_state(
        &mut target,
        &fonts,
        ui,
        Some(&popup),
        ShellDockEntries {
            workspace: &[],
            acp: &[],
        },
        ShellChrome {
            user_library: &NullUserLibrary,
            theme_registry,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize { width, height },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width,
                line_height,
                ascent: 12,
            },
            pulse: FramePulse {
                now: Instant::now(),
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    Ok((scene, popup_rect))
}

#[test]
fn render_shell_state_uses_theme_background_for_docked_runtime_popup_surface() -> Result<(), String>
{
    let base_background = Color::RGB(15, 16, 20);
    let (scene, popup_rect) = render_shell_state_scene_with_docked_runtime_popup(None)?;
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

    assert_eq!(popup_surface_fills, vec![to_render_color(base_background)]);
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

fn render_shell_state_scene_with_notification_overlay(
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(Vec<DrawCommand>, Rect), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let now = Instant::now();
    shell_ui_mut(&mut state.runtime)?.apply_notification(
        NotificationUpdate {
            key: "toast".to_owned(),
            severity: NotificationSeverity::Info,
            title: "Overlay".to_owned(),
            body_lines: vec!["Readability check".to_owned()],
            progress: None,
            active: true,
            action: None,
            workspace_id: None,
        },
        now,
    );
    let ui = shell_ui(&state.runtime)?;
    let width = 320;
    let height = 180;
    let cell_width = 8;
    let line_height = 16;
    let rect = notification_overlay_layouts(
        &ui.visible_notifications(now),
        width,
        height,
        cell_width,
        line_height,
    )
    .first()
    .map(|layout| layout.rect)
    .ok_or_else(|| "notification overlay was not created".to_owned())?;

    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
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
            theme_registry,
            workspace_name: "default",
            lsp_server: None,
            lsp_workspace_loaded: false,
            acp_connected: false,
        },
        ShellFrameView {
            size: WindowSize { width, height },
            fps_overlay: None,
            metrics: TextMetrics {
                cell_width,
                line_height,
                ascent: 12,
            },
            pulse: FramePulse {
                now,
                typing_active: false,
            },
        },
    )
    .map_err(|error| error.to_string())?;

    Ok((scene, rect))
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
fn theme_runtime_settings_resolve_window_effects_from_theme_options() {
    let mut registry = ThemeRegistry::new();
    registry
        .register(
            editor_theme::Theme::new("test-theme", "Test Theme")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.65)
                .with_option(crate::window_effects::OPTION_WINDOW_BLUR, 18.0),
        )
        .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    let settings = theme_runtime_settings(Some(&registry), &ShellConfig::default(), 1.0);

    assert_eq!(
        settings.window_effects,
        crate::window_effects::WindowEffects {
            opacity: 0.65,
            blur: 18.0,
            transparency: crate::window_effects::WindowTransparency::Blur,
        }
    );
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
fn preferred_primary_font_hinting_matches_transparent_window_policy() {
    if cfg!(target_os = "windows") {
        assert!(matches!(
            preferred_primary_font_hinting(),
            Some(Hinting::NONE)
        ));
    } else {
        assert!(preferred_primary_font_hinting().is_none());
    }
}

#[test]
fn rebuild_theme_registry_preserves_active_theme_when_still_present() {
    let registry = rebuild_theme_registry(
        vec![
            editor_theme::Theme::new("default", "Default"),
            editor_theme::Theme::new("night", "Night")
                .with_option(crate::window_effects::OPTION_WINDOW_OPACITY, 0.55),
        ],
        Some("night"),
    )
    .unwrap_or_else(|error| panic!("unexpected error: {error}"));

    assert_eq!(
        registry.active_theme().map(|theme| theme.id()),
        Some("night")
    );
    assert_eq!(
        registry.resolve_number(crate::window_effects::OPTION_WINDOW_OPACITY),
        Some(0.55)
    );
}

#[test]
fn theme_source_fingerprint_from_dir_changes_when_global_toml_changes() -> Result<(), String> {
    let temp = TempTestDir::new("theme-source-fingerprint");
    let themes_dir = temp.path().join("user").join("themes");
    fs::create_dir_all(&themes_dir).map_err(|error| error.to_string())?;
    let global = themes_dir.join("global.toml");
    fs::write(&global, "[options]\n\"window.opacity\" = 1.0\n")
        .map_err(|error| error.to_string())?;

    let before = theme_source_fingerprint_from_dir(&themes_dir)
        .ok_or_else(|| "missing initial theme fingerprint".to_owned())?;

    thread::sleep(Duration::from_millis(20));
    fs::write(
        &global,
        "[options]\n\"window.opacity\" = 0.35\n\"window.blur\" = 12.0\n",
    )
    .map_err(|error| error.to_string())?;

    let after = theme_source_fingerprint_from_dir(&themes_dir)
        .ok_or_else(|| "missing updated theme fingerprint".to_owned())?;

    assert_ne!(before, after);
    Ok(())
}

#[test]
fn user_config_source_fingerprint_changes_when_child_yaml_changes() -> Result<(), String> {
    let temp = TempTestDir::new("user-config-source-fingerprint");
    let user_dir = temp.path().join("user");
    let config_dir = user_dir.join("config");
    fs::create_dir_all(&config_dir).map_err(|error| error.to_string())?;
    fs::write(
        user_dir.join("config.yaml"),
        "workspace: config/workspace.yaml\nui: config/ui.yaml\n",
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        config_dir.join("workspace.yaml"),
        "project_search_roots:\n  - path: P:/\n    max_depth: 4\n",
    )
    .map_err(|error| error.to_string())?;
    let ui = config_dir.join("ui.yaml");
    fs::write(&ui, "ligatures_enabled: true\n").map_err(|error| error.to_string())?;

    let before = user_config_source_fingerprint_from_files(vec![
        user_dir.join("config.yaml"),
        config_dir.join("workspace.yaml"),
        config_dir.join("ui.yaml"),
    ])
    .ok_or_else(|| "missing initial user config fingerprint".to_owned())?;

    thread::sleep(Duration::from_millis(20));
    fs::write(&ui, "ligatures_enabled: false\n").map_err(|error| error.to_string())?;

    let after = user_config_source_fingerprint_from_files(vec![
        user_dir.join("config.yaml"),
        config_dir.join("workspace.yaml"),
        config_dir.join("ui.yaml"),
    ])
    .ok_or_else(|| "missing updated user config fingerprint".to_owned())?;

    assert_ne!(before, after);
    Ok(())
}

#[test]
fn hidden_window_startup_smoke_supports_window_effects() -> Result<(), String> {
    let _guard = crate::window_effects::lock_window_effects_for_tests();
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let video = sdl_context.video().map_err(|error| error.to_string())?;
    crate::window_effects::configure_window_opacity_driver(Some(video.current_video_driver()));
    let window_effects = crate::window_effects::WindowEffects {
        opacity: 0.35,
        blur: 0.0,
        transparency: crate::window_effects::WindowTransparency::None,
    };

    let mut window_builder = video.window("Volt Smoke", 320, 180);
    window_builder.hidden().high_pixel_density();
    window_builder.set_flags(
        window_builder.flags() | crate::window_effects::window_creation_flags(window_effects),
    );
    let mut window = window_builder.build().map_err(|error| error.to_string())?;
    assert!(WindowFlags::from(window.window_flags()).contains(WindowFlags::HIGH_PIXEL_DENSITY));
    apply_window_effects(&mut window, window_effects).map_err(|error| error.to_string())?;

    let mut canvas = window.into_canvas();
    canvas.set_draw_color(Color::RGBA(29, 32, 40, 128));
    canvas.clear();
    canvas.present();

    let size = canvas.output_size().map_err(|error| error.to_string())?;
    assert_eq!(size, (320, 180));
    Ok(())
}

#[test]
fn scaled_font_size_uses_window_display_scale() {
    assert_eq!(scaled_font_size(18, 2.0), 36.0);
    assert_eq!(scaled_font_size(18, 1.25), 22.5);
    assert_eq!(scaled_font_size(18, -1.0), 18.0);
}

#[test]
fn normalized_raster_pixel_size_matches_target_line_height() {
    let pixel_size = normalized_raster_pixel_size(
        18.0,
        24,
        Some(fontdue::LineMetrics {
            ascent: 15.0,
            descent: -5.0,
            line_gap: 0.0,
            new_line_size: 20.0,
        }),
    );

    assert!((pixel_size - 21.6).abs() < f32::EPSILON);
}

#[test]
fn load_font_set_normalizes_icon_raster_sizes_to_primary_line_height() -> Result<(), String> {
    let sdl_context = sdl3::init().map_err(|error| error.to_string())?;
    let _video = sdl_context.video().map_err(|error| error.to_string())?;
    let ttf = sdl3::ttf::init().map_err(|error| error.to_string())?;
    let (fonts, _) = load_font_set(
        &ttf,
        &ThemeRuntimeSettings {
            font_request: None,
            emoji_font_request: None,
            font_size: 18,
            emoji_font_size: 18,
            display_scale: 1.0,
            window_effects: crate::window_effects::WindowEffects::default(),
        },
        &NullUserLibrary,
    )
    .map_err(|error| error.to_string())?;
    let primary_line_height = fonts.primary().height().max(1) as f32;

    for icon_font in fonts.icon_fonts() {
        let line_metrics = icon_font
            .raster_font
            .horizontal_line_metrics(icon_font.pixel_size)
            .ok_or_else(|| format!("icon font `{}` is missing line metrics", icon_font.name))?;
        let icon_line_height = line_metrics.ascent - line_metrics.descent;
        assert!(
            (icon_line_height - primary_line_height).abs() <= 1.0,
            "expected icon font `{}` to target line height {primary_line_height}, got {icon_line_height}",
            icon_font.name,
        );
    }
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
fn plugin_sections_layout_reserves_extra_bottom_padding() -> Result<(), String> {
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

    assert_eq!(
        panes.panes[1].rect.height(),
        (plugin_section_panel_chrome_height("Output", 18) + panes.panes[1].visible_rows as i32 * 18)
            as u32
    );
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
fn calculator_ctrl_tab_switches_sections_without_changing_workspace_pane() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(
        &mut state,
        user::calculator::BUFFER_NAME,
        user::calculator::CALCULATOR_KIND,
    )?;
    split_runtime_pane(&mut state.runtime, PaneSplitDirection::Vertical)?;
    let active_pane_id = shell_ui(&state.runtime)?
        .active_pane_id()
        .ok_or_else(|| "active pane is missing".to_owned())?;

    let handled = state
        .try_runtime_keybinding(Keycode::Tab, ctrl_mod())
        .map_err(|error| error.to_string())?;

    assert!(handled);
    assert_eq!(
        shell_ui(&state.runtime)?.active_pane_id(),
        Some(active_pane_id)
    );
    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());
    Ok(())
}

#[test]
fn calculator_switch_pane_command_targets_workspace_buffer_when_popup_has_focus()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(
        &mut state,
        user::calculator::BUFFER_NAME,
        user::calculator::CALCULATOR_KIND,
    )?;
    let _popup_buffer_id = install_terminal_popup_test_buffer(&mut state)?;

    state
        .runtime
        .execute_command("calculator.switch-pane")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.plugin_active_section_index(), Some(1));
    assert!(buffer.is_read_only());
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
fn render_plugin_sections_active_header_keeps_neutral_background() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_plugin_sections_test_buffer(&mut state, &["alpha"], &["beta"])?;
    let buffer = state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?;

    let rect = PixelRectToRect::rect(0, 0, 640, 360);
    let layout = buffer_footer_layout(buffer, rect, 16, 8);
    let pane_layout = plugin_section_buffer_layout(buffer, rect, layout, 8, 16)
        .ok_or_else(|| "plugin section layout missing".to_owned())?;
    let header_height = (16 + 10) as u32;
    let header_rect = PixelRectToRect::rect(
        pane_layout.panes[0].rect.x() + 1,
        pane_layout.panes[0].rect.y() + 1,
        pane_layout.panes[0].rect.width().saturating_sub(2),
        header_height,
    );
    let base_background = Color::RGB(15, 16, 20);
    let panel_background = buffer_section_panel_background(base_background);
    let header_background = buffer_section_header_background(None, panel_background);
    let mut scene = Vec::new();
    let mut target = DrawTarget::Scene(&mut scene);
    render_plugin_section_buffer_body(
        &mut target,
        PluginSectionDraw {
            buffer,
            view_state: buffer.view_state(),
            pane: PaneSlot { rect, active: true },
            layout,
            visual_selection: None,
            yank_flash: None,
            input_mode: InputMode::Normal,
        },
        BufferBodyPalette {
            theme_registry: None,
            base_background,
            foreground: Color::RGB(215, 221, 232),
            muted: Color::RGB(140, 144, 152),
            border_color: Color::RGB(40, 44, 52),
            selection: Color::RGBA(55, 71, 99, 255),
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
        DrawCommand::FillTopRoundedRect { rect, color, .. }
            if rect.x == header_rect.x()
                && rect.y == header_rect.y()
                && rect.width == header_rect.width()
                && rect.height == header_rect.height()
                && *color == to_render_color(header_background)
    )));
    Ok(())
}
