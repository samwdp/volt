use editor_render::{DrawCommand, PixelRect, RenderColor};

use crate::text::{OVERLAY_ACCENT_BAR_WIDTH, OVERLAY_SHADOW_OFFSET};

/// Extra right-side pixels so selection glyphs are not flush against the highlight edge.
pub const SELECTION_HIGHLIGHT_RIGHT_PAD_PX: u32 = 2;

const SCROLLBAR_THUMB_WIDTH: u32 = 4;
const SCROLLBAR_THUMB_INSET: i32 = 8;
const SCROLLBAR_MIN_THUMB_HEIGHT: u32 = 12;

/// Overlay card chrome: optional drop shadow, border ring, fill, optional left accent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OverlayCard {
    /// Card bounds.
    pub rect: PixelRect,
    /// Corner radius.
    pub radius: u32,
    /// Outer border fill (already opacity-scaled).
    pub border: RenderColor,
    /// Inner background fill (already opacity-scaled).
    pub background: RenderColor,
    /// Optional left accent bar color (already opacity-scaled).
    pub accent: Option<RenderColor>,
    /// Optional drop-shadow color (already opacity-scaled). `None` skips the shadow.
    pub shadow: Option<RenderColor>,
}

/// Panel chrome: rounded fill, with an optional 1px border ring on opaque windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PanelFrame {
    /// Panel bounds.
    pub rect: PixelRect,
    /// Corner radius.
    pub radius: u32,
    /// Border color used when `opaque_border` is true.
    pub border: RenderColor,
    /// Background fill.
    pub background: RenderColor,
    /// When true, paint a 1px border ring then inset background. When false, a
    /// single rounded fill so translucent windows do not stack alpha.
    pub opaque_border: bool,
}

/// Buffer-body scrollbar thumb geometry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScrollbarThumb {
    /// Full pane rectangle.
    pub pane_rect: PixelRect,
    /// Body origin Y.
    pub body_y: i32,
    /// Visible row count.
    pub visible_rows: usize,
    /// Row height in pixels.
    pub line_height: i32,
    /// Current scroll row.
    pub scroll_row: usize,
    /// Maximum scroll row.
    pub max_scroll: usize,
    /// Thumb color (already opacity-scaled, including the 120/255 thumb alpha).
    pub color: RenderColor,
}

fn push_fill_rect(out: &mut Vec<DrawCommand>, rect: PixelRect, color: RenderColor) {
    out.push(DrawCommand::FillRect { rect, color });
}

fn push_rounded(out: &mut Vec<DrawCommand>, rect: PixelRect, radius: u32, color: RenderColor) {
    out.push(DrawCommand::FillRoundedRect {
        rect,
        radius,
        color,
    });
}

fn push_top_rounded(out: &mut Vec<DrawCommand>, rect: PixelRect, radius: u32, color: RenderColor) {
    out.push(DrawCommand::FillTopRoundedRect {
        rect,
        radius,
        color,
    });
}

/// Paints the overlay drop shadow used by cards.
pub fn paint_overlay_shadow(
    out: &mut Vec<DrawCommand>,
    rect: PixelRect,
    radius: u32,
    color: RenderColor,
) {
    push_rounded(
        out,
        PixelRect::new(
            rect.x + OVERLAY_SHADOW_OFFSET,
            rect.y + OVERLAY_SHADOW_OFFSET,
            rect.width,
            rect.height,
        ),
        radius,
        color,
    );
}

/// Paints a rounded fill with a left accent bar (same geometry as the shell).
pub fn paint_left_accent(
    out: &mut Vec<DrawCommand>,
    rect: PixelRect,
    radius: u32,
    fill: RenderColor,
    accent: RenderColor,
) {
    push_rounded(out, rect, radius, accent);
    let body_width = rect.width.saturating_sub(OVERLAY_ACCENT_BAR_WIDTH);
    if body_width == 0 {
        return;
    }
    push_rounded(
        out,
        PixelRect::new(
            rect.x + OVERLAY_ACCENT_BAR_WIDTH as i32,
            rect.y,
            body_width,
            rect.height,
        ),
        radius,
        fill,
    );
}

/// Paints overlay card chrome.
pub fn paint_overlay_card(out: &mut Vec<DrawCommand>, card: OverlayCard) {
    let OverlayCard {
        rect,
        radius,
        border,
        background,
        accent,
        shadow,
    } = card;
    if let Some(shadow) = shadow {
        paint_overlay_shadow(out, rect, radius, shadow);
    }
    push_rounded(out, rect, radius, border);
    let inner = PixelRect::new(
        rect.x + 1,
        rect.y + 1,
        rect.width.saturating_sub(2),
        rect.height.saturating_sub(2),
    );
    let inner_radius = radius.saturating_sub(1);
    match accent {
        Some(accent) => paint_left_accent(out, inner, inner_radius, background, accent),
        None => push_rounded(out, inner, inner_radius, background),
    }
}

/// Paints a rounded fill, then squares off the left edge so only the right side stays rounded.
pub fn paint_right_band(
    out: &mut Vec<DrawCommand>,
    rect: PixelRect,
    radius: u32,
    color: RenderColor,
) {
    let radius = radius.min(rect.width / 2).min(rect.height / 2);
    push_rounded(out, rect, radius, color);
    if rect.width > radius {
        push_fill_rect(
            out,
            PixelRect::new(rect.x, rect.y, radius, rect.height),
            color,
        );
    }
}

