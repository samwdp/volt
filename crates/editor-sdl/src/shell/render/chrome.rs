pub(super) fn paint_overlay_card(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    style: OverlayCardStyle,
) -> Result<(), ShellError> {
    let OverlayCardStyle {
        radius,
        border,
        background,
        window_effects,
        accent,
        shadow,
    } = style;
    paint_ui_overlay_card(
        target.scene(),
        OverlayCard {
            rect: to_pixel_rect(rect),
            radius,
            border: to_render_color(overlay_window_surface_color(border, window_effects)),
            background: to_render_color(overlay_window_surface_color(background, window_effects)),
            accent: accent.map(|color| {
                to_render_color(overlay_window_surface_color(color, window_effects))
            }),
            shadow: shadow.then(|| {
                to_render_color(overlay_window_surface_color(
                    Color::RGBA(0, 0, 0, 72),
                    window_effects,
                ))
            }),
        },
    );
    Ok(())
}

pub(super) fn fill_rounded_rect_with_left_accent(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    fill: Color,
    accent: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    paint_left_accent(
        target.scene(),
        to_pixel_rect(rect),
        radius,
        to_render_color(overlay_window_surface_color(fill, window_effects)),
        to_render_color(overlay_window_surface_color(accent, window_effects)),
    );
    Ok(())
}

fn fill_overlay_right_band(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    paint_right_band(
        target.scene(),
        to_pixel_rect(rect),
        radius,
        to_render_color(overlay_window_surface_color(color, window_effects)),
    );
    Ok(())
}

fn fill_overlay_top_header_band(
    target: &mut DrawTarget<'_>,
    header_rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    // CONTEXT: top-only rounded fill with no self-overlap. The old "rounded rect
    // then square overpaint" stacked alpha on the lower header and left the
    // rounded cap looking broken once window.opacity < 1.
    paint_top_header_band(
        target.scene(),
        to_pixel_rect(header_rect),
        radius,
        to_render_color(overlay_window_surface_color(color, window_effects)),
    );
    Ok(())
}

fn fill_window_top_header_band(
    target: &mut DrawTarget<'_>,
    header_rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    // CONTEXT: top-only rounded fill with no self-overlap. The old "rounded rect
    // then square overpaint" stacked alpha on the lower header and left the
    // rounded cap looking broken once window.opacity < 1.
    paint_top_header_band(
        target.scene(),
        to_pixel_rect(header_rect),
        radius,
        to_render_color(window_surface_color(color, window_effects)),
    );
    Ok(())
}

/// Panel frame for ACP/plugin/input sections. When `window.opacity` < 1, paint a
/// single rounded fill so every section shares one opacity layer (no border+inner
/// stack). Opaque windows keep the 1px border ring.
fn fill_window_panel_frame(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    border: Color,
    background: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    let opacity = crate::window_effects::window_surface_opacity(window_effects);
    paint_panel_frame(
        target.scene(),
        PanelFrame {
            rect: to_pixel_rect(rect),
            radius,
            border: to_render_color(window_surface_color(border, window_effects)),
            background: to_render_color(window_surface_color(background, window_effects)),
            opaque_border: opacity >= 1.0,
        },
    );
    Ok(())
}

/// Darken (or lighten on light themes) from the editor base so ACP/plugin section
/// bodies stay distinct once `window.opacity` scales their alpha.
pub(super) fn buffer_section_panel_background(base_background: Color) -> Color {
    adjust_color(
        base_background,
        if is_dark_color(base_background) {
            -10
        } else {
            10
        },
    )
}

pub(super) fn buffer_section_header_background(
    theme_registry: Option<&ThemeRegistry>,
    panel_background: Color,
) -> Color {
    theme_color(
        theme_registry,
        "ui.panel.header.background",
        adjust_color(
            panel_background,
            if is_dark_color(panel_background) {
                12
            } else {
                -12
            },
        ),
    )
}

#[derive(Debug, Clone, Copy)]
pub(super) struct CursorScreenAnchor {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) pane_bottom: i32,
}

pub(super) fn popup_content_rect(popup_rect: Rect) -> Rect {
    PixelRectToRect::rect(
        popup_rect.x() + 6,
        popup_rect.y() + 14,
        popup_rect.width().saturating_sub(12),
        popup_rect.height().saturating_sub(20),
    )
}

