const TEXT_TEXTURE_CACHE_MAX_BYTES: usize = 32 * 1024 * 1024;
const TEXT_TEXTURE_CACHE_MAX_ENTRY_BYTES: usize = 256 * 1024;
const TEXT_TEXTURE_CACHE_MAX_ENTRIES: usize = 4096;
const LIGATURE_SHAPE_CACHE_MAX_ENTRIES: usize = 4096;
const PRIMARY_TEXT_RUN_CACHE_MAX_ENTRIES: usize = 4096;

type WindowTextureCreator = TextureCreator<WindowContext>;

pub(super) struct CanvasTextSink<'a, 'texture, 'ttf> {
    pub canvas: &'a mut Canvas<Window>,
    pub texture_creator: &'texture WindowTextureCreator,
    pub cache: &'a mut TextTextureCache<'texture>,
    pub cache_mode: TextTextureCacheMode,
    pub fonts: &'a FontSet<'ttf>,
}

pub(super) fn render_color_cache_key(color: RenderColor) -> u32 {
    u32::from_be_bytes([color.r, color.g, color.b, color.a])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TextTextureCacheMode {
    ReadWrite,
    ReuseOnly,
}

impl TextTextureCacheMode {
    const fn allows_inserts(self) -> bool {
        matches!(self, Self::ReadWrite)
    }
}

struct ManagedTexture<'texture> {
    texture: Texture<'texture>,
    width: u32,
    height: u32,
}

impl<'texture> ManagedTexture<'texture> {
    fn from_surface(
        texture_creator: &'texture WindowTextureCreator,
        surface: &Surface<'_>,
    ) -> Result<Self, ShellError> {
        Self::from_surface_with_scale_mode(texture_creator, surface, None)
    }

    fn from_surface_nearest(
        texture_creator: &'texture WindowTextureCreator,
        surface: &Surface<'_>,
    ) -> Result<Self, ShellError> {
        Self::from_surface_with_scale_mode(texture_creator, surface, Some(ScaleMode::Nearest))
    }

    fn from_surface_with_scale_mode(
        texture_creator: &'texture WindowTextureCreator,
        surface: &Surface<'_>,
        scale_mode: Option<ScaleMode>,
    ) -> Result<Self, ShellError> {
        let mut texture = texture_creator
            .create_texture_from_surface(surface)
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
        if let Some(scale_mode) = scale_mode {
            texture.set_scale_mode(scale_mode);
        }
        let query = texture.query();
        Ok(Self {
            texture,
            width: query.width,
            height: query.height,
        })
    }

    fn byte_len(&self) -> usize {
        (self.width as usize)
            .saturating_mul(self.height as usize)
            .saturating_mul(4)
    }

    fn copy_to_canvas(&self, canvas: &mut Canvas<Window>, rect: Rect) -> Result<(), ShellError> {
        canvas
            .copy(&self.texture, None, rect)
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
        Ok(())
    }

    fn copy_to_canvas_clipped(
        &self,
        canvas: &mut Canvas<Window>,
        src: Rect,
        dst: Rect,
    ) -> Result<(), ShellError> {
        canvas
            .copy(&self.texture, src, dst)
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
        Ok(())
    }

    const fn width(&self) -> u32 {
        self.width
    }

    const fn height(&self) -> u32 {
        self.height
    }
}

pub(super) struct RenderedTextTexture<'texture> {
    texture: Option<ManagedTexture<'texture>>,
    offset_x: i32,
    offset_y: i32,
    draw_width: Option<u32>,
    advance: i32,
}

impl<'texture> RenderedTextTexture<'texture> {
    fn from_texture(
        texture: ManagedTexture<'texture>,
        offset_x: i32,
        offset_y: i32,
        advance: i32,
    ) -> Self {
        Self {
            texture: Some(texture),
            offset_x,
            offset_y,
            draw_width: None,
            advance,
        }
    }

    fn from_texture_with_draw_width(
        texture: ManagedTexture<'texture>,
        offset_x: i32,
        offset_y: i32,
        advance: i32,
        draw_width: u32,
    ) -> Self {
        Self {
            texture: Some(texture),
            offset_x,
            offset_y,
            draw_width: Some(draw_width),
            advance,
        }
    }

    fn empty(advance: i32) -> Self {
        Self {
            texture: None,
            offset_x: 0,
            offset_y: 0,
            draw_width: None,
            advance,
        }
    }

    fn byte_len(&self) -> usize {
        self.texture.as_ref().map_or(0, ManagedTexture::byte_len)
    }

    fn blit(&self, canvas: &mut Canvas<Window>, x: i32, y: i32) -> Result<i32, ShellError> {
        if let Some(texture) = self.texture.as_ref() {
            let draw_width = self
                .draw_width
                .map(|width| width.min(texture.width()))
                .unwrap_or_else(|| texture.width());
            texture.copy_to_canvas_clipped(
                canvas,
                Rect::new(0, 0, draw_width, texture.height()),
                Rect::new(
                    x.saturating_add(self.offset_x),
                    y.saturating_add(self.offset_y),
                    draw_width,
                    texture.height(),
                ),
            )?;
        }
        Ok(self.advance)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(super) enum TextTextureCacheKey {
    Primary {
        text: String,
        color: u32,
        style: TextStyle,
    },
    Emoji {
        text: String,
        color: u32,
    },
    Ligature {
        text: String,
        color: u32,
    },
    Icon {
        font_index: usize,
        character: char,
        color: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CachedLigatureGlyphPlacement {
    pub(super) glyph_id: u16,
    pub(super) draw_x: i32,
    pub(super) draw_y: i32,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) raster_px_64: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CachedGlyphRasterPlacement {
    pub(super) glyph_id: u16,
    pub(super) draw_x: i32,
    pub(super) draw_y: i32,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) raster_px_64: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CachedLigatureLayout {
    pub(super) glyphs: Vec<CachedLigatureGlyphPlacement>,
    pub(super) offset_x: i32,
    pub(super) offset_y: i32,
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) advance: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LigatureShapeCacheValue {
    NotLigature,
    Layout(CachedLigatureLayout),
}

pub(super) struct TextTextureCacheEntry<'texture> {
    rendered: RenderedTextTexture<'texture>,
    last_used: u64,
}

pub(super) struct LigatureShapeCacheEntry {
    value: LigatureShapeCacheValue,
    last_used: u64,
}

pub(super) struct PrimaryTextRunCacheEntry {
    value: Vec<PrimaryTextRun>,
    last_used: u64,
}

pub(super) struct TextTextureCache<'texture> {
    entries: HashMap<TextTextureCacheKey, TextTextureCacheEntry<'texture>>,
    ligature_shapes: HashMap<String, LigatureShapeCacheEntry>,
    primary_text_runs: HashMap<String, PrimaryTextRunCacheEntry>,
    access_tick: u64,
    used_bytes: usize,
}

impl<'texture> TextTextureCache<'texture> {
    pub(super) fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ligature_shapes: HashMap::new(),
            primary_text_runs: HashMap::new(),
            access_tick: 0,
            used_bytes: 0,
        }
    }

