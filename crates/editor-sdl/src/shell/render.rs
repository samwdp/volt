use super::*;

#[derive(Debug, Clone)]
pub(super) struct MarkdownInlineImageDraw {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
    rows: usize,
    alt: String,
}

impl MarkdownInlineImageDraw {
    pub(super) fn rows(&self) -> usize {
        self.rows
    }
}

#[derive(Debug, Default)]
pub(super) struct MarkdownPrettyPaintPlan {
    pub(super) text_overrides: BTreeMap<usize, String>,
    pub(super) images: BTreeMap<usize, MarkdownInlineImageDraw>,
}

fn markdown_inline_image_rows(
    image_width: u32,
    image_height: u32,
    pane_width_px: u32,
    line_height: i32,
    max_rows: usize,
) -> usize {
    let line_height = line_height.max(1) as u32;
    let pane_width_px = pane_width_px.max(1);
    let scaled_height = if image_width == 0 {
        image_height
    } else {
        let scaled = (u64::from(image_height) * u64::from(pane_width_px)) / u64::from(image_width);
        scaled.min(u64::from(u32::MAX)) as u32
    };
    let rows = scaled_height.div_ceil(line_height).max(1) as usize;
    rows.min(max_rows.max(1))
}

pub(super) fn markdown_pretty_paint_plan(
    buffer: &ShellBuffer,
    user_library: &dyn UserLibrary,
    args: MarkdownPrettyPaintArgs,
) -> MarkdownPrettyPaintPlan {
    let MarkdownPrettyPaintArgs {
        visible_start,
        visible_end,
        visual_selection,
        input_mode,
        pane_width_px,
        line_height,
    } = args;
    let mut paint = MarkdownPrettyPaintPlan::default();
    if buffer.language_id() != Some("markdown") {
        return paint;
    }
    let config = markdown_pretty::user_library_pretty_config(user_library);
    let enabled = buffer.markdown_pretty_enabled().unwrap_or(config.enabled);
    let plan = markdown_pretty::cached_plan_for_buffer(buffer, &config, enabled, None);
    if !enabled || plan.skipped_by_kill_switch || plan.lines.is_empty() {
        return paint;
    }
    let cursor_line = buffer.cursor_row();
    let visual_range = if matches!(input_mode, InputMode::Visual) {
        visual_selection.map(|selection| match selection {
            VisualSelection::Range(range) => {
                let start = range.start().line.min(range.end().line);
                let end = range.start().line.max(range.end().line);
                start..end.saturating_add(1)
            }
            VisualSelection::Block(block) => {
                let start = block.start_line.min(block.end_line);
                let end = block.start_line.max(block.end_line);
                start..end.saturating_add(1)
            }
        })
    } else {
        None
    };
    let line_count = buffer.line_count();
    for line_index in visible_start..visible_end.min(line_count) {
        let source = buffer.text.line(line_index).unwrap_or_default();
        let anti = editor_markdown::line_is_anti_concealed(
            Some(cursor_line),
            visual_range.as_ref(),
            line_index,
        );
        if !anti
            && let Some(image) =
                markdown_pretty::line_plan(&plan, line_index).and_then(|line| line.image.as_ref())
        {
            let entry =
                markdown_pretty::ensure_image_loaded(&image.destination, config.image_max_bytes);
            match entry {
                markdown_pretty::MarkdownImageCacheEntry::Ready(decoded) => {
                    let rows = markdown_inline_image_rows(
                        decoded.width,
                        decoded.height,
                        pane_width_px,
                        line_height,
                        config.image_max_rows,
                    );
                    paint.images.insert(
                        line_index,
                        MarkdownInlineImageDraw {
                            width: decoded.width,
                            height: decoded.height,
                            pixels: Arc::clone(&decoded.pixels),
                            rows,
                            alt: image.alt.clone(),
                        },
                    );
                    // Empty display text — image occupies the visual rows.
                    paint.text_overrides.insert(line_index, String::new());
                }
                markdown_pretty::MarkdownImageCacheEntry::Loading => {
                    paint.text_overrides.insert(
                        line_index,
                        format!("{} loading…", editor_icons::symbols::md::MD_IMAGE),
                    );
                }
                markdown_pretty::MarkdownImageCacheEntry::Failed(error) => {
                    paint.text_overrides.insert(
                        line_index,
                        format!("{} {}", editor_icons::symbols::md::MD_IMAGE_BROKEN, error),
                    );
                }
            }
            continue;
        }
        let display = markdown_pretty::pretty_display_line(&plan, anti, line_index, &source);
        if display != source {
            paint.text_overrides.insert(line_index, display);
        }
    }
    paint
}

pub(super) fn render_shell_state(
    target: &mut DrawTarget<'_>,
    fonts: &FontSet<'_>,
    state: &ShellUiState,
    runtime_popup: Option<&RuntimePopupSnapshot>,
    workspace_dock_entries: &[WorkspaceDockEntry],
    chrome: ShellChrome<'_>,
    view: ShellFrameView<'_>,
) -> Result<(), ShellError> {
    let ShellChrome {
        user_library,
        theme_registry,
        ..
    } = chrome;
    let ShellFrameView {
        size: WindowSize { width, height },
        fps_overlay,
        metrics,
        pulse: FramePulse { now, typing_active },
    } = view;
    let TextMetrics {
        cell_width,
        line_height,
        ascent,
    } = metrics;
    let dock = workspace_dock_layout(user_library, state, width, height, cell_width);
    let content_height = height;
    let popup_height = runtime_popup
        .map(|_| popup_window_height(height, line_height))
        .unwrap_or(0);
    let pane_height = content_height.saturating_sub(popup_height);
    let panes = state
        .panes()
        .ok_or_else(|| ShellError::Runtime("active workspace view is missing".to_owned()))?;
    let mut pane_rects = workspace_pane_rects(
        user_library,
        state,
        dock.content_width,
        pane_height,
        panes.len(),
    );
    for rect in &mut pane_rects {
        rect.x = rect.x.saturating_add(dock.content_x);
    }
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let is_dark = is_dark_color(base_background);
    let pane_active_background = base_background;
    let pane_inactive_background = theme_color(
        theme_registry,
        TOKEN_PANE_INACTIVE,
        adjust_color(base_background, if is_dark { -6 } else { 6 }),
    );
    let git_summary = state.git_summary();
    let popup_focus = runtime_popup
        .map(|popup| state.popup_focus_active(popup))
        .unwrap_or(false);
    let dock_focus = state.workspace_dock_focus_active(user_library);
    let command_line_row_visible = state.command_line().is_some() || state.input_prompt_visible();

    clear_window_surface(target, base_background, window_effects);

    render_workspace_dock(
        target,
        &dock,
        workspace_dock_entries,
        theme_registry,
        cell_width,
        line_height,
        ascent,
    )?;

    for (pane_index, pane) in panes.iter().enumerate() {
        let rect = pane_rects[pane_index];
        let active = pane_index == state.active_pane_index()
            && !state.picker_visible()
            && !popup_focus
            && !dock_focus;
        let background = if active {
            pane_active_background
        } else {
            pane_inactive_background
        };
        fill_window_surface_rect(
            target,
            PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
            background,
            window_effects,
        )?;

        if let Some(buffer) = state.buffer(pane.buffer_id) {
            let input_mode = state.input_mode_for_buffer(buffer.id(), active);
            let vim_targets_input =
                state.vim_target_for_buffer(buffer.id(), active) == VimTarget::Input;
            let visual_range = state.visual_selection_for_buffer(buffer, active);
            let multicursor = state.multicursor_for_buffer(buffer.id(), active).cloned();
            let yank_flash = state.yank_flash(buffer.id(), now);
            // Prefer the Vim command line; fall back to a generic input prompt
            // (e.g. workspace.compile) which shares the same footer row.
            let command_line_input = active
                .then(|| {
                    state
                        .command_line()
                        .map(CommandLineOverlay::input)
                        .or_else(|| state.input_prompt().map(InputPromptOverlay::input))
                })
                .flatten();
            let view_state = if active {
                buffer.view_state()
            } else {
                state
                    .buffer_view_state(pane.pane_id, buffer.id())
                    .unwrap_or_else(|| buffer.view_state())
            };
            render_buffer(
                target,
                BufferDrawRequest {
                    buffer,
                    view_state,
                    pane: PaneSlot {
                        rect: PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
                        active,
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
                        input: command_line_input,
                        row_visible: command_line_row_visible,
                    },
                },
                BufferChrome::from_shell(&chrome, git_summary.as_ref()),
                metrics,
            )?;
        }
    }

    if let Some(popup) = runtime_popup {
        render_runtime_popup_overlay(
            target,
            state,
            popup,
            PixelRectToRect::rect(
                dock.content_x,
                pane_height as i32,
                dock.content_width,
                popup_height,
            ),
            chrome,
            metrics,
            FramePulse { now, typing_active },
        )?;
    }

    if let Some(pane) = panes.get(state.active_pane_index())
        && let Some(buffer) = state.buffer(pane.buffer_id)
        && let Some((matches, selected_index)) = state
            .command_line()
            .and_then(CommandLineOverlay::completion_list)
        && let Some(active_rect) = pane_rects.get(state.active_pane_index())
    {
        let layout = buffer_footer_layout_with_command_line(
            buffer,
            PixelRectToRect::rect(
                active_rect.x,
                active_rect.y,
                active_rect.width,
                active_rect.height,
            ),
            line_height,
            cell_width,
            true,
        );
        if let Some(commandline_y) = layout.commandline_y {
            render_command_line_completion_popup(
                target,
                matches,
                selected_index,
                PixelRectToRect::rect(
                    active_rect.x,
                    active_rect.y,
                    active_rect.width,
                    active_rect.height,
                ),
                commandline_y,
                theme_registry,
                metrics.cells(),
            )?;
        }
    }

    if let Some(autocomplete) = state
        .autocomplete()
        .filter(|autocomplete| autocomplete.is_visible())
        && let Some(active_rect) = pane_rects.get(state.active_pane_index())
    {
        render_autocomplete_overlay(
            target,
            state,
            autocomplete,
            OverlayAnchorContext {
                pane_rect: PixelRectToRect::rect(
                    active_rect.x,
                    active_rect.y,
                    active_rect.width,
                    active_rect.height,
                ),
                user_library,
                theme_registry,
                metrics: metrics.cells(),
                typing_active,
            },
        )?;
    }

    if let Some(hover) = state.hover()
        && let Some(active_rect) = pane_rects.get(state.active_pane_index())
    {
        render_hover_overlay(
            target,
            state,
            hover,
            OverlayAnchorContext {
                pane_rect: PixelRectToRect::rect(
                    active_rect.x,
                    active_rect.y,
                    active_rect.width,
                    active_rect.height,
                ),
                user_library,
                theme_registry,
                metrics: metrics.cells(),
                typing_active,
            },
        )?;
    }

    if let Some(picker) = state.picker() {
        picker::render_picker_overlay(
            target,
            fonts,
            PickerOverlayDraw {
                picker,
                size: WindowSize { width, height },
                line_height,
                theme_registry,
                picker_layout: user_library.picker_layout(),
                truncate_strategy: user_library.picker_truncate_strategy(),
            },
        )?;
    }

    render_notification_overlay(
        target,
        state,
        WindowSize { width, height },
        theme_registry,
        metrics.cells(),
        now,
    )?;
    render_fps_overlay(
        target,
        width,
        theme_registry,
        fps_overlay,
        cell_width,
        line_height,
    )?;

    Ok(())
}

fn frame_time_to_fps(frame_time: Duration) -> f64 {
    if frame_time.is_zero() {
        0.0
    } else {
        1.0 / frame_time.as_secs_f64()
    }
}

fn format_fps_overlay_text(snapshot: &FpsOverlaySnapshot) -> String {
    format!(
        "FPS {:>5.1}  frame {:>5.2}ms  max {:>5.2}ms",
        frame_time_to_fps(snapshot.average_frame_time),
        snapshot.latest_frame_time.as_secs_f64() * 1_000.0,
        snapshot.worst_frame_time.as_secs_f64() * 1_000.0,
    )
}

fn render_fps_overlay(
    target: &mut DrawTarget<'_>,
    width: u32,
    theme_registry: Option<&ThemeRegistry>,
    fps_overlay: Option<&FpsOverlaySnapshot>,
    cell_width: i32,
    line_height: i32,
) -> Result<(), ShellError> {
    let Some(snapshot) = fps_overlay else {
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
    let background = theme_color(
        theme_registry,
        "ui.fps.background",
        adjust_color(base_background, if is_dark { 18 } else { -18 }),
    );
    let border = theme_color(
        theme_registry,
        "ui.fps.border",
        adjust_color(base_background, if is_dark { 30 } else { -30 }),
    );
    let accent = if snapshot.average_frame_time <= FRAME_PACING_TARGET_120FPS {
        theme_color(
            theme_registry,
            "ui.fps.good",
            Color::RGBA(95, 196, 122, 255),
        )
    } else if snapshot.average_frame_time <= FRAME_PACING_TARGET_120FPS + Duration::from_millis(4) {
        theme_color(
            theme_registry,
            "ui.fps.warn",
            Color::RGBA(236, 191, 74, 255),
        )
    } else {
        theme_color(theme_registry, "ui.fps.bad", Color::RGBA(224, 97, 97, 255))
    };
    let text = format_fps_overlay_text(snapshot);
    let panel_width = monospace_text_width(&text, cell_width) + 24;
    let panel_height = line_height.max(1) as u32 + 16;
    let rect = PixelRectToRect::rect(
        width as i32 - panel_width as i32 - 12,
        12,
        panel_width,
        panel_height,
    );
    let radius = overlay_radius(theme_registry);
    paint_overlay_card(
        target,
        rect,
        OverlayCardStyle {
            radius,
            border,
            background,
            window_effects,
            accent: Some(accent),
            shadow: false,
        },
    )?;
    draw_text(target, rect.x() + 14, rect.y() + 8, &text, base_foreground)?;
    Ok(())
}

pub(super) fn shared_corner_radius(theme_registry: Option<&ThemeRegistry>) -> u32 {
    theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_CORNER_RADIUS))
        .map(|value| value.clamp(0.0, 64.0).round() as u32)
        .unwrap_or(16)
}

pub(super) fn overlay_radius(theme_registry: Option<&ThemeRegistry>) -> u32 {
    shared_corner_radius(theme_registry)
}

pub(super) fn picker_card_rect(
    window_width: u32,
    window_height: u32,
    layout: editor_plugin_api::PickerLayout,
) -> PixelRect {
    let width_fraction = layout.width_fraction.clamp(0.15, 1.0);
    let height_fraction = layout.height_fraction.clamp(0.15, 1.0);
    let width = ((window_width as f32) * width_fraction).round() as u32;
    let height = ((window_height as f32) * height_fraction).round() as u32;
    centered_rect(window_width, window_height, width.max(1), height.max(1))
}

fn paint_overlay_shadow(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_overlay_surface_rounded_rect(
        target,
        PixelRectToRect::rect(
            rect.x() + OVERLAY_SHADOW_OFFSET,
            rect.y() + OVERLAY_SHADOW_OFFSET,
            rect.width(),
            rect.height(),
        ),
        radius,
        Color::RGBA(0, 0, 0, 72),
        window_effects,
    )
}

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
    if shadow {
        paint_overlay_shadow(target, rect, radius, window_effects)?;
    }
    fill_overlay_surface_rounded_rect(target, rect, radius, border, window_effects)?;
    let inner = PixelRectToRect::rect(
        rect.x() + 1,
        rect.y() + 1,
        rect.width().saturating_sub(2),
        rect.height().saturating_sub(2),
    );
    let inner_radius = radius.saturating_sub(1);
    match accent {
        Some(accent) => fill_rounded_rect_with_left_accent(
            target,
            inner,
            inner_radius,
            background,
            accent,
            window_effects,
        )?,
        None => fill_overlay_surface_rounded_rect(
            target,
            inner,
            inner_radius,
            background,
            window_effects,
        )?,
    }
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
    fill_overlay_surface_rounded_rect(target, rect, radius, accent, window_effects)?;
    let body_width = rect.width().saturating_sub(OVERLAY_ACCENT_BAR_WIDTH);
    if body_width == 0 {
        return Ok(());
    }
    fill_overlay_surface_rounded_rect(
        target,
        PixelRectToRect::rect(
            rect.x() + OVERLAY_ACCENT_BAR_WIDTH as i32,
            rect.y(),
            body_width,
            rect.height(),
        ),
        radius,
        fill,
        window_effects,
    )
}

fn fill_overlay_right_band(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    let radius = radius.min(rect.width() / 2).min(rect.height() / 2);
    fill_overlay_surface_rounded_rect(target, rect, radius, color, window_effects)?;
    if rect.width() > radius {
        fill_overlay_surface_rect(
            target,
            PixelRectToRect::rect(rect.x(), rect.y(), radius, rect.height()),
            color,
            window_effects,
        )?;
    }
    Ok(())
}

