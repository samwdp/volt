pub(super) fn fill_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    color: Color,
) -> Result<(), ShellError> {
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::FillRect {
            rect: to_pixel_rect(rect),
            color: to_render_color(color),
        }),
    }
    Ok(())
}

pub(super) fn fill_selection_highlight(
    target: &mut DrawTarget<'_>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    paint_selection_highlight(
        target.scene(),
        x,
        y,
        width,
        height,
        radius,
        to_render_color(color),
    );
    Ok(())
}

pub(super) fn window_surface_color(color: Color, window_effects: WindowEffects) -> Color {
    let alpha = (f32::from(color.a) * crate::window_effects::window_surface_opacity(window_effects))
        .round()
        .clamp(0.0, 255.0) as u8;
    Color::RGBA(color.r, color.g, color.b, alpha)
}

pub(super) fn overlay_window_surface_color(color: Color, window_effects: WindowEffects) -> Color {
    let alpha = (f32::from(color.a)
        * crate::window_effects::overlay_window_surface_opacity(window_effects))
    .round()
    .clamp(0.0, 255.0) as u8;
    Color::RGBA(color.r, color.g, color.b, alpha)
}

pub(super) fn clear_window_surface(
    target: &mut DrawTarget<'_>,
    color: Color,
    window_effects: WindowEffects,
) {
    // CONTEXT: when window.opacity < 1, clear to a fully transparent frame so
    // later surface fills are the only opacity layer. Clearing with the scaled
    // background first would stack under pane/panel fills and make chrome
    // (especially ACP/plugin sections) look darker than the configured opacity.
    let opacity = crate::window_effects::window_surface_opacity(window_effects);
    if opacity < 1.0 {
        target.clear(Color::RGBA(color.r, color.g, color.b, 0));
    } else {
        target.clear(window_surface_color(color, window_effects));
    }
}

pub(super) fn fill_window_surface_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rect(target, rect, window_surface_color(color, window_effects))
}

pub(super) fn fill_overlay_surface_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rect(
        target,
        rect,
        overlay_window_surface_color(color, window_effects),
    )
}

fn paint_buffer_scrollbar(
    target: &mut DrawTarget<'_>,
    paint: ScrollbarPaint,
) -> Result<(), ShellError> {
    let ScrollbarPaint {
        pane_rect,
        body_y,
        visible_rows,
        line_height,
        scroll_row,
        max_scroll,
        color,
        window_effects,
    } = paint;
    paint_scrollbar_thumb(
        target.scene(),
        ScrollbarThumb {
            pane_rect: to_pixel_rect(pane_rect),
            body_y,
            visible_rows,
            line_height,
            scroll_row,
            max_scroll,
            color: to_render_color(window_surface_color(
                Color::RGBA(color.r, color.g, color.b, 120),
                window_effects,
            )),
        },
    );
    Ok(())
}

pub(super) fn fill_window_surface_rounded_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rounded_rect(
        target,
        rect,
        radius,
        window_surface_color(color, window_effects),
    )
}

pub(super) fn fill_overlay_surface_rounded_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rounded_rect(
        target,
        rect,
        radius,
        overlay_window_surface_color(color, window_effects),
    )
}

pub(super) fn fill_rounded_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::FillRoundedRect {
            rect: to_pixel_rect(rect),
            radius,
            color: to_render_color(color),
        }),
    }
    Ok(())
}
