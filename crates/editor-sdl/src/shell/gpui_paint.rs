struct GpuiTextPaint {
    family: gpui::SharedString,
    font_size: gpui::Pixels,
    line_height: gpui::Pixels,
}

fn hsla_color(color: RenderColor) -> gpui::Hsla {
    gpui::Rgba {
        r: f32::from(color.r) / 255.0,
        g: f32::from(color.g) / 255.0,
        b: f32::from(color.b) / 255.0,
        a: f32::from(color.a) / 255.0,
    }
    .into()
}

fn scene_bounds(origin_x: f32, origin_y: f32, x: i32, y: i32, width: u32, height: u32) -> gpui::Bounds<gpui::Pixels> {
    gpui::Bounds {
        origin: gpui::point(
            gpui::px(origin_x + x as f32),
            gpui::px(origin_y + y as f32),
        ),
        size: gpui::size(gpui::px(width as f32), gpui::px(height as f32)),
    }
}

fn paint_fill(
    window: &mut gpui::Window,
    bounds: gpui::Bounds<gpui::Pixels>,
    radii: gpui::Corners<gpui::Pixels>,
    color: RenderColor,
) {
    window.paint_quad(gpui::quad(
        bounds,
        radii,
        hsla_color(color),
        gpui::px(0.0),
        gpui::transparent_black(),
        gpui::BorderStyle::default(),
    ));
}

struct ScenePaint<'a> {
    origin_x: f32,
    origin_y: f32,
    style: &'a GpuiTextPaint,
}

struct PaintedText<'a> {
    x: f32,
    y: f32,
    text: &'a str,
    color: RenderColor,
    text_style: TextStyle,
}

fn paint_text_run(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    paint: &ScenePaint<'_>,
    run: PaintedText<'_>,
) {
    let text = run.text.replace(['\n', '\r'], " ");
    if text.is_empty() {
        return;
    }
    let mut font = gpui::font(paint.style.family.clone());
    if run.text_style.bold {
        font.weight = gpui::FontWeight::BOLD;
    }
    if run.text_style.italic {
        font.style = gpui::FontStyle::Italic;
    }
    let shaped = window.text_system().shape_line(
        text.clone().into(),
        paint.style.font_size,
        &[gpui::TextRun {
            len: text.len(),
            font,
            color: hsla_color(run.color),
            background_color: None,
            underline: None,
            strikethrough: None,
        }],
        None,
    );
    let _ = shaped.paint(
        gpui::point(gpui::px(run.x), gpui::px(run.y)),
        paint.style.line_height,
        gpui::TextAlign::Left,
        None,
        window,
        cx,
    );
}

fn bgra_frame(width: u32, height: u32, pixels: &[u8]) -> Option<image::Frame> {
    let expected = (width as usize).checked_mul(height as usize)?.checked_mul(4)?;
    if pixels.len() < expected || width == 0 || height == 0 {
        return None;
    }
    let mut bgra = Vec::with_capacity(expected);
    for pixel in pixels[..expected].as_chunks::<4>().0 {
        bgra.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
    }
    let buffer = image::RgbaImage::from_raw(width, height, bgra)?;
    Some(image::Frame::new(buffer))
}

fn paint_scene_image(
    window: &mut gpui::Window,
    paint: &ScenePaint<'_>,
    rect: PixelRect,
    clip_rect: Option<PixelRect>,
    image_width: u32,
    image_height: u32,
    pixels: &[u8],
) {
    let Some(frame) = bgra_frame(image_width, image_height, pixels) else {
        return;
    };
    let image = Arc::new(gpui::RenderImage::new([frame]));
    let image_bounds = scene_bounds(
        paint.origin_x,
        paint.origin_y,
        rect.x,
        rect.y,
        rect.width,
        rect.height,
    );
    let bounds = clip_rect.map_or(image_bounds, |clip| {
        scene_bounds(
            paint.origin_x,
            paint.origin_y,
            clip.x,
            clip.y,
            clip.width,
            clip.height,
        )
    });
    let _ = window.paint_image(
        bounds,
        image_bounds,
        gpui::Corners::default(),
        image,
        0,
        false,
    );
}