    pub(super) fn clear(&mut self) {
        self.entries.clear();
        self.ligature_shapes.clear();
        self.primary_text_runs.clear();
        self.access_tick = 0;
        self.used_bytes = 0;
    }

    fn can_cache(rendered: &RenderedTextTexture<'texture>) -> bool {
        rendered.byte_len() <= TEXT_TEXTURE_CACHE_MAX_ENTRY_BYTES
    }

    fn get(&mut self, key: &TextTextureCacheKey) -> Option<&RenderedTextTexture<'texture>> {
        let last_used = self.next_access_tick();
        let entry = self.entries.get_mut(key)?;
        entry.last_used = last_used;
        Some(&entry.rendered)
    }

    fn insert(
        &mut self,
        key: TextTextureCacheKey,
        rendered: RenderedTextTexture<'texture>,
    ) -> Result<&RenderedTextTexture<'texture>, ShellError> {
        let byte_len = rendered.byte_len();
        let last_used = self.next_access_tick();
        if let Some(previous) = self.entries.insert(
            key.clone(),
            TextTextureCacheEntry {
                rendered,
                last_used,
            },
        ) {
            self.used_bytes = self.used_bytes.saturating_sub(previous.rendered.byte_len());
        }
        self.used_bytes = self.used_bytes.saturating_add(byte_len);
        self.evict_to_budget();
        self.entries
            .get(&key)
            .map(|entry| &entry.rendered)
            .ok_or_else(|| {
                ShellError::Runtime(
                    "text texture cache entry disappeared after insertion".to_owned(),
                )
            })
    }

    pub(super) fn get_ligature_shape(&mut self, text: &str) -> Option<LigatureShapeCacheValue> {
        let last_used = self.next_access_tick();
        let entry = self.ligature_shapes.get_mut(text)?;
        entry.last_used = last_used;
        Some(entry.value.clone())
    }

    pub(super) fn insert_ligature_shape(
        &mut self,
        text: String,
        value: LigatureShapeCacheValue,
    ) -> LigatureShapeCacheValue {
        let last_used = self.next_access_tick();
        self.ligature_shapes.insert(
            text,
            LigatureShapeCacheEntry {
                value: value.clone(),
                last_used,
            },
        );
        self.evict_ligature_shapes();
        value
    }

    pub(super) fn get_primary_text_runs(&mut self, text: &str) -> Option<Vec<PrimaryTextRun>> {
        let last_used = self.next_access_tick();
        let entry = self.primary_text_runs.get_mut(text)?;
        entry.last_used = last_used;
        Some(entry.value.clone())
    }

    pub(super) fn insert_primary_text_runs(
        &mut self,
        text: String,
        value: Vec<PrimaryTextRun>,
    ) -> Vec<PrimaryTextRun> {
        let last_used = self.next_access_tick();
        self.primary_text_runs.insert(
            text,
            PrimaryTextRunCacheEntry {
                value: value.clone(),
                last_used,
            },
        );
        self.evict_primary_text_runs();
        value
    }

    fn next_access_tick(&mut self) -> u64 {
        self.access_tick = self.access_tick.saturating_add(1);
        self.access_tick
    }

    fn evict_to_budget(&mut self) {
        while self.entries.len() > TEXT_TEXTURE_CACHE_MAX_ENTRIES
            || self.used_bytes > TEXT_TEXTURE_CACHE_MAX_BYTES
        {
            let Some(oldest_key) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            if let Some(entry) = self.entries.remove(&oldest_key) {
                self.used_bytes = self.used_bytes.saturating_sub(entry.rendered.byte_len());
            }
        }
    }

    fn evict_ligature_shapes(&mut self) {
        while self.ligature_shapes.len() > LIGATURE_SHAPE_CACHE_MAX_ENTRIES {
            let Some(oldest_key) = self
                .ligature_shapes
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.ligature_shapes.remove(&oldest_key);
        }
    }

    fn evict_primary_text_runs(&mut self) {
        while self.primary_text_runs.len() > PRIMARY_TEXT_RUN_CACHE_MAX_ENTRIES {
            let Some(oldest_key) = self
                .primary_text_runs
                .iter()
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| key.clone())
            else {
                break;
            };
            self.primary_text_runs.remove(&oldest_key);
        }
    }
}

