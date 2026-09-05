#![doc = r#"Shared chrome widgets used by the native editor shell.

These helpers emit the same `DrawCommand`s the SDL shell already paints. Callers
must apply window/overlay opacity to colors before painting so pixels stay
identical to the previous inline implementations.
"#]

mod chrome;
mod text;

pub use chrome::{
    OverlayCard, PanelFrame, SELECTION_HIGHLIGHT_RIGHT_PAD_PX, ScrollbarThumb, paint_left_accent,
    paint_overlay_card, paint_overlay_shadow, paint_panel_frame, paint_right_band,
    paint_scrollbar_thumb, paint_selection_highlight, paint_top_header_band,
};
pub use editor_render::{DrawCommand, PixelRect, RenderColor};
pub use text::{
    OVERLAY_ACCENT_BAR_WIDTH, OVERLAY_SHADOW_OFFSET, truncate_text_to_width,
    truncate_text_to_width_preserving_end,
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Shared chrome widgets (cards, panels, scrollbars, chips geometry) for consistent shell UI.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}
