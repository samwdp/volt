impl ShellState {
    fn render(
        &mut self,
        target: &mut DrawTarget<'_>,
        fonts: &FontSet<'_>,
        size: WindowSize,
        metrics: TextMetrics,
        fps_overlay: Option<&FpsOverlaySnapshot>,
    ) -> Result<(), ShellError> {
        let WindowSize { width, height } = size;
        let TextMetrics {
            cell_width,
            line_height,
            ascent,
        } = metrics;
        self.refresh_picker_preview_syntax();
        let runtime_popup = self.runtime_popup()?;
        let now = Instant::now();
        let dock_visible = {
            let ui = self.ui()?;
            workspace_dock_visible(&*shell_user_library(&self.runtime), ui)
        };
        let dock_entries =
            collect_workspace_dock_entries(&self.runtime).map_err(ShellError::Runtime)?;
        let acp_dock_entries =
            collect_acp_dock_entries(&self.runtime).map_err(ShellError::Runtime)?;
        if dock_visible {
            let roots = workspace_dock_project_roots(&self.runtime).map_err(ShellError::Runtime)?;
            let cache = self.ui_mut()?.workspace_dock_branches_mut();
            refresh_workspace_dock_branches(cache, &roots, now);
        }
        let ui = self.ui()?;
        let acp_connected = acp::acp_connected(&self.runtime).unwrap_or(false);
        let lsp_workspace_loaded = active_lsp_workspace_loaded(&self.runtime, ui);
        let theme_registry = self.runtime.services().get::<ThemeRegistry>();
        let workspace_name = self
            .runtime
            .model()
            .active_workspace()
            .map_err(|error| ShellError::Runtime(error.to_string()))?
            .name()
            .to_owned();
        let typing_active = self.typing_refresh_budget_active(now);
        render_shell_state(
            target,
            fonts,
            ui,
            runtime_popup.as_ref(),
            ShellDockEntries {
                workspace: &dock_entries,
                acp: &acp_dock_entries,
            },
            ShellChrome {
                user_library: &*shell_user_library(&self.runtime),
                theme_registry,
                workspace_name: &workspace_name,
                lsp_server: ui.attached_lsp_server(),
                lsp_workspace_loaded,
                acp_connected,
            },
            ShellFrameView {
                size: WindowSize { width, height },
                fps_overlay,
                metrics: TextMetrics {
                    cell_width,
                    line_height,
                    ascent,
                },
                pulse: FramePulse { now, typing_active },
            },
        )
    }

    fn sync_browser_hosts(
        &mut self,
        window: &Window,
        width: u32,
        height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let runtime_popup = self.runtime_popup()?;
        let plan = browser_sync_plan(
            self.ui()?,
            BrowserSyncView {
                runtime_popup: runtime_popup.as_ref(),
                user_library: &*shell_user_library(&self.runtime),
                size: WindowSize { width, height },
                metrics: CellMetrics {
                    cell_width,
                    line_height,
                },
                now: Instant::now(),
            },
        )?;
        let updates = self
            .browser_host
            .sync_window(window, &plan)
            .map_err(ShellError::Runtime)?;
        if !updates.is_empty() {
            apply_browser_location_updates(&mut self.runtime, &updates)
                .map_err(ShellError::Runtime)?;
        }
        let events = self
            .browser_host
            .drain_events()
            .map_err(ShellError::Runtime)?;
        if !events.is_empty() {
            self.apply_browser_host_events(&events)?;
        }
        Ok(())
    }

    fn apply_browser_host_events(&mut self, events: &[BrowserHostEvent]) -> Result<(), ShellError> {
        for event in events {
            match event {
                BrowserHostEvent::FocusParentRequested { .. } => {
                    self.browser_host
                        .focus_parent()
                        .map_err(ShellError::Runtime)?;
                    self.ui_mut()?.enter_normal_mode();
                }
                BrowserHostEvent::OpenDevtoolsRequested { buffer_id } => {
                    self.browser_host
                        .open_devtools(*buffer_id)
                        .map_err(ShellError::Runtime)?;
                }
                BrowserHostEvent::DocumentTitleChanged { buffer_id, title } => {
                    if let Some(buffer) = self.ui_mut()?.buffer_mut(*buffer_id) {
                        set_browser_buffer_title(buffer, title.as_deref());
                    }
                }
                BrowserHostEvent::PageLoadStateChanged {
                    buffer_id,
                    current_url,
                    is_loading,
                } => {
                    let user_library = shell_user_library(&self.runtime);
                    if let Some(buffer) = self.ui_mut()?.buffer_mut(*buffer_id) {
                        apply_browser_page_load_state(
                            buffer,
                            current_url,
                            *is_loading,
                            &*user_library,
                        );
                    }
                }
                BrowserHostEvent::NewWindowRequested { url, .. } => {
                    open_browser_buffer_in_popup(&mut self.runtime, Some(url))
                        .map_err(ShellError::Runtime)?;
                }
            }
        }
        Ok(())
    }

    fn pane_count(&self) -> Result<usize, ShellError> {
        Ok(self.ui()?.pane_count())
    }

    pub(crate) fn picker_visible(&self) -> Result<bool, ShellError> {
        Ok(self.ui()?.picker_visible())
    }

    pub(crate) fn command_line_visible(&self) -> Result<bool, ShellError> {
        Ok(self.ui()?.command_line_visible())
    }

    fn popup_visible(&mut self) -> Result<bool, ShellError> {
        Ok(self.picker_visible()? || self.runtime_popup()?.is_some())
    }

    pub(crate) fn ui(&self) -> Result<&ShellUiState, ShellError> {
        shell_ui(&self.runtime).map_err(ShellError::Runtime)
    }

    fn ui_mut(&mut self) -> Result<&mut ShellUiState, ShellError> {
        shell_ui_mut(&mut self.runtime).map_err(ShellError::Runtime)
    }

    pub(crate) fn input_mode(&self) -> Result<InputMode, ShellError> {
        Ok(self.ui()?.input_mode())
    }

    pub(crate) fn active_buffer_mut(&mut self) -> Result<&mut ShellBuffer, ShellError> {
        let buffer_id = active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
        ensure_shell_buffer(&mut self.runtime, buffer_id).map_err(ShellError::Runtime)?;
        let ui = self.ui_mut()?;
        ui.buffer_mut(buffer_id)
            .ok_or_else(|| ShellError::Runtime("active shell buffer is missing".to_owned()))
    }
}