pub(super) fn present_scene_to_canvas<'texture>(
    canvas: &mut Canvas<Window>,
    texture_creator: &'texture WindowTextureCreator,
    text_texture_cache: &mut TextTextureCache<'texture>,
    text_texture_cache_mode: TextTextureCacheMode,
    fonts: &FontSet<'_>,
    scene: &[DrawCommand],
) -> Result<(), ShellError> {
    for command in scene {
        match command {
            DrawCommand::Clear { color } => {
                canvas.set_draw_color(from_render_color(*color));
                canvas.clear();
            }
            DrawCommand::FillRect { rect, color } => {
                canvas.set_draw_color(from_render_color(*color));
                canvas
                    .fill_rect(PixelRectToRect::from_pixel_rect(*rect))
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
            }
            DrawCommand::FillRoundedRect {
                rect,
                radius,
                color,
            } => fill_rounded_rect_canvas(
                canvas,
                PixelRectToRect::from_pixel_rect(*rect),
                *radius,
                from_render_color(*color),
            )?,
            DrawCommand::FillTopRoundedRect {
                rect,
                radius,
                color,
            } => fill_top_rounded_rect_canvas(
                canvas,
                PixelRectToRect::from_pixel_rect(*rect),
                *radius,
                from_render_color(*color),
            )?,
            DrawCommand::Undercurl {
                x,
                y,
                width,
                line_height,
                color,
            } => draw_undercurl_canvas(
                canvas,
                *x,
                *y,
                *width,
                *line_height,
                from_render_color(*color),
            )?,
            DrawCommand::Text { x, y, text, color } => render_text_with_fonts(
                &mut CanvasTextSink {
                    canvas,
                    texture_creator,
                    cache: text_texture_cache,
                    cache_mode: text_texture_cache_mode,
                    fonts,
                },
                *x,
                *y,
                text,
                *color,
                TextStyle::plain(),
            )?,
            DrawCommand::StyledText {
                x,
                y,
                text,
                color,
                style,
            } => render_text_with_fonts(
                &mut CanvasTextSink {
                    canvas,
                    texture_creator,
                    cache: text_texture_cache,
                    cache_mode: text_texture_cache_mode,
                    fonts,
                },
                *x,
                *y,
                text,
                *color,
                *style,
            )?,
            DrawCommand::Image {
                rect,
                image_width,
                image_height,
                pixels,
                clip_rect,
            } => {
                let mut pixels = pixels.to_vec();
                let surface = Surface::from_data(
                    pixels.as_mut_slice(),
                    *image_width,
                    *image_height,
                    image_width.saturating_mul(4),
                    PixelFormat::RGBA32,
                )
                .map_err(|error| ShellError::Sdl(error.to_string()))?;
                let texture = ManagedTexture::from_surface(texture_creator, &surface)?;
                canvas.set_clip_rect(clip_rect.as_ref().map(|clip_rect| {
                    Rect::new(clip_rect.x, clip_rect.y, clip_rect.width, clip_rect.height)
                }));
                texture
                    .copy_to_canvas(canvas, Rect::new(rect.x, rect.y, rect.width, rect.height))?;
                canvas.set_clip_rect(None);
            }
        }
    }

    canvas.present();
    Ok(())
}

#[derive(Clone, Copy)]
pub(super) struct IconGlyphRenderStyle<'a> {
    icon_font: &'a IconFont<'a>,
    icon_pixel_size: f32,
    cell_width: i32,
    primary_line_height: i32,
    primary_ascent: i32,
    color: RenderColor,
}

#[derive(Debug, Clone)]
pub(super) struct RasterizedIconGlyph {
    pub(super) metrics: fontdue::Metrics,
    pub(super) bitmap: Vec<u8>,
    pub(super) pixel_size: f32,
}

pub(super) fn rasterize_icon_glyph_for_cell(
    raster_font: &RasterFont,
    character: char,
    icon_pixel_size: f32,
    cell_width: i32,
) -> RasterizedIconGlyph {
    let cell_width = cell_width.max(1) as usize;
    let mut pixel_size = icon_pixel_size.max(1.0);
    let mut rasterized = raster_font.rasterize(character, pixel_size);
    for _ in 0..4 {
        if rasterized.0.width <= cell_width {
            break;
        }
        let next_pixel_size = (pixel_size * cell_width as f32 / rasterized.0.width as f32)
            .floor()
            .max(1.0);
        if next_pixel_size >= pixel_size {
            break;
        }
        pixel_size = next_pixel_size;
        rasterized = raster_font.rasterize(character, pixel_size);
    }
    let (metrics, bitmap) = rasterized;
    RasterizedIconGlyph {
        metrics,
        bitmap,
        pixel_size,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IconGlyphCellLayout {
    pub(super) draw_offset_x: i32,
    pub(super) advance: i32,
}

pub(super) fn icon_glyph_cell_layout(
    metrics: &fontdue::Metrics,
    cell_width: i32,
) -> IconGlyphCellLayout {
    let advance = cell_width.max(1);
    IconGlyphCellLayout {
        draw_offset_x: advance.saturating_sub(metrics.width as i32) / 2,
        advance,
    }
}

pub(super) fn icon_glyph_draw_offset_y(
    metrics: &fontdue::Metrics,
    primary_line_height: i32,
    primary_ascent: i32,
    icon_line_metrics: Option<fontdue::LineMetrics>,
) -> i32 {
    let fallback = primary_ascent - metrics.height as i32 - metrics.ymin;
    let Some(line_metrics) = icon_line_metrics else {
        return fallback;
    };
    if !line_metrics.ascent.is_finite() || !line_metrics.descent.is_finite() {
        return fallback;
    }
    let icon_line_height = line_metrics.ascent - line_metrics.descent;
    if icon_line_height <= f32::EPSILON {
        return fallback;
    }
    (((primary_line_height.max(1) as f32 - icon_line_height) * 0.5) + line_metrics.ascent
        - metrics.height as f32
        - metrics.ymin as f32)
        .round() as i32
}

pub(super) fn alpha_bitmap_surface(
    width: usize,
    height: usize,
    bitmap: &[u8],
    color: RenderColor,
) -> Result<Surface<'static>, ShellError> {
    let mut surface = Surface::new(width as u32, height as u32, PixelFormat::RGBA32)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let pitch = surface.pitch() as usize;
    surface.with_lock_mut(|pixels| {
        for row in 0..height {
            let src = &bitmap[row * width..(row + 1) * width];
            let row_start = row * pitch;
            let dst = &mut pixels[row_start..row_start + width * 4];
            for (alpha, rgba) in src.iter().zip(dst.as_chunks_mut::<4>().0) {
                let alpha = ((*alpha as u16 * color.a as u16) / 255) as u8;
                rgba[0] = color.r;
                rgba[1] = color.g;
                rgba[2] = color.b;
                rgba[3] = alpha;
            }
        }
    });
    Ok(surface)
}

fn convert_surface_to_rgba32<'surface>(
    mut surface: Surface<'surface>,
) -> Result<Surface<'surface>, ShellError> {
    if surface.pixel_format_enum() != PixelFormat::RGBA32 {
        surface = surface
            .convert_format(PixelFormat::RGBA32)
            .map_err(|error| ShellError::Sdl(error.to_string()))?;
    }
    Ok(surface)
}

pub(super) fn render_primary_text_surface(
    fonts: &FontSet<'_>,
    text: &str,
    color: RenderColor,
    style: TextStyle,
) -> Result<Surface<'static>, ShellError> {
    let surface = fonts
        .primary_for_style(style)
        .render(text)
        .blended(from_render_color(color))
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    convert_surface_to_rgba32(surface)
}