fn fill_overlay_top_header_band(
    target: &mut DrawTarget<'_>,
    header_rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    let radius = radius
        .min(header_rect.width() / 2)
        .min(header_rect.height() / 2);
    fill_overlay_surface_rounded_rect(target, header_rect, radius, color, window_effects)?;
    if header_rect.height() > radius {
        fill_overlay_surface_rect(
            target,
            PixelRectToRect::rect(
                header_rect.x(),
                header_rect.y() + radius as i32,
                header_rect.width(),
                header_rect.height().saturating_sub(radius),
            ),
            color,
            window_effects,
        )?;
    }
    Ok(())
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
    let width = overlay_width(pane_rect.width(), cell_width, 44, 74);
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

pub(super) fn overlay_text_columns(width: u32, horizontal_padding: u32, cell_width: i32) -> usize {
    (width.saturating_sub(horizontal_padding) / cell_width.max(1) as u32)
        .max(1)
        .try_into()
        .unwrap_or(1)
}

const BUFFER_BODY_TOP_PADDING: i32 = 10;
const BUFFER_BODY_BOTTOM_PADDING: i32 = 10;
const BUFFER_STATUSLINE_BOTTOM_PADDING: i32 = 8;
const BUFFER_STATUSLINE_COMMANDLINE_GAP: i32 = 8;
const BUFFER_FOOTER_SEPARATOR_OFFSET: i32 = 6;
const BUFFER_INPUT_BOX_EXTRA_HEIGHT: i32 = 8;
const BUFFER_INPUT_HINT_GAP: i32 = 4;
const BUFFER_INPUT_FOOTER_GAP: i32 = 10;
const BUFFER_OVERLAY_BOTTOM_GAP: i32 = 8;
const INPUT_PANEL_VERTICAL_PADDING: i32 = 10;
const MAX_VISIBLE_INPUT_TEXT_ROWS: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BufferFooterLayout {
    pub(super) body_y: i32,
    pub(super) statusline_y: i32,
    pub(super) commandline_y: Option<i32>,
    pub(super) input_y: i32,
    pub(super) input_box_height: i32,
    pub(super) input_hint_gap: i32,
    pub(super) visible_rows: usize,
    pub(super) pane_bottom: i32,
}

#[cfg(test)]
pub(super) fn buffer_footer_layout(
    buffer: &ShellBuffer,
    rect: Rect,
    line_height: i32,
    cell_width: i32,
) -> BufferFooterLayout {
    buffer_footer_layout_with_command_line(buffer, rect, line_height, cell_width, false)
}

pub(super) fn buffer_footer_layout_with_command_line(
    buffer: &ShellBuffer,
    rect: Rect,
    line_height: i32,
    cell_width: i32,
    command_line_visible: bool,
) -> BufferFooterLayout {
    let line_height = line_height.max(1);
    let body_y = rect.y() + BUFFER_BODY_TOP_PADDING;
    let command_line_reserved = if command_line_visible {
        line_height + BUFFER_STATUSLINE_COMMANDLINE_GAP
    } else {
        0
    };
    let statusline_y = rect.y() + rect.height() as i32
        - line_height
        - command_line_reserved
        - BUFFER_STATUSLINE_BOTTOM_PADDING;
    let commandline_y = command_line_visible
        .then_some(statusline_y + line_height + BUFFER_STATUSLINE_COMMANDLINE_GAP);
    let available_input_cols = if cell_width > 0 {
        ((rect.width() as i32 - 16) / cell_width).max(1) as usize
    } else {
        0
    };
    let (input_text_lines, has_hint) = buffer
        .standalone_input_field()
        .map(|input| {
            let line_count = if available_input_cols > 0 {
                input.visual_line_count(available_input_cols)
            } else {
                input.text_line_count()
            };
            (line_count, input.hint().is_some())
        })
        .unwrap_or((0, false));
    let input_hint_gap = if has_hint { BUFFER_INPUT_HINT_GAP } else { 0 };
    let input_footer_gap = if has_hint { BUFFER_INPUT_FOOTER_GAP } else { 0 };
    let fixed_input_reserved = if input_text_lines > 0 {
        input_hint_gap + i32::from(has_hint) * line_height + input_footer_gap
    } else {
        0
    };
    let min_input_y = body_y + BUFFER_BODY_BOTTOM_PADDING + line_height;
    let max_input_reserved = statusline_y.saturating_sub(min_input_y).max(0);
    let capped_input_text_lines = input_text_lines.min(MAX_VISIBLE_INPUT_TEXT_ROWS);
    let desired_input_box_height = if capped_input_text_lines > 0 {
        (line_height * capped_input_text_lines as i32 + BUFFER_INPUT_BOX_EXTRA_HEIGHT)
            .max(line_height)
    } else {
        0
    };
    let max_input_box_height = max_input_reserved.saturating_sub(fixed_input_reserved);
    let input_box_height = if input_text_lines > 0 && max_input_box_height > 0 {
        desired_input_box_height.clamp(line_height.min(max_input_box_height), max_input_box_height)
    } else {
        0
    };
    let input_reserved = if input_text_lines > 0 {
        input_box_height + fixed_input_reserved
    } else {
        0
    };
    let input_y = statusline_y - input_reserved;
    let visible_body_height = (input_y - body_y - BUFFER_BODY_BOTTOM_PADDING).max(line_height);
    let visible_rows = (visible_body_height / line_height).max(1) as usize;
    BufferFooterLayout {
        body_y,
        statusline_y,
        commandline_y,
        input_y,
        input_box_height,
        input_hint_gap,
        visible_rows,
        pane_bottom: input_y - BUFFER_OVERLAY_BOTTOM_GAP,
    }
}

fn visible_input_text_rows(input_box_height: i32, line_height: i32) -> usize {
    if input_box_height <= 0 {
        return 0;
    }
    let rows = input_box_height
        .saturating_sub(BUFFER_INPUT_BOX_EXTRA_HEIGHT)
        .max(line_height)
        .saturating_div(line_height.max(1)) as usize;
    rows.min(MAX_VISIBLE_INPUT_TEXT_ROWS)
}

pub(super) fn buffer_visible_rows_for_height(
    buffer: &ShellBuffer,
    height: u32,
    line_height: i32,
    command_line_visible: bool,
) -> usize {
    buffer_footer_layout_with_command_line(
        buffer,
        PixelRectToRect::rect(0, 0, 1, height),
        line_height,
        0,
        command_line_visible,
    )
    .visible_rows
}

fn render_footer_separator(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    y: i32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_window_surface_rect(
        target,
        PixelRectToRect::rect(rect.x() + 8, y, rect.width().saturating_sub(16), 1),
        color,
        window_effects,
    )
}

fn buffer_visible_headerline_row_count(
    buffer: &ShellBuffer,
    user_library: &dyn UserLibrary,
    visible_rows: usize,
    typing_active: bool,
) -> usize {
    buffer_context_overlay_snapshot(buffer, true, typing_active, user_library)
        .map(|snapshot| visible_headerline_row_count(&snapshot.headerline_lines, visible_rows))
        .unwrap_or(0)
}

pub(super) fn image_buffer_viewport_rect(rect: Rect, layout: BufferFooterLayout) -> Option<Rect> {
    let x = rect.x().saturating_add(8);
    let y = layout.body_y;
    let width = rect.width().saturating_sub(16);
    let height = layout.pane_bottom.saturating_sub(y);
    (width > 0 && height > 0).then(|| Rect::new(x, y, width, height as u32))
}

pub(super) fn centered_image_draw_rect(
    viewport: Rect,
    image_width: u32,
    image_height: u32,
    zoom: f32,
) -> Option<Rect> {
    if image_width == 0 || image_height == 0 || viewport.width() == 0 || viewport.height() == 0 {
        return None;
    }
    let fit_scale = (viewport.width() as f32 / image_width as f32)
        .min(viewport.height() as f32 / image_height as f32);
    let scale = (fit_scale * zoom).max(0.000_1);
    let draw_width = ((image_width as f32 * scale).round() as u32).max(1);
    let draw_height = ((image_height as f32 * scale).round() as u32).max(1);
    let x = viewport.x() + (viewport.width() as i32 - draw_width as i32) / 2;
    let y = viewport.y() + (viewport.height() as i32 - draw_height as i32) / 2;
    Some(Rect::new(x, y, draw_width, draw_height))
}

pub(super) fn render_image_buffer_body(
    target: &mut DrawTarget<'_>,
    buffer: &ShellBuffer,
    rect: Rect,
    layout: BufferFooterLayout,
    theme_registry: Option<&ThemeRegistry>,
    base_background: Color,
) -> Result<(), ShellError> {
    let window_effects = current_window_effect_settings(theme_registry);
    let Some(state) = buffer.image_state() else {
        return Ok(());
    };
    if state.mode != ImageBufferMode::Rendered {
        return Ok(());
    }
    let Some(viewport) = image_buffer_viewport_rect(rect, layout) else {
        return Ok(());
    };
    let viewport_background = theme_color(
        theme_registry,
        "ui.panel.background",
        adjust_color(
            base_background,
            if is_dark_color(base_background) {
                4
            } else {
                -4
            },
        ),
    );
    fill_window_surface_rect(target, viewport, viewport_background, window_effects)?;
    let Some(draw_rect) = centered_image_draw_rect(
        viewport,
        state.decoded.width,
        state.decoded.height,
        state.zoom,
    ) else {
        return Ok(());
    };
    draw_image(
        target,
        draw_rect,
        state.decoded.width,
        state.decoded.height,
        Arc::clone(&state.decoded.pixels),
        Some(viewport),
    )?;
    Ok(())
}

pub(super) fn render_pdf_buffer_body(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    layout: BufferFooterLayout,
    theme_registry: Option<&ThemeRegistry>,
    base_background: Color,
) -> Result<(), ShellError> {
    let window_effects = current_window_effect_settings(theme_registry);
    let Some(viewport) = image_buffer_viewport_rect(rect, layout) else {
        return Ok(());
    };
    let viewport_background = theme_color(
        theme_registry,
        "ui.panel.background",
        adjust_color(
            base_background,
            if is_dark_color(base_background) {
                4
            } else {
                -4
            },
        ),
    );
    fill_window_surface_rect(target, viewport, viewport_background, window_effects)
}

pub(super) fn autocomplete_preview_lines(
    entry: Option<&AutocompleteEntry>,
    token: &str,
    max_columns: usize,
    max_lines: usize,
    token_icon: &str,
) -> Vec<String> {
    let max_lines = max_lines.max(1);
    let Some(entry) = entry else {
        return wrap_overlay_text(
            &format!("{token_icon} {token}\n\nSelect a completion to preview details."),
            max_columns,
            max_lines,
        );
    };
    let mut lines = Vec::new();
    lines.extend(wrap_overlay_text(
        &format!("{} {}", entry.item_icon, entry.label),
        max_columns,
        max_lines,
    ));
    if lines.len() < max_lines {
        let meta = entry
            .detail
            .as_deref()
            .filter(|detail| !detail.is_empty())
            .map(|detail| {
                format!(
                    "{} {} · {detail}",
                    entry.provider_icon, entry.provider_label
                )
            })
            .unwrap_or_else(|| format!("{} {}", entry.provider_icon, entry.provider_label));
        lines.extend(wrap_overlay_text(
            &meta,
            max_columns,
            max_lines - lines.len(),
        ));
    }
    if lines.len() < max_lines {
        lines.push(String::new());
    }
    if lines.len() < max_lines {
        let body = entry
            .documentation
            .as_deref()
            .filter(|documentation| !documentation.trim().is_empty())
            .unwrap_or("No documentation available for this completion.");
        lines.extend(wrap_overlay_text(
            body,
            max_columns,
            max_lines - lines.len(),
        ));
    }
    lines.truncate(max_lines);
    lines
}

pub(super) fn wrap_overlay_lines(
    lines: &[String],
    max_columns: usize,
    max_lines: usize,
) -> Vec<String> {
    if max_lines == 0 {
        return Vec::new();
    }
    let mut wrapped = Vec::new();
    for line in lines {
        if wrapped.len() >= max_lines {
            break;
        }
        wrapped.extend(wrap_overlay_text(
            line,
            max_columns,
            max_lines.saturating_sub(wrapped.len()),
        ));
    }
    wrapped.truncate(max_lines);
    wrapped
}

pub(super) fn wrap_overlay_text(text: &str, max_columns: usize, max_lines: usize) -> Vec<String> {
    if max_lines == 0 {
        return Vec::new();
    }
    let max_columns = max_columns.max(1);
    let mut wrapped = Vec::new();
    for raw_line in text.lines() {
        if wrapped.len() >= max_lines {
            break;
        }
        if raw_line.is_empty() {
            wrapped.push(String::new());
            continue;
        }
        let mut remaining = raw_line;
        while !remaining.is_empty() && wrapped.len() < max_lines {
            if remaining.chars().count() <= max_columns {
                wrapped.push(remaining.to_owned());
                break;
            }
            let mut split_at = 0usize;
            let mut last_whitespace = None;
            let mut columns = 0usize;
            for (byte_index, character) in remaining.char_indices() {
                columns += 1;
                if character.is_whitespace() {
                    last_whitespace = Some(byte_index);
                }
                split_at = byte_index + character.len_utf8();
                if columns >= max_columns {
                    break;
                }
            }
            let split_at = last_whitespace
                .filter(|index| *index > 0)
                .unwrap_or(split_at);
            let (head, tail) = remaining.split_at(split_at);
            wrapped.push(head.trim_end().to_owned());
            remaining = tail.trim_start();
        }
    }
    if wrapped.is_empty() {
        wrapped.push(String::new());
    }
    wrapped.truncate(max_lines);
    wrapped
}

pub(super) fn statusline_icon_segments<'a>(
    text: &'a str,
    icons: &[&'a str],
) -> Vec<(&'a str, bool)> {
    if text.is_empty() {
        return Vec::new();
    }
    let mut remaining = text;
    let mut segments = Vec::new();
    while !remaining.is_empty() {
        let next_icon = icons
            .iter()
            .filter_map(|icon| remaining.find(icon).map(|index| (index, *icon)))
            .min_by_key(|(index, _)| *index);
        let Some((index, icon)) = next_icon else {
            segments.push((remaining, false));
            break;
        };
        if index > 0 {
            let (before, after) = remaining.split_at(index);
            segments.push((before, false));
            remaining = after;
            continue;
        }
        let after = &remaining[icon.len()..];
        segments.push((icon, true));
        remaining = after;
    }
    segments
}

pub(super) fn statusline_icon_colors(
    statusline: &str,
    user_library: &dyn UserLibrary,
    acp_connected: bool,
    lsp_server_visible: bool,
    lsp_workspace_loaded: bool,
    theme_registry: Option<&ThemeRegistry>,
    connected_color: Color,
) -> Vec<(&'static str, Color)> {
    let acp_icon = editor_icons::symbols::fa::FA_CONNECTDEVELOP;
    let lsp_icon = user_library.statusline_lsp_connected_icon();
    let error_icon = user_library.statusline_lsp_error_icon();
    let warning_icon = user_library.statusline_lsp_warning_icon();
    let mut icon_colors = Vec::new();
    if acp_connected && statusline.contains(acp_icon) {
        icon_colors.push((acp_icon, connected_color));
    }
    if lsp_server_visible && lsp_workspace_loaded && statusline.contains(lsp_icon) {
        icon_colors.push((lsp_icon, connected_color));
    }
    if statusline.contains(error_icon) {
        icon_colors.push((
            error_icon,
            diagnostic_color(LspDiagnosticSeverity::Error, theme_registry),
        ));
    }
    if statusline.contains(warning_icon) {
        icon_colors.push((
            warning_icon,
            diagnostic_color(LspDiagnosticSeverity::Warning, theme_registry),
        ));
    }
    icon_colors
}

fn draw_modeline(
    target: &mut DrawTarget<'_>,
    segments: &[ModelineSegment],
    draw: ModelineDraw<'_>,
) -> Result<(), ShellError> {
    let ModelineDraw {
        x,
        y,
        max_width,
        default_color,
        apply_tokens,
        theme_registry,
        user_library,
        acp_connected,
        lsp_server_visible,
        lsp_workspace_loaded,
        connected_color,
        cell_width,
        line_height,
    } = draw;
    let cell_width = cell_width.max(1);
    let gap_width = monospace_text_width(" ", cell_width);
    let chip_height = line_height.unwrap_or(cell_width).max(1) as u32;

    let left: Vec<&ModelineSegment> = segments
        .iter()
        .filter(|segment| segment.alignment == ModelineAlignment::Left)
        .collect();
    let right: Vec<&ModelineSegment> = segments
        .iter()
        .filter(|segment| segment.alignment == ModelineAlignment::Right)
        .collect();

    let right_width = modeline_side_width(&right, gap_width, cell_width);
    let left_budget = max_width.saturating_sub(right_width.saturating_add(gap_width));

    let joined = flatten_modeline_text(segments);
    let icon_colors = statusline_icon_colors(
        &joined,
        user_library,
        acp_connected,
        lsp_server_visible,
        lsp_workspace_loaded,
        theme_registry,
        connected_color,
    );
    let highlighted_icons = icon_colors
        .iter()
        .map(|(icon, _)| *icon)
        .collect::<Vec<_>>();

    draw_modeline_side(
        target,
        &left,
        ModelineSideDraw {
            x,
            y,
            max_width: left_budget,
            default_color,
            apply_tokens,
            theme_registry,
            icon_colors: &icon_colors,
            highlighted_icons: &highlighted_icons,
            cell_width,
            gap_width,
            chip_height,
            preserve_end: false,
        },
    )?;

    if right_width > 0 && right_width <= max_width {
        let right_x = x + max_width.saturating_sub(right_width) as i32;
        draw_modeline_side(
            target,
            &right,
            ModelineSideDraw {
                x: right_x,
                y,
                max_width: right_width,
                default_color,
                apply_tokens,
                theme_registry,
                icon_colors: &icon_colors,
                highlighted_icons: &highlighted_icons,
                cell_width,
                gap_width,
                chip_height,
                preserve_end: true,
            },
        )?;
    }
    Ok(())
}

fn modeline_side_width(segments: &[&ModelineSegment], gap_width: u32, cell_width: i32) -> u32 {
    let mut width = 0u32;
    for (index, segment) in segments.iter().enumerate() {
        if index > 0 {
            width = width.saturating_add(gap_width);
        }
        for (part_index, part) in segment.parts.iter().enumerate() {
            if part_index > 0 {
                width = width.saturating_add(gap_width);
            }
            width = width.saturating_add(monospace_text_width(&part.text, cell_width));
        }
    }
    width
}

fn draw_modeline_side(
    target: &mut DrawTarget<'_>,
    segments: &[&ModelineSegment],
    draw: ModelineSideDraw<'_>,
) -> Result<(), ShellError> {
    let ModelineSideDraw {
        x,
        y,
        max_width,
        default_color,
        apply_tokens,
        theme_registry,
        icon_colors,
        highlighted_icons,
        cell_width,
        gap_width,
        chip_height,
        preserve_end,
    } = draw;
    let mut draw_x = x;
    let mut remaining_width = max_width;
    for (index, segment) in segments.iter().enumerate() {
        if remaining_width == 0 {
            break;
        }
        if index > 0 {
            if remaining_width < gap_width {
                break;
            }
            draw_x += gap_width as i32;
            remaining_width = remaining_width.saturating_sub(gap_width);
        }
        for (part_index, part) in segment.parts.iter().enumerate() {
            if remaining_width == 0 {
                break;
            }
            if part_index > 0 {
                if remaining_width < gap_width {
                    break;
                }
                draw_x += gap_width as i32;
                remaining_width = remaining_width.saturating_sub(gap_width);
            }
            let text = if preserve_end {
                truncate_text_to_width_preserving_end(&part.text, remaining_width, cell_width)
            } else {
                truncate_text_to_width(&part.text, remaining_width, cell_width)
            };
            if text.is_empty() {
                break;
            }
            let painted_width = monospace_text_width(&text, cell_width);
            if let Some(background) = part.background.as_deref().filter(|token| !token.is_empty()) {
                let bg = theme_color(theme_registry, background, default_color);
                fill_rounded_rect(
                    target,
                    PixelRectToRect::rect(draw_x, y, painted_width, chip_height),
                    chip_height / 4,
                    bg,
                )?;
            }
            let token_color = (!part.foreground.is_empty() && apply_tokens)
                .then(|| theme_color(theme_registry, &part.foreground, default_color));
            for (segment_text, highlighted) in statusline_icon_segments(&text, highlighted_icons) {
                if remaining_width == 0 {
                    break;
                }
                let piece = if preserve_end {
                    truncate_text_to_width_preserving_end(segment_text, remaining_width, cell_width)
                } else {
                    truncate_text_to_width(segment_text, remaining_width, cell_width)
                };
                if piece.is_empty() {
                    break;
                }
                let color = if highlighted {
                    icon_colors
                        .iter()
                        .find_map(|(icon, color)| (*icon == segment_text).then_some(*color))
                        .unwrap_or(default_color)
                } else {
                    token_color.unwrap_or(default_color)
                };
                draw_text(target, draw_x, y, &piece, color)?;
                let piece_width = monospace_text_width(&piece, cell_width);
                draw_x += piece_width as i32;
                remaining_width = remaining_width.saturating_sub(piece_width);
            }
        }
    }
    Ok(())
}

pub(super) fn buffer_cursor_screen_anchor(
    buffer: &ShellBuffer,
    rect: Rect,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
    cell_width: i32,
    line_height: i32,
    typing_active: bool,
) -> Option<CursorScreenAnchor> {
    let cell_width = cell_width.max(1);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let headerline_rows = buffer_visible_headerline_row_count(
        buffer,
        user_library,
        layout.visible_rows,
        typing_active,
    );
    let body_y = layout.body_y + headerline_rows as i32 * line_height;
    let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
    let fringe_width = editor_fringe_width_px(cell_width, buffer.dap_fringe_live());
    let line_number_width = cell_width * 5;
    let wrap_cols = wrap_columns_for_width_with_fringe(
        rect.width(),
        cell_width,
        debug_fringe_cell_count(buffer.dap_fringe_live()),
    );
    let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
    let cursor_row = buffer.cursor_row();
    let cursor_col = buffer.cursor_col();
    let wrapped_lines = collect_wrapped_lines(
        buffer,
        WrapCollect {
            start_line: buffer.scroll_row,
            max_rows: visible_rows,
            wrap_cols,
            indent_size,
            scroll_col: buffer.scroll_col,
            line_wrap: buffer.line_wrap(),
        },
    );
    let mut cursor_row_on_screen = None;
    let mut cursor_col_on_screen = None;
    let mut cursor_indent_cols = 0usize;
    let mut visual_row = 0usize;
    for wrapped in &wrapped_lines {
        if wrapped.line_index == cursor_row {
            let display_cursor_col = wrapped.char_map.cursor_anchor_col(cursor_col);
            let segment_index = segment_index_for_column(&wrapped.segments, display_cursor_col);
            if let Some(segment) = wrapped.segments.get(segment_index) {
                cursor_row_on_screen = Some(visual_row + segment_index);
                cursor_col_on_screen = Some(
                    wrapped
                        .char_map
                        .display_cols_between(segment.start_col, display_cursor_col),
                );
                cursor_indent_cols = if segment_index == 0 {
                    0
                } else {
                    wrapped.continuation_indent_cols
                };
            }
        }
        visual_row = visual_row.saturating_add(wrapped.visual_row_count());
        if visual_row >= visible_rows {
            break;
        }
    }
    let cursor_row_on_screen = cursor_row_on_screen?;
    let cursor_col_on_screen = cursor_col_on_screen?;
    Some(CursorScreenAnchor {
        x: rect.x()
            + 12
            + fringe_width
            + line_number_width
            + ((cursor_indent_cols + cursor_col_on_screen) as i32 * cell_width),
        y: body_y + cursor_row_on_screen as i32 * line_height,
        pane_bottom: layout.pane_bottom,
    })
}

#[cfg(test)]
pub(super) fn pretty_cursor_body_row(
    buffer: &ShellBuffer,
    rect: Rect,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
    cell_width: i32,
    line_height: i32,
) -> Option<usize> {
    let cell_width = cell_width.max(1);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let headerline_rows =
        buffer_visible_headerline_row_count(buffer, user_library, layout.visible_rows, false);
    let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
    let wrap_cols = wrap_columns_for_width(rect.width(), cell_width);
    let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
    let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
    let pretty_paint = markdown_pretty_paint_plan(
        buffer,
        user_library,
        MarkdownPrettyPaintArgs {
            visible_start: buffer.scroll_row,
            visible_end: buffer
                .scroll_row
                .saturating_add(visible_rows.saturating_add(8)),
            visual_selection: None,
            input_mode: InputMode::Normal,
            pane_width_px: text_width_px,
            line_height,
        },
    );
    let wrapped_lines = collect_wrapped_lines_with_display(
        buffer,
        WrapCollect {
            start_line: buffer.scroll_row,
            max_rows: visible_rows,
            wrap_cols,
            indent_size,
            scroll_col: buffer.scroll_col,
            line_wrap: buffer.line_wrap(),
        },
        Some(&pretty_paint.text_overrides),
        Some(&pretty_paint.images),
    );
    let cursor_row = buffer.cursor_row();
    let mut visual_row = 0usize;
    for wrapped in &wrapped_lines {
        if wrapped.line_index == cursor_row {
            return Some(visual_row);
        }
        visual_row = visual_row.saturating_add(wrapped.visual_row_count());
        if visual_row >= visible_rows {
            break;
        }
    }
    None
}

pub(super) fn buffer_point_at_screen(
    buffer: &ShellBuffer,
    rect: Rect,
    user_library: &dyn UserLibrary,
    theme_registry: Option<&ThemeRegistry>,
    hit: ScreenHit,
    metrics: CellMetrics,
) -> Option<TextPoint> {
    let ScreenHit {
        x,
        y,
        clamp_body,
        typing_active,
    } = hit;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let line_height = line_height.max(1);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        user_library.commandline_enabled(),
    );
    let headerline_rows = buffer_visible_headerline_row_count(
        buffer,
        user_library,
        layout.visible_rows,
        typing_active,
    );
    let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
    let body_top = layout.body_y + headerline_rows as i32 * line_height;
    let body_height = visible_rows as i32 * line_height;
    let body_bottom = body_top + body_height;
    if body_height <= 0 {
        return None;
    }
    if !clamp_body && (y < body_top || y >= body_bottom) {
        return None;
    }
    let y = y.clamp(body_top, body_bottom.saturating_sub(1));
    let visual_row_target = ((y - body_top) / line_height) as usize;
    let cell_width = cell_width.max(1);
    let fringe_width = editor_fringe_width_px(cell_width, buffer.dap_fringe_live());
    let line_number_width = cell_width * 5;
    let text_x = rect.x() + 12 + fringe_width + line_number_width;
    let wrap_cols = wrap_columns_for_width_with_fringe(
        rect.width(),
        cell_width,
        debug_fringe_cell_count(buffer.dap_fringe_live()),
    );
    let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
    let wrapped_lines = collect_wrapped_lines(
        buffer,
        WrapCollect {
            start_line: buffer.scroll_row,
            max_rows: visible_rows,
            wrap_cols,
            indent_size,
            scroll_col: buffer.scroll_col,
            line_wrap: buffer.line_wrap(),
        },
    );
    let mut visual_row = 0usize;
    for wrapped in wrapped_lines {
        let line_len = buffer.line_len_chars(wrapped.line_index);
        let row_count = wrapped.visual_row_count();
        if row_count > wrapped.segments.len() {
            if visual_row_target >= visual_row
                && visual_row_target < visual_row.saturating_add(row_count)
            {
                return Some(TextPoint::new(wrapped.line_index, 0));
            }
            visual_row = visual_row.saturating_add(row_count);
            continue;
        }
        for (segment_index, segment) in wrapped.segments.iter().enumerate() {
            if visual_row == visual_row_target {
                let segment_indent_cols = if segment_index == 0 {
                    0
                } else {
                    wrapped.continuation_indent_cols
                };
                let segment_x = text_x + (segment_indent_cols as i32 * cell_width);
                let display_offset = (x.saturating_sub(segment_x) / cell_width).max(0) as usize;
                let max_col = if line_len == 0 {
                    0
                } else {
                    segment.end_col.saturating_sub(1)
                };
                let display_col = wrapped
                    .char_map
                    .display_col_at(segment.start_col)
                    .saturating_add(display_offset);
                let column = wrapped
                    .char_map
                    .char_col_for_display_col(display_col)
                    .min(max_col);
                return Some(TextPoint::new(wrapped.line_index, column));
            }
            visual_row = visual_row.saturating_add(1);
        }
    }
    None
}

