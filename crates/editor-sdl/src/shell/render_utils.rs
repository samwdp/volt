use sdl3::{
    pixels::Color,
    rect::Rect,
    render::{Canvas, FPoint, RenderTarget},
};

use super::{DrawTarget, ShellError, to_pixel_rect, to_render_color};
use editor_render::DrawCommand;

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

pub(super) fn fill_rounded_rect_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    let radius = radius.min(rect.width() / 2).min(rect.height() / 2) as i32;
    if radius <= 1 {
        canvas.set_draw_color(color);
        return canvas
            .fill_rect(rect)
            .map_err(|error| ShellError::Sdl(error.to_string()));
    }

    canvas.set_draw_color(color);
    let rect_height = rect.height() as i32;
    let rect_width = rect.width() as i32;

    for row in 0..rect_height {
        let inset = if row < radius {
            let dy = radius - row - 1;
            radius - ((radius * radius - dy * dy) as f64).sqrt().floor() as i32
        } else if row >= rect_height - radius {
            let dy = row - (rect_height - radius);
            radius - ((radius * radius - dy * dy) as f64).sqrt().floor() as i32
        } else {
            0
        };

        let width = rect_width - (inset * 2);
        if width <= 0 {
            continue;
        }

        canvas
            .fill_rect(Rect::new(rect.x() + inset, rect.y() + row, width as u32, 1))
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
    }

    Ok(())
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
    if text.is_empty() || max_width == 0 {
        return String::new();
    }

    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    let ellipsis = "...";
    if text.chars().take(max_cells.saturating_add(1)).count() <= max_cells {
        return text.to_owned();
    }
    let ellipsis_cells = ellipsis.chars().count();
    if max_cells <= ellipsis_cells {
        return ellipsis.to_owned();
    }

    let available_cells = max_cells.saturating_sub(ellipsis_cells);
    let mut truncated: String = text.chars().take(available_cells).collect();
    truncated.push_str(ellipsis);
    truncated
}