pub(super) fn render_runtime_popup_overlay(
    target: &mut DrawTarget<'_>,
    state: &ShellUiState,
    popup: &RuntimePopupSnapshot,
    popup_rect: Rect,
    chrome: ShellChrome<'_>,
    metrics: TextMetrics,
    pulse: FramePulse,
) -> Result<(), ShellError> {
    let ShellChrome { theme_registry, .. } = chrome;
    let FramePulse { now, typing_active } = pulse;
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let is_dark = is_dark_color(base_background);
    let popup_background = base_background;
    let border_color = adjust_color(base_background, if is_dark { 24 } else { -24 });
    let git_summary = state.git_summary();
    let popup_radius = overlay_radius(theme_registry);
    fill_overlay_surface_rounded_rect(
        target,
        popup_rect,
        popup_radius,
        popup_background,
        window_effects,
    )?;
    fill_overlay_surface_rounded_rect(
        target,
        PixelRectToRect::rect(
            popup_rect.x() + popup_rect.width() as i32 / 2 - 16,
            popup_rect.y() + 6,
            32,
            4,
        ),
        2,
        border_color,
        window_effects,
    )?;
    let popup_focus = state.popup_focus_active(popup);
    if let Some(buffer) = state.buffer(popup.active_buffer) {
        let view_state = buffer.view_state();
        let input_mode = state.input_mode_for_buffer(buffer.id(), popup_focus);
        let vim_targets_input =
            state.vim_target_for_buffer(buffer.id(), popup_focus) == VimTarget::Input;
        let visual_range = state.visual_selection_for_buffer(buffer, popup_focus);
        let multicursor = state
            .multicursor_for_buffer(buffer.id(), popup_focus)
            .cloned();
        let yank_flash = state.yank_flash(buffer.id(), now);
        render_buffer(
            target,
            BufferDrawRequest {
                buffer,
                view_state,
                pane: PaneSlot {
                    rect: popup_content_rect(popup_rect),
                    active: popup_focus,
                },
                decorations: BufferDecorations {
                    visual_selection: visual_range,
                    yank_flash,
                    input_mode,
                    multicursor: multicursor.as_ref(),
                    vim_targets_input,
                    recording_macro: state.vim().recording_macro,
                    typing_active,
                },
                command_line: CommandLineSlot {
                    input: None,
                    row_visible: false,
                },
            },
            BufferChrome::from_shell(&chrome, git_summary.as_ref()),
            metrics,
        )?;
    }

    Ok(())
}

