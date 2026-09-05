fn frame_time_to_fps(frame_time: Duration) -> f64 {
    if frame_time.is_zero() {
        0.0
    } else {
        1.0 / frame_time.as_secs_f64()
    }
}

fn format_fps_overlay_text(snapshot: &FpsOverlaySnapshot) -> String {
    format!(
        "FPS {:>5.1}  frame {:>5.2}ms  max {:>5.2}ms",
        frame_time_to_fps(snapshot.average_frame_time),
        snapshot.latest_frame_time.as_secs_f64() * 1_000.0,
        snapshot.worst_frame_time.as_secs_f64() * 1_000.0,
    )
}

fn render_fps_overlay(
    target: &mut DrawTarget<'_>,
    width: u32,
    theme_registry: Option<&ThemeRegistry>,
    fps_overlay: Option<&FpsOverlaySnapshot>,
    cell_width: i32,
    line_height: i32,
) -> Result<(), ShellError> {
    let Some(snapshot) = fps_overlay else {
        return Ok(());
    };
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let base_foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let background = theme_color(
        theme_registry,
        "ui.fps.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let border = theme_color(
        theme_registry,
        "ui.fps.border",
        adjust_color(base_background, if is_dark { 30 } else { -30 }),
    );
    let accent = if snapshot.average_frame_time <= FRAME_PACING_TARGET_120FPS {
        theme_color(
            theme_registry,
            "ui.fps.good",
            Color::RGBA(95, 196, 122, 255),
        )
    } else if snapshot.average_frame_time <= FRAME_PACING_TARGET_120FPS + Duration::from_millis(4) {
        theme_color(
            theme_registry,
            "ui.fps.warn",
            Color::RGBA(236, 191, 74, 255),
        )
    } else {
        theme_color(theme_registry, "ui.fps.bad", Color::RGBA(224, 97, 97, 255))
    };
    let text = format_fps_overlay_text(snapshot);
    let panel_width = monospace_text_width(&text, cell_width) + 24;
    let panel_height = line_height.max(1) as u32 + 16;
    let rect = PixelRectToRect::rect(
        width as i32 - panel_width as i32 - 12,
        12,
        panel_width,
        panel_height,
    );
    let radius = overlay_radius(theme_registry);
    paint_overlay_card(
        target,
        rect,
        OverlayCardStyle {
            radius,
            border,
            background,
            window_effects,
            accent: Some(accent),
            shadow: false,
        },
    )?;
    draw_text(target, rect.x() + 14, rect.y() + 8, &text, base_foreground)?;
    Ok(())
}

pub(super) fn shared_corner_radius(theme_registry: Option<&ThemeRegistry>) -> u32 {
    theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_CORNER_RADIUS))
        .map(|value| value.clamp(0.0, 64.0).round() as u32)
        .unwrap_or(16)
}

pub(super) fn overlay_radius(theme_registry: Option<&ThemeRegistry>) -> u32 {
    shared_corner_radius(theme_registry)
}

pub(super) fn picker_card_rect(
    window_width: u32,
    window_height: u32,
    layout: editor_plugin_api::PickerLayout,
) -> PixelRect {
    let width_fraction = layout.width_fraction.clamp(0.15, 1.0);
    let height_fraction = layout.height_fraction.clamp(0.15, 1.0);
    let width = ((window_width as f32) * width_fraction).round() as u32;
    let height = ((window_height as f32) * height_fraction).round() as u32;
    centered_rect(window_width, window_height, width.max(1), height.max(1))
}
