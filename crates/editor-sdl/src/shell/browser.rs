use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BrowserPane {
    Input,
    Footer,
}

impl Default for BrowserBufferState {
    fn default() -> Self {
        let mut input = InputField::new("");
        input.set_placeholder(Some("https://example.com".to_owned()));
        Self {
            current_url: None,
            active_pane: BrowserPane::Input,
            input,
            footer_pane: PluginTextPaneState {
                min_rows: Some(1),
                ..PluginTextPaneState::default()
            },
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct BrowserBufferState {
    pub(super) current_url: Option<String>,
    pub(super) active_pane: BrowserPane,
    pub(super) input: InputField,
    pub(super) footer_pane: PluginTextPaneState,
}

pub(super) fn focus_browser_input_section(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if shell_buffer_mut(runtime, buffer_id)?.focus_browser_input() {
        start_change_recording(runtime)?;
        mark_change_finish_on_normal(runtime)?;
        let ui = shell_ui_mut(runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }
    Ok(())
}

pub(super) fn navigate_browser_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    raw_url: &str,
) -> Result<(), String> {
    let url = normalize_browser_url(raw_url);
    let user_library = shell_user_library(runtime);
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    set_browser_buffer_location(buffer, &url, true, &*user_library);
    Ok(())
}

pub(super) fn normalize_browser_url(raw_url: &str) -> String {
    let trimmed = raw_url.trim();
    if trimmed.contains("://")
        || trimmed.starts_with("about:")
        || trimmed.starts_with("file:")
        || trimmed.starts_with("data:")
    {
        return trimmed.to_owned();
    }
    format!("https://{trimmed}")
}

pub(super) fn open_detected_browser_url(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let Some(url) = shell_buffer(runtime, buffer_id)
        .ok()
        .and_then(detect_browser_url)
    else {
        record_runtime_error(
            runtime,
            "browser.url",
            "no URL found at the cursor or on the current line",
        );
        return Ok(());
    };
    open_browser_popup_with_url(runtime, &url)
}

pub(super) fn open_browser_popup_with_url(
    runtime: &mut EditorRuntime,
    raw_url: &str,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(
            workspace_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, "Browser", buffer_id)
        .map_err(|error| error.to_string())?;
    {
        let user_library = shell_user_library(runtime);
        let ui = shell_ui_mut(runtime)?;
        ui.ensure_popup_buffer(
            buffer_id,
            BROWSER_BUFFER_NAME,
            BufferKind::Plugin(BROWSER_KIND.to_owned()),
            &*user_library,
        );
        ui.set_popup_buffer(buffer_id);
    }
    shell_ui_mut(runtime)?.set_popup_focus(true);
    enter_insert_mode_for_input_buffer(runtime, buffer_id)?;
    navigate_browser_buffer(runtime, buffer_id, raw_url)
}

pub(super) fn browser_buffer_display_name(current_url: Option<&str>) -> String {
    match current_url {
        Some(url) => format!("{} {url}", BROWSER_BUFFER_NAME),
        None => BROWSER_BUFFER_NAME.to_owned(),
    }
}

pub(super) fn set_browser_buffer_location(
    buffer: &mut ShellBuffer,
    url: &str,
    clear_input: bool,
    user_library: &dyn UserLibrary,
) {
    let state = buffer
        .browser_state
        .get_or_insert_with(BrowserBufferState::default);
    let changed = state.current_url.as_deref() != Some(url);
    if changed {
        state.current_url = Some(url.to_owned());
        buffer.name = browser_buffer_display_name(Some(url));
        buffer.replace_with_lines(user_library.browser_buffer_lines(Some(url)));
    }
    if let Some(state) = buffer.browser_state.as_mut() {
        if clear_input {
            state.input.clear();
        }
        state
            .footer_pane
            .replace_lines(vec![user_library.browser_input_hint(Some(url))], true);
    }
}

pub(super) fn apply_browser_location_updates(
    runtime: &mut EditorRuntime,
    updates: &[BrowserLocationUpdate],
) -> Result<(), String> {
    let user_library = shell_user_library(runtime);
    let ui = shell_ui_mut(runtime)?;
    for update in updates {
        if let Some(buffer) = ui.buffer_mut(update.buffer_id) {
            set_browser_buffer_location(buffer, &update.current_url, false, &*user_library);
        }
    }
    Ok(())
}

pub(super) fn detect_browser_url(buffer: &ShellBuffer) -> Option<String> {
    let cursor = buffer.cursor_point();
    let line = buffer.text.line(cursor.line)?;
    let candidates = browser_url_candidates(&line);
    if candidates.is_empty() {
        return None;
    }
    let cursor_col = cursor.column.min(line.chars().count());
    candidates
        .iter()
        .find(|(start, end, _)| cursor_col >= *start && cursor_col <= *end)
        .map(|(_, _, url)| url.clone())
        .or_else(|| (candidates.len() == 1).then(|| candidates[0].2.clone()))
}

pub(super) fn browser_url_candidates(line: &str) -> Vec<(usize, usize, String)> {
    let characters = line.chars().collect::<Vec<_>>();
    let mut candidates = Vec::new();
    let mut index = 0usize;
    while index < characters.len() {
        let suffix = characters[index..].iter().collect::<String>();
        let Some(_) = browser_url_prefix_len(&suffix) else {
            index += 1;
            continue;
        };
        let mut end = index;
        while end < characters.len() && !is_browser_url_terminator(characters[end]) {
            end += 1;
        }
        let mut candidate = characters[index..end].iter().collect::<String>();
        while candidate
            .chars()
            .last()
            .is_some_and(is_browser_url_trailing_punctuation)
        {
            candidate.pop();
            end = end.saturating_sub(1);
        }
        if !candidate.is_empty() {
            candidates.push((index, end, candidate));
        }
        index = end.max(index + 1);
    }
    candidates
}

pub(super) fn browser_url_prefix_len(text: &str) -> Option<usize> {
    ["https://", "http://", "file://", "www."]
        .iter()
        .find(|prefix| text.starts_with(**prefix))
        .map(|prefix| prefix.len())
}

pub(super) fn is_browser_url_terminator(character: char) -> bool {
    character.is_whitespace() || matches!(character, '<' | '>' | '"' | '\'')
}

pub(super) fn is_browser_url_trailing_punctuation(character: char) -> bool {
    matches!(character, ')' | ']' | '}' | ',' | '.' | ';' | '!' | '?')
}

pub(super) fn browser_state_for_kind(
    kind: &BufferKind,
    user_library: &dyn UserLibrary,
) -> Option<BrowserBufferState> {
    if !buffer_is_browser(kind) {
        return None;
    }
    let mut state = BrowserBufferState::default();
    state.input.prompt = user_library.browser_url_prompt();
    state
        .input
        .set_placeholder(Some(user_library.browser_url_placeholder()));
    state
        .footer_pane
        .replace_lines(vec![user_library.browser_input_hint(None)], true);
    Some(state)
}

#[expect(
    clippy::too_many_arguments,
    reason = "browser sync needs the full viewport and overlay context"
)]
pub(super) fn browser_sync_plan(
    state: &ShellUiState,
    runtime_popup: Option<&RuntimePopupSnapshot>,
    user_library: &dyn UserLibrary,
    width: u32,
    height: u32,
    cell_width: i32,
    line_height: i32,
    now: Instant,
) -> Result<BrowserSyncPlan, ShellError> {
    let buffers = state
        .buffers
        .iter()
        .filter(|buffer| buffer_is_browser(&buffer.kind))
        .map(|buffer| BrowserBufferPlan {
            buffer_id: buffer.id(),
            current_url: buffer
                .browser_state
                .as_ref()
                .and_then(|browser| browser.current_url.clone()),
        })
        .collect::<Vec<_>>();
    let overlay_occludes_browsers = state.picker_visible()
        || runtime_popup
            .and_then(|popup| state.buffer(popup.active_buffer))
            .is_some_and(|buffer| !buffer_is_browser(&buffer.kind));
    if overlay_occludes_browsers {
        return Ok(BrowserSyncPlan {
            buffers,
            visible_surfaces: Vec::new(),
        });
    }
    let popup_height = runtime_popup
        .map(|_| popup_window_height(height, line_height))
        .unwrap_or(0);
    let pane_height = height.saturating_sub(popup_height);
    let panes = state
        .panes()
        .ok_or_else(|| ShellError::Runtime("active workspace view is missing".to_owned()))?;
    let pane_rects = match state.pane_split_direction() {
        PaneSplitDirection::Vertical => vertical_pane_rects(width, pane_height, panes.len()),
        PaneSplitDirection::Horizontal => horizontal_pane_rects(width, pane_height, panes.len()),
    };
    let notification_rects = notification_overlay_layouts(
        &state.visible_notifications(now),
        width,
        height,
        cell_width,
        line_height,
    )
    .into_iter()
    .map(|layout| layout.rect)
    .collect::<Vec<_>>();
    let mut visible_surfaces = Vec::new();
    for (pane_index, pane) in panes.iter().enumerate() {
        let Some(buffer) = state.buffer(pane.buffer_id) else {
            continue;
        };
        if !buffer_is_browser(&buffer.kind) {
            continue;
        }
        let Some(rect) = browser_viewport_rect(
            buffer,
            PixelRectToRect::rect(
                pane_rects[pane_index].x,
                pane_rects[pane_index].y,
                pane_rects[pane_index].width,
                pane_rects[pane_index].height,
            ),
            cell_width,
            line_height,
            user_library.commandline_enabled(),
        ) else {
            continue;
        };
        if notification_rects
            .iter()
            .any(|overlay| rects_intersect(browser_viewport_rect_rect(rect), *overlay))
        {
            continue;
        }
        visible_surfaces.push(BrowserSurfacePlan {
            buffer_id: buffer.id(),
            rect,
        });
    }
    if let Some(popup) = runtime_popup
        && let Some(buffer) = state.buffer(popup.active_buffer)
        && buffer_is_browser(&buffer.kind)
        && let Some(rect) = browser_viewport_rect(
            buffer,
            PixelRectToRect::rect(0, pane_height as i32, width, popup_height),
            cell_width,
            line_height,
            user_library.commandline_enabled(),
        )
        && !notification_rects
            .iter()
            .any(|overlay| rects_intersect(browser_viewport_rect_rect(rect), *overlay))
    {
        visible_surfaces.push(BrowserSurfacePlan {
            buffer_id: buffer.id(),
            rect,
        });
    }
    Ok(BrowserSyncPlan {
        buffers,
        visible_surfaces,
    })
}

pub(super) fn browser_viewport_rect_rect(rect: BrowserViewportRect) -> Rect {
    PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height)
}