pub(super) fn fallback_overlay_anchor(
    buffer: &ShellBuffer,
    rect: Rect,
    line_height: i32,
    command_line_visible: bool,
) -> CursorScreenAnchor {
    let layout =
        buffer_footer_layout_with_command_line(buffer, rect, line_height, 0, command_line_visible);
    CursorScreenAnchor {
        x: rect.x() + 24,
        y: layout.body_y + 6,
        pane_bottom: layout.pane_bottom,
    }
}

#[derive(Debug)]
pub(super) struct WrappedLine {
    pub(super) line_index: usize,
    pub(super) line: String,
    pub(super) char_map: LineCharMap,
    pub(super) segments: Vec<LineWrapSegment>,
    pub(super) continuation_indent_cols: usize,
    pub(super) inline_image: Option<MarkdownInlineImageDraw>,
    visual_rows: usize,
}

impl WrappedLine {
    fn visual_row_count(&self) -> usize {
        self.visual_rows.max(1)
    }
}

pub(super) fn collect_wrapped_lines(buffer: &ShellBuffer, wrap: WrapCollect) -> Vec<WrappedLine> {
    collect_wrapped_lines_with_display(buffer, wrap, None, None)
}

pub(super) fn collect_wrapped_lines_with_display(
    buffer: &ShellBuffer,
    wrap: WrapCollect,
    display_overrides: Option<&BTreeMap<usize, String>>,
    inline_images: Option<&BTreeMap<usize, MarkdownInlineImageDraw>>,
) -> Vec<WrappedLine> {
    let WrapCollect {
        start_line,
        max_rows,
        wrap_cols,
        indent_size,
        scroll_col,
        line_wrap,
    } = wrap;
    if max_rows == 0 {
        return Vec::new();
    }

    let wrap_cols = wrap_cols.max(1);
    let mut lines = Vec::new();
    let mut visual_rows = 0usize;
    let mut line_index = start_line;
    let line_count = buffer.line_count();
    while line_index < line_count && visual_rows < max_rows {
        let inline_image = inline_images.and_then(|images| images.get(&line_index).cloned());
        let pretty_rows = buffer.pretty_display_rows.get(&line_index).copied();
        let is_pretty_image = inline_image.is_some() || pretty_rows.is_some();
        let line = if is_pretty_image {
            String::new()
        } else {
            display_overrides
                .and_then(|overrides| overrides.get(&line_index).cloned())
                .unwrap_or_else(|| buffer.text.line(line_index).unwrap_or_default())
        };
        let tab_width = resolved_tab_width(indent_size);
        let char_map = LineCharMap::with_tab_width(&line, tab_width);
        let (leading_indent_cols, _) = leading_whitespace_info(&line, tab_width);
        let continuation_indent_cols = leading_indent_cols.saturating_add(indent_size);
        let (continuation_indent_cols, segments) = if is_pretty_image {
            (
                0,
                vec![LineWrapSegment {
                    start_col: 0,
                    end_col: 0,
                }],
            )
        } else if line_wrap {
            let continuation_cols = wrap_cols.saturating_sub(continuation_indent_cols).max(1);
            (
                continuation_indent_cols,
                wrap_line_segments(&char_map, wrap_cols, continuation_cols),
            )
        } else {
            let start_col = char_map.char_col_for_display_col(scroll_col);
            let end_display_col = scroll_col.saturating_add(wrap_cols);
            let end_col = if end_display_col >= char_map.display_col_at(char_map.len()) {
                char_map.len()
            } else {
                char_map.char_col_for_display_col(end_display_col)
            };
            (
                0,
                vec![LineWrapSegment {
                    start_col: start_col.min(end_col),
                    end_col,
                }],
            )
        };
        let row_count = inline_image
            .as_ref()
            .map(|image| image.rows.max(1))
            .or(pretty_rows.map(|rows| rows.max(1)))
            .unwrap_or_else(|| segments.len().max(1));
        visual_rows = visual_rows.saturating_add(row_count);
        lines.push(WrappedLine {
            line_index,
            line,
            char_map,
            segments,
            continuation_indent_cols,
            inline_image,
            visual_rows: row_count,
        });
        line_index = line_index.saturating_add(1);
    }

    lines
}

