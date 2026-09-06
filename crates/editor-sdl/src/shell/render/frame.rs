pub(super) fn render_shell_state(
    target: &mut DrawTarget<'_>,
    fonts: &FontSet<'_>,
    state: &ShellUiState,
    runtime_popup: Option<&RuntimePopupSnapshot>,
    dock_entries: ShellDockEntries<'_>,
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
    let docks = shell_docks_layout(user_library, state, width, height, cell_width);
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
        docks.content_width,
        pane_height,
        panes.len(),
    );
    for rect in &mut pane_rects {
        rect.x = rect.x.saturating_add(docks.content_x);
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
    let dock_focus =
        state.workspace_dock_focus_active(user_library) || state.acp_dock_focus_active();
    let command_line_row_visible = state.command_line().is_some() || state.input_prompt_visible();

    clear_window_surface(target, base_background, window_effects);

    render_workspace_dock(
        target,
        &docks.workspace,
        dock_entries.workspace,
        theme_registry,
        cell_width,
        line_height,
        ascent,
    )?;
    render_acp_dock(
        target,
        &docks.acp,
        dock_entries.acp,
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
        // CONTEXT: ACP and plugin-section buffers paint their own translucent
        // panel chrome. Filling the pane first would stack another opacity
        // layer and make those sections look darker/more opaque than the
        // editor. Leave gaps at the cleared window surface so acrylic shows.
        let paints_own_panels = state
            .buffer(pane.buffer_id)
            .is_some_and(|buffer| buffer.is_acp_buffer() || buffer.has_plugin_sections());
        if !paints_own_panels {
            fill_window_surface_rect(
                target,
                PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
                background,
                window_effects,
            )?;
        }

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
                docks.content_x,
                pane_height as i32,
                docks.content_width,
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
