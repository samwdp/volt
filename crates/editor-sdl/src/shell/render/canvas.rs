pub(super) fn fill_rounded_rect_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    let radius = radius.min(rect.width() / 2).min(rect.height() / 2) as i32;
    if radius <= 0 {
        canvas.set_draw_color(color);
        return canvas
            .fill_rect(rect)
            .map_err(|error| ShellError::Sdl(error.to_string()));
    }

    let previous_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
    let rect_height = rect.height() as i32;
    let rect_width = rect.width() as i32;
    let x0 = rect.x();
    let y0 = rect.y();

    let result = (|| {
        // Body: full-width band between the corner caps.
        if rect_height > radius * 2 {
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(
                    x0,
                    y0 + radius,
                    rect_width as u32,
                    (rect_height - radius * 2) as u32,
                ))
                .map_err(|error| ShellError::Sdl(error.to_string()))?;
        }

        // Straight edge caps between the four circular corners.
        let mid_width = rect_width - radius * 2;
        if mid_width > 0 {
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(x0 + radius, y0, mid_width as u32, radius as u32))
                .map_err(|error| ShellError::Sdl(error.to_string()))?;
            canvas
                .fill_rect(Rect::new(
                    x0 + radius,
                    y0 + rect_height - radius,
                    mid_width as u32,
                    radius as u32,
                ))
                .map_err(|error| ShellError::Sdl(error.to_string()))?;
        }

        fill_rounded_corner_canvas(canvas, x0, y0, radius, color, RoundedCorner::TopLeft)?;
        fill_rounded_corner_canvas(
            canvas,
            x0 + rect_width - radius,
            y0,
            radius,
            color,
            RoundedCorner::TopRight,
        )?;
        fill_rounded_corner_canvas(
            canvas,
            x0,
            y0 + rect_height - radius,
            radius,
            color,
            RoundedCorner::BottomLeft,
        )?;
        fill_rounded_corner_canvas(
            canvas,
            x0 + rect_width - radius,
            y0 + rect_height - radius,
            radius,
            color,
            RoundedCorner::BottomRight,
        )?;
        Ok(())
    })();

    canvas.set_blend_mode(previous_blend_mode);
    result
}

pub(super) fn fill_top_rounded_rect_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    // Only the top edge is rounded; clamp radius by full height so short header
    // bands can still use the panel corner radius.
    let radius = radius.min(rect.width() / 2).min(rect.height()) as i32;
    if radius <= 0 {
        canvas.set_draw_color(color);
        return canvas
            .fill_rect(rect)
            .map_err(|error| ShellError::Sdl(error.to_string()));
    }

    let previous_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
    let rect_height = rect.height() as i32;
    let rect_width = rect.width() as i32;
    let x0 = rect.x();
    let y0 = rect.y();

    let result = (|| {
        if rect_height > radius {
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(
                    x0,
                    y0 + radius,
                    rect_width as u32,
                    (rect_height - radius) as u32,
                ))
                .map_err(|error| ShellError::Sdl(error.to_string()))?;
        }

        let mid_width = rect_width - radius * 2;
        if mid_width > 0 {
            canvas.set_draw_color(color);
            canvas
                .fill_rect(Rect::new(x0 + radius, y0, mid_width as u32, radius as u32))
                .map_err(|error| ShellError::Sdl(error.to_string()))?;
        }

        fill_rounded_corner_canvas(canvas, x0, y0, radius, color, RoundedCorner::TopLeft)?;
        fill_rounded_corner_canvas(
            canvas,
            x0 + rect_width - radius,
            y0,
            radius,
            color,
            RoundedCorner::TopRight,
        )?;
        Ok(())
    })();

    canvas.set_blend_mode(previous_blend_mode);
    result
}

#[derive(Clone, Copy)]
enum RoundedCorner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