/// Paints a top-only rounded header band.
pub fn paint_top_header_band(
    out: &mut Vec<DrawCommand>,
    header_rect: PixelRect,
    radius: u32,
    color: RenderColor,
) {
    let radius = radius.min(header_rect.width / 2).min(header_rect.height);
    push_top_rounded(out, header_rect, radius, color);
}

/// Paints a panel frame (border ring or single fill).
pub fn paint_panel_frame(out: &mut Vec<DrawCommand>, frame: PanelFrame) {
    let PanelFrame {
        rect,
        radius,
        border,
        background,
        opaque_border,
    } = frame;
    if opaque_border {
        push_rounded(out, rect, radius, border);
        let inner_rect = PixelRect::new(
            rect.x + 1,
            rect.y + 1,
            rect.width.saturating_sub(2),
            rect.height.saturating_sub(2),
        );
        push_rounded(out, inner_rect, radius.saturating_sub(1), background);
    } else {
        push_rounded(out, rect, radius, background);
    }
}

/// Paints the buffer scrollbar thumb. No-op when there is nothing to scroll.
pub fn paint_scrollbar_thumb(out: &mut Vec<DrawCommand>, thumb: ScrollbarThumb) {
    let ScrollbarThumb {
        pane_rect,
        body_y,
        visible_rows,
        line_height,
        scroll_row,
        max_scroll,
        color,
    } = thumb;
    if max_scroll == 0 {
        return;
    }
    let track_height = (visible_rows as i32 * line_height.max(1)).max(1) as u32;
    let thumb_height = ((track_height as u64 * visible_rows as u64)
        / (visible_rows as u64 + max_scroll as u64))
        .clamp(
            u64::from(SCROLLBAR_MIN_THUMB_HEIGHT),
            u64::from(track_height),
        ) as u32;
    let travel = track_height.saturating_sub(thumb_height);
    let thumb_y = body_y + ((travel as u64 * scroll_row as u64) / max_scroll.max(1) as u64) as i32;
    push_rounded(
        out,
        PixelRect::new(
            pane_rect.x + pane_rect.width as i32 - SCROLLBAR_THUMB_INSET,
            thumb_y,
            SCROLLBAR_THUMB_WIDTH,
            thumb_height,
        ),
        2,
        color,
    );
}

/// Paints a selection highlight with the trailing pad used by buffer selections.
pub fn paint_selection_highlight(
    out: &mut Vec<DrawCommand>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    radius: u32,
    color: RenderColor,
) {
    push_rounded(
        out,
        PixelRect::new(
            x,
            y,
            width.saturating_add(SELECTION_HIGHLIGHT_RIGHT_PAD_PX),
            height,
        ),
        radius,
        color,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect() -> PixelRect {
        PixelRect::new(10, 20, 100, 40)
    }

    fn color(a: u8) -> RenderColor {
        RenderColor::rgba(1, 2, 3, a)
    }

    #[test]
    fn overlay_card_without_shadow_matches_border_inner_fill() {
        let mut out = Vec::new();
        paint_overlay_card(
            &mut out,
            OverlayCard {
                rect: rect(),
                radius: 8,
                border: color(255),
                background: color(200),
                accent: None,
                shadow: None,
            },
        );
        assert_eq!(out.len(), 2);
        assert!(matches!(
            out[0],
            DrawCommand::FillRoundedRect { radius: 8, .. }
        ));
        assert!(matches!(
            out[1],
            DrawCommand::FillRoundedRect { radius: 7, .. }
        ));
    }

    #[test]
    fn overlay_card_shadow_and_accent_emit_expected_command_count() {
        let mut out = Vec::new();
        paint_overlay_card(
            &mut out,
            OverlayCard {
                rect: rect(),
                radius: 8,
                border: color(255),
                background: color(200),
                accent: Some(color(180)),
                shadow: Some(RenderColor::rgba(0, 0, 0, 72)),
            },
        );
        // shadow + border + accent fill + body fill
        assert_eq!(out.len(), 4);
    }

    #[test]
    fn panel_frame_opaque_paints_border_then_inset() {
        let mut out = Vec::new();
        paint_panel_frame(
            &mut out,
            PanelFrame {
                rect: rect(),
                radius: 6,
                border: color(255),
                background: color(128),
                opaque_border: true,
            },
        );
        assert_eq!(out.len(), 2);
        match &out[1] {
            DrawCommand::FillRoundedRect { rect, radius, .. } => {
                assert_eq!(rect.x, 11);
                assert_eq!(rect.y, 21);
                assert_eq!(rect.width, 98);
                assert_eq!(rect.height, 38);
                assert_eq!(*radius, 5);
            }
            other => panic!("unexpected command {other:?}"),
        }
    }

    #[test]
    fn panel_frame_translucent_is_single_fill() {
        let mut out = Vec::new();
        paint_panel_frame(
            &mut out,
            PanelFrame {
                rect: rect(),
                radius: 6,
                border: color(255),
                background: color(128),
                opaque_border: false,
            },
        );
        assert_eq!(out.len(), 1);
    }

    #[test]
    fn scrollbar_is_noop_when_nothing_to_scroll() {
        let mut out = Vec::new();
        paint_scrollbar_thumb(
            &mut out,
            ScrollbarThumb {
                pane_rect: rect(),
                body_y: 20,
                visible_rows: 10,
                line_height: 16,
                scroll_row: 0,
                max_scroll: 0,
                color: color(255),
            },
        );
        assert!(out.is_empty());
    }
}