fn paint_draw_scene(
    window: &mut gpui::Window,
    cx: &mut gpui::App,
    paint: &ScenePaint<'_>,
    width: u32,
    height: u32,
    scene: &[DrawCommand],
) {
    for command in scene {
        match command {
            DrawCommand::Clear { color } => {
                paint_fill(
                    window,
                    scene_bounds(paint.origin_x, paint.origin_y, 0, 0, width, height),
                    gpui::Corners::default(),
                    *color,
                );
            }
            DrawCommand::FillRect { rect, color } => {
                paint_fill(
                    window,
                    scene_bounds(
                        paint.origin_x,
                        paint.origin_y,
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                    ),
                    gpui::Corners::default(),
                    *color,
                );
            }
            DrawCommand::FillRoundedRect { rect, radius, color } => {
                paint_fill(
                    window,
                    scene_bounds(
                        paint.origin_x,
                        paint.origin_y,
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                    ),
                    gpui::px(*radius as f32).into(),
                    *color,
                );
            }
            DrawCommand::FillTopRoundedRect { rect, radius, color } => {
                let radius = gpui::px(*radius as f32);
                paint_fill(
                    window,
                    scene_bounds(
                        paint.origin_x,
                        paint.origin_y,
                        rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                    ),
                    gpui::Corners {
                        top_left: radius,
                        top_right: radius,
                        bottom_left: gpui::px(0.0),
                        bottom_right: gpui::px(0.0),
                    },
                    *color,
                );
            }
            DrawCommand::Undercurl {
                x,
                y,
                width,
                line_height,
                color,
            } => {
                let baseline = y + *line_height as i32 - 2;
                let mut dx = 0i32;
                let width = *width as i32;
                while dx < width {
                    let segment_y = if (dx / 2) % 2 == 0 {
                        baseline
                    } else {
                        baseline.saturating_sub(1)
                    };
                    let segment_width = (width - dx).clamp(0, 2) as u32;
                    paint_fill(
                        window,
                        scene_bounds(
                            paint.origin_x,
                            paint.origin_y,
                            x + dx,
                            segment_y,
                            segment_width,
                            1,
                        ),
                        gpui::Corners::default(),
                        *color,
                    );
                    dx += 2;
                }
            }
            DrawCommand::Text { x, y, text, color } => paint_text_run(
                window,
                cx,
                paint,
                PaintedText {
                    x: paint.origin_x + *x as f32,
                    y: paint.origin_y + *y as f32,
                    text,
                    color: *color,
                    text_style: TextStyle::plain(),
                },
            ),
            DrawCommand::StyledText {
                x,
                y,
                text,
                color,
                style: text_style,
            } => paint_text_run(
                window,
                cx,
                paint,
                PaintedText {
                    x: paint.origin_x + *x as f32,
                    y: paint.origin_y + *y as f32,
                    text,
                    color: *color,
                    text_style: *text_style,
                },
            ),
            DrawCommand::Image {
                rect,
                clip_rect,
                image_width,
                image_height,
                pixels,
            } => paint_scene_image(
                window,
                paint,
                *rect,
                *clip_rect,
                *image_width,
                *image_height,
                pixels,
            ),
        }
    }
}

fn gpui_font_bytes(font_path: &Path, icon_paths: &[PathBuf]) -> Vec<Cow<'static, [u8]>> {
    let mut fonts = Vec::new();
    if let Ok(bytes) = std::fs::read(font_path) {
        fonts.push(Cow::Owned(bytes));
    }
    for path in icon_paths {
        if let Ok(bytes) = std::fs::read(path) {
            fonts.push(Cow::Owned(bytes));
        }
    }
    fonts
}
