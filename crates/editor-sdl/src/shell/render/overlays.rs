fn autocomplete_visible_start(
    entry_count: usize,
    selected_index: usize,
    visible_rows: usize,
) -> usize {
    if entry_count <= visible_rows {
        return 0;
    }
    selected_index
        .saturating_sub(visible_rows.saturating_sub(1))
        .min(entry_count - visible_rows)
}

pub(super) fn render_hover_overlay(
    target: &mut DrawTarget<'_>,
    state: &ShellUiState,
    hover: &HoverOverlay,
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
    let Some(buffer) = state.buffer(hover.buffer_id) else {
        return Ok(());
    };
    let Some(provider) = hover.current_provider() else {
        return Ok(());
    };
    let anchor = buffer_cursor_screen_anchor(
        buffer,
        pane_rect,
        user_library,
        theme_registry,
        cell_width,
        line_height,
        typing_active,
    )
    .unwrap_or_else(|| {
        fallback_overlay_anchor(
            buffer,
            pane_rect,
            line_height,
            user_library.commandline_enabled(),
        )
    });
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
    let background = theme_color(
        theme_registry,
        "ui.hover.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let foreground = theme_color(theme_registry, "ui.hover.foreground", base_foreground);
    let border = theme_color(
        theme_registry,
        "ui.hover.border",
        adjust_color(base_background, if is_dark { 30 } else { -30 }),
    );
    let header_background = theme_color(
        theme_registry,
        "ui.hover.header.background",
        adjust_color(background, if is_dark { 6 } else { -6 }),
    );
    let focus_border = if hover.focused {
        theme_color(theme_registry, "ui.hover.focused.border", accent)
    } else {
        border
    };
    let selected_tab = theme_color(
        theme_registry,
        "ui.hover.selection",
        blend_color(accent, header_background, 0.68),
    );
    let muted = theme_color(
        theme_registry,
        "ui.hover.muted",
        blend_color(base_foreground, background, 0.46),
    );
    let row_height = line_height.max(1);
    let width = hover_overlay_width(hover, provider, pane_rect.width(), cell_width);
    let body_columns = overlay_text_columns(width, 28, cell_width);
    let body_lines = wrap_hover_overlay_lines(
        provider,
        hover.scroll_offset,
        hover.line_limit.max(1),
        body_columns,
        hover.line_limit.max(1),
    );
    let footer_text = if provider.lines.len() > hover.visible_lines().len() {
        Some(format!(
            "Lines {}-{} of {}",
            hover.scroll_offset + 1,
            hover.scroll_offset + hover.visible_lines().len(),
            provider.lines.len()
        ))
    } else if hover.focused {
        Some("Esc returns to the buffer".to_owned())
    } else {
        Some("Run hover.focus to enter the panel".to_owned())
    };
    let tabs_height = row_height as u32 + 10;
    let title_rows = 1u32;
    let body_rows = body_lines.len().max(1) as u32;
    let footer_rows = u32::from(footer_text.is_some());
    let height = tabs_height + row_height as u32 * (title_rows + body_rows + footer_rows) + 22;
    let min_x = pane_rect.x() + 8;
    let max_x = pane_rect.x() + pane_rect.width() as i32 - width as i32 - 8;
    let preferred_x = anchor.x - (cell_width.max(1) * 6);
    let x = preferred_x.clamp(min_x, max_x.max(min_x));
    let below_y = (anchor.y + row_height + 6)
        .min(anchor.pane_bottom - height as i32)
        .max(pane_rect.y() + 8);
    let above_y = anchor.y - height as i32 - 6;
    let y = if above_y >= pane_rect.y() + 8 {
        above_y
    } else {
        below_y
    };
    let outer_rect = PixelRectToRect::rect(x, y, width, height);
    let radius = overlay_radius(theme_registry);
    paint_overlay_card(
        target,
        outer_rect,
        OverlayCardStyle {
            radius,
            border: focus_border,
            background,
            window_effects,
            accent: None,
            shadow: false,
        },
    )?;
    fill_overlay_top_header_band(
        target,
        PixelRectToRect::rect(x + 1, y + 1, width.saturating_sub(2), tabs_height),
        radius.saturating_sub(1),
        header_background,
        window_effects,
    )?;
    fill_overlay_surface_rect(
        target,
        PixelRectToRect::rect(x + 1, y + tabs_height as i32, width.saturating_sub(2), 1),
        border,
        window_effects,
    )?;

    let mut tab_x = x + 10;
    let tab_y = y + 6;
    for (index, tab) in hover.providers.iter().enumerate() {
        let label = format!("{} {}", tab.provider_icon, tab.provider_label);
        let tab_width = monospace_text_width(&label, cell_width).saturating_add(16);
        if index == hover.provider_index {
            fill_overlay_surface_rounded_rect(
                target,
                PixelRectToRect::rect(tab_x - 4, tab_y - 2, tab_width, row_height as u32 + 4),
                5,
                selected_tab,
                window_effects,
            )?;
        }
        draw_text(
            target,
            tab_x,
            tab_y,
            &label,
            if index == hover.provider_index {
                foreground
            } else {
                muted
            },
        )?;
        tab_x += tab_width as i32 + 4;
    }

    let title_y = y + tabs_height as i32 + 8;
    let title = truncate_text_to_width(
        &format!("{} {}", provider.provider_icon, hover.token),
        width.saturating_sub(28),
        cell_width,
    );
    draw_text(target, x + 12, title_y, &title, foreground)?;
    let status = if hover.focused { "Focused" } else { "Preview" };
    let status_width = monospace_text_width(status, cell_width) as i32;
    draw_text(
        target,
        x + width as i32 - status_width - 12,
        title_y,
        status,
        muted,
    )?;

    if body_lines.is_empty() {
        draw_text(
            target,
            x + 12,
            title_y + row_height,
            "No hover details",
            muted,
        )?;
    } else {
        for (index, line) in body_lines.iter().enumerate() {
            let row_y = title_y + row_height + index as i32 * row_height;
            draw_buffer_text(
                target,
                BufferTextRun {
                    x: x + 12,
                    y: row_y,
                    line: &line.line,
                    segment: line.segment,
                    char_map: &line.char_map,
                    line_syntax_spans: provider.line_syntax_spans(line.source_line_index),
                    default_color: foreground,
                    cell_width,
                },
                theme_registry,
            )?;
        }
    }
    if let Some(footer_text) = footer_text {
        draw_text(
            target,
            x + 12,
            title_y + row_height + body_lines.len() as i32 * row_height,
            &truncate_text_to_width(&footer_text, width.saturating_sub(20), cell_width),
            muted,
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct WrappedHoverOverlayLine {
    source_line_index: usize,
    line: String,
    char_map: LineCharMap,
    segment: LineWrapSegment,
}

fn wrap_hover_overlay_lines(
    provider: &HoverProviderContent,
    scroll_offset: usize,
    source_line_limit: usize,
    max_columns: usize,
    max_rows: usize,
) -> Vec<WrappedHoverOverlayLine> {
    if max_rows == 0 {
        return Vec::new();
    }
    let start = scroll_offset.min(provider.lines.len());
    let end = (start + source_line_limit).min(provider.lines.len());
    let mut wrapped = Vec::new();
    for source_line_index in start..end {
        let line = provider.lines[source_line_index].clone();
        let char_map = LineCharMap::new(&line);
        for segment in wrap_line_segments(&char_map, max_columns, max_columns) {
            if wrapped.len() >= max_rows {
                return wrapped;
            }
            wrapped.push(WrappedHoverOverlayLine {
                source_line_index,
                line: line.clone(),
                char_map: char_map.clone(),
                segment,
            });
        }
    }
    wrapped
}

pub(super) fn notification_accent_color(
    theme_registry: Option<&ThemeRegistry>,
    severity: NotificationSeverity,
    fallback: Color,
) -> Color {
    let token = match severity {
        NotificationSeverity::Info => "ui.notification.info",
        NotificationSeverity::Success => "ui.notification.success",
        NotificationSeverity::Warning => "ui.notification.warning",
        NotificationSeverity::Error => "ui.notification.error",
    };
    theme_color(theme_registry, token, fallback)
}

pub(super) fn notification_status_text(notification: &ShellNotification) -> Option<String> {
    match notification
        .progress
        .and_then(|progress| progress.percentage)
    {
        Some(percentage) if notification.active => Some(format!("{percentage}%")),
        None if notification.active && notification.progress.is_some() => {
            Some("Working".to_owned())
        }
        _ => None,
    }
}

pub(super) fn notification_overlay_layouts(
    notifications: &[&ShellNotification],
    width: u32,
    height: u32,
    cell_width: i32,
    line_height: i32,
) -> Vec<NotificationOverlayLayout> {
    if notifications.is_empty() {
        return Vec::new();
    }
    let row_height = line_height.max(1) as u32;
    let toast_width = overlay_width(width, cell_width, 34, 56);
    let body_columns = overlay_text_columns(toast_width, 28, cell_width);
    let x = width as i32 - toast_width as i32 - 12;
    let mut layouts = Vec::new();
    let mut bottom = height as i32 - 12;
    for notification in notifications {
        let body_lines = wrap_overlay_lines(
            &notification.body_lines,
            body_columns,
            NOTIFICATION_MAX_BODY_LINES,
        );
        let body_rows = body_lines.len() as u32;
        let progress_height = u32::from(notification.progress.is_some()) * 10;
        let body_gap = u32::from(!body_lines.is_empty()) * 4;
        let panel_height = row_height + body_rows * row_height + body_gap + progress_height + 20;
        let y = bottom - panel_height as i32;
        if y < 8 {
            break;
        }
        layouts.push(NotificationOverlayLayout {
            rect: PixelRectToRect::rect(x, y, toast_width, panel_height),
            title: notification.title.clone(),
            body_lines,
            status_text: notification_status_text(notification),
            severity: notification.severity,
            progress: notification.progress,
            active: notification.active,
            action: notification.action.clone(),
        });
        bottom = y - NOTIFICATION_STACK_GAP;
    }
    layouts
}

pub(super) fn notification_action_at_point(
    state: &ShellUiState,
    width: u32,
    height: u32,
    cell_width: i32,
    line_height: i32,
    now: Instant,
    point: (i32, i32),
) -> Option<NotificationAction> {
    let (x, y) = point;
    let notifications = state.visible_notifications(now);
    notification_overlay_layouts(&notifications, width, height, cell_width, line_height)
        .into_iter()
        .find(|layout| {
            let rect = layout.rect;
            let right = rect.x().saturating_add(rect.width() as i32);
            let bottom = rect.y().saturating_add(rect.height() as i32);
            x >= rect.x() && y >= rect.y() && x < right && y < bottom
        })
        .and_then(|layout| layout.action)
}

pub(super) fn render_notification_overlay(
    target: &mut DrawTarget<'_>,
    state: &ShellUiState,
    size: WindowSize,
    theme_registry: Option<&ThemeRegistry>,
    metrics: CellMetrics,
    now: Instant,
) -> Result<(), ShellError> {
    let WindowSize { width, height } = size;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let notifications = state.visible_notifications(now);
    let layouts =
        notification_overlay_layouts(&notifications, width, height, cell_width, line_height);
    if layouts.is_empty() {
        return Ok(());
    }

    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let base_foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let background = theme_color(
        theme_registry,
        "ui.notification.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let foreground = theme_color(
        theme_registry,
        "ui.notification.foreground",
        base_foreground,
    );
    let title_color = theme_color(theme_registry, "ui.notification.title", foreground);
    let muted = theme_color(
        theme_registry,
        "ui.notification.muted",
        blend_color(base_foreground, background, 0.46),
    );
    let border = theme_color(
        theme_registry,
        "ui.notification.border",
        adjust_color(base_background, if is_dark { 30 } else { -30 }),
    );
    let progress_background = theme_color(
        theme_registry,
        "ui.notification.progress.background",
        adjust_color(background, if is_dark { 10 } else { -10 }),
    );
    let default_info = theme_color(
        theme_registry,
        "ui.statusline.active",
        adjust_color(base_background, if is_dark { 56 } else { -56 }),
    );

    for layout in layouts {
        let accent = notification_accent_color(theme_registry, layout.severity, default_info);
        let outer_rect = layout.rect;
        let radius = overlay_radius(theme_registry);
        paint_overlay_card(
            target,
            outer_rect,
            OverlayCardStyle {
                radius,
                border,
                background,
                window_effects,
                accent: Some(accent),
                shadow: false,
            },
        )?;

        let title_y = layout.rect.y() + 10;
        let status_width = layout
            .status_text
            .as_ref()
            .map(|status| monospace_text_width(status, cell_width) as i32)
            .unwrap_or(0);
        let title_width = layout
            .rect
            .width()
            .saturating_sub((28 + status_width.max(0) as u32).max(28));
        let title = truncate_text_to_width(&layout.title, title_width, cell_width);
        draw_text(target, layout.rect.x() + 14, title_y, &title, title_color)?;
        if let Some(status_text) = layout.status_text.as_ref() {
            draw_text(
                target,
                layout.rect.x() + layout.rect.width() as i32 - status_width - 12,
                title_y,
                status_text,
                accent,
            )?;
        }

        for (index, line) in layout.body_lines.iter().enumerate() {
            let row_y = title_y + line_height.max(1) + 4 + index as i32 * line_height.max(1);
            let clipped =
                truncate_text_to_width(line, layout.rect.width().saturating_sub(28), cell_width);
            draw_text(
                target,
                layout.rect.x() + 14,
                row_y,
                &clipped,
                if index == 0 { foreground } else { muted },
            )?;
        }

        if let Some(progress) = layout.progress {
            let bar_width = layout.rect.width().saturating_sub(28);
            let bar_x = layout.rect.x() + 14;
            let bar_y = layout.rect.y() + layout.rect.height() as i32 - 10;
            fill_overlay_surface_rounded_rect(
                target,
                PixelRectToRect::rect(bar_x, bar_y, bar_width, 4),
                2,
                progress_background,
                window_effects,
            )?;
            let fill_width = if layout.active {
                progress
                    .percentage
                    .map(|percentage| {
                        ((bar_width.saturating_mul(u32::from(percentage))) / 100).max(1)
                    })
                    .unwrap_or(bar_width / 3)
            } else {
                bar_width
            };
            if fill_width > 0 {
                let fill = theme_color(theme_registry, "ui.notification.progress.fill", accent);
                fill_rounded_rect(
                    target,
                    PixelRectToRect::rect(bar_x, bar_y, fill_width, 4),
                    2,
                    fill,
                )?;
            }
        }
    }
    Ok(())
}

pub(super) fn overlay_width(
    pane_width: u32,
    cell_width: i32,
    min_cells: u32,
    max_cells: u32,
) -> u32 {
    let available = pane_width.saturating_sub(16);
    let min_width = ((cell_width.max(1) as u32) * min_cells).min(available);
    let max_width = ((cell_width.max(1) as u32) * max_cells)
        .min(available)
        .max(min_width);
    ((pane_width.saturating_mul(3)) / 4).clamp(min_width, max_width)
}

pub(super) fn hover_overlay_width(
    hover: &HoverOverlay,
    provider: &HoverProviderContent,
    pane_width: u32,
    cell_width: i32,
) -> u32 {
    // Body rows are drawn at x + 12 and wrapped via overlay_text_columns(width, 28, ...).
    let mut content_width = 0u32;
    for line in &provider.lines {
        content_width =
            content_width.max(monospace_text_width(line, cell_width).saturating_add(28));
    }
    // Title row: "{icon} {token}" on the left, "Focused"/"Preview" right-aligned.
    let title = format!("{} {}", provider.provider_icon, hover.token);
    let status_width = monospace_text_width("Preview", cell_width)
        .max(monospace_text_width("Focused", cell_width));
    content_width = content_width.max(
        monospace_text_width(&title, cell_width)
            .saturating_add(status_width)
            .saturating_add(40),
    );
    // Tab strip: each tab is label + 16px padding + 4px gap, starting at x + 10.
    let mut tabs_width = 14u32;
    for tab in &hover.providers {
        let label = format!("{} {}", tab.provider_icon, tab.provider_label);
        tabs_width = tabs_width
            .saturating_add(monospace_text_width(&label, cell_width))
            .saturating_add(20);
    }
    content_width = content_width.max(tabs_width);
    let available = pane_width.saturating_sub(16);
    let min_width = ((cell_width.max(1) as u32) * 44).min(available);
    content_width.clamp(min_width, available.max(min_width))
}

pub(super) fn overlay_text_columns(width: u32, horizontal_padding: u32, cell_width: i32) -> usize {
    (width.saturating_sub(horizontal_padding) / cell_width.max(1) as u32)
        .max(1)
        .try_into()
        .unwrap_or(1)
}
