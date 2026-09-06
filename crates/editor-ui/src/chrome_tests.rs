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