pub(super) fn composite_alpha_bitmap(
    surface: &mut Surface<'_>,
    dest_x: i32,
    dest_y: i32,
    width: usize,
    height: usize,
    bitmap: &[u8],
    color: RenderColor,
) {
    let pitch = surface.pitch() as usize;
    let surface_width = surface.width() as i32;
    let surface_height = surface.height() as i32;
    surface.with_lock_mut(|pixels| {
        for row in 0..height {
            let y = dest_y.saturating_add(row as i32);
            if !(0..surface_height).contains(&y) {
                continue;
            }
            let src_row_start = row * width;
            let dst_row_start = y as usize * pitch;
            for col in 0..width {
                let x = dest_x.saturating_add(col as i32);
                if !(0..surface_width).contains(&x) {
                    continue;
                }
                let src_alpha = bitmap[src_row_start + col];
                if src_alpha == 0 {
                    continue;
                }
                let src_alpha = ((src_alpha as u16 * color.a as u16) / 255) as u8;
                if src_alpha == 0 {
                    continue;
                }
                let pixel_start = dst_row_start + x as usize * 4;
                let dst_alpha = pixels[pixel_start + 3];
                let out_alpha = src_alpha as u16
                    + ((dst_alpha as u16 * (255u16.saturating_sub(src_alpha as u16))) / 255);
                pixels[pixel_start] = color.r;
                pixels[pixel_start + 1] = color.g;
                pixels[pixel_start + 2] = color.b;
                pixels[pixel_start + 3] = out_alpha.min(255) as u8;
            }
        }
    });
}

pub(super) fn encode_raster_px_64(pixel_size: f32) -> u16 {
    (pixel_size.max(1.0) * 64.0)
        .round()
        .clamp(1.0, u16::MAX as f32) as u16
}

pub(super) fn decode_raster_px_64(encoded: u16) -> f32 {
    (encoded.max(1) as f32) / 64.0
}

pub(super) fn adjusted_contextual_ligature_pixel_size(
    _raster_font: &RasterFont,
    base_pixel_size: f32,
    _nominal_character: char,
    _ligature_glyph_id: u16,
) -> f32 {
    // Same-length contextual substitutions stay visually closest to the primary
    // SDL_ttf path when they are rasterized at the unscaled base size.
    base_pixel_size
}

pub(super) fn render_primary_text_texture<'texture>(
    texture_creator: &'texture WindowTextureCreator,
    fonts: &FontSet<'_>,
    text: &str,
    color: RenderColor,
    style: TextStyle,
) -> Result<RenderedTextTexture<'texture>, ShellError> {
    // SDL_ttf's blended glyph surfaces already use straight alpha, so upload
    // them as-is to avoid brightening partially transparent edge pixels.
    let surface = render_primary_text_surface(fonts, text, color, style)?;
    let texture = ManagedTexture::from_surface(texture_creator, &surface)?;
    if style == TextStyle::plain() {
        let advance = surface.width() as i32;
        return Ok(RenderedTextTexture::from_texture(texture, 0, 0, advance));
    }

    let advance = monospace_text_width(text, fonts.cell_width()) as i32;
    Ok(RenderedTextTexture::from_texture_with_draw_width(
        texture,
        0,
        0,
        advance,
        advance.max(0) as u32,
    ))
}

pub(super) fn render_emoji_text_texture<'texture>(
    texture_creator: &'texture WindowTextureCreator,
    fonts: &FontSet<'_>,
    text: &str,
    primary_ascent: i32,
    color: RenderColor,
) -> Result<RenderedTextTexture<'texture>, ShellError> {
    let advance = monospace_text_width(text, fonts.cell_width()) as i32;
    let Some(layout) = cached_emoji_layout(fonts, text, primary_ascent) else {
        return Ok(RenderedTextTexture::empty(advance));
    };
    let Some(surface) = compose_emoji_surface(fonts, &layout, color)? else {
        return Ok(RenderedTextTexture::empty(layout.advance));
    };
    let texture = ManagedTexture::from_surface(texture_creator, &surface)?;
    Ok(RenderedTextTexture::from_texture(
        texture,
        layout.offset_x,
        layout.offset_y,
        layout.advance,
    ))
}

pub(super) fn draw_text_texture_with_cache<'texture, F>(
    canvas: &mut Canvas<Window>,
    text_texture_cache: &mut TextTextureCache<'texture>,
    text_texture_cache_mode: TextTextureCacheMode,
    key: TextTextureCacheKey,
    create: F,
    x: i32,
    y: i32,
) -> Result<i32, ShellError>
where
    F: FnOnce() -> Result<RenderedTextTexture<'texture>, ShellError>,
{
    if let Some(rendered) = text_texture_cache.get(&key) {
        return rendered.blit(canvas, x, y);
    }

    let rendered = create()?;
    if !text_texture_cache_mode.allows_inserts() || !TextTextureCache::can_cache(&rendered) {
        return rendered.blit(canvas, x, y);
    }

    let rendered = text_texture_cache.insert(key, rendered)?;
    rendered.blit(canvas, x, y)
}

pub(super) fn render_icon_glyph_texture<'texture>(
    texture_creator: &'texture WindowTextureCreator,
    style: IconGlyphRenderStyle<'_>,
    character: char,
) -> Result<RenderedTextTexture<'texture>, ShellError> {
    let rasterized = rasterize_icon_glyph_for_cell(
        &style.icon_font.raster_font,
        character,
        style.icon_pixel_size,
        style.cell_width,
    );
    let layout = icon_glyph_cell_layout(&rasterized.metrics, style.cell_width);
    if rasterized.metrics.width == 0 || rasterized.metrics.height == 0 {
        return Ok(RenderedTextTexture::empty(layout.advance));
    }

    let surface = alpha_bitmap_surface(
        rasterized.metrics.width,
        rasterized.metrics.height,
        &rasterized.bitmap,
        style.color,
    )?;
    let texture = ManagedTexture::from_surface_nearest(texture_creator, &surface)?;
    let draw_offset_y = icon_glyph_draw_offset_y(
        &rasterized.metrics,
        style.primary_line_height,
        style.primary_ascent,
        style
            .icon_font
            .raster_font
            .horizontal_line_metrics(rasterized.pixel_size),
    );
    Ok(RenderedTextTexture::from_texture(
        texture,
        layout.draw_offset_x,
        draw_offset_y,
        layout.advance,
    ))
}

pub(super) fn scale_shaping_units(value: i32, pixel_size: f32, units_per_em: i32) -> f32 {
    value as f32 * (pixel_size / units_per_em.max(1) as f32)
}