fn fill_rounded_corner_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    origin_x: i32,
    origin_y: i32,
    radius: i32,
    color: Color,
    corner: RoundedCorner,
) -> Result<(), ShellError> {
    for ly in 0..radius {
        let mut run_start = 0i32;
        let mut run_alpha: Option<u8> = None;
        let mut run_len = 0i32;

        for lx in 0..radius {
            let (outer_x, outer_y) = match corner {
                RoundedCorner::TopLeft => (lx, ly),
                RoundedCorner::TopRight => (radius - 1 - lx, ly),
                RoundedCorner::BottomLeft => (lx, radius - 1 - ly),
                RoundedCorner::BottomRight => (radius - 1 - lx, radius - 1 - ly),
            };
            let alpha =
                scaled_coverage_alpha(rounded_corner_coverage(outer_x, outer_y, radius), color.a);
            match run_alpha {
                Some(current) if current == alpha => run_len += 1,
                Some(current) => {
                    fill_rounded_corner_run(
                        canvas,
                        origin_x + run_start,
                        origin_y + ly,
                        run_len,
                        Color::RGBA(color.r, color.g, color.b, current),
                    )?;
                    run_start = lx;
                    run_len = 1;
                    run_alpha = Some(alpha);
                }
                None => {
                    run_start = lx;
                    run_len = 1;
                    run_alpha = Some(alpha);
                }
            }
        }

        if let Some(alpha) = run_alpha {
            fill_rounded_corner_run(
                canvas,
                origin_x + run_start,
                origin_y + ly,
                run_len,
                Color::RGBA(color.r, color.g, color.b, alpha),
            )?;
        }
    }
    Ok(())
}

fn fill_rounded_corner_run<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    x: i32,
    y: i32,
    len: i32,
    color: Color,
) -> Result<(), ShellError> {
    if len <= 0 || color.a == 0 {
        return Ok(());
    }
    canvas.set_draw_color(color);
    canvas
        .fill_rect(Rect::new(x, y, len as u32, 1))
        .map_err(|error| ShellError::Sdl(error.to_string()))
}

/// Coverage for a pixel in a circular corner, with `(0,0)` at the outer corner.
/// Uses a signed-distance falloff so several fringe pixels blend instead of one stair step.
fn rounded_corner_coverage(local_x: i32, local_y: i32, radius: i32) -> f32 {
    if radius <= 0 {
        return 0.0;
    }
    let radius_f = radius as f32;
    let dx = local_x as f32 + 0.5 - radius_f;
    let dy = local_y as f32 + 0.5 - radius_f;
    let signed_distance = (dx * dx + dy * dy).sqrt() - radius_f;
    // ~1.5px filter: smoother than a hard 1px edge, still sharp enough for UI chrome.
    const AA_HALF_WIDTH: f32 = 0.75;
    ((AA_HALF_WIDTH - signed_distance) / (2.0 * AA_HALF_WIDTH)).clamp(0.0, 1.0)
}

fn scaled_coverage_alpha(coverage: f32, alpha: u8) -> u8 {
    ((coverage * f32::from(alpha)).round()) as u8
}

pub(super) fn draw_undercurl_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    x: i32,
    y: i32,
    width: u32,
    line_height: u32,
    color: Color,
) -> Result<(), ShellError> {
    if width == 0 || line_height == 0 {
        return Ok(());
    }

    let baseline_y = y + line_height as i32 - 2;
    let upper_y = baseline_y.saturating_sub(1);
    let width = width as i32;
    let mut points = Vec::with_capacity(width.max(1) as usize);
    let mut dx = 0i32;
    while dx < width {
        let segment_width = (width - dx).min(2);
        let segment_y = if (dx / 2) % 2 == 0 {
            baseline_y
        } else {
            upper_y
        };
        for offset in 0..segment_width {
            points.push(FPoint::new((x + dx + offset) as f32, segment_y as f32));
        }
        dx += 2;
    }

    canvas.set_draw_color(color);
    canvas
        .draw_points(points.as_slice())
        .map_err(|error| ShellError::Sdl(error.to_string()))
}

pub(super) fn truncate_text_to_width(text: &str, max_width: u32, cell_width: i32) -> String {
    editor_ui::truncate_text_to_width(text, max_width, cell_width)
}

pub(super) fn truncate_text_to_width_preserving_end(
    text: &str,
    max_width: u32,
    cell_width: i32,
) -> String {
    editor_ui::truncate_text_to_width_preserving_end(text, max_width, cell_width)
}