pub(super) fn render_buffer(
    target: &mut DrawTarget<'_>,
    request: BufferDrawRequest<'_>,
    chrome: BufferChrome<'_>,
    metrics: TextMetrics,
) -> Result<(), ShellError> {
    let BufferDrawRequest {
        buffer,
        view_state,
        pane: PaneSlot { rect, active },
        decorations:
            BufferDecorations {
                visual_selection,
                yank_flash,
                input_mode,
                multicursor,
                vim_targets_input,
                recording_macro,
                typing_active,
            },
        command_line:
            CommandLineSlot {
                input: command_line_input,
                row_visible: command_line_row_visible,
            },
    } = request;
    let BufferChrome {
        user_library,
        theme_registry,
        workspace_name,
        lsp_server,
        lsp_workspace_loaded,
        acp_connected,
        git_summary,
    } = chrome;
    let TextMetrics {
        cell_width,
        line_height,
        ascent,
    } = metrics;
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let muted = blend_color(foreground, base_background, 0.5);
    let border_color = adjust_color(base_background, if is_dark { 24 } else { -24 });
    let commandline_background = theme_color(
        theme_registry,
        TOKEN_COMMANDLINE_BACKGROUND,
        Color::RGBA(base_background.r, base_background.g, base_background.b, 0),
    );
    let statusline_active = theme_color(
        theme_registry,
        TOKEN_STATUSLINE_ACTIVE,
        Color::RGBA(110, 170, 255, 255),
    );
    let statusline_inactive = theme_color(theme_registry, TOKEN_STATUSLINE_INACTIVE, muted);
    let statusline_text_color = if active {
        theme_color(
            theme_registry,
            TOKEN_STATUSLINE_FOREGROUND,
            statusline_active,
        )
    } else {
        theme_color(
            theme_registry,
            TOKEN_STATUSLINE_INACTIVE_FOREGROUND,
            theme_color(
                theme_registry,
                TOKEN_STATUSLINE_FOREGROUND,
                statusline_inactive,
            ),
        )
    };
    let text_color = foreground;
    let line_number_color = theme_color(theme_registry, TOKEN_LINE_NUMBER, muted);
    let line_number_current_color =
        theme_color(theme_registry, TOKEN_LINE_NUMBER_CURRENT, foreground);
    let ghost_text_color = theme_color(theme_registry, TOKEN_GHOST_TEXT, muted);
    let headerline_color = theme_color(theme_registry, TOKEN_HEADERLINE, statusline_active);
    let headerline_background =
        theme_color(theme_registry, TOKEN_HEADERLINE_BACKGROUND, base_background);
    let cursor = theme_color(theme_registry, "ui.cursor", Color::RGB(110, 170, 255));
    let selection = theme_color(theme_registry, "ui.selection", Color::RGBA(55, 71, 99, 255));
    let current_line_wash = theme_color(
        theme_registry,
        TOKEN_CURRENT_LINE,
        adjust_color(base_background, if is_dark { 12 } else { -12 }),
    );
    let relative_line_numbers = theme_registry
        .and_then(|registry| registry.resolve_bool(OPTION_LINE_NUMBER_RELATIVE))
        .unwrap_or(false);
    let cursor_roundness = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_CURSOR_ROUNDNESS))
        .map(|value| value.clamp(0.0, 16.0).round() as u32)
        .unwrap_or(2);
    let yank_flash_color = theme_registry
        .and_then(|registry| registry.resolve("ui.yank-flash"))
        .map(to_sdl_color)
        .unwrap_or(Color::RGBA(112, 196, 255, 120));
    let show_paren_match_color = theme_color(
        theme_registry,
        TOKEN_SHOW_PAREN_MATCH,
        Color::RGBA(110, 170, 255, 110),
    );
    let show_paren_mismatch_color = theme_color(
        theme_registry,
        TOKEN_SHOW_PAREN_MISMATCH,
        Color::RGBA(220, 80, 80, 160),
    );
    let git_added_fallback = theme_color(
        theme_registry,
        "git.status.entry.added",
        Color::RGB(108, 193, 118),
    );
    let git_modified_fallback = theme_color(
        theme_registry,
        "git.status.entry.modified",
        Color::RGB(209, 154, 102),
    );
    let git_removed_fallback = theme_color(
        theme_registry,
        "git.status.entry.deleted",
        Color::RGB(224, 107, 117),
    );
    let git_fringe_added = theme_color(
        theme_registry,
        user_library.gitfringe_token_added(),
        git_added_fallback,
    );
    let git_fringe_modified = theme_color(
        theme_registry,
        user_library.gitfringe_token_modified(),
        git_modified_fallback,
    );
    let git_fringe_removed = theme_color(
        theme_registry,
        user_library.gitfringe_token_removed(),
        git_removed_fallback,
    );
    let debug_fringe_verified = theme_color(
        theme_registry,
        TOKEN_DEBUG_FRINGE_BREAKPOINT,
        Color::RGB(224, 107, 117),
    );
    let debug_fringe_pending = theme_color(
        theme_registry,
        TOKEN_DEBUG_FRINGE_PENDING,
        Color::RGB(209, 154, 102),
    );
    let debug_fringe_execution = theme_color(
        theme_registry,
        TOKEN_DEBUG_FRINGE_EXECUTION,
        Color::RGB(86, 182, 194),
    );
    let debug_line_execution = theme_color(
        theme_registry,
        TOKEN_DEBUG_LINE_EXECUTION,
        Color::RGBA(86, 182, 194, 48),
    );
    let cell_width = cell_width.max(1);
    let (git_branch, git_added, git_removed) = git_summary
        .map(|summary| (summary.branch.as_deref(), summary.added, summary.removed))
        .unwrap_or((None, 0, 0));
    let lsp_diagnostics = statusline_lsp_diagnostics(buffer.lsp_diagnostics());
    let terminal_cursor = (buffer_is_terminal(&buffer.kind)
        && active
        && matches!(input_mode, InputMode::Insert | InputMode::Replace))
    .then(|| {
        buffer
            .terminal_render()
            .and_then(TerminalRenderSnapshot::cursor)
    })
    .flatten();
    let cursor_row = view_state.cursor.line;
    let cursor_col = view_state.cursor.column;
    let scroll_row = view_state.scroll_row;
    let statusline_line = terminal_cursor
        .map(|cursor| cursor.row() as usize + 1)
        .unwrap_or(cursor_row + 1);
    let statusline_column = terminal_cursor
        .map(|cursor| cursor.col() as usize + 1)
        .unwrap_or(cursor_col + 1);
    let statusline_context = HostStatuslineContext {
        vim_mode: statusline_mode_label(input_mode, multicursor.is_some()),
        recording_macro,
        workspace_name,
        buffer_name: buffer.display_name(),
        buffer_modified: buffer.is_dirty(),
        language_id: buffer.language_id(),
        line: statusline_line,
        column: statusline_column,
        lsp_server,
        lsp_diagnostics,
        acp_connected,
        git_branch,
        git_added,
        git_removed,
    };
    let modeline_segments = user_library.modeline_segments(&statusline_context);
    let statusline = truncate_text_to_width(
        &flatten_modeline_text(&modeline_segments),
        rect.width().saturating_sub(24),
        cell_width,
    );
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        line_height,
        cell_width,
        command_line_row_visible,
    );
    if buffer_is_terminal(&buffer.kind)
        && let Some(terminal_render) = buffer.terminal_render()
    {
        render_terminal_buffer(
            target,
            TerminalBufferDraw {
                buffer,
                terminal_render,
                rect,
                layout,
                active,
                input_mode,
                visual_selection,
                yank_flash,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground: text_color,
                muted: text_color,
                border_color,
                selection,
                yank_flash_color,
                cursor,
                cursor_roundness,
            },
            TerminalStatusline {
                text: statusline,
                active: statusline_active,
                inactive: statusline_inactive,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
        if let Some(commandline_y) = layout.commandline_y {
            render_footer_separator(
                target,
                rect,
                commandline_y - BUFFER_FOOTER_SEPARATOR_OFFSET,
                border_color,
                window_effects,
            )?;
        }
        render_command_line_overlay(
            target,
            CommandLineOverlayDraw {
                input: command_line_input,
                rect,
                layout,
                active,
                input_mode,
                paint: CommandLinePaint {
                    window_effects,
                    background: commandline_background,
                    foreground: text_color,
                    muted,
                    cursor,
                    cursor_roundness,
                    chip_radius: overlay_radius(theme_registry).min(8),
                },
                metrics: CellMetrics {
                    cell_width,
                    line_height,
                },
            },
        )?;
        return Ok(());
    }
    let text_x = rect.x() + 12 + cell_width + cell_width * 5;
    if buffer_is_browser(&buffer.kind) {
        render_browser_buffer_body(
            target,
            BrowserBufferDraw {
                buffer,
                rect,
                layout,
                active,
                input_mode,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground,
                muted,
                border_color,
                selection,
                yank_flash_color: selection,
                cursor,
                cursor_roundness,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
    } else if buffer.has_pdf_preview_surface() {
        render_pdf_buffer_body(target, rect, layout, theme_registry, base_background)?;
    } else if buffer.is_acp_buffer() {
        render_acp_buffer_body(
            target,
            AcpBufferDraw {
                buffer,
                rect,
                layout,
                active,
                visual_selection,
                yank_flash,
                input_mode,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground,
                muted,
                border_color,
                selection,
                yank_flash_color,
                cursor,
                cursor_roundness,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
    } else if buffer.is_rendered_image_buffer() {
        render_image_buffer_body(
            target,
            buffer,
            rect,
            layout,
            theme_registry,
            base_background,
        )?;
    } else if buffer.has_plugin_sections() {
        render_plugin_section_buffer_body(
            target,
            PluginSectionDraw {
                buffer,
                view_state,
                pane: PaneSlot { rect, active },
                layout,
                visual_selection,
                yank_flash,
                input_mode,
            },
            BufferBodyPalette {
                theme_registry,
                base_background,
                foreground,
                muted,
                border_color,
                selection,
                yank_flash_color,
                cursor,
                cursor_roundness,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
    } else {
        let debug_fringe_live = buffer.dap_fringe_live();
        let fringe_width = editor_fringe_width_px(cell_width, debug_fringe_live);
        let line_number_width = cell_width * 5;
        let wrap_cols = wrap_columns_for_width_with_fringe(
            rect.width(),
            cell_width,
            debug_fringe_cell_count(debug_fringe_live),
        );
        let indent_size = theme_lang_indent(theme_registry, buffer.language_id());
        let context_overlay =
            buffer_context_overlay_snapshot(buffer, active, typing_active, user_library);
        let headerline_lines = context_overlay
            .as_ref()
            .map(|snapshot| {
                visible_headerline_lines(&snapshot.headerline_lines, layout.visible_rows)
            })
            .unwrap_or_default();
        let headerline_rows = headerline_lines.len();
        let body_y = layout.body_y + headerline_rows as i32 * line_height;
        let visible_rows = layout.visible_rows.saturating_sub(headerline_rows).max(1);
        let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
        let pretty_paint = markdown_pretty_paint_plan(
            buffer,
            user_library,
            MarkdownPrettyPaintArgs {
                visible_start: view_state.scroll_row,
                visible_end: view_state
                    .scroll_row
                    .saturating_add(visible_rows.saturating_add(8)),
                visual_selection,
                input_mode,
                pane_width_px: text_width_px,
                line_height,
            },
        );
        let wrapped_lines = collect_wrapped_lines_with_display(
            buffer,
            WrapCollect {
                start_line: scroll_row,
                max_rows: visible_rows,
                wrap_cols,
                indent_size,
                scroll_col: view_state.scroll_col,
                line_wrap: buffer.line_wrap(),
            },
            Some(&pretty_paint.text_overrides),
            Some(&pretty_paint.images),
        );
        let mut cursor_row_on_screen = None;
        let mut cursor_col_on_screen = None;
        let mut cursor_indent_cols = 0usize;
        let multicursor_points = if active {
            multicursor
                .map(multicursor_cursor_points)
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let mut visual_row = 0usize;
        for wrapped in &wrapped_lines {
            if wrapped.line_index == cursor_row {
                let display_cursor_col = wrapped.char_map.cursor_anchor_col(cursor_col);
                let segment_index = segment_index_for_column(&wrapped.segments, display_cursor_col);
                if let Some(segment) = wrapped.segments.get(segment_index) {
                    cursor_row_on_screen = Some(visual_row + segment_index);
                    cursor_col_on_screen = Some(
                        wrapped
                            .char_map
                            .display_cols_between(segment.start_col, display_cursor_col),
                    );
                    cursor_indent_cols = if segment_index == 0 {
                        0
                    } else {
                        wrapped.continuation_indent_cols
                    };
                }
            }
            visual_row = visual_row.saturating_add(wrapped.visual_row_count());
            if visual_row >= visible_rows {
                break;
            }
        }

        let gutter_x = rect.x() + 12;
        let fringe_x = gutter_x;
        let line_number_x = gutter_x + fringe_width;
        let text_x = line_number_x + line_number_width;
        let headerline_width = rect
            .width()
            .saturating_sub((text_x - rect.x()).max(0) as u32 + 12);
        let show_text_cursor = !buffer.has_input_field()
            || !active
            || !vim_targets_input
            || !matches!(input_mode, InputMode::Insert | InputMode::Replace);
        let cursor_width = match input_mode {
            InputMode::Normal | InputMode::Visual => cell_width.max(2) as u32,
            InputMode::Insert | InputMode::Replace => (cell_width / 4).max(2) as u32,
        };
        let primary_cursor_rect = if show_text_cursor
            && let (Some(cursor_row_on_screen), Some(cursor_col_on_screen)) =
                (cursor_row_on_screen, cursor_col_on_screen)
            && cursor_row_on_screen < visible_rows
        {
            Some(PixelRectToRect::rect(
                text_x + ((cursor_indent_cols + cursor_col_on_screen) as i32 * cell_width),
                body_y + cursor_row_on_screen as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ))
        } else {
            None
        };
        let mut primary_cursor_text_overlay: Option<(i32, CursorTextOverlay)> = None;
        let mut multicursor_rects = Vec::new();
        let mut visual_row = 0usize;
        let show_paren = buffer
            .show_paren_enabled(user_library.show_paren_config().enabled)
            .then(|| {
                buffer
                    .text
                    .show_paren_at(view_state.cursor, buffer.language_id())
            })
            .flatten();
        for wrapped in wrapped_lines {
            let line_index = wrapped.line_index;
            let line_len = buffer.line_len_chars(line_index);
            let selection_range = visual_selection.and_then(|selection_state| {
                selection_columns_for_visual(selection_state, line_index, line_len)
            });
            let multicursor_ranges = multicursor
                .map(|state| multicursor_ranges_for_line(state, input_mode, line_index, line_len))
                .unwrap_or_default();
            let yank_range = yank_flash.and_then(|selection_state| {
                selection_columns_for_visual(selection_state, line_index, line_len)
            });
            let show_paren_color = show_paren.as_ref().map(|pair| {
                if pair.matched {
                    show_paren_match_color
                } else {
                    show_paren_mismatch_color
                }
            });
            if let Some(image) = wrapped.inline_image.as_ref() {
                if visual_row >= visible_rows {
                    break;
                }
                let remaining_rows = visible_rows.saturating_sub(visual_row);
                let image_rows_draw = image.rows.min(remaining_rows).max(1);
                let y = body_y + visual_row as i32 * line_height;
                let text_area_width = rect
                    .width()
                    .saturating_sub((text_x - rect.x()).max(0) as u32)
                    .saturating_sub(12)
                    .max(1);
                let image_rect = PixelRectToRect::rect(
                    text_x,
                    y,
                    text_area_width,
                    (image_rows_draw as i32 * line_height).max(line_height) as u32,
                );
                let line_number = if relative_line_numbers {
                    if line_index == cursor_row {
                        0
                    } else {
                        cursor_row.abs_diff(line_index)
                    }
                } else {
                    line_index + 1
                };
                draw_text(
                    target,
                    line_number_x,
                    y,
                    &format!("{:>4}", line_number),
                    if line_index == cursor_row {
                        line_number_current_color
                    } else {
                        line_number_color
                    },
                )?;
                draw_image(
                    target,
                    image_rect,
                    image.width,
                    image.height,
                    Arc::clone(&image.pixels),
                    Some(image_rect),
                )?;
                let _ = &image.alt;
                visual_row = visual_row.saturating_add(image_rows_draw);
                if visual_row >= visible_rows {
                    break;
                }
                continue;
            }
            for (segment_index, segment) in wrapped.segments.iter().enumerate() {
                if visual_row >= visible_rows {
                    break;
                }
                let y = body_y + visual_row as i32 * line_height;
                let is_execution_line = buffer.dap_execution_line() == Some(line_index);
                if is_execution_line {
                    fill_window_surface_rect(
                        target,
                        PixelRectToRect::rect(
                            gutter_x,
                            y,
                            rect.width().saturating_sub(24),
                            line_height.max(1) as u32,
                        ),
                        debug_line_execution,
                        window_effects,
                    )?;
                } else if active
                    && line_index == cursor_row
                    && !matches!(input_mode, InputMode::Visual)
                {
                    fill_window_surface_rect(
                        target,
                        PixelRectToRect::rect(
                            gutter_x,
                            y,
                            rect.width().saturating_sub(24),
                            line_height.max(1) as u32,
                        ),
                        current_line_wash,
                        window_effects,
                    )?;
                }
                let segment_indent_cols = if segment_index == 0 {
                    0
                } else {
                    wrapped.continuation_indent_cols
                };
                let segment_x = text_x + (segment_indent_cols as i32 * cell_width);
                if let Some((selection_start, selection_end)) = selection_range {
                    let start = selection_start.max(segment.start_col);
                    let end = selection_end.min(segment.end_col);
                    if start < end {
                        let start_display = wrapped
                            .char_map
                            .display_cols_between(segment.start_col, start);
                        let width_display = wrapped.char_map.display_cols_between(start, end);
                        fill_selection_highlight(
                            target,
                            segment_x + (start_display as i32 * cell_width),
                            y,
                            (width_display as i32 * cell_width) as u32,
                            line_height.max(1) as u32,
                            cursor_roundness,
                            selection,
                        )?;
                    }
                }
                for (selection_start, selection_end) in multicursor_ranges.iter().copied() {
                    let start = selection_start.max(segment.start_col);
                    let end = selection_end.min(segment.end_col);
                    if start < end {
                        let start_display = wrapped
                            .char_map
                            .display_cols_between(segment.start_col, start);
                        let width_display = wrapped.char_map.display_cols_between(start, end);
                        fill_selection_highlight(
                            target,
                            segment_x + (start_display as i32 * cell_width),
                            y,
                            (width_display as i32 * cell_width) as u32,
                            line_height.max(1) as u32,
                            cursor_roundness,
                            blend_color(selection, cursor, 0.25),
                        )?;
                    }
                }
                if let Some((selection_start, selection_end)) = yank_range {
                    let start = selection_start.max(segment.start_col);
                    let end = selection_end.min(segment.end_col);
                    if start < end {
                        let start_display = wrapped
                            .char_map
                            .display_cols_between(segment.start_col, start);
                        let width_display = wrapped.char_map.display_cols_between(start, end);
                        fill_selection_highlight(
                            target,
                            segment_x + (start_display as i32 * cell_width),
                            y,
                            (width_display as i32 * cell_width) as u32,
                            line_height.max(1) as u32,
                            cursor_roundness,
                            yank_flash_color,
                        )?;
                    }
                }
                if let (Some(pair), Some(color)) = (show_paren.as_ref(), show_paren_color) {
                    for range in [Some(pair.origin), pair.counterpart].into_iter().flatten() {
                        if let Some((selection_start, selection_end)) =
                            selection_columns_for_line(range, line_index, line_len)
                        {
                            let start = selection_start.max(segment.start_col);
                            let end = selection_end.min(segment.end_col);
                            if start < end {
                                let start_display = wrapped
                                    .char_map
                                    .display_cols_between(segment.start_col, start);
                                let width_display =
                                    wrapped.char_map.display_cols_between(start, end);
                                fill_selection_highlight(
                                    target,
                                    segment_x + (start_display as i32 * cell_width),
                                    y,
                                    (width_display as i32 * cell_width) as u32,
                                    line_height.max(1) as u32,
                                    cursor_roundness,
                                    color,
                                )?;
                            }
                        }
                    }
                }
                if segment_index == 0 {
                    let diagnostic_severity = user_library
                        .lsp_show_buffer_diagnostics()
                        .then(|| buffer.lsp_diagnostic_severity(line_index))
                        .flatten();
                    let dap_marker = buffer.dap_fringe_marker(line_index);
                    let dap_execution = buffer.dap_execution_line() == Some(line_index);
                    let shows_dap_glyph = dap_execution || dap_marker.is_some();
                    let git_fringe_x = if debug_fringe_live {
                        fringe_x + cell_width
                    } else {
                        fringe_x
                    };
                    if dap_execution {
                        draw_text(
                            target,
                            fringe_x,
                            y,
                            DEBUG_FRINGE_EXECUTION_GLYPH,
                            debug_fringe_execution,
                        )?;
                    } else if let Some(state) = dap_marker {
                        let (glyph, color) = match state {
                            BreakpointState::Verified => {
                                (DEBUG_FRINGE_VERIFIED_GLYPH, debug_fringe_verified)
                            }
                            BreakpointState::Pending | BreakpointState::Unverified => {
                                (DEBUG_FRINGE_PENDING_GLYPH, debug_fringe_pending)
                            }
                        };
                        draw_text(target, fringe_x, y, glyph, color)?;
                    }
                    // Idle: Breakpoint glyph replaces git on that line. Live Session:
                    // DAP markers keep the left cell; git uses the widened right cell.
                    let draw_git_or_diagnostic = debug_fringe_live || !shows_dap_glyph;
                    if draw_git_or_diagnostic {
                        if let Some(severity) = diagnostic_severity {
                            let color = diagnostic_color(severity, theme_registry);
                            draw_text(
                                target,
                                git_fringe_x,
                                y,
                                user_library.lsp_diagnostic_icon(),
                                color,
                            )?;
                        } else if let Some(kind) = buffer.git_fringe_kind(line_index) {
                            let color = match kind {
                                GitFringeKind::Added => git_fringe_added,
                                GitFringeKind::Modified => git_fringe_modified,
                                GitFringeKind::Removed => git_fringe_removed,
                            };
                            fill_window_surface_rect(
                                target,
                                PixelRectToRect::rect(
                                    git_fringe_x - 4,
                                    y,
                                    GIT_FRINGE_BAR_WIDTH,
                                    line_height.max(1) as u32,
                                ),
                                color,
                                window_effects,
                            )?;
                        }
                    }
                    let line_number = if relative_line_numbers {
                        if line_index == cursor_row {
                            0
                        } else {
                            cursor_row.abs_diff(line_index)
                        }
                    } else {
                        line_index + 1
                    };
                    let line_number_color = diagnostic_severity
                        .map(|severity| diagnostic_color(severity, theme_registry))
                        .unwrap_or(if line_index == cursor_row {
                            line_number_current_color
                        } else {
                            line_number_color
                        });
                    draw_text(
                        target,
                        line_number_x,
                        y,
                        &format!("{:>4}", line_number),
                        line_number_color,
                    )?;
                }
                draw_buffer_text(
                    target,
                    BufferTextRun {
                        x: segment_x,
                        y,
                        line: &wrapped.line,
                        segment: *segment,
                        char_map: &wrapped.char_map,
                        line_syntax_spans: buffer.line_syntax_spans(line_index),
                        default_color: text_color,
                        cell_width,
                    },
                    theme_registry,
                )?;
                if primary_cursor_text_overlay.is_none()
                    && let Some(overlay) = block_cursor_text_overlay(CursorOverlayQuery {
                        x: segment_x,
                        line: &wrapped.line,
                        char_map: &wrapped.char_map,
                        segment: *segment,
                        line_index,
                        cursor: TextPoint::new(cursor_row, cursor_col),
                        color: (matches!(input_mode, InputMode::Normal | InputMode::Visual)
                            && !vim_targets_input)
                            .then_some(base_background),
                        cell_width,
                    })
                {
                    primary_cursor_text_overlay = Some((y, overlay));
                }
                if user_library.lsp_show_buffer_diagnostics() && buffer.lsp_enabled() {
                    draw_diagnostic_underlines_for_segment(
                        target,
                        DiagnosticUnderlineDraw {
                            diagnostics: buffer.lsp_diagnostic_line_spans(line_index),
                            syntax_spans: buffer.line_syntax_spans(line_index),
                            char_map: &wrapped.char_map,
                            segment_x,
                            y,
                            line_len,
                            segment: *segment,
                            metrics: CellMetrics {
                                cell_width,
                                line_height,
                            },
                            theme_registry,
                        },
                    )?;
                }
                draw_line_ghost_text_for_segment(
                    target,
                    GhostTextSegmentDraw {
                        x: segment_x,
                        y,
                        segment: *segment,
                        char_map: &wrapped.char_map,
                        line_len,
                        ghost_text: context_overlay
                            .as_ref()
                            .and_then(|snapshot| snapshot.ghost_text_by_line.get(&line_index))
                            .map(String::as_str),
                        color: ghost_text_color,
                        cell_width,
                    },
                )?;
                for point in multicursor_points.iter().copied().filter(|point| {
                    point.line == line_index
                        && point.column >= segment.start_col
                        && point.column <= segment.end_col
                }) {
                    multicursor_rects.push(PixelRectToRect::rect(
                        segment_x
                            + (wrapped
                                .char_map
                                .display_cols_between(segment.start_col, point.column)
                                as i32
                                * cell_width),
                        y,
                        cursor_width,
                        line_height.max(2) as u32,
                    ));
                }
                visual_row = visual_row.saturating_add(1);
            }
            if visual_row >= visible_rows {
                break;
            }
        }
        if headerline_rows > 0 {
            for (index, headerline) in headerline_lines.iter().enumerate() {
                let y = layout.body_y + index as i32 * line_height;
                fill_window_surface_rounded_rect(
                    target,
                    PixelRectToRect::rect(
                        rect.x() + 8,
                        y + 1,
                        rect.width().saturating_sub(16),
                        line_height.saturating_sub(2).max(1) as u32,
                    ),
                    overlay_radius(theme_registry).min(8),
                    headerline_background,
                    window_effects,
                )?;
                draw_text(
                    target,
                    text_x,
                    y,
                    &truncate_text_to_width_preserving_end(
                        headerline,
                        headerline_width,
                        cell_width,
                    ),
                    headerline_color,
                )?;
            }
            fill_window_surface_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 8,
                    body_y.saturating_sub(1),
                    rect.width().saturating_sub(16),
                    1,
                ),
                border_color,
                window_effects,
            )?;
        }
        for rect in multicursor_rects {
            fill_rounded_rect(target, rect, cursor_roundness, cursor)?;
        }
        if let Some(rect) = primary_cursor_rect {
            fill_rounded_rect(target, rect, cursor_roundness, cursor)?;
        }
        if let Some((y, overlay)) = primary_cursor_text_overlay {
            draw_text(target, overlay.draw_x, y, &overlay.text, overlay.color)?;
        }
        paint_buffer_scrollbar(
            target,
            ScrollbarPaint {
                pane_rect: rect,
                body_y,
                visible_rows,
                line_height,
                scroll_row: view_state.scroll_row,
                max_scroll: buffer.max_scroll_row_for_wrapped_rows(
                    visible_rows,
                    wrap_cols,
                    indent_size,
                ),
                color: muted,
                window_effects,
            },
        )?;
    }

    if let Some(input) = buffer.standalone_input_field() {
        let input_background = theme_color(
            theme_registry,
            "ui.input.background",
            adjust_color(base_background, if is_dark { 8 } else { -8 }),
        );
        let input_foreground = theme_color(theme_registry, "ui.input.foreground", foreground);
        let placeholder_color = theme_color(theme_registry, "ui.input.placeholder", muted);
        fill_window_surface_rect(
            target,
            PixelRectToRect::rect(
                rect.x() + 8,
                layout.input_y - 4,
                rect.width().saturating_sub(16),
                layout.input_box_height as u32,
            ),
            input_background,
            window_effects,
        )?;
        if buffer_is_acp(&buffer.kind) {
            let border = if acp_connected {
                git_added_fallback
            } else {
                border_color
            };
            fill_window_surface_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 8,
                    layout.input_y - 4,
                    rect.width().saturating_sub(16),
                    1,
                ),
                border,
                window_effects,
            )?;
            fill_window_surface_rect(
                target,
                PixelRectToRect::rect(
                    rect.x() + 8,
                    layout.input_y - 4 + layout.input_box_height,
                    rect.width().saturating_sub(16),
                    1,
                ),
                border,
                window_effects,
            )?;
        }
        let input_x = text_x;
        let prompt = input.prompt();
        let prompt_len = prompt.chars().count();
        let prompt_padding = " ".repeat(prompt_len);
        let text_width = rect.width() as i32 - (text_x - rect.x()) - 12;
        let available_input_cols = (text_width / cell_width.max(1)).max(1) as usize;
        if active && vim_targets_input && matches!(input_mode, InputMode::Visual) {
            for (row, start_col, end_col) in
                input.selection_visual_ranges(VisualSelectionKind::Character, available_input_cols)
            {
                fill_selection_highlight(
                    target,
                    input_x + ((prompt_len + start_col) as i32 * cell_width),
                    layout.input_y + row as i32 * line_height,
                    ((end_col.saturating_sub(start_col)) as i32 * cell_width.max(1)) as u32,
                    line_height.max(1) as u32,
                    cursor_roundness,
                    selection,
                )?;
            }
        }
        let max_visible_rows = visible_input_text_rows(layout.input_box_height, line_height);
        let (visible_rows, first_visible_row) =
            input.visible_wrapped_visual_rows(available_input_cols, max_visible_rows);
        if input.text().is_empty() {
            if let Some(placeholder) = input.placeholder() {
                let line = format!("{prompt}{placeholder}");
                draw_text(target, input_x, layout.input_y, &line, placeholder_color)?;
            } else {
                draw_text(target, input_x, layout.input_y, prompt, input_foreground)?;
            }
        } else {
            for (index, line) in visible_rows.into_iter().enumerate() {
                let prefix = if index == 0 { prompt } else { &prompt_padding };
                let rendered = format!("{prefix}{line}");
                draw_text(
                    target,
                    input_x,
                    layout.input_y + index as i32 * line_height,
                    &rendered,
                    input_foreground,
                )?;
            }
        }
        if let Some(hint) = input.hint() {
            let hint_y = layout.input_y + layout.input_box_height + layout.input_hint_gap;
            if let Some((mode_label, rest)) = hint.split_once(" · ") {
                let prefix = format!("{prompt_padding}{mode_label}");
                draw_text(target, input_x, hint_y, &prefix, git_added_fallback)?;
                let prefix_width = monospace_text_width(&prefix, cell_width) as i32;
                let suffix = format!(" · {rest}");
                draw_text(
                    target,
                    input_x + prefix_width,
                    hint_y,
                    &suffix,
                    placeholder_color,
                )?;
            } else {
                let hint_line = format!("{prompt_padding}{hint}");
                draw_text(target, input_x, hint_y, &hint_line, placeholder_color)?;
            }
        }
        if active
            && vim_targets_input
            && matches!(input_mode, InputMode::Insert | InputMode::Replace)
        {
            let (input_row, col_in_visual_row) = input.cursor_visual_row_col(available_input_cols);
            let input_row = input_row.saturating_sub(first_visible_row);
            let input_col = prompt_len + col_in_visual_row;
            let cursor_width = (cell_width / 4).max(2) as u32;
            fill_rounded_rect(
                target,
                PixelRectToRect::rect(
                    input_x + (input_col as i32 * cell_width),
                    layout.input_y + input_row as i32 * line_height,
                    cursor_width,
                    line_height.max(2) as u32,
                ),
                cursor_roundness,
                cursor,
            )?;
        } else if active
            && vim_targets_input
            && matches!(input_mode, InputMode::Normal | InputMode::Visual)
        {
            let cursor_char = input.cursor_char();
            let char_count = input.char_count();
            if char_count > 0 {
                let cursor_index = cursor_char.min(char_count.saturating_sub(1));
                let mut cursor_input = input.clone();
                cursor_input.cursor = cursor_index;
                cursor_input.clear_selection();
                let (input_row, col_in_visual_row) =
                    cursor_input.cursor_visual_row_col(available_input_cols);
                let input_row = input_row.saturating_sub(first_visible_row);
                fill_rect(
                    target,
                    PixelRectToRect::rect(
                        input_x + ((prompt_len + col_in_visual_row) as i32 * cell_width),
                        layout.input_y + input_row as i32 * line_height,
                        cell_width.max(1) as u32,
                        line_height.max(1) as u32,
                    ),
                    cursor,
                )?;
            }
        }
    }

    render_footer_separator(
        target,
        rect,
        layout.statusline_y - BUFFER_FOOTER_SEPARATOR_OFFSET,
        border_color,
        window_effects,
    )?;
    let statusline_x = rect.x() + 12;
    draw_modeline(
        target,
        &modeline_segments,
        ModelineDraw {
            x: statusline_x,
            y: layout.statusline_y,
            max_width: rect.width().saturating_sub(24),
            default_color: statusline_text_color,
            apply_tokens: active,
            theme_registry,
            user_library,
            acp_connected,
            lsp_server_visible: lsp_server.is_some(),
            lsp_workspace_loaded,
            connected_color: git_added_fallback,
            cell_width,
            line_height: Some(line_height),
        },
    )?;
    if let Some(commandline_y) = layout.commandline_y {
        render_footer_separator(
            target,
            rect,
            commandline_y - BUFFER_FOOTER_SEPARATOR_OFFSET,
            border_color,
            window_effects,
        )?;
    }
    render_command_line_overlay(
        target,
        CommandLineOverlayDraw {
            input: command_line_input,
            rect,
            layout,
            active,
            input_mode,
            paint: CommandLinePaint {
                window_effects,
                background: commandline_background,
                foreground,
                muted,
                cursor,
                cursor_roundness,
                chip_radius: overlay_radius(theme_registry).min(8),
            },
            metrics: CellMetrics {
                cell_width,
                line_height,
            },
        },
    )?;

    let _ = ascent;
    fill_window_surface_rect(
        target,
        PixelRectToRect::rect(
            rect.x(),
            rect.y() + rect.height() as i32 - 2,
            rect.width(),
            1,
        ),
        border_color,
        window_effects,
    )?;
    Ok(())
}

pub(super) fn render_command_line_overlay(
    target: &mut DrawTarget<'_>,
    draw: CommandLineOverlayDraw<'_>,
) -> Result<(), ShellError> {
    let CommandLineOverlayDraw {
        input: command_line_input,
        rect,
        layout,
        active,
        input_mode,
        paint:
            CommandLinePaint {
                window_effects,
                background,
                foreground,
                muted,
                cursor,
                cursor_roundness,
                chip_radius,
            },
        metrics: CellMetrics {
            cell_width,
            line_height,
        },
    } = draw;
    let Some(commandline_y) = layout.commandline_y else {
        return Ok(());
    };
    let Some(input) = command_line_input else {
        return Ok(());
    };
    let chip_background = if background.a == 0 {
        Color::RGBA(foreground.r, foreground.g, foreground.b, 18)
    } else {
        background
    };
    fill_window_surface_rounded_rect(
        target,
        PixelRectToRect::rect(
            rect.x() + 8,
            commandline_y,
            rect.width().saturating_sub(16),
            line_height.max(1) as u32,
        ),
        chip_radius,
        chip_background,
        window_effects,
    )?;
    let text_x = rect.x() + 12;
    let prompt = input.prompt();
    let rendered = if input.text().is_empty() {
        input.placeholder().map_or_else(
            || prompt.to_owned(),
            |placeholder| format!("{prompt}{placeholder}"),
        )
    } else {
        format!("{prompt}{}", input.text())
    };
    let color = if input.text().is_empty() {
        muted
    } else {
        foreground
    };
    draw_text(target, text_x, commandline_y, &rendered, color)?;
    if active {
        let cursor_color = if matches!(input_mode, InputMode::Replace) {
            adjust_color(cursor, -24)
        } else {
            cursor
        };
        let cursor_col = prompt.chars().count() + input.cursor;
        let cursor_width = (cell_width / 4).max(2) as u32;
        fill_rounded_rect(
            target,
            PixelRectToRect::rect(
                text_x + cursor_col as i32 * cell_width.max(1),
                commandline_y,
                cursor_width,
                line_height.max(2) as u32,
            ),
            cursor_roundness,
            cursor_color,
        )?;
    }
    Ok(())
}

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

fn plugin_section_tree_layout(
    state: &PluginSectionBufferState,
    tree: &PluginBufferLayout,
    bounds: PixelRect,
    cell_width: i32,
    line_height: i32,
    gap: i32,
) -> Option<PluginSectionLayout> {
    let line_height = line_height.max(1);
    let split = plugin_layout_to_split_node(state, tree, line_height)?;
    let leaves = layout_split_tree(bounds, &split, gap.max(0) as u32);
    let mut panes = vec![
        TextPaneLayout {
            rect: Rect::new(bounds.x, bounds.y, 0, 0),
            visible_rows: 1,
            wrap_cols: 1,
        };
        state.section_count()
    ];
    for (index, leaf) in leaves {
        let Some(pane) = panes.get_mut(index) else {
            continue;
        };
        let chrome = plugin_section_panel_chrome_height(state.section_title(index), line_height);
        let inner_height = (leaf.height as i32).saturating_sub(chrome).max(line_height);
        let wrap_cols = overlay_text_columns(leaf.width.saturating_sub(20), 0, cell_width);
        *pane = TextPaneLayout {
            rect: Rect::new(leaf.x, leaf.y, leaf.width, leaf.height),
            visible_rows: (inner_height / line_height).max(1) as usize,
            wrap_cols,
        };
    }
    Some(PluginSectionLayout { panes })
}

fn plugin_layout_to_split_node(
    state: &PluginSectionBufferState,
    layout: &PluginBufferLayout,
    line_height: i32,
) -> Option<SplitNode> {
    let children = layout
        .children()
        .iter()
        .filter_map(|child| plugin_layout_node_to_child(state, child, line_height))
        .collect::<Vec<_>>();
    (!children.is_empty()).then_some(SplitNode::new(plugin_layout_axis(layout.axis()), children))
}

fn plugin_layout_node_to_child(
    state: &PluginSectionBufferState,
    node: &PluginBufferLayoutNode,
    line_height: i32,
) -> Option<SplitChild> {
    match node {
        PluginBufferLayoutNode::Section { name, weight } => {
            let index = state.section_index_by_name(name.as_str())?;
            let chrome =
                plugin_section_panel_chrome_height(state.section_title(index), line_height);
            let min_rows = state.section_min_rows(index).unwrap_or(1).max(1) as i32;
            Some(SplitChild::leaf(
                index,
                (*weight).max(1),
                (chrome + min_rows * line_height).max(1) as u32,
            ))
        }
        PluginBufferLayoutNode::Split {
            axis,
            weight,
            children,
        } => {
            let mapped = children
                .iter()
                .filter_map(|child| plugin_layout_node_to_child(state, child, line_height))
                .collect::<Vec<_>>();
            if mapped.is_empty() {
                return None;
            }
            let min_px = mapped.iter().map(|child| child.min_px).sum::<u32>();
            Some(SplitChild::node(
                SplitNode::new(plugin_layout_axis(*axis), mapped),
                (*weight).max(1),
                min_px.max(1),
            ))
        }
    }
}

fn plugin_layout_axis(axis: PluginBufferLayoutAxis) -> SplitAxis {
    match axis {
        PluginBufferLayoutAxis::Rows => SplitAxis::Rows,
        PluginBufferLayoutAxis::Columns => SplitAxis::Columns,
    }
}

pub(super) fn render_plugin_section_buffer_body(
    target: &mut DrawTarget<'_>,
    draw: PluginSectionDraw<'_>,
    palette: BufferBodyPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let PluginSectionDraw {
        buffer,
        view_state: base_view_state,
        pane: PaneSlot { rect, active },
        layout,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let BufferBodyPalette {
        theme_registry,
        base_background,
        foreground,
        muted,
        border_color,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let Some(state) = buffer.plugin_sections() else {
        return Ok(());
    };
    let Some(section_layout) =
        plugin_section_buffer_layout(buffer, rect, layout, cell_width, line_height)
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
    let header_background = theme_color(
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
    );
    let active_border = theme_color(theme_registry, TOKEN_STATUSLINE_ACTIVE, cursor);
    for (index, pane_layout) in section_layout.panes.iter().copied().enumerate() {
        let pane_active = active && state.active_section == index;
        let pane_visual_selection = if state.active_section == index {
            visual_selection
        } else {
            None
        };
        let pane_yank_flash = if state.active_section == index {
            yank_flash
        } else {
            None
        };
        let (text, scroll_row, cursor_point, title, pane_mode, pane_syntax) = if index == 0 {
            (
                &buffer.text,
                base_view_state.scroll_row,
                (state.active_section == 0).then_some(base_view_state.cursor),
                state.base_title.as_str(),
                if state.base_writable {
                    input_mode
                } else {
                    InputMode::Normal
                },
                Some(&buffer.syntax_lines),
            )
        } else {
            let Some(pane) = state.attached_section(index) else {
                continue;
            };
            (
                &pane.text,
                pane.scroll_row,
                (state.active_section == index).then_some(pane.cursor()),
                pane.title.as_str(),
                if pane.writable {
                    input_mode
                } else {
                    InputMode::Normal
                },
                Some(&pane.syntax_lines),
            )
        };
        render_text_panel(
            target,
            TextPanelDraw {
                text,
                syntax_lines: pane_syntax,
                scroll_row,
                cursor_point,
                pane_active,
                pane_layout,
                title,
                visual_selection: pane_visual_selection,
                yank_flash: pane_yank_flash,
                input_mode: pane_mode,
            },
            PanelPalette {
                theme_registry,
                panel_background,
                header_background,
                foreground,
                muted,
                border_color,
                active_border,
                selection,
                yank_flash_color,
                cursor,
                cursor_roundness,
            },
            CellMetrics {
                cell_width,
                line_height,
            },
        )?;
    }
    Ok(())
}

pub(super) fn render_text_panel(
    target: &mut DrawTarget<'_>,
    draw: TextPanelDraw<'_>,
    palette: PanelPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let TextPanelDraw {
        text,
        syntax_lines,
        scroll_row,
        cursor_point,
        pane_active,
        pane_layout,
        title,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let PanelPalette {
        theme_registry,
        panel_background,
        header_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let window_effects = current_window_effect_settings(theme_registry);
    let corner_radius = shared_corner_radius(theme_registry);
    let rect = pane_layout.rect;
    let border = if pane_active {
        active_border
    } else {
        border_color
    };
    fill_overlay_surface_rounded_rect(target, rect, corner_radius, border, window_effects)?;
    let inner_rect = PixelRectToRect::rect(
        rect.x() + 1,
        rect.y() + 1,
        rect.width().saturating_sub(2),
        rect.height().saturating_sub(2),
    );
    let inner_radius = corner_radius.saturating_sub(1);
    fill_overlay_surface_rounded_rect(
        target,
        inner_rect,
        inner_radius,
        panel_background,
        window_effects,
    )?;
    let header_height = text_panel_header_height(title, line_height.max(1));
    if header_height > 0 {
        let header_rect = PixelRectToRect::rect(
            rect.x() + 1,
            rect.y() + 1,
            rect.width().saturating_sub(2),
            header_height as u32,
        );
        let header_color = header_background;
        fill_overlay_top_header_band(
            target,
            header_rect,
            inner_radius,
            header_color,
            window_effects,
        )?;
        draw_text(target, rect.x() + 10, rect.y() + 6, title, foreground)?;
    }
    let body_x = rect.x() + 10;
    let body_y = if header_height > 0 {
        rect.y() + header_height + 6
    } else {
        rect.y() + 10
    };
    let mut visual_row = 0usize;
    let line_count = text.line_count();
    let mut cursor_screen: Option<(usize, usize)> = None;
    for line_index in scroll_row.min(line_count.saturating_sub(1))..line_count {
        let line = text.line(line_index).unwrap_or_default();
        let line_len = text.line_len_chars(line_index).unwrap_or(0);
        let char_map = LineCharMap::new(&line);
        let selection_range = visual_selection.and_then(|selection_state| {
            selection_columns_for_visual(selection_state, line_index, line_len)
        });
        let yank_range = yank_flash.and_then(|selection_state| {
            selection_columns_for_visual(selection_state, line_index, line_len)
        });
        let segments = wrap_line_segments(&char_map, pane_layout.wrap_cols, pane_layout.wrap_cols);
        for segment in &segments {
            if visual_row >= pane_layout.visible_rows {
                break;
            }
            let y = body_y + visual_row as i32 * line_height;
            if let Some((selection_start, selection_end)) = selection_range {
                let start = selection_start.max(segment.start_col);
                let end = selection_end.min(segment.end_col);
                if start < end {
                    let start_display = char_map.display_cols_between(segment.start_col, start);
                    let width_display = char_map.display_cols_between(start, end);
                    fill_selection_highlight(
                        target,
                        body_x + (start_display as i32 * cell_width),
                        y,
                        (width_display as i32 * cell_width) as u32,
                        line_height.max(1) as u32,
                        cursor_roundness,
                        selection,
                    )?;
                }
            }
            if let Some((selection_start, selection_end)) = yank_range {
                let start = selection_start.max(segment.start_col);
                let end = selection_end.min(segment.end_col);
                if start < end {
                    let start_display = char_map.display_cols_between(segment.start_col, start);
                    let width_display = char_map.display_cols_between(start, end);
                    fill_selection_highlight(
                        target,
                        body_x + (start_display as i32 * cell_width),
                        y,
                        (width_display as i32 * cell_width) as u32,
                        line_height.max(1) as u32,
                        cursor_roundness,
                        yank_flash_color,
                    )?;
                }
            }
            if cursor_screen.is_none()
                && let Some(cursor_point) = cursor_point
                && cursor_point.line == line_index
                && char_map.cursor_anchor_col(cursor_point.column) >= segment.start_col
                && char_map.cursor_anchor_col(cursor_point.column) <= segment.end_col
            {
                let cursor_col = char_map.cursor_anchor_col(cursor_point.column);
                cursor_screen = Some((
                    visual_row,
                    char_map.display_cols_between(segment.start_col, cursor_col),
                ));
            }
            draw_buffer_text(
                target,
                BufferTextRun {
                    x: body_x,
                    y,
                    line: &line,
                    segment: *segment,
                    char_map: &char_map,
                    line_syntax_spans: syntax_lines
                        .and_then(|lines| lines.get(&line_index))
                        .map(Vec::as_slice),
                    default_color: foreground,
                    cell_width,
                },
                theme_registry,
            )?;
            visual_row = visual_row.saturating_add(1);
        }
        if visual_row >= pane_layout.visible_rows {
            break;
        }
    }
    if let Some((cursor_row, cursor_col)) = cursor_screen
        && pane_active
        && cursor_row < pane_layout.visible_rows
    {
        let cursor_width = match input_mode {
            InputMode::Normal | InputMode::Visual => cell_width.max(2) as u32,
            InputMode::Insert | InputMode::Replace => (cell_width / 4).max(2) as u32,
        };
        fill_rounded_rect(
            target,
            PixelRectToRect::rect(
                body_x + (cursor_col as i32 * cell_width),
                body_y + cursor_row as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ),
            cursor_roundness,
            cursor,
        )?;
    } else if line_count == 0 {
        draw_text(target, body_x, body_y, "", muted)?;
    }
    Ok(())
}

pub(super) fn render_input_panel(
    target: &mut DrawTarget<'_>,
    draw: InputPanelDraw<'_>,
    palette: PanelPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let InputPanelDraw {
        input,
        pane_active,
        pane_layout,
        input_mode,
        window_effects,
        corner_radius,
    } = draw;
    let PanelPalette {
        panel_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        cursor,
        cursor_roundness,
        ..
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let rect = pane_layout.rect;
    let border = if pane_active {
        active_border
    } else {
        border_color
    };
    fill_overlay_surface_rounded_rect(target, rect, corner_radius, border, window_effects)?;
    let inner_rect = PixelRectToRect::rect(
        rect.x() + 1,
        rect.y() + 1,
        rect.width().saturating_sub(2),
        rect.height().saturating_sub(2),
    );
    let inner_radius = corner_radius.saturating_sub(1);
    fill_overlay_surface_rounded_rect(
        target,
        inner_rect,
        inner_radius,
        panel_background,
        window_effects,
    )?;
    let input_x = rect.x() + INPUT_PANEL_VERTICAL_PADDING;
    let input_y = rect.y() + INPUT_PANEL_VERTICAL_PADDING;
    let prompt = input.prompt();
    let prompt_len = prompt.chars().count();
    let prompt_padding = " ".repeat(prompt_len);
    let available_input_cols = pane_layout.wrap_cols.max(prompt_len.saturating_add(1));
    if pane_active && matches!(input_mode, InputMode::Visual) {
        for (row, start_col, end_col) in
            input.selection_visual_ranges(VisualSelectionKind::Character, available_input_cols)
        {
            fill_selection_highlight(
                target,
                input_x + ((prompt_len + start_col) as i32 * cell_width),
                input_y + row as i32 * line_height,
                ((end_col.saturating_sub(start_col)) as i32 * cell_width.max(1)) as u32,
                line_height.max(1) as u32,
                cursor_roundness,
                selection,
            )?;
        }
    }
    let max_visible_rows = visible_input_text_rows(pane_layout.rect.height() as i32, line_height);
    let (visible_rows, first_visible_row) =
        input.visible_wrapped_visual_rows(available_input_cols, max_visible_rows);
    if input.text().is_empty() {
        if let Some(placeholder) = input.placeholder() {
            let line = format!("{prompt}{placeholder}");
            draw_text(target, input_x, input_y, &line, muted)?;
        } else {
            draw_text(target, input_x, input_y, prompt, foreground)?;
        }
    } else {
        for (index, line) in visible_rows.into_iter().enumerate() {
            let prefix = if index == 0 { prompt } else { &prompt_padding };
            let rendered = format!("{prefix}{line}");
            draw_text(
                target,
                input_x,
                input_y + index as i32 * line_height,
                &rendered,
                foreground,
            )?;
        }
    }
    if pane_active && matches!(input_mode, InputMode::Insert | InputMode::Replace) {
        let (input_row, col_in_visual_row) = input.cursor_visual_row_col(available_input_cols);
        let input_row = input_row.saturating_sub(first_visible_row);
        let input_col = prompt_len + col_in_visual_row;
        let cursor_width = (cell_width / 4).max(2) as u32;
        fill_rounded_rect(
            target,
            PixelRectToRect::rect(
                input_x + (input_col as i32 * cell_width),
                input_y + input_row as i32 * line_height,
                cursor_width,
                line_height.max(2) as u32,
            ),
            cursor_roundness,
            cursor,
        )?;
    } else if pane_active && matches!(input_mode, InputMode::Normal | InputMode::Visual) {
        let cursor_char = input.cursor_char();
        let char_count = input.char_count();
        if char_count > 0 {
            let cursor_index = cursor_char.min(char_count.saturating_sub(1));
            let (input_row, col_in_visual_row) =
                input.visual_row_col_for_cursor(cursor_index, available_input_cols);
            let input_row = input_row.saturating_sub(first_visible_row);
            fill_rounded_rect(
                target,
                PixelRectToRect::rect(
                    input_x + ((prompt_len + col_in_visual_row) as i32 * cell_width),
                    input_y + input_row as i32 * line_height,
                    cell_width.max(1) as u32,
                    line_height.max(2) as u32,
                ),
                cursor_roundness,
                cursor,
            )?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AcpPaneLayout {
    pub(super) rect: Rect,
    pub(super) visible_rows: usize,
    pub(super) wrap_cols: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct AcpBufferLayout {
    pub(super) plan: AcpPaneLayout,
    pub(super) output: AcpPaneLayout,
    pub(super) input: TextPaneLayout,
    pub(super) footer: TextPaneLayout,
}

pub(super) fn acp_pane_body_visible_rows(rect_height: u32, line_height: i32) -> usize {
    let line_height = line_height.max(1);
    let header_height = (line_height + 10).max(line_height);
    let body_height = rect_height as i32 - header_height - 4;
    (body_height.max(line_height) / line_height).max(1) as usize
}

pub(super) fn acp_buffer_layout(
    buffer: &ShellBuffer,
    rect: Rect,
    layout: BufferFooterLayout,
    cell_width: i32,
    line_height: i32,
) -> Option<AcpBufferLayout> {
    let state = buffer.acp_state.as_ref()?;
    let line_height = line_height.max(1);
    let panel_x = rect.x() + 8;
    let panel_width = rect.width().saturating_sub(16);
    let gap = 8i32;
    let body_width = panel_width.saturating_sub(20);
    let wrap_cols = overlay_text_columns(body_width, 0, cell_width);
    let desired_input_rows = state
        .input
        .visual_line_count(wrap_cols)
        .clamp(1, MAX_VISIBLE_INPUT_TEXT_ROWS);
    let footer_line_count = state.footer_pane.line_count().max(1);
    let footer_rows = state
        .footer_pane
        .min_rows
        .unwrap_or(footer_line_count)
        .max(footer_line_count);
    let input_chrome = input_panel_chrome_height();
    let footer_chrome = text_panel_chrome_height("", line_height);
    let plan_chrome = text_panel_chrome_height("Plan", line_height);
    let output_chrome = text_panel_chrome_height("Output", line_height);
    let total_height = layout.pane_bottom.saturating_sub(layout.body_y).max(
        plan_chrome + output_chrome + input_chrome + footer_chrome + gap * 3 + line_height * 4,
    );
    let min_top_rows = 2i32;
    let max_input_rows = total_height
        .saturating_sub(
            plan_chrome
                + output_chrome
                + footer_chrome
                + footer_rows as i32 * line_height
                + input_chrome
                + gap * 3
                + min_top_rows * line_height,
        )
        .saturating_div(line_height.max(1))
        .max(1) as usize;
    let input_rows = desired_input_rows.min(max_input_rows);
    let bottom_reserved = input_chrome
        + input_rows as i32 * line_height
        + footer_chrome
        + footer_rows as i32 * line_height;
    let top_height = total_height.saturating_sub(bottom_reserved + gap * 3);
    let total_row_budget = ((top_height - plan_chrome - output_chrome).max(line_height * 2)
        / line_height)
        .max(2) as usize;
    let plan_target_rows = acp_pane_content_rows(&state.plan_pane, wrap_cols).clamp(1, 10);
    let plan_rows = plan_target_rows.min(total_row_budget.saturating_sub(1).max(1));
    let output_rows = total_row_budget.saturating_sub(plan_rows).max(1);
    let used_top_height = plan_chrome
        + output_chrome
        + gap
        + ((plan_rows.saturating_add(output_rows)) as i32 * line_height);
    let output_extra = top_height.saturating_sub(used_top_height);
    let plan_height = plan_chrome + plan_rows as i32 * line_height;
    let output_height = output_chrome + output_rows as i32 * line_height + output_extra;
    let input_height = input_chrome + input_rows as i32 * line_height;
    let footer_height = footer_chrome + footer_rows as i32 * line_height;
    let output_y = layout.body_y + plan_height + gap;
    let input_y = output_y + output_height + gap;
    let footer_y = input_y + input_height + gap;
    Some(AcpBufferLayout {
        plan: AcpPaneLayout {
            rect: Rect::new(panel_x, layout.body_y, panel_width, plan_height as u32),
            visible_rows: acp_pane_body_visible_rows(plan_height as u32, line_height),
            wrap_cols,
        },
        output: AcpPaneLayout {
            rect: Rect::new(panel_x, output_y, panel_width, output_height as u32),
            visible_rows: acp_pane_body_visible_rows(output_height as u32, line_height),
            wrap_cols,
        },
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

pub(super) fn render_acp_buffer_body(
    target: &mut DrawTarget<'_>,
    draw: AcpBufferDraw<'_>,
    palette: BufferBodyPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let AcpBufferDraw {
        buffer,
        rect,
        layout,
        active,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let BufferBodyPalette {
        theme_registry,
        base_background,
        foreground,
        muted,
        border_color,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let Some(state) = buffer.acp_state.as_ref() else {
        return Ok(());
    };
    let window_effects = current_window_effect_settings(theme_registry);
    let Some(acp_layout) = acp_buffer_layout(buffer, rect, layout, cell_width, line_height) else {
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
    let header_background = theme_color(
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
    );
    let active_border = theme_color(theme_registry, TOKEN_STATUSLINE_ACTIVE, cursor);
    let active_pane = state.active_pane;
    let corner_radius = shared_corner_radius(theme_registry);

    render_acp_pane(
        target,
        AcpPaneDraw {
            pane: &state.plan_pane,
            pane_active: active_pane == AcpPane::Plan,
            pane_layout: acp_layout.plan,
            title: "Plan",
            shell_active: active,
            visual_selection: if active_pane == AcpPane::Plan {
                visual_selection
            } else {
                None
            },
            yank_flash: if active_pane == AcpPane::Plan {
                yank_flash
            } else {
                None
            },
            input_mode,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    render_acp_pane(
        target,
        AcpPaneDraw {
            pane: &state.output_pane,
            pane_active: active_pane == AcpPane::Output,
            pane_layout: acp_layout.output,
            title: &acp_output_header_title(state),
            shell_active: active,
            visual_selection: if active_pane == AcpPane::Output {
                visual_selection
            } else {
                None
            },
            yank_flash: if active_pane == AcpPane::Output {
                yank_flash
            } else {
                None
            },
            input_mode,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    render_input_panel(
        target,
        InputPanelDraw {
            input: &state.input,
            pane_active: active && active_pane == AcpPane::Input,
            pane_layout: acp_layout.input,
            input_mode,
            window_effects,
            corner_radius,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    render_text_panel(
        target,
        TextPanelDraw {
            text: &state.footer_pane.text,
            syntax_lines: Some(&state.footer_pane.syntax_lines),
            scroll_row: state.footer_pane.scroll_row,
            cursor_point: (active && active_pane == AcpPane::Footer)
                .then_some(state.footer_pane.cursor()),
            pane_active: active && active_pane == AcpPane::Footer,
            pane_layout: acp_layout.footer,
            title: "",
            visual_selection: if active_pane == AcpPane::Footer {
                visual_selection
            } else {
                None
            },
            yank_flash: if active_pane == AcpPane::Footer {
                yank_flash
            } else {
                None
            },
            input_mode: InputMode::Normal,
        },
        PanelPalette {
            theme_registry,
            panel_background,
            header_background,
            foreground,
            muted,
            border_color,
            active_border,
            selection,
            yank_flash_color,
            cursor,
            cursor_roundness,
        },
        CellMetrics {
            cell_width,
            line_height,
        },
    )?;
    Ok(())
}

pub(super) fn render_acp_pane(
    target: &mut DrawTarget<'_>,
    draw: AcpPaneDraw<'_>,
    palette: PanelPalette<'_>,
    metrics: CellMetrics,
) -> Result<(), ShellError> {
    let AcpPaneDraw {
        pane,
        pane_active,
        pane_layout,
        title,
        shell_active,
        visual_selection,
        yank_flash,
        input_mode,
    } = draw;
    let PanelPalette {
        theme_registry,
        panel_background,
        header_background,
        foreground,
        muted,
        border_color,
        active_border,
        selection,
        yank_flash_color,
        cursor,
        cursor_roundness,
    } = palette;
    let CellMetrics {
        cell_width,
        line_height,
    } = metrics;
    let window_effects = current_window_effect_settings(theme_registry);
    let corner_radius = acp_chat_corner_radius(theme_registry);
    let rect = pane_layout.rect;
    let border = if pane_active {
        active_border
    } else {
        border_color
    };
    fill_overlay_surface_rounded_rect(target, rect, corner_radius, border, window_effects)?;
    let inner_rect = PixelRectToRect::rect(
        rect.x() + 1,
        rect.y() + 1,
        rect.width().saturating_sub(2),
        rect.height().saturating_sub(2),
    );
    let inner_radius = corner_radius.saturating_sub(1);
    fill_overlay_surface_rounded_rect(
        target,
        inner_rect,
        inner_radius,
        panel_background,
        window_effects,
    )?;
    let header_height = (line_height + 10).max(line_height);
    let header_rect = PixelRectToRect::rect(
        rect.x() + 1,
        rect.y() + 1,
        rect.width().saturating_sub(2),
        header_height as u32,
    );
    let header_color = if pane_active {
        blend_color(selection, header_background, 0.45)
    } else {
        header_background
    };
    fill_overlay_top_header_band(
        target,
        header_rect,
        inner_radius,
        header_color,
        window_effects,
    )?;
    draw_text(target, rect.x() + 12, rect.y() + 6, title, foreground)?;
    let body_x = rect.x() + 10;
    let body_y = rect.y() + header_height + 4;
    let body_width = rect.width().saturating_sub(20);
    let max_draw_rows = acp_pane_body_visible_rows(pane_layout.rect.height(), line_height)
        .min(pane_layout.visible_rows);
    let spinner_frame = acp_spinner_frame();
    let cursor_point = pane.cursor();
    let show_text_cursor = pane_active
        && shell_active
        && !matches!(input_mode, InputMode::Insert | InputMode::Replace);
    let mut global_visual_row = 0usize;
    let mut drawn_rows = 0usize;
    let scroll_top = pane.viewport_scroll_top();
    let mut painted_bubble_groups: Vec<u32> = Vec::new();
    for (line_index, rendered_line) in pane.render_lines.iter().enumerate() {
        if drawn_rows >= max_draw_rows {
            break;
        }
        match rendered_line {
            AcpRenderedLine::Text(line) => {
                let prefix_cols = acp_prefix_columns(&line.prefix, spinner_frame);
                let line_len = pane.line_len_chars(line_index);
                let selection_range = visual_selection.and_then(|selection_state| {
                    selection_columns_for_visual(selection_state, line_index, line_len)
                });
                let yank_range = yank_flash.and_then(|selection_state| {
                    selection_columns_for_visual(selection_state, line_index, line_len)
                });
                let segments = acp_rendered_text_segments(line, pane_layout.wrap_cols);
                let cursor_segment = segment_index_for_column(&segments, cursor_point.column);
                let origin_x = acp_chat_origin_x(line, body_x, pane_layout.wrap_cols, cell_width);
                let bubble_width =
                    acp_chat_bubble_width_px(line, pane_layout.wrap_cols, cell_width, body_width);
                for (segment_index, segment) in segments.iter().enumerate() {
                    if global_visual_row < scroll_top {
                        global_visual_row = global_visual_row.saturating_add(1);
                        continue;
                    }
                    if drawn_rows >= max_draw_rows {
                        break;
                    }
                    let y = body_y + drawn_rows as i32 * line_height;
                    if line.bubble
                        && line.bubble_group != 0
                        && !painted_bubble_groups.contains(&line.bubble_group)
                    {
                        painted_bubble_groups.push(line.bubble_group);
                        let remaining = acp_bubble_remaining_rows(
                            pane,
                            line_index,
                            segment_index,
                            line.bubble_group,
                            pane_layout.wrap_cols,
                        );
                        let rows = remaining
                            .min(max_draw_rows.saturating_sub(drawn_rows))
                            .max(1);
                        let fill_role = line.row_fill.unwrap_or(match line.align {
                            AcpChatAlign::End => AcpColorRole::Accent,
                            _ => AcpColorRole::Muted,
                        });
                        let fill = blend_color(
                            panel_background,
                            acp_color(fill_role, theme_registry, foreground, muted, cursor),
                            match line.align {
                                AcpChatAlign::End => 0.24,
                                _ => 0.16,
                            },
                        );
                        fill_overlay_surface_rounded_rect(
                            target,
                            PixelRectToRect::rect(
                                origin_x.saturating_sub(6),
                                y,
                                bubble_width.saturating_add(12),
                                (rows as i32 * line_height).max(line_height) as u32,
                            ),
                            corner_radius.min(18),
                            fill,
                            window_effects,
                        )?;
                    }
                    let segment_x = origin_x + (prefix_cols as i32 * cell_width);
                    if let Some(fill_role) = line.row_fill.filter(|_| !line.bubble) {
                        let fill = blend_color(
                            panel_background,
                            acp_color(fill_role, theme_registry, foreground, muted, cursor),
                            0.22,
                        );
                        fill_overlay_surface_rounded_rect(
                            target,
                            PixelRectToRect::rect(body_x, y, body_width, line_height.max(1) as u32),
                            4,
                            fill,
                            window_effects,
                        )?;
                    }
                    if line.gutter {
                        fill_overlay_surface_rect(
                            target,
                            PixelRectToRect::rect(origin_x + 2, y, 2, line_height.max(1) as u32),
                            muted,
                            window_effects,
                        )?;
                    }
                    if let Some((selection_start, selection_end)) = selection_range {
                        let start = selection_start.max(segment.start_col);
                        let end = selection_end.min(segment.end_col);
                        if start < end {
                            fill_selection_highlight(
                                target,
                                segment_x
                                    + (start.saturating_sub(segment.start_col) as i32 * cell_width),
                                y,
                                (end.saturating_sub(start) as i32 * cell_width) as u32,
                                line_height.max(1) as u32,
                                cursor_roundness,
                                selection,
                            )?;
                        }
                    }
                    if let Some((selection_start, selection_end)) = yank_range {
                        let start = selection_start.max(segment.start_col);
                        let end = selection_end.min(segment.end_col);
                        if start < end {
                            fill_selection_highlight(
                                target,
                                segment_x
                                    + (start.saturating_sub(segment.start_col) as i32 * cell_width),
                                y,
                                (end.saturating_sub(start) as i32 * cell_width) as u32,
                                line_height.max(1) as u32,
                                cursor_roundness,
                                yank_flash_color,
                            )?;
                        }
                    }
                    if show_text_cursor
                        && cursor_point.line == line_index
                        && cursor_segment == segment_index
                    {
                        let cursor_x = origin_x
                            + (prefix_cols as i32 * cell_width)
                            + (cursor_point.column.saturating_sub(segment.start_col) as i32
                                * cell_width);
                        let cursor_width = match input_mode {
                            InputMode::Normal | InputMode::Visual => cell_width.max(2) as u32,
                            InputMode::Insert | InputMode::Replace => {
                                (cell_width / 4).max(2) as u32
                            }
                        };
                        fill_rounded_rect(
                            target,
                            PixelRectToRect::rect(
                                cursor_x,
                                y,
                                cursor_width,
                                line_height.max(2) as u32,
                            ),
                            cursor_roundness,
                            cursor,
                        )?;
                    }
                    if segment_index == 0 {
                        acp_draw_prefix_segments(
                            target,
                            AcpPrefixDraw {
                                x: origin_x,
                                y,
                                segments: &line.prefix,
                                spinner_frame,
                                theme_registry,
                                foreground,
                                muted,
                                accent: cursor,
                                cell_width,
                            },
                        )?;
                    }
                    let segment_text =
                        acp_slice_chars(&line.text, segment.start_col, segment.end_col);
                    let default_color =
                        acp_color(line.text_role, theme_registry, foreground, muted, cursor);
                    if line.syntax_spans.is_empty() {
                        draw_text(target, segment_x, y, &segment_text, default_color)?;
                    } else {
                        let char_map = LineCharMap::new(&line.text);
                        draw_buffer_text(
                            target,
                            BufferTextRun {
                                x: segment_x,
                                y,
                                line: &line.text,
                                segment: *segment,
                                char_map: &char_map,
                                line_syntax_spans: Some(line.syntax_spans.as_slice()),
                                default_color,
                                cell_width,
                            },
                            theme_registry,
                        )?;
                    }
                    drawn_rows = drawn_rows.saturating_add(1);
                    global_visual_row = global_visual_row.saturating_add(1);
                }
            }
            AcpRenderedLine::Image(image) => {
                let image_rows = image.rows.max(1);
                if global_visual_row.saturating_add(image_rows) <= scroll_top {
                    global_visual_row = global_visual_row.saturating_add(image_rows);
                    continue;
                }
                if drawn_rows >= max_draw_rows {
                    break;
                }
                let remaining_rows = max_draw_rows.saturating_sub(drawn_rows);
                let image_rows_draw = image.rows.min(remaining_rows).max(1);
                let y = body_y + drawn_rows as i32 * line_height;
                let image_rect = PixelRectToRect::rect(
                    body_x,
                    y,
                    body_width,
                    (image_rows_draw as i32 * line_height).max(line_height) as u32,
                );
                fill_window_surface_rounded_rect(
                    target,
                    image_rect,
                    8,
                    adjust_color(
                        panel_background,
                        if is_dark_color(panel_background) {
                            10
                        } else {
                            -10
                        },
                    ),
                    window_effects,
                )?;
                draw_text(target, body_x + 8, y + 6, &image.label, muted)?;
                if let Some(decoded) = image.image.as_ref() {
                    let top = y + line_height;
                    let height = image_rect
                        .height()
                        .saturating_sub(line_height.max(1) as u32)
                        .saturating_sub(8);
                    if height > 0 {
                        let width = body_width.saturating_sub(12);
                        let draw_height = height;
                        draw_image(
                            target,
                            PixelRectToRect::rect(body_x + 6, top + 2, width, draw_height),
                            decoded.width,
                            decoded.height,
                            Arc::clone(&decoded.pixels),
                            Some(image_rect),
                        )?;
                    }
                }
                drawn_rows = drawn_rows.saturating_add(image_rows_draw);
                global_visual_row = global_visual_row.saturating_add(image_rows);
            }
            AcpRenderedLine::Spacer => {
                if global_visual_row < scroll_top {
                    global_visual_row = global_visual_row.saturating_add(1);
                    continue;
                }
                if drawn_rows >= max_draw_rows {
                    break;
                }
                drawn_rows = drawn_rows.saturating_add(1);
                global_visual_row = global_visual_row.saturating_add(1);
            }
        }
    }
    Ok(())
}

const ACP_CHAT_ROUNDED_OPTION: &str = "acp.chat.rounded";

pub(super) fn acp_chat_rounded(theme_registry: Option<&ThemeRegistry>) -> bool {
    theme_registry
        .and_then(|registry| registry.resolve_bool(ACP_CHAT_ROUNDED_OPTION))
        .unwrap_or(true)
}

pub(super) fn acp_chat_corner_radius(theme_registry: Option<&ThemeRegistry>) -> u32 {
    if acp_chat_rounded(theme_registry) {
        shared_corner_radius(theme_registry)
    } else {
        0
    }
}

fn acp_chat_origin_x(
    line: &AcpRenderedTextLine,
    body_x: i32,
    wrap_cols: usize,
    cell_width: i32,
) -> i32 {
    match line.align {
        AcpChatAlign::End => {
            let bubble_cols = acp_chat_bubble_cols(wrap_cols);
            let shift = wrap_cols.saturating_sub(bubble_cols) as i32 * cell_width.max(1);
            body_x + shift
        }
        AcpChatAlign::Start | AcpChatAlign::Full => body_x,
    }
}

fn acp_chat_bubble_width_px(
    line: &AcpRenderedTextLine,
    wrap_cols: usize,
    cell_width: i32,
    body_width: u32,
) -> u32 {
    match line.align {
        AcpChatAlign::Full => body_width,
        AcpChatAlign::Start | AcpChatAlign::End => {
            (acp_chat_bubble_cols(wrap_cols) as i32 * cell_width.max(1)).max(1) as u32
        }
    }
}

fn acp_bubble_remaining_rows(
    pane: &AcpPaneState,
    line_index: usize,
    segment_index: usize,
    bubble_group: u32,
    wrap_cols: usize,
) -> usize {
    let mut rows = 0usize;
    for (index, rendered_line) in pane.render_lines.iter().enumerate().skip(line_index) {
        let AcpRenderedLine::Text(line) = rendered_line else {
            break;
        };
        if !line.bubble || line.bubble_group != bubble_group {
            break;
        }
        let segments = acp_rendered_text_segments(line, wrap_cols);
        if index == line_index {
            rows = rows.saturating_add(segments.len().saturating_sub(segment_index).max(1));
        } else {
            rows = rows.saturating_add(segments.len().max(1));
        }
    }
    rows.max(1)
}

pub(super) fn acp_draw_prefix_segments(
    target: &mut DrawTarget<'_>,
    draw: AcpPrefixDraw<'_>,
) -> Result<(), ShellError> {
    let AcpPrefixDraw {
        x,
        y,
        segments,
        spinner_frame,
        theme_registry,
        foreground,
        muted,
        accent,
        cell_width,
    } = draw;
    let mut draw_x = x;
    for segment in segments {
        let text = if segment.animate {
            spinner_frame
        } else {
            segment.text.as_str()
        };
        let color = acp_color(segment.role, theme_registry, foreground, muted, accent);
        draw_text(target, draw_x, y, text, color)?;
        draw_x += monospace_text_width(text, cell_width) as i32;
    }
    Ok(())
}

pub(super) fn acp_prefix_columns(segments: &[AcpRenderedSegment], spinner_frame: &str) -> usize {
    segments
        .iter()
        .map(|segment| {
            if segment.animate {
                spinner_frame.chars().count()
            } else {
                segment.text.chars().count()
            }
        })
        .sum()
}

pub(super) fn acp_color(
    role: AcpColorRole,
    theme_registry: Option<&ThemeRegistry>,
    foreground: Color,
    muted: Color,
    accent: Color,
) -> Color {
    match role {
        AcpColorRole::Default => foreground,
        AcpColorRole::Muted => muted,
        AcpColorRole::Accent => accent,
        AcpColorRole::Success => theme_color(
            theme_registry,
            "git.status.entry.added",
            Color::RGB(108, 193, 118),
        ),
        AcpColorRole::Warning => theme_color(
            theme_registry,
            "ui.notification.warning",
            Color::RGB(209, 154, 102),
        ),
        AcpColorRole::Error => theme_color(
            theme_registry,
            "ui.notification.error",
            Color::RGB(224, 107, 117),
        ),
        AcpColorRole::PriorityHigh => theme_color(
            theme_registry,
            "ui.notification.error",
            Color::RGB(224, 107, 117),
        ),
        AcpColorRole::PriorityMedium => theme_color(
            theme_registry,
            "ui.notification.warning",
            Color::RGB(209, 154, 102),
        ),
        AcpColorRole::PriorityLow => theme_color(
            theme_registry,
            "ui.notification.info",
            Color::RGB(110, 170, 255),
        ),
    }
}

pub(super) fn acp_spinner_frame() -> &'static str {
    const FRAMES: [&str; 6] = ["◜", "◠", "◝", "◞", "◡", "◟"];
    let frame = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| ((duration.as_millis() / 120) % FRAMES.len() as u128) as usize)
        .unwrap_or(0);
    FRAMES[frame]
}

pub(super) fn acp_slice_chars(text: &str, start: usize, end: usize) -> String {
    if start >= end {
        return String::new();
    }
    let mut start_byte = text.len();
    let mut end_byte = text.len();
    let mut seen = 0usize;
    for (index, character) in text.char_indices() {
        if seen == start {
            start_byte = index;
        }
        if seen == end {
            end_byte = index;
            break;
        }
        seen = seen.saturating_add(1);
        if seen == text.chars().count() {
            end_byte = text.len();
        }
        let _ = character;
    }
    if start == 0 {
        start_byte = 0;
    }
    if end >= text.chars().count() {
        end_byte = text.len();
    }
    text.get(start_byte..end_byte)
        .unwrap_or_default()
        .to_owned()
}

pub(super) fn block_cursor_text_overlay(
    query: CursorOverlayQuery<'_>,
) -> Option<CursorTextOverlay> {
    let CursorOverlayQuery {
        x,
        line,
        char_map,
        segment,
        line_index,
        cursor,
        color,
        cell_width,
    } = query;
    let cursor_col = char_map.cursor_anchor_col(cursor.column);
    let color = color?;
    if line_index != cursor.line || cursor_col < segment.start_col || cursor_col >= segment.end_col
    {
        return None;
    }
    if cursor_col >= char_map.len() {
        return None;
    }
    let text = char_map.display_text_for_range(line, cursor_col, cursor_col.saturating_add(1));
    (!text.is_empty()).then_some(CursorTextOverlay {
        draw_x: x
            + (char_map.display_cols_between(segment.start_col, cursor_col) as i32 * cell_width),
        text,
        color,
    })
}

pub(super) fn draw_buffer_text(
    target: &mut DrawTarget<'_>,
    run: BufferTextRun<'_>,
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(), ShellError> {
    let BufferTextRun {
        x,
        y,
        line,
        segment,
        char_map,
        line_syntax_spans,
        default_color,
        cell_width,
    } = run;
    let segment_end_col = segment.end_col.min(char_map.len());
    let segment_start_col = segment.start_col.min(segment_end_col);
    let segment_text = char_map.slice(line, segment_start_col, segment_end_col);
    let segment_base_byte = char_map
        .bytes
        .get(segment_start_col)
        .copied()
        .unwrap_or_default();
    let segment_byte_offsets = &char_map.bytes[segment_start_col..=segment_end_col];
    let mut clipped_spans = Vec::new();
    if let Some(line_syntax_spans) = line_syntax_spans {
        for span in line_syntax_spans {
            let start = span.start.max(segment.start_col);
            let end = span.end.min(segment.end_col);
            if start < end {
                clipped_spans.push(LineSyntaxSpan {
                    start: start - segment.start_col,
                    end: end - segment.start_col,
                    capture_name: span.capture_name.clone(),
                    theme_token: span.theme_token.clone(),
                });
            }
        }
    }
    let clipped_spans = if clipped_spans.is_empty() {
        None
    } else {
        Some(clipped_spans.as_slice())
    };

    let mut draw_x = x;
    let mut segment_char_offset = 0usize;
    for (colored_segment, color, style) in line_color_segments(
        segment_text,
        clipped_spans,
        theme_registry,
        default_color,
        segment_byte_offsets,
        segment_base_byte,
    ) {
        let colored_segment_chars = colored_segment.chars().count();
        if colored_segment.is_empty() {
            segment_char_offset = segment_char_offset.saturating_add(colored_segment_chars);
            continue;
        }
        let rendered_segment = char_map.display_text_for_range(
            line,
            segment_start_col + segment_char_offset,
            segment_start_col + segment_char_offset + colored_segment_chars,
        );
        if style == TextStyle::plain() {
            draw_text(target, draw_x, y, &rendered_segment, color)?;
        } else {
            let mut character_x = draw_x;
            for character in rendered_segment.chars() {
                let character_text = character.to_string();
                draw_styled_text(target, character_x, y, &character_text, color, style)?;
                character_x += monospace_text_width(&character_text, cell_width) as i32;
            }
        }
        draw_x += monospace_text_width(&rendered_segment, cell_width) as i32;
        segment_char_offset = segment_char_offset.saturating_add(colored_segment_chars);
    }
    Ok(())
}

pub(super) struct GhostTextSegmentDraw<'a> {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) segment: LineWrapSegment,
    pub(super) char_map: &'a LineCharMap,
    pub(super) line_len: usize,
    pub(super) ghost_text: Option<&'a str>,
    pub(super) color: Color,
    pub(super) cell_width: i32,
}

pub(super) fn draw_line_ghost_text_for_segment(
    target: &mut DrawTarget<'_>,
    draw: GhostTextSegmentDraw<'_>,
) -> Result<(), ShellError> {
    let Some(ghost_text) = draw.ghost_text.filter(|text| !text.is_empty()) else {
        return Ok(());
    };
    let visible_end = draw.segment.end_col.min(draw.line_len);
    if visible_end < draw.line_len {
        return Ok(());
    }
    let visible_cols = draw
        .char_map
        .display_cols_between(draw.segment.start_col, visible_end);
    // Leave one monospace cell between the closing delimiter and the ghost text.
    let draw_x = draw.x + visible_cols as i32 * draw.cell_width + draw.cell_width;
    draw_text(target, draw_x, draw.y, ghost_text, draw.color)
}

fn headerline_max_rows(visible_rows: usize) -> usize {
    visible_rows.saturating_sub(1)
}

pub(super) fn visible_headerline_row_count(lines: &[String], visible_rows: usize) -> usize {
    let max_rows = headerline_max_rows(visible_rows);
    if max_rows == 0 {
        return 0;
    }
    lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .count()
        .min(max_rows)
}

pub(super) fn visible_headerline_lines(lines: &[String], visible_rows: usize) -> Vec<&str> {
    let max_rows = headerline_max_rows(visible_rows);
    if max_rows == 0 {
        return Vec::new();
    }
    let mut kept: Vec<&str> = lines
        .iter()
        .map(String::as_str)
        .filter(|line| !line.trim().is_empty())
        .collect();
    let extra = kept.len().saturating_sub(max_rows);
    if extra > 0 {
        kept.drain(..extra);
    }
    kept
}

pub(super) fn line_color_segments(
    line: &str,
    line_syntax_spans: Option<&[LineSyntaxSpan]>,
    theme_registry: Option<&ThemeRegistry>,
    default_color: Color,
    column_byte_offsets: &[usize],
    base_byte: usize,
) -> Vec<(String, Color, TextStyle)> {
    let Some(line_syntax_spans) = line_syntax_spans else {
        return vec![(line.to_owned(), default_color, TextStyle::plain())];
    };

    let relevant_spans = line_syntax_spans
        .iter()
        .filter_map(|span| {
            let start = column_to_relative_byte_offset(
                line.len(),
                column_byte_offsets,
                base_byte,
                span.start,
            );
            let end = column_to_relative_byte_offset(
                line.len(),
                column_byte_offsets,
                base_byte,
                span.end,
            );
            if start >= end {
                return None;
            }

            Some((start, end, span.theme_token.as_ref()))
        })
        .collect::<Vec<_>>();
    if relevant_spans.is_empty() {
        return vec![(line.to_owned(), default_color, TextStyle::plain())];
    }

    let mut breakpoints = vec![0, line.len()];
    for (start, end, _) in &relevant_spans {
        breakpoints.push(*start);
        breakpoints.push(*end);
    }
    breakpoints.sort_unstable();
    breakpoints.dedup();

    let mut segments = Vec::new();
    for window in breakpoints.windows(2) {
        let start = window[0];
        let end = window[1];
        if start >= end {
            continue;
        }
        let Some(text) = line.get(start..end) else {
            continue;
        };
        let token_style = relevant_spans
            .iter()
            .filter(|(span_start, span_end, _)| start >= *span_start && end <= *span_end)
            .min_by_key(|(span_start, span_end, token)| {
                (
                    span_end.saturating_sub(*span_start),
                    usize::from(!theme_token_has_render_priority(token)),
                )
            })
            .and_then(|(_, _, token)| {
                theme_registry.and_then(|registry| registry.resolve_style(token))
            });
        let color = token_style
            .map(|token_style| to_sdl_color(token_style.color))
            .unwrap_or(default_color);
        let style = token_style
            .map(|token_style| text_style_from_theme_style(token_style.style))
            .unwrap_or_else(TextStyle::plain);
        segments.push((text.to_owned(), color, style));
    }

    if segments.is_empty() {
        vec![(line.to_owned(), default_color, TextStyle::plain())]
    } else {
        segments
    }
}

fn theme_token_has_render_priority(token: &str) -> bool {
    token.starts_with("rainbow.paren.")
}

pub(super) fn column_to_relative_byte_offset(
    line_len: usize,
    column_byte_offsets: &[usize],
    base_byte: usize,
    column: usize,
) -> usize {
    column_byte_offsets
        .get(column)
        .copied()
        .unwrap_or_else(|| base_byte.saturating_add(line_len))
        .saturating_sub(base_byte)
        .min(line_len)
}

pub(super) fn draw_diagnostic_undercurl(
    target: &mut DrawTarget<'_>,
    x: i32,
    y: i32,
    width: i32,
    line_height: i32,
    color: Color,
) -> Result<(), ShellError> {
    if width <= 0 || line_height <= 0 {
        return Ok(());
    }
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::Undercurl {
            x,
            y,
            width: width as u32,
            line_height: line_height as u32,
            color: to_render_color(color),
        }),
    }
    Ok(())
}

pub(super) fn selection_columns_for_line(
    range: TextRange,
    line_index: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    let range = range.normalized();
    if line_index < range.start().line || line_index > range.end().line {
        return None;
    }

    let start = if line_index == range.start().line {
        range.start().column
    } else {
        0
    };
    let end = if line_index == range.end().line {
        range.end().column
    } else {
        line_len
    };
    let start = start.min(line_len);
    let end = end.min(line_len);
    (start < end).then_some((start, end))
}

pub(super) fn selection_columns_for_visual(
    selection: VisualSelection,
    line_index: usize,
    line_len: usize,
) -> Option<(usize, usize)> {
    match selection {
        VisualSelection::Range(range) => selection_columns_for_line(range, line_index, line_len),
        VisualSelection::Block(block) => {
            if line_index < block.start_line || line_index > block.end_line {
                return None;
            }
            let start = block.start_col.min(line_len);
            let end = block.end_col.min(line_len);
            (start < end).then_some((start, end))
        }
    }
}

pub(super) fn multicursor_ranges_for_line(
    state: &MulticursorState,
    input_mode: InputMode,
    line_index: usize,
    line_len: usize,
) -> Vec<(usize, usize)> {
    let Some((start_offset, end_offset)) = multicursor_selection_offsets(state, input_mode) else {
        return Vec::new();
    };
    let start_text = state
        .match_text
        .chars()
        .take(start_offset)
        .collect::<String>();
    let end_text = state
        .match_text
        .chars()
        .take(end_offset)
        .collect::<String>();
    state
        .ranges
        .iter()
        .filter_map(|range| {
            selection_columns_for_visual(
                VisualSelection::Range(TextRange::new(
                    advance_point_by_text(range.start(), &start_text),
                    advance_point_by_text(range.start(), &end_text),
                )),
                line_index,
                line_len,
            )
        })
        .collect()
}

pub(super) fn multicursor_cursor_points(state: &MulticursorState) -> Vec<TextPoint> {
    let prefix = state
        .match_text
        .chars()
        .take(state.cursor_offset)
        .collect::<String>();
    state
        .ranges
        .iter()
        .map(|range| advance_point_by_text(range.start(), &prefix))
        .collect()
}

fn relative_byte_column_to_char_column(line: &str, byte_column: usize) -> usize {
    let mut bytes = 0usize;
    let mut chars = 0usize;
    for character in line.chars() {
        if bytes >= byte_column {
            break;
        }
        bytes = bytes.saturating_add(character.len_utf8());
        chars = chars.saturating_add(1);
    }
    chars
}

pub(super) fn index_syntax_lines(
    snapshot: SyntaxSnapshot,
    text: &TextBuffer,
) -> IndexedSyntaxLines {
    let mut syntax_lines = BTreeMap::new();
    for span in snapshot.highlight_spans {
        let start_line = span.start_position.line;
        let end_line = span.end_position.line;
        let mut capture_name = span.capture_name;
        let mut theme_token = span.theme_token;
        for line_index in start_line..=end_line {
            let Some(line_text) = text.line(line_index) else {
                continue;
            };
            let Some(line_start_byte) = text.line_start_byte(line_index) else {
                continue;
            };
            let start_byte = if line_index == start_line {
                span.start_byte.saturating_sub(line_start_byte)
            } else {
                0
            };
            let end_byte = if line_index == end_line {
                span.end_byte.saturating_sub(line_start_byte)
            } else {
                line_text.len()
            };
            let start =
                relative_byte_column_to_char_column(&line_text, start_byte.min(line_text.len()));
            let end =
                relative_byte_column_to_char_column(&line_text, end_byte.min(line_text.len()));
            if start >= end {
                continue;
            }
            syntax_lines
                .entry(line_index)
                .or_insert_with(Vec::new)
                .push(LineSyntaxSpan {
                    start,
                    end,
                    capture_name: if line_index == end_line {
                        std::mem::take(&mut capture_name)
                    } else {
                        capture_name.clone()
                    },
                    theme_token: if line_index == end_line {
                        std::mem::take(&mut theme_token)
                    } else {
                        theme_token.clone()
                    },
                });
        }
    }

    syntax_lines
}

pub(super) fn clamp_to_char_boundary(text: &str, index: usize) -> usize {
    let mut clamped = index.min(text.len());
    while clamped > 0 && !text.is_char_boundary(clamped) {
        clamped -= 1;
    }
    clamped
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FontRole {
    Primary,
    Icon(usize),
    Emoji,
}

#[derive(Debug, Clone)]
pub(super) struct FontRun {
    role: FontRole,
    text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PrimaryTextRenderMode {
    Normal,
    Ligature,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PrimaryTextRun {
    pub(super) render_mode: PrimaryTextRenderMode,
    pub(super) text: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct ShapedGlyph {
    pub(super) cluster: usize,
    pub(super) glyph_id: u16,
    pub(super) x_advance: f32,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ShapedRun {
    pub(super) glyphs: Vec<ShapedGlyph>,
    pub(super) total_advance: f32,
}

pub(super) fn shaped_run_uses_cell_grid(text: &str, shaped: &ShapedRun) -> bool {
    shaped.glyphs.len() == text.chars().count()
}

pub(super) fn shaped_run_preserves_monospace_layout(
    text: &str,
    shaped: &ShapedRun,
    cell_width: i32,
) -> bool {
    shaped_run_uses_cell_grid(text, shaped)
        || (shaped.total_advance - monospace_text_width(text, cell_width) as f32).abs() <= 1.0
}

pub(super) fn is_private_use_character(character: char) -> bool {
    matches!(
        character as u32,
        0xE000..=0xF8FF | 0xF0000..=0xFFFFD | 0x100000..=0x10FFFD
    )
}

pub(super) fn is_symbol_like_character(character: char) -> bool {
    matches!(
        character as u32,
        0x2190..=0x21FF
            | 0x2300..=0x23FF
            | 0x2500..=0x257F
            | 0x2580..=0x259F
            | 0x25A0..=0x25FF
            | 0x2600..=0x27BF
            | 0x2B00..=0x2BFF
    )
}

pub(super) fn is_emoji_character(character: char) -> bool {
    matches!(
        character as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x27BF | 0xFE00..=0xFE0F
    )
}

pub(super) fn resolve_font_role_for_char(
    icon_font_index: Option<usize>,
    primary_has_glyph: bool,
    prefers_icon_font: bool,
    emoji_has_glyph: bool,
    character: char,
) -> FontRole {
    // Emoji characters get their own font role when available
    if emoji_has_glyph && is_emoji_character(character) {
        return FontRole::Emoji;
    }
    if let Some(index) = icon_font_index
        && (prefers_icon_font
            || is_private_use_character(character)
            || is_symbol_like_character(character))
    {
        return FontRole::Icon(index);
    }
    if primary_has_glyph {
        return FontRole::Primary;
    }
    icon_font_index
        .map(FontRole::Icon)
        .unwrap_or(FontRole::Primary)
}

pub(super) fn font_role_for_char(fonts: &FontSet<'_>, character: char) -> FontRole {
    resolve_font_role_for_char(
        fonts.icon_font_index_for_char(character),
        fonts.primary().find_glyph(character).is_some(),
        fonts.prefers_icon_font(character),
        fonts.emoji_font_has_char(character),
        character,
    )
}

pub(super) fn font_runs(text: &str, fonts: &FontSet<'_>) -> Vec<FontRun> {
    if text.is_empty() {
        return Vec::new();
    }
    if fonts.icon_fonts().is_empty() || text.is_ascii() {
        return vec![FontRun {
            role: FontRole::Primary,
            text: text.to_owned(),
        }];
    }
    let mut runs = Vec::new();
    let mut current_role = FontRole::Primary;
    let mut current_text = String::new();
    for character in text.chars() {
        if is_zero_width_display_character(character) {
            if current_role == FontRole::Emoji && !current_text.is_empty() {
                current_text.push(character);
            }
            continue;
        }
        let next_role = font_role_for_char(fonts, character);
        if next_role != current_role && !current_text.is_empty() {
            runs.push(FontRun {
                role: current_role,
                text: std::mem::take(&mut current_text),
            });
        }
        current_role = next_role;
        current_text.push(character);
    }
    if !current_text.is_empty() {
        runs.push(FontRun {
            role: current_role,
            text: current_text,
        });
    }
    runs
}

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

pub(super) fn fill_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    color: Color,
) -> Result<(), ShellError> {
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::FillRect {
            rect: to_pixel_rect(rect),
            color: to_render_color(color),
        }),
    }
    Ok(())
}

/// Extra right-side pixels so selection glyphs are not flush against the highlight edge.
const SELECTION_HIGHLIGHT_RIGHT_PAD_PX: u32 = 2;

pub(super) fn fill_selection_highlight(
    target: &mut DrawTarget<'_>,
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    fill_rounded_rect(
        target,
        PixelRectToRect::rect(
            x,
            y,
            width.saturating_add(SELECTION_HIGHLIGHT_RIGHT_PAD_PX),
            height,
        ),
        radius,
        color,
    )
}

pub(super) fn window_surface_color(color: Color, window_effects: WindowEffects) -> Color {
    let alpha = (f32::from(color.a) * crate::window_effects::window_surface_opacity(window_effects))
        .round()
        .clamp(0.0, 255.0) as u8;
    Color::RGBA(color.r, color.g, color.b, alpha)
}

pub(super) fn overlay_window_surface_color(color: Color, window_effects: WindowEffects) -> Color {
    let alpha = (f32::from(color.a)
        * crate::window_effects::overlay_window_surface_opacity(window_effects))
    .round()
    .clamp(0.0, 255.0) as u8;
    Color::RGBA(color.r, color.g, color.b, alpha)
}

pub(super) fn clear_window_surface(
    target: &mut DrawTarget<'_>,
    color: Color,
    window_effects: WindowEffects,
) {
    target.clear(window_surface_color(color, window_effects));
}

pub(super) fn fill_window_surface_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rect(target, rect, window_surface_color(color, window_effects))
}

pub(super) fn fill_overlay_surface_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rect(
        target,
        rect,
        overlay_window_surface_color(color, window_effects),
    )
}

fn paint_buffer_scrollbar(
    target: &mut DrawTarget<'_>,
    paint: ScrollbarPaint,
) -> Result<(), ShellError> {
    let ScrollbarPaint {
        pane_rect,
        body_y,
        visible_rows,
        line_height,
        scroll_row,
        max_scroll,
        color,
        window_effects,
    } = paint;
    if max_scroll == 0 {
        return Ok(());
    }
    let track_height = (visible_rows as i32 * line_height.max(1)).max(1) as u32;
    let thumb_height = ((track_height as u64 * visible_rows as u64)
        / (visible_rows as u64 + max_scroll as u64))
        .clamp(12, u64::from(track_height)) as u32;
    let travel = track_height.saturating_sub(thumb_height);
    let thumb_y = body_y + ((travel as u64 * scroll_row as u64) / max_scroll.max(1) as u64) as i32;
    fill_window_surface_rounded_rect(
        target,
        PixelRectToRect::rect(
            pane_rect.x() + pane_rect.width() as i32 - 8,
            thumb_y,
            4,
            thumb_height,
        ),
        2,
        Color::RGBA(color.r, color.g, color.b, 120),
        window_effects,
    )
}

pub(super) fn fill_rounded_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    match target {
        DrawTarget::Scene(scene) => scene.push(DrawCommand::FillRoundedRect {
            rect: to_pixel_rect(rect),
            radius,
            color: to_render_color(color),
        }),
    }
    Ok(())
}

pub(super) fn fill_window_surface_rounded_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rounded_rect(
        target,
        rect,
        radius,
        window_surface_color(color, window_effects),
    )
}

pub(super) fn fill_overlay_surface_rounded_rect(
    target: &mut DrawTarget<'_>,
    rect: Rect,
    radius: u32,
    color: Color,
    window_effects: WindowEffects,
) -> Result<(), ShellError> {
    fill_rounded_rect(
        target,
        rect,
        radius,
        overlay_window_surface_color(color, window_effects),
    )
}

pub(super) fn fill_rounded_rect_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    rect: Rect,
    radius: u32,
    color: Color,
) -> Result<(), ShellError> {
    let radius = radius.min(rect.width() / 2).min(rect.height() / 2) as i32;
    if radius <= 1 {
        canvas.set_draw_color(color);
        return canvas
            .fill_rect(rect)
            .map_err(|error| ShellError::Sdl(error.to_string()));
    }

    let previous_blend_mode = canvas.blend_mode();
    canvas.set_blend_mode(sdl3::render::BlendMode::Blend);
    let rect_height = rect.height() as i32;
    let rect_width = rect.width() as i32;

    let result = (|| {
        for row in 0..rect_height {
            let (inset, edge_alpha) =
                rounded_rect_row_coverage(row, rect_height, rect_width, radius, color.a);
            let width = rect_width - (inset * 2);
            if width > 0 {
                canvas.set_draw_color(color);
                canvas
                    .fill_rect(Rect::new(rect.x() + inset, rect.y() + row, width as u32, 1))
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
            }

            if edge_alpha > 0 && inset > 0 {
                let edge_color = Color::RGBA(color.r, color.g, color.b, edge_alpha);
                canvas.set_draw_color(edge_color);
                canvas
                    .fill_rect(Rect::new(rect.x() + inset - 1, rect.y() + row, 1, 1))
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
                canvas
                    .fill_rect(Rect::new(
                        rect.x() + rect_width - inset,
                        rect.y() + row,
                        1,
                        1,
                    ))
                    .map_err(|error| ShellError::Sdl(error.to_string()))?;
            }
        }
        Ok(())
    })();

    canvas.set_blend_mode(previous_blend_mode);
    result
}

fn rounded_rect_row_coverage(
    row: i32,
    rect_height: i32,
    rect_width: i32,
    radius: i32,
    alpha: u8,
) -> (i32, u8) {
    // Measure from pixel centers so the scanline coverage matches the blended edge pixels.
    let corner_distance = if row < radius {
        radius as f32 - row as f32 - 0.5
    } else if row >= rect_height - radius {
        row as f32 - (rect_height - radius) as f32 + 0.5
    } else {
        return (0, 0);
    };
    let radius_f = radius as f32;
    let inset = radius_f - (radius_f * radius_f - corner_distance * corner_distance).sqrt();
    let full_inset = inset.ceil() as i32;
    let full_inset = full_inset.clamp(0, rect_width / 2);
    let coverage = (full_inset as f32 - inset).clamp(0.0, 1.0);
    (full_inset, scaled_coverage_alpha(coverage, alpha))
}

fn scaled_coverage_alpha(coverage: f32, alpha: u8) -> u8 {
    ((coverage * f32::from(alpha)).round()) as u8
}

pub(super) fn draw_undercurl_canvas<T: RenderTarget>(
    canvas: &mut Canvas<T>,
    x: i32,
    y: i32,
    width: u32,
    line_height: u32,
    color: Color,
) -> Result<(), ShellError> {
    if width == 0 || line_height == 0 {
        return Ok(());
    }

    let baseline_y = y + line_height as i32 - 2;
    let upper_y = baseline_y.saturating_sub(1);
    let width = width as i32;
    let mut points = Vec::with_capacity(width.max(1) as usize);
    let mut dx = 0i32;
    while dx < width {
        let segment_width = (width - dx).min(2);
        let segment_y = if (dx / 2) % 2 == 0 {
            baseline_y
        } else {
            upper_y
        };
        for offset in 0..segment_width {
            points.push(FPoint::new((x + dx + offset) as f32, segment_y as f32));
        }
        dx += 2;
    }

    canvas.set_draw_color(color);
    canvas
        .draw_points(points.as_slice())
        .map_err(|error| ShellError::Sdl(error.to_string()))
}

pub(super) fn truncate_text_to_width(text: &str, max_width: u32, cell_width: i32) -> String {
    if text.is_empty() || max_width == 0 {
        return String::new();
    }

    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    if text.chars().count() <= max_cells {
        return text.to_owned();
    }

    let ellipsis = "...";
    let ellipsis_cells = ellipsis.chars().count();
    if max_cells <= ellipsis_cells {
        return "...".to_owned();
    }

    let mut truncated = String::new();
    let available_cells = max_cells.saturating_sub(ellipsis_cells);
    for character in text.chars() {
        if truncated.chars().count() >= available_cells {
            break;
        }
        truncated.push(character);
    }

    truncated.push_str(ellipsis);
    truncated
}

pub(super) fn truncate_text_to_width_preserving_end(
    text: &str,
    max_width: u32,
    cell_width: i32,
) -> String {
    if text.is_empty() || max_width == 0 {
        return String::new();
    }

    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    if text.chars().count() <= max_cells {
        return text.to_owned();
    }

    let ellipsis = "...";
    let ellipsis_cells = ellipsis.chars().count();
    if max_cells <= ellipsis_cells {
        return "...".to_owned();
    }

    let available_cells = max_cells.saturating_sub(ellipsis_cells);
    let suffix = text
        .chars()
        .rev()
        .take(available_cells)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{ellipsis}{suffix}")
}

const PICKER_PATH_TAIL_KEEP: usize = 3;

fn text_fits_width(text: &str, max_width: u32, cell_width: i32) -> bool {
    if text.is_empty() || max_width == 0 {
        return true;
    }
    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    text.chars().count() <= max_cells
}

fn path_separator_for(path: &str) -> char {
    if path.contains('\\') && !path.contains('/') {
        '\\'
    } else {
        '/'
    }
}

fn is_path_like(text: &str) -> bool {
    text.contains('/') || text.contains('\\')
}

fn shrink_directory_name(name: &str) -> String {
    name.chars()
        .next()
        .map(|character| character.to_string())
        .unwrap_or_default()
}

fn path_segments(path_str: &str) -> Option<Vec<String>> {
    if !is_path_like(path_str) {
        return None;
    }
    let path = Path::new(path_str);
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                segments.push(prefix.as_os_str().to_string_lossy().into_owned());
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => segments.push("..".to_owned()),
            Component::Normal(name) => segments.push(name.to_string_lossy().into_owned()),
        }
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

fn join_path_segments(segments: &[String], sep: char) -> String {
    if segments.is_empty() {
        return String::new();
    }
    let mut result = segments[0].clone();
    for segment in segments.iter().skip(1) {
        if !result.is_empty() {
            result.push(sep);
        }
        result.push_str(segment);
    }
    result
}

fn shrink_path_directories(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    if segments.len() == 1 {
        return Some(segments[0].clone());
    }
    let sep = path_separator_for(path_str);
    let file_name = segments.last()?.clone();
    let parents = segments
        .iter()
        .take(segments.len().saturating_sub(1))
        .map(|segment| {
            if segment.ends_with(':') || segment == ".." {
                segment.clone()
            } else {
                shrink_directory_name(segment)
            }
        })
        .collect::<Vec<_>>();
    let mut joined = join_path_segments(&parents, sep);
    if joined.is_empty() {
        return Some(file_name);
    }
    if !joined.ends_with(':') {
        joined.push(sep);
    }
    joined.push_str(&file_name);
    Some(joined)
}

fn split_file_stem_ext(file_name: &str) -> (String, String) {
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_owned());
    let extension = path
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    (stem, extension)
}

fn shrink_path_all(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    if segments.len() == 1 {
        let (stem, extension) = split_file_stem_ext(&segments[0]);
        return Some(format!("{}{extension}", shrink_directory_name(&stem)));
    }
    let sep = path_separator_for(path_str);
    let file_name = segments.last()?.clone();
    let (stem, extension) = split_file_stem_ext(&file_name);
    let shrunk_file = format!("{}{extension}", shrink_directory_name(&stem));
    let mut parents = segments
        .iter()
        .take(segments.len().saturating_sub(1))
        .map(|segment| {
            if segment.ends_with(':') || segment == ".." {
                segment.clone()
            } else {
                shrink_directory_name(segment)
            }
        })
        .collect::<Vec<_>>();
    parents.push(shrunk_file);
    Some(join_path_segments(&parents, sep))
}

fn file_name_with_parent(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    let sep = path_separator_for(path_str);
    if segments.len() < 2 {
        return segments.last().cloned();
    }
    Some(format!(
        "{}{}{}",
        segments[segments.len() - 2],
        sep,
        segments[segments.len() - 1]
    ))
}

fn parent_initial_file_name(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    let sep = path_separator_for(path_str);
    if segments.len() < 2 {
        return segments.last().cloned();
    }
    Some(format!(
        "{}{}{}",
        shrink_directory_name(&segments[segments.len() - 2]),
        sep,
        segments[segments.len() - 1]
    ))
}

fn shrink_leading_keep_tail(path_str: &str, tail_keep: usize) -> Option<String> {
    let segments = path_segments(path_str)?;
    if segments.len() <= tail_keep {
        return shrink_path_directories(path_str);
    }
    let sep = path_separator_for(path_str);
    let split_at = segments.len() - tail_keep;
    let (leading, tail) = segments.split_at(split_at);
    let mut parts = leading
        .iter()
        .map(|segment| {
            if segment.ends_with(':') || segment == ".." {
                segment.clone()
            } else {
                shrink_directory_name(segment)
            }
        })
        .collect::<Vec<_>>();
    parts.extend_from_slice(tail);
    Some(join_path_segments(&parts, sep))
}

pub(super) fn truncate_text_to_width_middle(text: &str, max_width: u32, cell_width: i32) -> String {
    if text.is_empty() || max_width == 0 {
        return String::new();
    }

    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    if text.chars().count() <= max_cells {
        return text.to_owned();
    }

    let ellipsis = "...";
    let ellipsis_cells = ellipsis.chars().count();
    if max_cells <= ellipsis_cells {
        return ellipsis.to_owned();
    }

    let available_cells = max_cells.saturating_sub(ellipsis_cells);
    let head_cells = available_cells / 2;
    let tail_cells = available_cells - head_cells;
    let chars = text.chars().collect::<Vec<_>>();
    let head = chars.iter().take(head_cells).copied().collect::<String>();
    let tail = chars
        .iter()
        .skip(chars.len().saturating_sub(tail_cells))
        .copied()
        .collect::<String>();
    format!("{head}{ellipsis}{tail}")
}

fn fit_picker_label_after_transform(
    text: &str,
    max_width: u32,
    cell_width: i32,
    overflow: PickerTruncateStrategy,
) -> String {
    if text_fits_width(text, max_width, cell_width) {
        return text.to_owned();
    }
    match overflow {
        PickerTruncateStrategy::EndEllipsis => truncate_text_to_width(text, max_width, cell_width),
        PickerTruncateStrategy::MiddleEllipsis => {
            truncate_text_to_width_middle(text, max_width, cell_width)
        }
        _ => truncate_text_to_width_preserving_end(text, max_width, cell_width),
    }
}

pub(super) fn truncate_picker_label(
    text: &str,
    max_width: u32,
    cell_width: i32,
    strategy: PickerTruncateStrategy,
) -> String {
    match strategy {
        PickerTruncateStrategy::EndEllipsis => truncate_text_to_width(text, max_width, cell_width),
        PickerTruncateStrategy::StartEllipsis => {
            truncate_text_to_width_preserving_end(text, max_width, cell_width)
        }
        PickerTruncateStrategy::MiddleEllipsis => {
            truncate_text_to_width_middle(text, max_width, cell_width)
        }
        PickerTruncateStrategy::Full => {
            if text_fits_width(text, max_width, cell_width) {
                text.to_owned()
            } else {
                truncate_text_to_width_preserving_end(text, max_width, cell_width)
            }
        }
        PickerTruncateStrategy::Auto => {
            if text_fits_width(text, max_width, cell_width) {
                return text.to_owned();
            }
            if let Some(shrunk) = shrink_path_directories(text)
                && text_fits_width(&shrunk, max_width, cell_width)
            {
                return shrunk;
            }
            truncate_text_to_width_preserving_end(text, max_width, cell_width)
        }
        PickerTruncateStrategy::ShrinkDirectories => {
            let display = shrink_path_directories(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::ShrinkAll => {
            let display = shrink_path_all(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::FileName => {
            let display = Path::new(text)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::FileNameWithParent => {
            let display = file_name_with_parent(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::ParentInitialFileName => {
            let display = parent_initial_file_name(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::ShrinkLeadingKeepTail => {
            let display = shrink_leading_keep_tail(text, PICKER_PATH_TAIL_KEEP)
                .unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
    }
}

pub(super) struct PixelRectToRect;

impl PixelRectToRect {
    pub(super) fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect::new(x, y, width, height)
    }

    pub(super) fn from_pixel_rect(rect: PixelRect) -> Rect {
        Self::rect(rect.x, rect.y, rect.width, rect.height)
    }
}

#[cfg(test)]
mod render_rounded_rect_tests {
    use super::rounded_rect_row_coverage;

    #[test]
    fn rounded_rect_row_coverage_is_symmetric() {
        let top = rounded_rect_row_coverage(0, 12, 20, 4, 255);
        let bottom = rounded_rect_row_coverage(11, 12, 20, 4, 255);
        assert_eq!(top, bottom);
    }

    #[test]
    fn rounded_rect_row_coverage_adds_partial_edge_pixels() {
        let (inset, alpha) = rounded_rect_row_coverage(0, 12, 20, 4, 255);
        assert!(inset > 0);
        assert!(alpha > 0);
        assert!(alpha < 255);
    }
}