pub(super) fn shape_text_run_with_face(
    face: &ShapeFace<'_>,
    pixel_size: f32,
    features: &[ShapeFeature],
    text: &str,
) -> Option<ShapedRun> {
    if text.is_empty() {
        return None;
    }

    let mut face = face.clone();
    let pixel_size = pixel_size.max(1.0);
    let ppem = pixel_size.round().clamp(1.0, u16::MAX as f32) as u16;
    face.set_pixels_per_em(Some((ppem, ppem)));

    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.guess_segment_properties();
    let glyph_buffer = shape(&face, features, buffer);
    let glyph_infos = glyph_buffer.glyph_infos();

    let units_per_em = face.units_per_em();
    let glyphs = glyph_infos
        .iter()
        .zip(glyph_buffer.glyph_positions())
        .map(|(info, position)| ShapedGlyph {
            cluster: info.cluster as usize,
            glyph_id: info.glyph_id as u16,
            x_advance: scale_shaping_units(position.x_advance, pixel_size, units_per_em),
            x_offset: scale_shaping_units(position.x_offset, pixel_size, units_per_em),
            y_offset: scale_shaping_units(position.y_offset, pixel_size, units_per_em),
        })
        .collect::<Vec<_>>();
    let total_advance = glyphs.iter().map(|glyph| glyph.x_advance).sum::<f32>();
    Some(ShapedRun {
        glyphs,
        total_advance,
    })
}

pub(super) fn shape_ascii_ligature_run_with_face(
    face: &ShapeFace<'_>,
    pixel_size: f32,
    ligatures_enabled: bool,
    text: &str,
) -> Option<ShapedRun> {
    if !ligatures_enabled || !text.is_ascii() || text.chars().count() < 2 {
        return None;
    }
    let features = [ShapeFeature::new(Tag::from_bytes(b"calt"), 1, ..)];
    shape_text_run_with_face(face, pixel_size, &features, text)
}

pub(super) fn shape_ascii_ligature_run(fonts: &FontSet<'_>, text: &str) -> Option<ShapedRun> {
    let shaped = shape_ascii_ligature_run_with_face(
        fonts.primary_shape_face(),
        fonts.primary_pixel_size(),
        fonts.ligatures_enabled(),
        text,
    )?;
    let has_substitution = shaped.glyphs.len() != text.chars().count()
        || text
            .chars()
            .zip(shaped.glyphs.iter())
            .any(|(character, glyph)| {
                fonts.primary_raster_font().lookup_glyph_index(character) != glyph.glyph_id
            });
    let has_positioning = shaped
        .glyphs
        .iter()
        .any(|glyph| glyph.x_offset.abs() > 0.01 || glyph.y_offset.abs() > 0.01);
    (has_substitution || has_positioning).then_some(shaped)
}

pub(super) fn shape_emoji_run(fonts: &FontSet<'_>, text: &str) -> Option<ShapedRun> {
    let face = fonts.emoji_shape_face()?;
    let pixel_size = fonts.emoji_pixel_size()?;
    shape_text_run_with_face(face, pixel_size, &[], text)
}

fn glyphs_need_ligature_render_path(
    text: &str,
    glyphs: &[ShapedGlyph],
    source_start: usize,
    source_end: usize,
    raster_font: &RasterFont,
) -> bool {
    if glyphs.is_empty() || source_start >= source_end {
        return false;
    }
    let source_text = &text[source_start..source_end];
    let source_char_count = source_text.chars().count();
    if source_char_count != glyphs.len() {
        return true;
    }
    glyphs
        .iter()
        .any(|glyph| glyph.x_offset.abs() > 0.01 || glyph.y_offset.abs() > 0.01)
        || source_text
            .chars()
            .zip(glyphs.iter())
            .any(|(character, glyph)| raster_font.lookup_glyph_index(character) != glyph.glyph_id)
}

fn push_ligature_byte_range(ranges: &mut Vec<std::ops::Range<usize>>, start: usize, end: usize) {
    if start >= end {
        return;
    }
    if let Some(previous) = ranges.last_mut()
        && previous.end == start
    {
        previous.end = end;
        return;
    }
    ranges.push(start..end);
}

pub(super) fn ascii_ligature_byte_ranges_with_face(
    face: &ShapeFace<'_>,
    raster_font: &RasterFont,
    pixel_size: f32,
    ligatures_enabled: bool,
    text: &str,
    cell_width: i32,
) -> Vec<std::ops::Range<usize>> {
    if !ligatures_enabled || !text.is_ascii() || text.chars().count() < 2 {
        return Vec::new();
    }
    let Some(shaped) =
        shape_ascii_ligature_run_with_face(face, pixel_size, ligatures_enabled, text)
    else {
        return Vec::new();
    };
    if !shaped_run_preserves_monospace_layout(text, &shaped, cell_width) {
        return Vec::new();
    }

    let mut ranges = Vec::new();
    let mut glyph_index = 0;
    while glyph_index < shaped.glyphs.len() {
        let cluster = shaped.glyphs[glyph_index].cluster.min(text.len());
        let mut group_end = glyph_index + 1;
        while group_end < shaped.glyphs.len()
            && shaped.glyphs[group_end].cluster == shaped.glyphs[glyph_index].cluster
        {
            group_end += 1;
        }
        let next_cluster = shaped
            .glyphs
            .get(group_end)
            .map(|glyph| glyph.cluster.min(text.len()))
            .unwrap_or(text.len());
        let source_start = cluster.min(next_cluster);
        let source_end = cluster.max(next_cluster);
        if glyphs_need_ligature_render_path(
            text,
            &shaped.glyphs[glyph_index..group_end],
            source_start,
            source_end,
            raster_font,
        ) {
            push_ligature_byte_range(&mut ranges, source_start, source_end);
        }
        glyph_index = group_end;
    }
    ranges
}