pub(super) fn render_autocomplete_overlay(
    target: &mut DrawTarget<'_>,
    state: &ShellUiState,
    autocomplete: &AutocompleteOverlay,
    layout: OverlayAnchorContext<'_>,
) -> Result<(), ShellError> {
    let OverlayAnchorContext {
        pane_rect,
        user_library,
        theme_registry,
        metrics: CellMetrics {
            cell_width,
            line_height,
        },
        typing_active,
    } = layout;
    let Some(buffer) = state.buffer(autocomplete.buffer_id) else {
        return Ok(());
    };
    let Some(anchor) = buffer_cursor_screen_anchor(
        buffer,
        pane_rect,
        user_library,
        theme_registry,
        cell_width,
        line_height,
        typing_active,
    ) else {
        return Ok(());
    };
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let base_foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let accent = theme_color(
        theme_registry,
        "ui.selection",
        adjust_color(base_background, if is_dark { 48 } else { -48 }),
    );
    let panel_background = theme_color(
        theme_registry,
        "ui.autocomplete.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let foreground = theme_color(
        theme_registry,
        "ui.autocomplete.foreground",
        base_foreground,
    );
    let border = theme_color(
        theme_registry,
        "ui.autocomplete.border",
        adjust_color(base_background, if is_dark { 30 } else { -30 }),
    );
    let docs_background = theme_color(
        theme_registry,
        "ui.autocomplete.documentation.background",
        adjust_color(panel_background, if is_dark { 4 } else { -4 }),
    );
    let selected_background = theme_color(
        theme_registry,
        "ui.autocomplete.selection",
        blend_color(accent, panel_background, 0.72),
    );
    let muted = theme_color(
        theme_registry,
        "ui.autocomplete.muted",
        blend_color(base_foreground, panel_background, 0.46),
    );
    let row_height = line_height.max(1);
    let width = overlay_width(pane_rect.width(), cell_width, 48, 72);
    let list_width = ((width.saturating_mul(36)) / 100)
        .max((cell_width.max(1) as u32) * 18)
        .min((cell_width.max(1) as u32) * 28)
        .min(width.saturating_sub((cell_width.max(1) as u32) * 18));
    let docs_width = width.saturating_sub(list_width).saturating_sub(1);
    let docs_columns = overlay_text_columns(docs_width, 20, cell_width);
    let visible_result_limit = user_library.autocomplete_result_limit().max(1);
    let max_body_rows = ((pane_rect.height().saturating_sub(28)) / row_height as u32)
        .clamp(4, visible_result_limit.max(6) as u32 + 2) as usize;
    let preview_lines = autocomplete_preview_lines(
        autocomplete.selected(),
        &autocomplete.query.token,
        docs_columns,
        max_body_rows,
        user_library.autocomplete_token_icon(),
    );
    let body_rows = autocomplete
        .entries()
        .len()
        .max(preview_lines.len())
        .max(1)
        .min(max_body_rows);
    let height = row_height as u32 * body_rows as u32 + 18;
    let preferred_x = anchor.x - (cell_width.max(1) * 3);
    let max_x = pane_rect.x() + pane_rect.width() as i32 - width as i32 - 8;
    let x = preferred_x.clamp(pane_rect.x() + 8, max_x.max(pane_rect.x() + 8));
    let below_y = anchor.y + row_height + 6;
    let above_y = anchor.y - height as i32 - 6;
    let y = if below_y + height as i32 <= anchor.pane_bottom {
        below_y
    } else {
        above_y.max(pane_rect.y() + 8)
    };
    let outer_rect = PixelRectToRect::rect(x, y, width, height);
    let radius = overlay_radius(theme_registry);
    paint_overlay_card(
        target,
        outer_rect,
        OverlayCardStyle {
            radius,
            border,
            background: panel_background,
            window_effects,
            accent: None,
            shadow: false,
        },
    )?;
    fill_overlay_surface_rect(
        target,
        PixelRectToRect::rect(x + list_width as i32, y + 8, 1, height.saturating_sub(16)),
        border,
        window_effects,
    )?;
    fill_overlay_right_band(
        target,
        PixelRectToRect::rect(
            x + list_width as i32 + 1,
            y + 1,
            docs_width.saturating_sub(1),
            height.saturating_sub(2),
        ),
        radius.saturating_sub(1),
        docs_background,
        window_effects,
    )?;
    if autocomplete.entries().is_empty() {
        return Ok(());
    }
    let list_text_width = list_width.saturating_sub(24);
    let visible_start = autocomplete_visible_start(
        autocomplete.entries().len(),
        autocomplete.selected_index,
        body_rows,
    );
    for (row_index, entry) in autocomplete
        .entries()
        .iter()
        .enumerate()
        .skip(visible_start)
        .take(body_rows)
    {
        let index = row_index - visible_start;
        let row_y = y + 8 + index as i32 * row_height;
        if row_index == autocomplete.selected_index {
            fill_rounded_rect_with_left_accent(
                target,
                PixelRectToRect::rect(
                    x + 6,
                    row_y - 2,
                    list_width.saturating_sub(12),
                    row_height as u32,
                ),
                overlay_radius(theme_registry).min(8),
                selected_background,
                accent,
                window_effects,
            )?;
        }
        let label = truncate_text_to_width(
            &format!("{} {}", entry.item_icon, entry.label),
            list_text_width,
            cell_width,
        );
        draw_text(target, x + 10, row_y, &label, foreground)?;
    }
    for (index, line) in preview_lines.iter().take(body_rows).enumerate() {
        let row_y = y + 8 + index as i32 * row_height;
        let color = if index == 0 { foreground } else { muted };
        let clipped = truncate_text_to_width(line, docs_width.saturating_sub(20), cell_width);
        draw_text(target, x + list_width as i32 + 11, row_y, &clipped, color)?;
    }
    Ok(())
}
