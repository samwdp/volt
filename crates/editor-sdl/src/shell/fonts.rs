struct IconFont<'ttf> {
    name: String,
    font: Font<'ttf>,
    raster_font: RasterFont,
    pixel_size: f32,
}

struct EmojiFont<'ttf> {
    font: Font<'ttf>,
    raster_font: RasterFont,
    pixel_size: f32,
    shape_face: ShapeFace<'static>,
}

struct FontSetInit<'ttf> {
    primary: Font<'ttf>,
    primary_bold: Font<'ttf>,
    primary_italic: Font<'ttf>,
    primary_bold_italic: Font<'ttf>,
    primary_bold_is_synthetic: bool,
    primary_bold_italic_is_synthetic: bool,
    primary_raster_font: RasterFont,
    primary_shape_face: ShapeFace<'static>,
    primary_pixel_size: f32,
    emoji_font: Option<(Font<'ttf>, RasterFont, f32, ShapeFace<'static>)>,
    ligatures_enabled: bool,
    icon_fonts: Vec<(String, Font<'ttf>, RasterFont, f32)>,
    icon_chars: BTreeSet<char>,
    cell_width: i32,
}

struct FontSet<'ttf> {
    primary: Font<'ttf>,
    primary_bold: Font<'ttf>,
    primary_italic: Font<'ttf>,
    primary_bold_italic: Font<'ttf>,
    primary_bold_is_synthetic: bool,
    primary_bold_italic_is_synthetic: bool,
    primary_raster_font: RasterFont,
    primary_shape_face: ShapeFace<'static>,
    primary_pixel_size: f32,
    emoji_font: Option<EmojiFont<'ttf>>,
    ligatures_enabled: bool,
    icon_fonts: Vec<IconFont<'ttf>>,
    icon_chars: BTreeSet<char>,
    cell_width: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OptionalFontLoadMode {
    StartupPrimaryOnly,
    Eager,
}

impl<'ttf> FontSet<'ttf> {
    fn new(init: FontSetInit<'ttf>) -> Self {
        let icon_fonts = init
            .icon_fonts
            .into_iter()
            .map(|(name, font, raster_font, pixel_size)| IconFont {
                name,
                font,
                raster_font,
                pixel_size,
            })
            .collect();
        let emoji_font = init
            .emoji_font
            .map(|(font, raster_font, pixel_size, shape_face)| EmojiFont {
                font,
                raster_font,
                pixel_size,
                shape_face,
            });
        Self {
            primary: init.primary,
            primary_bold: init.primary_bold,
            primary_italic: init.primary_italic,
            primary_bold_italic: init.primary_bold_italic,
            primary_bold_is_synthetic: init.primary_bold_is_synthetic,
            primary_bold_italic_is_synthetic: init.primary_bold_italic_is_synthetic,
            primary_raster_font: init.primary_raster_font,
            primary_shape_face: init.primary_shape_face,
            primary_pixel_size: init.primary_pixel_size,
            emoji_font,
            ligatures_enabled: init.ligatures_enabled,
            icon_fonts,
            icon_chars: init.icon_chars,
            cell_width: init.cell_width.max(1),
        }
    }

    fn primary(&self) -> &Font<'ttf> {
        &self.primary
    }

    fn primary_for_style(&self, style: TextStyle) -> &Font<'ttf> {
        match (style.bold, style.italic) {
            (true, true) => &self.primary_bold_italic,
            (true, false) => &self.primary_bold,
            (false, true) => &self.primary_italic,
            (false, false) => &self.primary,
        }
    }

    fn primary_style_uses_synthetic_bold(&self, style: TextStyle) -> bool {
        match (style.bold, style.italic) {
            (true, true) => self.primary_bold_italic_is_synthetic,
            (true, false) => self.primary_bold_is_synthetic,
            _ => false,
        }
    }

    fn primary_raster_font(&self) -> &RasterFont {
        &self.primary_raster_font
    }

    fn primary_shape_face(&self) -> &ShapeFace<'static> {
        &self.primary_shape_face
    }

    fn primary_pixel_size(&self) -> f32 {
        self.primary_pixel_size
    }

    fn ligatures_enabled(&self) -> bool {
        self.ligatures_enabled
    }

    fn icon_font(&self, index: usize) -> Option<&IconFont<'ttf>> {
        self.icon_fonts.get(index)
    }

    fn icon_font_index_for_char(&self, character: char) -> Option<usize> {
        self.icon_fonts
            .iter()
            .position(|font| font.font.find_glyph(character).is_some())
    }

    fn icon_fonts(&self) -> &[IconFont<'ttf>] {
        &self.icon_fonts
    }

    fn push_icon_font(
        &mut self,
        name: String,
        font: Font<'ttf>,
        raster_font: RasterFont,
        pixel_size: f32,
    ) {
        self.icon_fonts.push(IconFont {
            name,
            font,
            raster_font,
            pixel_size,
        });
    }

    fn cell_width(&self) -> i32 {
        self.cell_width
    }

    fn prefers_icon_font(&self, character: char) -> bool {
        self.icon_chars.contains(&character)
    }

    /// Returns the emoji font's raster font when configured.
    pub(super) fn emoji_raster_font(&self) -> Option<&RasterFont> {
        self.emoji_font.as_ref().map(|emoji| &emoji.raster_font)
    }

    /// Returns the emoji font's shaping face when configured.
    pub(super) fn emoji_shape_face(&self) -> Option<&ShapeFace<'static>> {
        self.emoji_font.as_ref().map(|emoji| &emoji.shape_face)
    }

    /// Returns the emoji font's pixel size when configured.
    pub(super) fn emoji_pixel_size(&self) -> Option<f32> {
        self.emoji_font.as_ref().map(|emoji| emoji.pixel_size)
    }

    /// Returns true if the emoji font supports the given character.
    pub(super) fn emoji_font_has_char(&self, character: char) -> bool {
        if let Some(emoji_font) = self.emoji_font.as_ref() {
            emoji_font.font.find_glyph(character).is_some()
        } else {
            false
        }
    }

    fn set_emoji_font(
        &mut self,
        font: Font<'ttf>,
        raster_font: RasterFont,
        pixel_size: f32,
        shape_face: ShapeFace<'static>,
    ) {
        self.emoji_font = Some(EmojiFont {
            font,
            raster_font,
            pixel_size,
            shape_face,
        });
    }
}