pub(super) fn primary_ligature_byte_ranges(
    fonts: &FontSet<'_>,
    text: &str,
) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut ascii_start = None;
    for (index, character) in text.char_indices() {
        if character.is_ascii() {
            if ascii_start.is_none() {
                ascii_start = Some(index);
            }
            continue;
        }
        if let Some(start) = ascii_start.take() {
            ranges.extend(
                ascii_ligature_byte_ranges_with_face(
                    fonts.primary_shape_face(),
                    fonts.primary_raster_font(),
                    fonts.primary_pixel_size(),
                    fonts.ligatures_enabled(),
                    &text[start..index],
                    fonts.cell_width(),
                )
                .into_iter()
                .map(|range| start + range.start..start + range.end),
            );
        }
    }
    if let Some(start) = ascii_start {
        ranges.extend(
            ascii_ligature_byte_ranges_with_face(
                fonts.primary_shape_face(),
                fonts.primary_raster_font(),
                fonts.primary_pixel_size(),
                fonts.ligatures_enabled(),
                &text[start..],
                fonts.cell_width(),
            )
            .into_iter()
            .map(|range| start + range.start..start + range.end),
        );
    }
    ranges
}

fn push_primary_text_run(
    runs: &mut Vec<PrimaryTextRun>,
    render_mode: PrimaryTextRenderMode,
    text: &str,
) {
    if text.is_empty() {
        return;
    }
    if let Some(previous) = runs.last_mut()
        && previous.render_mode == render_mode
    {
        previous.text.push_str(text);
        return;
    }
    runs.push(PrimaryTextRun {
        render_mode,
        text: text.to_owned(),
    });
}

pub(super) fn split_primary_text_by_ligature_ranges(
    text: &str,
    ligature_ranges: &[std::ops::Range<usize>],
) -> Vec<PrimaryTextRun> {
    if text.is_empty() {
        return Vec::new();
    }
    if ligature_ranges.is_empty() {
        return vec![PrimaryTextRun {
            render_mode: PrimaryTextRenderMode::Normal,
            text: text.to_owned(),
        }];
    }

    let mut runs = Vec::new();
    let mut cursor = 0;
    for range in ligature_ranges {
        let start = clamp_to_char_boundary(text, range.start.min(text.len()));
        let end = clamp_to_char_boundary(text, range.end.min(text.len()));
        if cursor < start {
            push_primary_text_run(
                &mut runs,
                PrimaryTextRenderMode::Normal,
                &text[cursor..start],
            );
        }
        if start < end {
            push_primary_text_run(
                &mut runs,
                PrimaryTextRenderMode::Ligature,
                &text[start..end],
            );
            cursor = end;
        }
    }
    if cursor < text.len() {
        push_primary_text_run(&mut runs, PrimaryTextRenderMode::Normal, &text[cursor..]);
    }
    runs
}

pub(super) fn split_primary_text_for_ligatures(
    fonts: &FontSet<'_>,
    text: &str,
) -> Vec<PrimaryTextRun> {
    split_primary_text_by_ligature_ranges(text, &primary_ligature_byte_ranges(fonts, text))
}

pub(super) fn cached_primary_text_runs<'texture>(
    text_texture_cache: &mut TextTextureCache<'texture>,
    text_texture_cache_mode: TextTextureCacheMode,
    fonts: &FontSet<'_>,
    text: &str,
) -> Vec<PrimaryTextRun> {
    // Scrolling re-renders many of the same visible lines; cache the split so we
    // do not reshape identical runs before the texture caches can help.
    if let Some(runs) = text_texture_cache.get_primary_text_runs(text) {
        return runs;
    }

    let runs = split_primary_text_for_ligatures(fonts, text);
    if text_texture_cache_mode.allows_inserts() {
        text_texture_cache.insert_primary_text_runs(text.to_owned(), runs)
    } else {
        runs
    }
}

pub(super) fn build_cached_text_layout(
    glyphs: Vec<CachedGlyphRasterPlacement>,
    advance: i32,
) -> CachedLigatureLayout {
    if glyphs.is_empty() {
        return CachedLigatureLayout {
            glyphs: Vec::new(),
            offset_x: 0,
            offset_y: 0,
            width: 0,
            height: 0,
            advance,
        };
    }

    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    let glyphs = glyphs
        .into_iter()
        .map(|glyph| {
            min_x = min_x.min(glyph.draw_x);
            min_y = min_y.min(glyph.draw_y);
            max_x = max_x.max(glyph.draw_x.saturating_add(glyph.width as i32));
            max_y = max_y.max(glyph.draw_y.saturating_add(glyph.height as i32));
            CachedLigatureGlyphPlacement {
                glyph_id: glyph.glyph_id,
                draw_x: glyph.draw_x,
                draw_y: glyph.draw_y,
                width: glyph.width,
                height: glyph.height,
                raster_px_64: glyph.raster_px_64,
            }
        })
        .collect();

    CachedLigatureLayout {
        glyphs,
        offset_x: min_x,
        offset_y: min_y,
        width: (max_x - min_x).max(1) as u32,
        height: (max_y - min_y).max(1) as u32,
        advance,
    }
}

pub(super) fn cached_ligature_layout(
    fonts: &FontSet<'_>,
    text: &str,
    primary_ascent: i32,
) -> LigatureShapeCacheValue {
    let Some(shaped) = shape_ascii_ligature_run(fonts, text) else {
        return LigatureShapeCacheValue::NotLigature;
    };
    let uses_cell_grid = shaped_run_uses_cell_grid(text, &shaped);
    if !shaped_run_preserves_monospace_layout(text, &shaped, fonts.cell_width()) {
        return LigatureShapeCacheValue::NotLigature;
    }

    let mut pen_x = 0.0_f32;
    let mut glyphs = Vec::new();
    let text_characters = text.chars().collect::<Vec<_>>();
    for (index, glyph) in shaped.glyphs.iter().enumerate() {
        let raster_pixel_size = if uses_cell_grid {
            text_characters
                .get(index)
                .copied()
                .map(|character| {
                    adjusted_contextual_ligature_pixel_size(
                        fonts.primary_raster_font(),
                        fonts.primary_pixel_size(),
                        character,
                        glyph.glyph_id,
                    )
                })
                .unwrap_or_else(|| fonts.primary_pixel_size())
        } else {
            fonts.primary_pixel_size()
        };
        let metrics = fonts
            .primary_raster_font()
            .metrics_indexed(glyph.glyph_id, raster_pixel_size);
        if metrics.width != 0 && metrics.height != 0 {
            let glyph_origin_x = if uses_cell_grid {
                index as f32 * fonts.cell_width() as f32
            } else {
                pen_x
            };
            let draw_x = (glyph_origin_x + glyph.x_offset).round() as i32 + metrics.xmin;
            let draw_y = primary_ascent
                - metrics.height as i32
                - metrics.ymin
                - glyph.y_offset.round() as i32;
            glyphs.push(CachedGlyphRasterPlacement {
                glyph_id: glyph.glyph_id,
                draw_x,
                draw_y,
                width: metrics.width as u32,
                height: metrics.height as u32,
                raster_px_64: encode_raster_px_64(raster_pixel_size),
            });
        }
        pen_x += glyph.x_advance;
    }

    let advance = if uses_cell_grid {
        monospace_text_width(text, fonts.cell_width()) as i32
    } else {
        shaped.total_advance.round() as i32
    };
    LigatureShapeCacheValue::Layout(build_cached_text_layout(glyphs, advance))
}