pub(super) fn rects_intersect(left: Rect, right: Rect) -> bool {
    let left_right = left.x().saturating_add(left.width() as i32);
    let left_bottom = left.y().saturating_add(left.height() as i32);
    let right_right = right.x().saturating_add(right.width() as i32);
    let right_bottom = right.y().saturating_add(right.height() as i32);
    left.x() < right_right
        && left_right > right.x()
        && left.y() < right_bottom
        && left_bottom > right.y()
}

#[derive(Debug, Clone, Copy)]
pub(super) struct BrowserBufferLayout {
    pub(super) viewport: Rect,
    pub(super) input: TextPaneLayout,
    pub(super) footer: TextPaneLayout,
}

pub(super) fn browser_viewport_rect(
    buffer: &ShellBuffer,
    rect: Rect,
    cell_width: i32,
    line_height: i32,
    command_line_visible: bool,
) -> Option<BrowserViewportRect> {
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        command_line_visible,
    );
    let viewport = browser_buffer_layout(buffer, rect, layout, cell_width, line_height)?.viewport;
    let x = viewport.x();
    let y = viewport.y();
    let width = viewport.width();
    let height = viewport.height().max(line_height.max(1) as u32) as i32;
    (width > 0 && height > 0).then_some(BrowserViewportRect {
        x,
        y,
        width,
        height: height as u32,
    })
}

