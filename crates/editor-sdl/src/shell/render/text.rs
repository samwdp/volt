pub(super) fn strip_zero_width_display_characters(text: &str) -> std::borrow::Cow<'_, str> {
    if !text.chars().any(is_zero_width_display_character) {
        return std::borrow::Cow::Borrowed(text);
    }
    std::borrow::Cow::Owned(
        text.chars()
            .filter(|character| !is_zero_width_display_character(*character))
            .collect(),
    )
}

pub(super) fn monospace_text_width(text: &str, cell_width: i32) -> u32 {
    let char_map = LineCharMap::new(text);
    (char_map.display_col_at(char_map.len()) as u32).saturating_mul(cell_width.max(1) as u32)
}

pub(super) fn to_sdl_color(color: ThemeColor) -> Color {
    Color::RGBA(color.r, color.g, color.b, color.a)
}

pub(super) fn to_render_color(color: Color) -> RenderColor {
    RenderColor::rgba(color.r, color.g, color.b, color.a)
}

pub(super) fn text_style_from_theme_style(style: ThemeStyle) -> TextStyle {
    TextStyle::new(style.bold, style.italic)
}

pub(super) fn from_render_color(color: RenderColor) -> Color {
    Color::RGBA(color.r, color.g, color.b, color.a)
}

pub(super) fn to_pixel_rect(rect: Rect) -> PixelRect {
    PixelRect::new(rect.x(), rect.y(), rect.width(), rect.height())
}