pub(super) fn cached_emoji_layout(
    fonts: &FontSet<'_>,
    text: &str,
    primary_ascent: i32,
) -> Option<CachedLigatureLayout> {
    let shaped = shape_emoji_run(fonts, text)?;
    let raster_font = fonts.emoji_raster_font()?;
    let raster_pixel_size = fonts.emoji_pixel_size()?;

    let mut pen_x = 0.0_f32;
    let mut glyphs = Vec::new();
    for glyph in &shaped.glyphs {
        let metrics = raster_font.metrics_indexed(glyph.glyph_id, raster_pixel_size);
        if metrics.width != 0 && metrics.height != 0 {
            let draw_x = (pen_x + glyph.x_offset).round() as i32 + metrics.xmin;
            let draw_y = primary_ascent
                - metrics.height as i32
                - metrics.ymin
                - glyph.y_offset.round() as i32;
            glyphs.push(CachedGlyphRasterPlacement {
                glyph_id: glyph.glyph_id,
                draw_x,
                draw_y,
                width: metrics.width as u32,
                height: metrics.height as u32,
                raster_px_64: encode_raster_px_64(raster_pixel_size),
            });
        }
        pen_x += glyph.x_advance;
    }

    let advance = (shaped.total_advance.round() as i32)
        .max(monospace_text_width(text, fonts.cell_width()) as i32);
    Some(build_cached_text_layout(glyphs, advance))
}

pub(super) fn compose_raster_surface(
    raster_font: &RasterFont,
    layout: &CachedLigatureLayout,
    color: RenderColor,
) -> Result<Option<Surface<'static>>, ShellError> {
    if layout.glyphs.is_empty() || layout.width == 0 || layout.height == 0 {
        return Ok(None);
    }

    let mut composed = Surface::new(layout.width, layout.height, PixelFormat::RGBA32)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    composed
        .fill_rect(None, Color::RGBA(0, 0, 0, 0))
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    for glyph in &layout.glyphs {
        if glyph.width == 0 || glyph.height == 0 {
            continue;
        }
        let raster_pixel_size = decode_raster_px_64(glyph.raster_px_64);
        // CONTEXT: fontdue's LCD/subpixel mask assumes channel-local filtering.
        // Collapsing that back into a single alpha channel changed the apparent
        // color and weight of ligatures in compositor-backed windows.
        let (_, bitmap) = raster_font.rasterize_indexed(glyph.glyph_id, raster_pixel_size);
        composite_alpha_bitmap(
            &mut composed,
            glyph.draw_x - layout.offset_x,
            glyph.draw_y - layout.offset_y,
            glyph.width as usize,
            glyph.height as usize,
            &bitmap,
            color,
        );
    }
    Ok(Some(composed))
}

pub(super) fn compose_ligature_surface(
    fonts: &FontSet<'_>,
    layout: &CachedLigatureLayout,
    color: RenderColor,
) -> Result<Option<Surface<'static>>, ShellError> {
    compose_raster_surface(fonts.primary_raster_font(), layout, color)
}

pub(super) fn compose_emoji_surface(
    fonts: &FontSet<'_>,
    layout: &CachedLigatureLayout,
    color: RenderColor,
) -> Result<Option<Surface<'static>>, ShellError> {
    let raster_font = fonts
        .emoji_raster_font()
        .ok_or_else(|| ShellError::Runtime("emoji raster font is not configured".to_owned()))?;
    compose_raster_surface(raster_font, layout, color)
}

pub(super) fn render_cached_ligature_texture<'texture>(
    texture_creator: &'texture WindowTextureCreator,
    fonts: &FontSet<'_>,
    layout: &CachedLigatureLayout,
    color: RenderColor,
) -> Result<RenderedTextTexture<'texture>, ShellError> {
    let Some(composed) = compose_ligature_surface(fonts, layout, color)? else {
        return Ok(RenderedTextTexture::empty(layout.advance));
    };
    let texture = ManagedTexture::from_surface(texture_creator, &composed)?;
    Ok(RenderedTextTexture::from_texture(
        texture,
        layout.offset_x,
        layout.offset_y,
        layout.advance,
    ))
}

pub(super) fn draw_primary_ligature_texture_if_available<'texture>(
    sink: &mut CanvasTextSink<'_, 'texture, '_>,
    x: i32,
    y: i32,
    text: &str,
    primary_ascent: i32,
    color: RenderColor,
) -> Result<Option<i32>, ShellError> {
    let key = TextTextureCacheKey::Ligature {
        text: text.to_owned(),
        color: render_color_cache_key(color),
    };
    if let Some(rendered) = sink.cache.get(&key) {
        return Ok(Some(rendered.blit(sink.canvas, x, y)?));
    }

    let shape = if let Some(shape) = sink.cache.get_ligature_shape(text) {
        shape
    } else {
        let shape = cached_ligature_layout(sink.fonts, text, primary_ascent);
        if sink.cache_mode.allows_inserts() {
            sink.cache.insert_ligature_shape(text.to_owned(), shape)
        } else {
            shape
        }
    };
    let LigatureShapeCacheValue::Layout(layout) = shape else {
        return Ok(None);
    };
    let rendered =
        render_cached_ligature_texture(sink.texture_creator, sink.fonts, &layout, color)?;
    if !sink.cache_mode.allows_inserts() || !TextTextureCache::can_cache(&rendered) {
        return Ok(Some(rendered.blit(sink.canvas, x, y)?));
    }

    let rendered = sink.cache.insert(key, rendered)?;
    Ok(Some(rendered.blit(sink.canvas, x, y)?))
}