pub(super) fn browser_viewport_contains_point(rect: BrowserViewportRect, x: i32, y: i32) -> bool {
    let right = rect.x.saturating_add(rect.width as i32);
    let bottom = rect.y.saturating_add(rect.height as i32);
    x >= rect.x && y >= rect.y && x < right && y < bottom
}

pub(super) fn browser_surface_buffer_at_point(
    plan: &BrowserSyncPlan,
    x: i32,
    y: i32,
) -> Option<BufferId> {
    plan.visible_surfaces.iter().find_map(|surface| {
        browser_viewport_contains_point(surface.rect, x, y).then_some(surface.buffer_id)
    })
}

pub(super) fn browser_buffer_layout(
    buffer: &ShellBuffer,
    rect: Rect,
    layout: BufferFooterLayout,
    cell_width: i32,
    line_height: i32,
) -> Option<BrowserBufferLayout> {
    let state = buffer.browser_state.as_ref()?;
    let line_height = line_height.max(1);
    let panel_x = rect.x() + 8;
    let panel_width = rect.width().saturating_sub(16);
    let gap = 8i32;
    let body_width = panel_width.saturating_sub(20);
    let wrap_cols = overlay_text_columns(body_width, 0, cell_width.max(1));
    let input_rows = if wrap_cols > 0 {
        state.input.visual_line_count(wrap_cols).max(1)
    } else {
        1
    };
    let footer_line_count = state.footer_pane.line_count().max(1);
    let footer_rows = state
        .footer_pane
        .min_rows
        .unwrap_or(footer_line_count)
        .max(footer_line_count);
    let footer_chrome = text_panel_chrome_height("", line_height);
    let input_chrome = input_panel_chrome_height();
    let footer_height = footer_chrome + footer_rows as i32 * line_height;
    let input_height = input_chrome + input_rows as i32 * line_height;
    let footer_y = layout.pane_bottom.saturating_sub(footer_height);
    let input_y = footer_y.saturating_sub(gap + input_height);
    let viewport_y = layout.body_y.saturating_sub(2);
    let viewport_height = input_y.saturating_sub(gap).saturating_sub(viewport_y);
    if panel_width == 0 || viewport_height <= 0 {
        return None;
    }
    Some(BrowserBufferLayout {
        viewport: Rect::new(panel_x, viewport_y, panel_width, viewport_height as u32),
        input: TextPaneLayout {
            rect: Rect::new(panel_x, input_y, panel_width, input_height as u32),
            visible_rows: input_rows,
            wrap_cols,
        },
        footer: TextPaneLayout {
            rect: Rect::new(panel_x, footer_y, panel_width, footer_height as u32),
            visible_rows: footer_rows,
            wrap_cols,
        },
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_browser_buffer_body(
    target: &mut DrawTarget<'_>,
    buffer: &ShellBuffer,
    rect: Rect,
    layout: BufferFooterLayout,
    active: bool,
    input_mode: InputMode,
    theme_registry: Option<&ThemeRegistry>,
    base_background: Color,
    foreground: Color,
    muted: Color,
    border_color: Color,
    selection: Color,
    cursor: Color,
    cursor_roundness: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<(), ShellError> {
    let Some(state) = buffer.browser_state.as_ref() else {
        return Ok(());
    };
    let Some(browser_layout) = browser_buffer_layout(buffer, rect, layout, cell_width, line_height)
    else {
        return Ok(());
    };
    let panel_background = theme_color(
        theme_registry,
        "ui.panel.background",
        adjust_color(
            base_background,
            if is_dark_color(base_background) {
                8
            } else {
                -8
            },
        ),
    );
    let active_border = theme_color(theme_registry, TOKEN_STATUSLINE_ACTIVE, cursor);
    fill_rect(target, browser_layout.viewport, panel_background)?;
    render_input_panel(
        target,
        &state.input,
        active && state.active_pane == BrowserPane::Input,
        browser_layout.input,
        input_mode,
        panel_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        cursor,
        cursor_roundness,
        cell_width,
        line_height,
    )?;
    render_text_panel(
        target,
        &state.footer_pane.text,
        state.footer_pane.scroll_row,
        (active && state.active_pane == BrowserPane::Footer).then_some(state.footer_pane.cursor()),
        active && state.active_pane == BrowserPane::Footer,
        browser_layout.footer,
        "",
        None,
        None,
        InputMode::Normal,
        theme_registry,
        panel_background,
        panel_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        selection,
        cursor,
        cursor_roundness,
        cell_width,
        line_height,
    )?;
    Ok(())
}
