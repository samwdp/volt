fn render_command_line_completion_popup(
    target: &mut DrawTarget<'_>,
    matches: &[String],
    selected_index: usize,
    pane_rect: Rect,
    commandline_y: i32,
    theme_registry: Option<&ThemeRegistry>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    if matches.is_empty() {
        return Ok(());
    }
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let background = adjust_color(base_background, if is_dark { 14 } else { -14 });
    let selection = theme_color(
        theme_registry,
        "ui.selection",
        adjust_color(base_background, if is_dark { 36 } else { -36 }),
    );
    let visible = matches.len().min(8);
    let row_height = line_height.max(1);
    let height = row_height as u32 * visible as u32 + 10;
    let width = pane_rect.width().saturating_sub(16);
    let y = (commandline_y - height as i32 - 6).max(pane_rect.y() + 8);
    let x = pane_rect.x() + 8;
    let radius = overlay_radius(theme_registry).min(8);
    paint_overlay_card(
        target,
        PixelRectToRect::rect(x, y, width, height),
        OverlayCardStyle {
            radius,
            border: adjust_color(background, if is_dark { 24 } else { -24 }),
            background,
            window_effects,
            accent: None,
            shadow: false,
        },
    )?;
    let start = selected_index.saturating_sub(visible.saturating_sub(1));
    for (row, (index, label)) in matches
        .iter()
        .enumerate()
        .skip(start)
        .take(visible)
        .enumerate()
    {
        let row_y = y + 5 + row as i32 * row_height;
        if index == selected_index {
            fill_overlay_surface_rounded_rect(
                target,
                PixelRectToRect::rect(x + 4, row_y - 1, width.saturating_sub(8), row_height as u32),
                4,
                selection,
                window_effects,
            )?;
        }
        draw_text(
            target,
            x + 10,
            row_y,
            &truncate_text_to_width(label, width.saturating_sub(20), cell_width),
            foreground,
        )?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(super) struct TextPaneLayout {
    pub(super) rect: Rect,
    pub(super) visible_rows: usize,
    pub(super) wrap_cols: usize,
}

#[derive(Debug, Clone)]
pub(super) struct PluginSectionLayout {
    pub(super) panes: Vec<TextPaneLayout>,
}

pub(super) fn plugin_section_row_budget(
    min_rows: &[Option<usize>],
    total_row_budget: usize,
) -> Vec<usize> {
    let section_count = min_rows.len().max(1);
    let mut rows = min_rows
        .iter()
        .map(|min_rows| min_rows.unwrap_or(1).max(1))
        .collect::<Vec<_>>();
    let mut used = rows.iter().sum::<usize>();
    while used > total_row_budget {
        let Some((index, _)) = rows.iter().enumerate().max_by_key(|(_, value)| **value) else {
            break;
        };
        if rows[index] <= 1 {
            break;
        }
        rows[index] = rows[index].saturating_sub(1);
        used = used.saturating_sub(1);
    }
    if used >= total_row_budget {
        return rows;
    }
    let flexible = min_rows
        .iter()
        .enumerate()
        .filter_map(|(index, value)| value.is_none().then_some(index))
        .collect::<Vec<_>>();
    let recipients = if flexible.is_empty() {
        vec![section_count.saturating_sub(1)]
    } else {
        flexible
    };
    let mut remaining = total_row_budget.saturating_sub(used);
    let mut recipient_index = 0usize;
    while remaining > 0 {
        let index = recipients[recipient_index % recipients.len()];
        rows[index] = rows[index].saturating_add(1);
        recipient_index = recipient_index.saturating_add(1);
        remaining = remaining.saturating_sub(1);
    }
    rows
}

pub(super) fn text_panel_header_height(title: &str, line_height: i32) -> i32 {
    if title.trim().is_empty() {
        0
    } else {
        line_height.max(1) + 10
    }
}

pub(super) fn text_panel_chrome_height(title: &str, line_height: i32) -> i32 {
    text_panel_header_height(title, line_height) + 12
}

pub(super) fn input_panel_chrome_height() -> i32 {
    INPUT_PANEL_VERTICAL_PADDING * 2
}

pub(super) fn plugin_section_panel_chrome_height(title: &str, line_height: i32) -> i32 {
    text_panel_chrome_height(title, line_height) + 4
}

pub(super) fn plugin_section_buffer_layout(
    buffer: &ShellBuffer,
    rect: Rect,
    layout: BufferFooterLayout,
    cell_width: i32,
    line_height: i32,
) -> Option<PluginSectionLayout> {
    let state = buffer.plugin_sections()?;
    let section_count = state.section_count();
    let line_height = line_height.max(1);
    let gap = 8i32;
    let panel_x = rect.x() + 8;
    let panel_y = layout.body_y;
    let panel_width = rect.width().saturating_sub(16);
    let panel_height = layout.pane_bottom.saturating_sub(layout.body_y).max(1);
    if let Some(tree) = state.layout.as_ref() {
        return plugin_section_tree_layout(
            state,
            tree,
            PixelRect::new(panel_x, panel_y, panel_width, panel_height as u32),
            cell_width,
            line_height,
            gap,
        );
    }
    let total_gap = gap.saturating_mul(section_count.saturating_sub(1) as i32);
    let titles = (0..section_count)
        .map(|index| state.section_title(index))
        .collect::<Vec<_>>();
    let pane_chrome = titles
        .iter()
        .map(|title| plugin_section_panel_chrome_height(title, line_height))
        .collect::<Vec<_>>();
    let total_height = panel_height
        .max(pane_chrome.iter().sum::<i32>() + total_gap + line_height * section_count as i32);
    let body_width = panel_width.saturating_sub(20);
    let wrap_cols = overlay_text_columns(body_width, 0, cell_width);
    let total_row_budget = ((total_height - pane_chrome.iter().sum::<i32>() - total_gap)
        .max(line_height * section_count as i32)
        / line_height)
        .max(section_count as i32) as usize;
    let min_rows = (0..section_count)
        .map(|index| state.section_min_rows(index))
        .collect::<Vec<_>>();
    let row_budget = plugin_section_row_budget(&min_rows, total_row_budget);
    let used_height = pane_chrome.iter().sum::<i32>()
        + total_gap
        + row_budget.iter().sum::<usize>() as i32 * line_height;
    let extra_height = total_height.saturating_sub(used_height);
    let mut pane_y = panel_y;
    let mut panes = Vec::with_capacity(section_count);
    for (index, rows) in row_budget.into_iter().enumerate() {
        let extra = if index == 0 { extra_height } else { 0 };
        let pane_height = pane_chrome[index] + rows as i32 * line_height + extra;
        panes.push(TextPaneLayout {
            rect: Rect::new(panel_x, pane_y, panel_width, pane_height as u32),
            visible_rows: rows,
            wrap_cols,
        });
        pane_y += pane_height + gap;
    }
    Some(PluginSectionLayout { panes })
}