pub(super) fn render_text_with_fonts<'texture>(
    sink: &mut CanvasTextSink<'_, 'texture, '_>,
    x: i32,
    y: i32,
    text: &str,
    color: RenderColor,
    style: TextStyle,
) -> Result<(), ShellError> {
    let runs = if sink.fonts.icon_fonts().is_empty() || text.is_ascii() {
        let text = strip_zero_width_display_characters(text);
        let text = text.as_ref();
        if text.is_empty() {
            return Ok(());
        }
        vec![FontRun {
            role: FontRole::Primary,
            text: text.to_owned(),
        }]
    } else {
        let runs = font_runs(text, sink.fonts);
        if runs.is_empty() {
            return Ok(());
        }
        runs
    };
    if runs.is_empty() {
        return Ok(());
    }
    let mut draw_x = x;
    let primary_line_height = sink.fonts.primary().height().max(1);
    let primary_ascent = sink.fonts.primary().ascent();
    let color_key = render_color_cache_key(color);
    for run in runs {
        if run.text.is_empty() {
            continue;
        }
        match run.role {
            FontRole::Primary => {
                let primary_runs = if style == TextStyle::plain() {
                    cached_primary_text_runs(sink.cache, sink.cache_mode, sink.fonts, &run.text)
                } else {
                    vec![PrimaryTextRun {
                        render_mode: PrimaryTextRenderMode::Normal,
                        text: run.text.clone(),
                    }]
                };
                for subrun in primary_runs {
                    let advance = match subrun.render_mode {
                        PrimaryTextRenderMode::Ligature => {
                            if let Some(advance) = draw_primary_ligature_texture_if_available(
                                sink,
                                draw_x,
                                y,
                                &subrun.text,
                                primary_ascent,
                                color,
                            )? {
                                advance
                            } else {
                                draw_text_texture_with_cache(
                                    sink.canvas,
                                    sink.cache,
                                    sink.cache_mode,
                                    TextTextureCacheKey::Primary {
                                        text: subrun.text.clone(),
                                        color: color_key,
                                        style,
                                    },
                                    || {
                                        render_primary_text_texture(
                                            sink.texture_creator,
                                            sink.fonts,
                                            &subrun.text,
                                            color,
                                            style,
                                        )
                                    },
                                    draw_x,
                                    y,
                                )?
                            }
                        }
                        PrimaryTextRenderMode::Normal => {
                            let advance = draw_text_texture_with_cache(
                                sink.canvas,
                                sink.cache,
                                sink.cache_mode,
                                TextTextureCacheKey::Primary {
                                    text: subrun.text.clone(),
                                    color: color_key,
                                    style,
                                },
                                || {
                                    render_primary_text_texture(
                                        sink.texture_creator,
                                        sink.fonts,
                                        &subrun.text,
                                        color,
                                        style,
                                    )
                                },
                                draw_x,
                                y,
                            )?;
                            if sink.fonts.primary_style_uses_synthetic_bold(style) {
                                let overlay =
                                    RenderColor::rgba(color.r, color.g, color.b, color.a / 2);
                                let overlay_key = render_color_cache_key(overlay);
                                let _ = draw_text_texture_with_cache(
                                    sink.canvas,
                                    sink.cache,
                                    sink.cache_mode,
                                    TextTextureCacheKey::Primary {
                                        text: subrun.text.clone(),
                                        color: overlay_key,
                                        style,
                                    },
                                    || {
                                        render_primary_text_texture(
                                            sink.texture_creator,
                                            sink.fonts,
                                            &subrun.text,
                                            overlay,
                                            style,
                                        )
                                    },
                                    draw_x.saturating_add(1),
                                    y,
                                )?;
                            }
                            advance
                        }
                    };
                    draw_x += advance;
                }
            }
            FontRole::Icon(index) => {
                let icon_font = sink.fonts.icon_font(index).ok_or_else(|| {
                    ShellError::Runtime(format!("icon font missing at index {index}"))
                })?;
                let style = IconGlyphRenderStyle {
                    icon_font,
                    icon_pixel_size: icon_font.pixel_size,
                    cell_width: sink.fonts.cell_width(),
                    primary_line_height,
                    primary_ascent,
                    color,
                };
                for character in run.text.chars() {
                    draw_x += draw_text_texture_with_cache(
                        sink.canvas,
                        sink.cache,
                        sink.cache_mode,
                        TextTextureCacheKey::Icon {
                            font_index: index,
                            character,
                            color: color_key,
                        },
                        || render_icon_glyph_texture(sink.texture_creator, style, character),
                        draw_x,
                        y,
                    )?;
                }
            }
            FontRole::Emoji => {
                draw_x += draw_text_texture_with_cache(
                    sink.canvas,
                    sink.cache,
                    sink.cache_mode,
                    TextTextureCacheKey::Emoji {
                        text: run.text.clone(),
                        color: color_key,
                    },
                    || {
                        // SDL_ttf still returns tofu for Segoe UI Emoji here on Windows,
                        // so emoji runs go through the fontdue/rustybuzz compositor path.
                        render_emoji_text_texture(
                            sink.texture_creator,
                            sink.fonts,
                            &run.text,
                            primary_ascent,
                            color,
                        )
                    },
                    draw_x,
                    y,
                )?;
            }
        }
    }
    Ok(())
}

pub(super) fn draw_text(
    target: &mut DrawTarget<'_>,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
) -> Result<(), ShellError> {
    draw_styled_text(target, x, y, text, color, TextStyle::plain())
}

pub(super) fn draw_styled_text(
    target: &mut DrawTarget<'_>,
    x: i32,
    y: i32,
    text: &str,
    color: Color,
    style: TextStyle,
) -> Result<(), ShellError> {
    if text.is_empty() {
        return Ok(());
    }

    match target {
        DrawTarget::Scene(scene) => {
            if style == TextStyle::plain() {
                scene.push(DrawCommand::Text {
                    x,
                    y,
                    text: text.to_owned(),
                    color: to_render_color(color),
                });
            } else {
                scene.push(DrawCommand::StyledText {
                    x,
                    y,
                    text: text.to_owned(),
                    color: to_render_color(color),
                    style,
                });
            }
        }
    }

    Ok(())
}

pub(super) fn draw_image(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    image_width: u32,
    image_height: u32,
    pixels: Arc<[u8]>,
    clip_rect: Option<Rect>,
) -> Result<(), ShellError> {
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::Image {
            rect: to_pixel_rect(rect),
            image_width,
            image_height,
            pixels,
            clip_rect: clip_rect.map(to_pixel_rect),
        }),
    }
    Ok(())
}
