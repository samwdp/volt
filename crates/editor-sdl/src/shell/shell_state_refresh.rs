impl ShellState {
    fn sync_active_buffer(&mut self) -> Result<(), String> {
        sync_active_buffer(&mut self.runtime)
    }

    fn sync_active_buffer_if_surface_changed(
        &mut self,
        previous_surface: Option<(PaneId, BufferId)>,
    ) -> Result<(), ShellError> {
        let runtime_surface = active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
        if runtime_surface != previous_surface {
            self.sync_active_buffer().map_err(ShellError::Runtime)?;
        }
        Ok(())
    }

    #[cfg(test)]
    fn sync_active_viewport(
        &mut self,
        viewport_height: u32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let buffer_id = active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
        let command_line_visible =
            self.ui()?.command_line().is_some() || self.ui()?.input_prompt_visible();
        let visible_rows = {
            let buffer = shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?;
            buffer_visible_rows_for_height(
                buffer,
                viewport_height,
                line_height,
                command_line_visible,
            )
        };
        self.active_buffer_mut()?.set_viewport_lines(visible_rows);
        Ok(())
    }

    #[cfg(test)]
    fn active_viewport_height(
        &mut self,
        render_width: u32,
        render_height: u32,
        line_height: i32,
    ) -> Result<u32, ShellError> {
        let runtime_popup = self.runtime_popup()?;
        let ui = self.ui()?;
        if let Some(popup) = runtime_popup.as_ref()
            && ui.popup_focus_active(popup)
        {
            return Ok(popup_window_height(render_height, line_height).max(1));
        }
        let popup_height = runtime_popup
            .as_ref()
            .map(|_| popup_window_height(render_height, line_height))
            .unwrap_or(0);
        let pane_height = render_height.saturating_sub(popup_height);
        let user_library = shell_user_library(&self.runtime);
        let docks = shell_docks_layout(&*user_library, ui, render_width, render_height, 8);
        let panes = ui
            .panes()
            .ok_or_else(|| ShellError::Runtime("active workspace view is missing".to_owned()))?;
        let pane_rects = workspace_pane_rects(
            &*user_library,
            ui,
            docks.content_width,
            pane_height,
            panes.len(),
        );
        let rect = pane_rects
            .get(ui.active_pane_index())
            .ok_or_else(|| ShellError::Runtime("active pane rect is missing".to_owned()))?;
        Ok(rect.height.max(1))
    }

    #[cfg(test)]
    fn sync_active_viewport_for_render_size(
        &mut self,
        render_width: u32,
        render_height: u32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let viewport_height =
            self.active_viewport_height(render_width, render_height, line_height)?;
        self.sync_active_viewport(viewport_height, line_height)
    }

    fn sync_visible_buffer_layouts(
        &mut self,
        render_width: u32,
        render_height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let typing_active = self.typing_refresh_budget_active(Instant::now());
        let runtime_popup = self.runtime_popup()?;
        let ui = self.ui()?;
        let command_line_visible = ui.command_line().is_some() || ui.input_prompt_visible();
        let popup_height = runtime_popup
            .as_ref()
            .map(|_| popup_window_height(render_height, line_height))
            .unwrap_or(0);
        let pane_height = render_height.saturating_sub(popup_height);
        let user_library = shell_user_library(&self.runtime);
        let docks = shell_docks_layout(&*user_library, ui, render_width, render_height, cell_width);
        let panes = ui
            .panes()
            .ok_or_else(|| ShellError::Runtime("active workspace view is missing".to_owned()))?;
        let pane_rects = workspace_pane_rects(
            &*user_library,
            ui,
            docks.content_width,
            pane_height,
            panes.len(),
        );
        let mut visible_buffers = panes
            .iter()
            .zip(pane_rects.iter())
            .enumerate()
            .map(|(pane_index, (pane, rect))| {
                (
                    pane.buffer_id,
                    rect.width,
                    rect.height,
                    pane_index == ui.active_pane_index()
                        && !ui.picker_visible()
                        && !runtime_popup
                            .as_ref()
                            .map(|popup| ui.popup_focus_active(popup))
                            .unwrap_or(false),
                )
            })
            .collect::<Vec<_>>();
        if let Some(popup) = runtime_popup.as_ref() {
            visible_buffers.push((
                popup.active_buffer,
                docks.content_width,
                popup_height.max(1),
                ui.popup_focus_active(popup),
            ));
        }
        let user_library = shell_user_library(&self.runtime);
        for (buffer_id, width, height, active) in visible_buffers {
            let (
                language_id,
                visible_rows,
                is_acp,
                has_plugin_sections,
                scrolloff,
                reserved_top_rows,
                input_mode,
                visual_selection,
            ) = {
                let theme_registry = self.runtime.services().get::<ThemeRegistry>();
                let ui = self.ui()?;
                let buffer = ui.buffer(buffer_id).ok_or_else(|| {
                    ShellError::Runtime(format!("buffer `{buffer_id}` is missing"))
                })?;
                let input_mode = ui.input_mode_for_buffer(buffer_id, active);
                let visual_selection = ui.visual_selection_for_buffer(buffer, active);
                let visible_rows = buffer_visible_rows_for_height(
                    buffer,
                    height,
                    line_height,
                    command_line_visible,
                );
                let is_acp = buffer.is_acp_buffer();
                let has_plugin_sections = buffer.has_plugin_sections();
                let reserved_top_rows = if active && !is_acp && !has_plugin_sections {
                    buffer_context_overlay_snapshot(buffer, true, typing_active, &*user_library)
                        .map(|snapshot| {
                            visible_headerline_row_count(&snapshot.headerline_lines, visible_rows)
                        })
                        .unwrap_or(0)
                } else {
                    0
                };
                (
                    buffer.language_id().map(str::to_owned),
                    visible_rows,
                    is_acp,
                    has_plugin_sections,
                    if !is_acp && !has_plugin_sections {
                        theme_scrolloff(theme_registry)
                    } else {
                        0
                    },
                    reserved_top_rows,
                    input_mode,
                    visual_selection,
                )
            };
            let wrap_cols = wrap_columns_for_width(width, cell_width);
            let indent_size = theme_lang_indent(
                self.runtime.services().get::<ThemeRegistry>(),
                language_id.as_deref(),
            );
            let buffer = self
                .ui_mut()?
                .buffer_mut(buffer_id)
                .ok_or_else(|| ShellError::Runtime(format!("buffer `{buffer_id}` is missing")))?;
            if is_acp {
                buffer.sync_acp_viewport_metrics(
                    width,
                    height,
                    cell_width,
                    line_height,
                    command_line_visible,
                );
            } else if has_plugin_sections {
                buffer.sync_plugin_section_viewport_metrics(
                    width,
                    height,
                    cell_width,
                    line_height,
                    command_line_visible,
                );
            } else {
                buffer.set_viewport_lines(visible_rows);
                let content_rows = visible_rows.saturating_sub(reserved_top_rows).max(1);
                buffer.set_scroll_layout(content_rows, wrap_cols, indent_size);
                let text_width_px = (wrap_cols as i32 * cell_width).max(1) as u32;
                buffer.refresh_pretty_display_rows(
                    &*user_library,
                    text_width_px,
                    line_height,
                    visual_selection,
                    input_mode,
                    content_rows,
                );
            }
            buffer.ensure_visible(
                visible_rows,
                wrap_cols,
                indent_size,
                reserved_top_rows,
                scrolloff,
            );
        }
        Ok(())
    }

    fn runtime_popup(&mut self) -> Result<Option<RuntimePopupSnapshot>, ShellError> {
        let popup = active_runtime_popup(&self.runtime).map_err(ShellError::Runtime)?;
        if let Some(popup) = popup.as_ref() {
            self.ui_mut()?.set_popup_buffer(popup.active_buffer);
            ensure_shell_buffer(&mut self.runtime, popup.active_buffer)
                .map_err(ShellError::Runtime)?;
            if ensure_terminal_session(&mut self.runtime, popup.active_buffer)
                .map_err(ShellError::Runtime)?
            {
                self.ui_mut()?.enter_insert_mode();
            }
        } else {
            let ui = self.ui_mut()?;
            ui.set_popup_focus(false);
            ui.clear_popup_buffer();
        }
        Ok(popup)
    }

    fn mark_active_buffer_syntax_dirty(&mut self) -> Result<(), ShellError> {
        self.active_buffer_mut()?.mark_syntax_dirty();
        Ok(())
    }

    fn refresh_pending_file_reloads(&mut self, now: Instant) -> Result<bool, ShellError> {
        refresh_pending_file_reloads(&mut self.runtime, now, false).map_err(ShellError::Runtime)
    }

    fn refresh_pending_syntax(
        &mut self,
        typing_active: bool,
    ) -> Result<SyntaxRefreshStats, ShellError> {
        if typing_active {
            return Ok(SyntaxRefreshStats::default());
        }
        self.active_buffer_mut()?.ensure_visible_syntax_window();
        refresh_pending_syntax(&mut self.runtime).map_err(ShellError::Runtime)
    }

    fn refresh_pending_git(&mut self, now: Instant, typing_active: bool) -> Result<(), ShellError> {
        refresh_pending_git(&mut self.runtime, now, typing_active).map_err(ShellError::Runtime)
    }

    fn refresh_pending_terminal(
        &mut self,
        render_width: u32,
        render_height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<bool, ShellError> {
        refresh_pending_terminal(
            &mut self.runtime,
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(ShellError::Runtime)
    }

    fn refresh_pending_streamed_commands(&mut self) -> Result<bool, ShellError> {
        refresh_pending_streamed_commands(&mut self.runtime).map_err(ShellError::Runtime)
    }

    fn refresh_pending_lsp(&mut self, typing_active: bool) -> Result<bool, ShellError> {
        refresh_pending_lsp(&mut self.runtime, typing_active).map_err(ShellError::Runtime)
    }

    fn refresh_pending_dap(&mut self) -> Result<bool, ShellError> {
        refresh_pending_dap(&mut self.runtime).map_err(ShellError::Runtime)
    }

    fn refresh_notifications(
        &mut self,
        now: Instant,
        typing_active: bool,
    ) -> Result<bool, ShellError> {
        if typing_active {
            return Ok(false);
        }
        Ok(self.ui_mut()?.prune_notifications(now))
    }

    fn refresh_pending_acp(
        &mut self,
        render_width: u32,
        render_height: u32,
        line_height: i32,
        cell_width: i32,
    ) -> Result<bool, ShellError> {
        self.sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)?;
        let active_buffer_id =
            active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
        let followed_output = {
            let buffer =
                shell_buffer(&self.runtime, active_buffer_id).map_err(ShellError::Runtime)?;
            buffer.has_input_field() && buffer.should_follow_output()
        };
        let changed = acp::refresh_pending_acp(&mut self.runtime).map_err(ShellError::Runtime)?;
        self.sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)?;
        if changed
            && followed_output
            && active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?
                == active_buffer_id
        {
            self.active_buffer_mut()?.scroll_output_to_end();
        }
        Ok(changed)
    }

    fn visual_refresh_key(
        &self,
        render_width: u32,
        render_height: u32,
        theme_settings: &ThemeRuntimeSettings,
        now: Instant,
    ) -> Result<ShellVisualRefreshKey, ShellError> {
        let ui = self.ui()?;
        let active_lsp_workspace_loaded = active_lsp_workspace_loaded(&self.runtime, ui);
        Ok(ShellVisualRefreshKey {
            render_width,
            render_height,
            theme_settings: theme_settings.clone(),
            git_summary_revision: ui.git_summary_revision(),
            git_fringe_revisions: ui
                .buffers
                .iter()
                .filter_map(|buffer| {
                    buffer
                        .git_fringe_revision()
                        .map(|revision| (buffer.id(), revision))
                })
                .collect(),
            lsp_diagnostics_revisions: ui
                .buffers
                .iter()
                .map(|buffer| (buffer.id(), buffer.lsp_diagnostics_revision()))
                .collect(),
            active_lsp_server: ui.attached_lsp_server().map(str::to_owned),
            active_lsp_workspace_loaded,
            notification_revision: ui.notification_revision(),
            notification_deadline: ui.notification_deadline(now),
            yank_flash_until: ui.yank_flash_deadline(now),
        })
    }
}
