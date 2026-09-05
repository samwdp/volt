impl ShellState {
    fn handle_event(
        &mut self,
        event: Event,
        render_width: u32,
        render_height: u32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<bool, ShellError> {
        let active_buffer =
            active_buffer_event_context(&self.runtime).map_err(ShellError::Runtime)?;
        let visible_rows = shell_buffer(&self.runtime, active_buffer.buffer_id)
            .map_err(ShellError::Runtime)?
            .viewport_lines();
        let page_rows = visible_rows as i32;
        let (input_mode, picker_visible) = {
            let ui = self.ui()?;
            (ui.input_mode(), ui.picker_visible())
        };
        match event {
            Event::Quit { .. } => return Ok(true),
            Event::MouseButtonDown {
                mouse_btn,
                clicks,
                x,
                y,
                ..
            } => {
                let mouse_x = x as i32;
                let mouse_y = y as i32;
                let now = Instant::now();
                if mouse_btn == MouseButton::Left
                    && let Some(action) = notification_action_at_point(
                        self.ui()?,
                        render_width,
                        render_height,
                        cell_width,
                        line_height,
                        now,
                        (mouse_x, mouse_y),
                    )
                {
                    match action {
                        NotificationAction::OpenAcpPermissionPicker { request_id } => {
                            acp::acp_open_permission_request(&mut self.runtime, request_id)
                                .map_err(ShellError::Runtime)?;
                        }
                        NotificationAction::OpenBrowserPopup { url } => {
                            open_browser_buffer_in_popup(&mut self.runtime, Some(&url))
                                .map_err(ShellError::Runtime)?;
                        }
                        NotificationAction::CopilotSignIn { root } => {
                            begin_copilot_sign_in(&mut self.runtime, root.as_deref())
                                .map_err(ShellError::Runtime)?;
                        }
                    }
                    return Ok(false);
                }
                if picker_visible {
                    return Ok(false);
                }
                let user_library = shell_user_library(&self.runtime);
                let docks = shell_docks_layout(
                    &*user_library,
                    self.ui()?,
                    render_width,
                    render_height,
                    cell_width,
                );
                if mouse_btn == MouseButton::Left {
                    let workspace_entries = collect_workspace_dock_entries(&self.runtime)
                        .map_err(ShellError::Runtime)?;
                    if let Some(workspace_id) = workspace_dock_entry_at_point(
                        &docks.workspace,
                        &workspace_entries,
                        line_height,
                        mouse_x,
                        mouse_y,
                    ) {
                        self.mouse_drag = None;
                        shell_ui_mut(&mut self.runtime)
                            .map_err(ShellError::Runtime)?
                            .set_workspace_dock_focus(true);
                        shell_ui_mut(&mut self.runtime)
                            .map_err(ShellError::Runtime)?
                            .set_popup_focus(false);
                        switch_runtime_workspace(&mut self.runtime, workspace_id)
                            .map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                    let acp_entries =
                        collect_acp_dock_entries(&self.runtime).map_err(ShellError::Runtime)?;
                    if let Some(buffer_id) = acp_dock_entry_at_point(
                        &docks.acp,
                        &acp_entries,
                        line_height,
                        mouse_x,
                        mouse_y,
                    ) {
                        self.mouse_drag = None;
                        shell_ui_mut(&mut self.runtime)
                            .map_err(ShellError::Runtime)?
                            .set_popup_focus(false);
                        shell_ui_mut(&mut self.runtime)
                            .map_err(ShellError::Runtime)?
                            .set_acp_dock_focus(true);
                        acp::focus_acp_buffer(&mut self.runtime, buffer_id)
                            .map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                }
                let runtime_popup = self.runtime_popup()?;
                let popup_height = runtime_popup
                    .as_ref()
                    .map(|_| popup_window_height(render_height, line_height))
                    .unwrap_or(0);
                let pane_height = render_height.saturating_sub(popup_height);
                let browser_plan = browser_sync_plan(
                    self.ui()?,
                    BrowserSyncView {
                        runtime_popup: runtime_popup.as_ref(),
                        user_library: &*user_library,
                        size: WindowSize {
                            width: render_width,
                            height: render_height,
                        },
                        metrics: CellMetrics {
                            cell_width,
                            line_height,
                        },
                        now: Instant::now(),
                    },
                )?;
                let clicked_browser_buffer =
                    browser_surface_buffer_at_point(&browser_plan, mouse_x, mouse_y);
                let in_popup = runtime_popup.is_some()
                    && mouse_y >= pane_height as i32
                    && mouse_x >= docks.content_x
                    && mouse_x < docks.content_x.saturating_add(docks.content_width as i32);
                if in_popup {
                    self.mouse_drag = None;
                    if let Some(popup) = runtime_popup.as_ref() {
                        let ui = self.ui_mut()?;
                        ui.set_popup_buffer(popup.active_buffer);
                        ui.set_popup_focus(true);
                    }
                    if let Some(buffer_id) = clicked_browser_buffer {
                        self.browser_host
                            .focus_buffer(buffer_id)
                            .map_err(ShellError::Runtime)?;
                    } else {
                        self.browser_host
                            .focus_parent()
                            .map_err(ShellError::Runtime)?;
                    }
                    return Ok(false);
                }
                if let Some((pane_id, buffer_id, pane_rect)) = self.pane_surface_at_point(
                    render_width,
                    render_height,
                    pane_height,
                    cell_width,
                    mouse_x,
                    mouse_y,
                )? {
                    self.focus_runtime_pane(pane_id)?;
                    if let Some(buffer_id) = clicked_browser_buffer {
                        self.browser_host
                            .focus_buffer(buffer_id)
                            .map_err(ShellError::Runtime)?;
                    } else {
                        self.browser_host
                            .focus_parent()
                            .map_err(ShellError::Runtime)?;
                    }
                    if mouse_btn == MouseButton::Left {
                        let (kind, uses_browser_host_surface) = {
                            let buffer = shell_buffer(&self.runtime, buffer_id)
                                .map_err(ShellError::Runtime)?;
                            (buffer.kind.clone(), buffer.uses_browser_host_surface())
                        };
                        if !uses_browser_host_surface && !buffer_is_terminal(&kind) {
                            self.begin_mouse_selection(
                                buffer_id,
                                pane_rect,
                                MouseClick {
                                    x: mouse_x,
                                    y: mouse_y,
                                    clicks,
                                },
                                CellMetrics {
                                    cell_width,
                                    line_height,
                                },
                            )?;
                        } else {
                            self.mouse_drag = None;
                        }
                    } else {
                        self.mouse_drag = None;
                    }
                } else if mouse_btn == MouseButton::Left {
                    self.mouse_drag = None;
                }
            }
            Event::MouseMotion { x, y, .. } => {
                self.update_mouse_selection(x as i32, y as i32, cell_width, line_height)?;
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                ..
            } => {
                self.finish_mouse_selection()?;
            }
            Event::MouseButtonUp { .. } => {}
            Event::MouseWheel {
                y,
                direction,
                mouse_x,
                mouse_y,
                ..
            } => {
                if picker_visible {
                    return Ok(false);
                }
                let wheel_delta = match direction {
                    MouseWheelDirection::Normal => y.round() as i32,
                    MouseWheelDirection::Flipped => -(y.round() as i32),
                    _ => y.round() as i32,
                };
                if wheel_delta == 0 {
                    return Ok(false);
                }
                let mouse_x = mouse_x as i32;
                let mouse_y = mouse_y as i32;
                let runtime_popup = self.runtime_popup()?;
                let popup_height = runtime_popup
                    .as_ref()
                    .map(|_| popup_window_height(render_height, line_height))
                    .unwrap_or(0);
                let pane_height = render_height.saturating_sub(popup_height);
                if runtime_popup.is_some() && mouse_y >= pane_height as i32 {
                    return Ok(false);
                }
                let browser_plan = browser_sync_plan(
                    self.ui()?,
                    BrowserSyncView {
                        runtime_popup: runtime_popup.as_ref(),
                        user_library: &*shell_user_library(&self.runtime),
                        size: WindowSize {
                            width: render_width,
                            height: render_height,
                        },
                        metrics: CellMetrics {
                            cell_width,
                            line_height,
                        },
                        now: Instant::now(),
                    },
                )?;
                if browser_surface_buffer_at_point(&browser_plan, mouse_x, mouse_y).is_some() {
                    return Ok(false);
                }
                let Some((pane_id, _, _)) = self.pane_surface_at_point(
                    render_width,
                    render_height,
                    pane_height,
                    cell_width,
                    mouse_x,
                    mouse_y,
                )?
                else {
                    return Ok(false);
                };
                self.focus_runtime_pane(pane_id)?;
                self.browser_host
                    .focus_parent()
                    .map_err(ShellError::Runtime)?;
                let scroll_lines = wheel_delta.saturating_mul(MOUSE_WHEEL_SCROLL_LINES);
                let active_buffer_id =
                    active_shell_buffer_id(&self.runtime).map_err(ShellError::Runtime)?;
                let (active_kind, active_uses_browser_host_surface) = {
                    let buffer = shell_buffer(&self.runtime, active_buffer_id)
                        .map_err(ShellError::Runtime)?;
                    (buffer.kind.clone(), buffer.uses_browser_host_surface())
                };
                if buffer_is_terminal(&active_kind) {
                    scroll_active_terminal_view(
                        &mut self.runtime,
                        TerminalViewportScroll::LineDelta(scroll_lines),
                    )
                    .map_err(ShellError::Runtime)?;
                } else if !active_uses_browser_host_surface {
                    scroll_buffer_viewport_only(
                        shell_buffer_mut(&mut self.runtime, active_buffer_id)
                            .map_err(ShellError::Runtime)?,
                        -scroll_lines,
                    );
                }
            }
            Event::KeyDown {
                keycode: Some(keycode),
                keymod,
                ..
            } => {
                let runtime_surface_before =
                    active_runtime_surface(&self.runtime).map_err(ShellError::Runtime)?;
                let is_ctrl_c = keymod.intersects(ctrl_mod()) && keycode == Keycode::C;
                let is_ctrl_k = keymod.intersects(ctrl_mod()) && keycode == Keycode::K;
                let is_ctrl_key = matches!(keycode, Keycode::LCtrl | Keycode::RCtrl);
                if active_buffer.is_git_commit {
                    let mut should_commit = false;
                    let mut should_cancel = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_commit = true;
                                consume = true;
                            } else if is_ctrl_k {
                                ui.pending_ctrl_c = None;
                                should_cancel = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_commit || should_cancel {
                        if should_commit {
                            commit_git_buffer(&mut self.runtime, active_buffer.buffer_id)
                                .map_err(ShellError::Runtime)?;
                        } else {
                            cancel_git_commit_buffer(&mut self.runtime, active_buffer.buffer_id)
                                .map_err(ShellError::Runtime)?;
                        }
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if active_buffer.is_git_editor {
                    let mut should_confirm = false;
                    let mut should_abort = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_confirm = true;
                                consume = true;
                            } else if is_ctrl_k {
                                ui.pending_ctrl_c = None;
                                should_abort = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_confirm || should_abort {
                        if should_confirm {
                            confirm_git_editor_buffer(&mut self.runtime, active_buffer.buffer_id)
                                .map_err(ShellError::Runtime)?;
                        } else {
                            abort_git_editor_buffer(&mut self.runtime, active_buffer.buffer_id)
                                .map_err(ShellError::Runtime)?;
                        }
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if active_buffer.is_db_query {
                    let mut should_execute_sql = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_execute_sql = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_execute_sql {
                        self.runtime
                            .execute_command("db.execute-sql")
                            .map_err(|error| ShellError::Runtime(error.to_string()))?;
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if active_buffer.is_plugin_evaluatable {
                    let mut should_evaluate = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_evaluate = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_evaluate {
                        evaluate_active_plugin_buffer(&mut self.runtime, active_buffer.buffer_id)
                            .map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if active_buffer.has_input && (active_buffer.is_acp || active_buffer.is_browser) {
                    let mut should_submit = false;
                    let mut consume = false;
                    {
                        let ui = self.ui_mut()?;
                        if ui.pending_ctrl_c.is_some() {
                            if is_ctrl_c {
                                ui.pending_ctrl_c = None;
                                should_submit = true;
                                consume = true;
                            } else if is_ctrl_key {
                                consume = true;
                            } else {
                                ui.pending_ctrl_c = None;
                            }
                        } else if is_ctrl_c {
                            ui.pending_ctrl_c = Some(Instant::now());
                            consume = true;
                        }
                    }
                    if should_submit {
                        submit_input_buffer(&mut self.runtime).map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                    if consume {
                        return Ok(false);
                    }
                }
                if !is_ctrl_c
                    && !is_ctrl_k
                    && !is_ctrl_key
                    && let Ok(ui) = self.ui_mut()
                {
                    ui.pending_ctrl_c = None;
                }
                if !picker_visible
                    && active_buffer.is_browser
                    && browser_devtools_shortcut_requested(keycode, keymod)
                {
                    self.browser_host
                        .open_devtools(active_buffer.buffer_id)
                        .map_err(ShellError::Runtime)?;
                    return Ok(false);
                }
                if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                    && active_buffer.has_input
                    && input_field_paste_shortcut_requested(keycode, keymod)
                {
                    paste_into_active_input_buffer(&mut self.runtime)
                        .map_err(ShellError::Runtime)?;
                    return Ok(false);
                }
                if keymod.intersects(ctrl_mod())
                    && keycode == Keycode::J
                    && matches!(input_mode, InputMode::Insert | InputMode::Replace)
                    && active_buffer.has_input
                    && active_buffer.is_acp
                {
                    if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                        input.append_text("\n");
                    }
                    return Ok(false);
                }
                if self.handle_input_prompt_keydown(keycode, keymod)? {
                    return Ok(false);
                }
                if self.handle_command_line_keydown(keycode, keymod)? {
                    return Ok(false);
                }
                if self.handle_focused_hover_keydown(keycode, keymod)? {
                    return Ok(false);
                }
                if self.handle_autocomplete_keydown(keycode, keymod)? {
                    return Ok(false);
                }
                if self.try_runtime_keybinding_cached(
                    keycode,
                    keymod,
                    input_mode,
                    picker_visible,
                    active_buffer.is_directory,
                )? {
                    self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                    return Ok(false);
                }

                if !picker_visible && keymod.intersects(ctrl_mod()) && keycode == Keycode::Q {
                    return Ok(true);
                }

                let acp_inline_picker_active = matches!(
                    self.ui()?.picker_kind(),
                    Some(kind) if kind.acp_inline_buffer_id() == Some(active_buffer.buffer_id)
                );
                if picker_visible
                    && acp_inline_picker_active
                    && matches!(
                        keycode,
                        Keycode::Return | Keycode::KpEnter | Keycode::Return2
                    )
                {
                    self.runtime
                        .execute_command("picker.submit")
                        .map_err(|error| ShellError::Runtime(error.to_string()))?;
                    self.sync_active_buffer().map_err(ShellError::Runtime)?;
                    return Ok(false);
                }
                if picker_visible && !acp_inline_picker_active {
                    if matches!(
                        keycode,
                        Keycode::Return | Keycode::KpEnter | Keycode::Return2
                    ) {
                        self.runtime
                            .execute_command("picker.submit")
                            .map_err(|error| ShellError::Runtime(error.to_string()))?;
                        self.sync_active_buffer().map_err(ShellError::Runtime)?;
                        return Ok(false);
                    }
                    if keycode == Keycode::Backspace
                        && let Some(picker) = self.ui_mut()?.picker_mut()
                    {
                        picker.backspace_query();
                        self.schedule_picker_search_refresh()?;
                    }
                    return Ok(false);
                }
                if picker_visible
                    && acp_inline_picker_active
                    && matches!(keycode, Keycode::Backspace | Keycode::Delete)
                {
                    self.ui_mut()?.close_picker();
                } else if picker_visible && acp_inline_picker_active {
                    return Ok(false);
                }

                if matches!(
                    keycode,
                    Keycode::Return | Keycode::KpEnter | Keycode::Return2
                ) && quickfix_open_selected_entry(&mut self.runtime)
                    .map_err(ShellError::Runtime)?
                {
                    self.sync_active_buffer_if_surface_changed(runtime_surface_before)?;
                    return Ok(false);
                }

                if active_buffer.is_terminal
                    && matches!(input_mode, InputMode::Insert | InputMode::Replace)
                    && !active_buffer.has_input
                    && let Some(terminal_key) = terminal_key_for_event(keycode, keymod)
                {
                    write_active_terminal_key(&mut self.runtime, terminal_key)
                        .map_err(ShellError::Runtime)?;
                    return Ok(false);
                }

                let mut refresh_autocomplete = false;
                let mut close_autocomplete = false;
                match keycode {
                    Keycode::Left => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_left();
                            }
                        } else {
                            let _ = self.active_buffer_mut()?.move_left();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::Right => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_right();
                            }
                        } else {
                            let _ = self.active_buffer_mut()?.move_right();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::Up => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_up();
                            }
                        } else if active_buffer.is_terminal
                            && matches!(input_mode, InputMode::Visual)
                        {
                            let _ = self.active_buffer_mut()?.move_up();
                        } else if active_buffer.is_terminal {
                            scroll_active_terminal_view(
                                &mut self.runtime,
                                TerminalViewportScroll::LineDelta(1),
                            )
                            .map_err(ShellError::Runtime)?;
                        } else {
                            let _ = self.active_buffer_mut()?.move_up();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::Down => {
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                        {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.move_down();
                            }
                        } else if active_buffer.is_terminal
                            && matches!(input_mode, InputMode::Visual)
                        {
                            let _ = self.active_buffer_mut()?.move_down();
                        } else if active_buffer.is_terminal {
                            scroll_active_terminal_view(
                                &mut self.runtime,
                                TerminalViewportScroll::LineDelta(-1),
                            )
                            .map_err(ShellError::Runtime)?;
                        } else {
                            let _ = self.active_buffer_mut()?.move_down();
                            refresh_autocomplete =
                                matches!(input_mode, InputMode::Insert | InputMode::Replace);
                        }
                    }
                    Keycode::PageDown
                        if active_buffer.is_terminal
                            && !matches!(input_mode, InputMode::Visual) =>
                    {
                        scroll_active_terminal_view(
                            &mut self.runtime,
                            TerminalViewportScroll::PageDown,
                        )
                        .map_err(ShellError::Runtime)?;
                    }
                    Keycode::PageDown => self.active_buffer_mut()?.scroll_by(page_rows),
                    Keycode::PageUp
                        if active_buffer.is_terminal
                            && !matches!(input_mode, InputMode::Visual) =>
                    {
                        scroll_active_terminal_view(
                            &mut self.runtime,
                            TerminalViewportScroll::PageUp,
                        )
                        .map_err(ShellError::Runtime)?;
                    }
                    Keycode::PageUp => self.active_buffer_mut()?.scroll_by(-page_rows),
                    Keycode::Return | Keycode::KpEnter | Keycode::Return2
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace) =>
                    {
                        if self.ui()?.vim().multicursor.is_some() && !active_buffer.has_input {
                            apply_multicursor_insert_text(
                                &mut self.runtime,
                                "\n",
                                matches!(input_mode, InputMode::Replace),
                            )
                            .map_err(ShellError::Runtime)?;
                            close_autocomplete = true;
                        } else if active_buffer.has_input {
                            submit_input_buffer(&mut self.runtime).map_err(ShellError::Runtime)?;
                        } else if !active_buffer.is_read_only {
                            let changed = {
                                let buffer = self.active_buffer_mut()?;
                                insert_markdown_table_row_at_cursor(buffer)
                            };
                            if let Some(changed) = changed {
                                if changed {
                                    self.mark_active_buffer_syntax_dirty()?;
                                }
                                close_autocomplete = true;
                            } else {
                                let (buffer_id, indent_size, use_tabs) = {
                                    let ui = self.ui()?;
                                    let buffer_id = active_shell_buffer_id(&self.runtime)
                                        .map_err(ShellError::Runtime)?;
                                    let language_id = ui
                                        .buffer(buffer_id)
                                        .and_then(|buffer| buffer.language_id());
                                    let theme_registry =
                                        self.runtime.services().get::<ThemeRegistry>();
                                    (
                                        buffer_id,
                                        theme_lang_indent(theme_registry, language_id),
                                        theme_lang_use_tabs(theme_registry, language_id),
                                    )
                                };
                                if !insert_newline_inside_pair(
                                    &mut self.runtime,
                                    buffer_id,
                                    indent_size,
                                    use_tabs,
                                )
                                .map_err(ShellError::Runtime)?
                                {
                                    {
                                        let buffer = self.active_buffer_mut()?;
                                        buffer.insert_text("\n");
                                    }
                                    format_current_line_indent(
                                        &mut self.runtime,
                                        buffer_id,
                                        indent_size,
                                        use_tabs,
                                    )
                                    .map_err(ShellError::Runtime)?;
                                }
                                self.mark_active_buffer_syntax_dirty()?;
                                close_autocomplete = true;
                            }
                        }
                    }
                    Keycode::Backspace
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace) =>
                    {
                        if self.ui()?.vim().multicursor.is_some() && !active_buffer.has_input {
                            apply_multicursor_delete(&mut self.runtime, true)
                                .map_err(ShellError::Runtime)?;
                            refresh_autocomplete = true;
                        } else if active_buffer.has_input {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.backspace();
                            }
                            if active_buffer.is_acp {
                                acp::maybe_open_acp_input_completion(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                                acp::refresh_acp_input_hint(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                            }
                        } else if !active_buffer.is_read_only {
                            self.active_buffer_mut()?.backspace();
                            {
                                let buffer = self.active_buffer_mut()?;
                                let _ = format_markdown_table_at_cursor(buffer);
                            }
                            self.mark_active_buffer_syntax_dirty()?;
                            refresh_autocomplete = true;
                        }
                    }
                    Keycode::Delete
                        if matches!(input_mode, InputMode::Insert | InputMode::Replace) =>
                    {
                        if self.ui()?.vim().multicursor.is_some() && !active_buffer.has_input {
                            apply_multicursor_delete(&mut self.runtime, false)
                                .map_err(ShellError::Runtime)?;
                            refresh_autocomplete = true;
                        } else if active_buffer.has_input {
                            if let Some(input) = self.active_buffer_mut()?.input_field_mut() {
                                input.delete_forward();
                            }
                            if active_buffer.is_acp {
                                acp::maybe_open_acp_input_completion(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                                acp::refresh_acp_input_hint(
                                    &mut self.runtime,
                                    active_buffer.buffer_id,
                                )
                                .map_err(ShellError::Runtime)?;
                            }
                        } else if !active_buffer.is_read_only {
                            self.active_buffer_mut()?.delete_forward();
                            {
                                let buffer = self.active_buffer_mut()?;
                                let _ = format_markdown_table_at_cursor(buffer);
                            }
                            self.mark_active_buffer_syntax_dirty()?;
                            refresh_autocomplete = true;
                        }
                    }
                    Keycode::Tab => {
                        if !keymod.intersects(shift_mod())
                            && matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.has_input
                            && active_buffer.is_acp
                        {
                            acp::acp_complete_slash(&mut self.runtime)
                                .map_err(ShellError::Runtime)?;
                        } else if !matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && active_buffer.is_git_status
                        {
                            toggle_git_section(&mut self.runtime).map_err(ShellError::Runtime)?;
                        } else if !keymod.intersects(shift_mod())
                            && matches!(input_mode, InputMode::Insert | InputMode::Replace)
                            && !active_buffer.has_input
                            && !active_buffer.is_read_only
                        {
                            if self.accept_inline_completion()? {
                                close_autocomplete = true;
                            } else {
                                let changed = {
                                    let buffer = self.active_buffer_mut()?;
                                    advance_markdown_table_insert_tab(buffer)
                                };
                                if let Some(changed) = changed {
                                    if changed {
                                        self.mark_active_buffer_syntax_dirty()?;
                                    }
                                    close_autocomplete = true;
                                } else {
                                    let insert = {
                                        let ui = self.ui()?;
                                        let language_id = ui
                                            .buffer(active_buffer.buffer_id)
                                            .and_then(|buffer| buffer.language_id());
                                        let theme_registry =
                                            self.runtime.services().get::<ThemeRegistry>();
                                        tab_insert_string(theme_registry, language_id)
                                    };
                                    self.handle_text_input(&insert)?;
                                }
                            }
                        } else {
                            cycle_runtime_pane(&mut self.runtime).map_err(ShellError::Runtime)?;
                            close_autocomplete = true;
                        }
                    }
                    Keycode::F2 => {
                        split_runtime_pane(&mut self.runtime, PaneSplitDirection::Horizontal)
                            .map_err(ShellError::Runtime)?;
                    }
                    _ => {}
                }
                if close_autocomplete {
                    self.ui_mut()?.close_autocomplete();
                } else if refresh_autocomplete {
                    self.schedule_autocomplete_refresh_if_active()?;
                    self.schedule_inline_completion_refresh()?;
                }
            }
            Event::TextInput { text, .. } => {
                self.handle_text_input(&text)?;
            }
            _ => {}
        }

        Ok(false)
    }

    fn queue_suppressed_text_input_for_chord(&mut self, chord: &str) {
        const SUPPRESSED_TEXT_INPUT_WINDOW: Duration = Duration::from_millis(50);
        if let Some(text) = suppressed_text_input_for_chord(chord) {
            self.pending_suppressed_text_input = Some(SuppressedTextInput {
                text,
                expires_at: Instant::now() + SUPPRESSED_TEXT_INPUT_WINDOW,
            });
        }
    }

    fn should_suppress_text_input(&mut self, text: &str) -> bool {
        self.pending_suppressed_text_input
            .take()
            .is_some_and(|pending| Instant::now() <= pending.expires_at && pending.text == text)
    }

    fn pane_surface_at_point(
        &self,
        width: u32,
        height: u32,
        pane_height: u32,
        cell_width: i32,
        x: i32,
        y: i32,
    ) -> Result<Option<(PaneId, BufferId, PixelRect)>, ShellError> {
        if x < 0 || y < 0 || y >= pane_height as i32 {
            return Ok(None);
        }
        let ui = self.ui()?;
        let Some(panes) = ui.panes() else {
            return Ok(None);
        };
        let user_library = shell_user_library(&self.runtime);
        let docks = shell_docks_layout(&*user_library, ui, width, height, cell_width);
        let mut pane_rects = workspace_pane_rects(
            &*user_library,
            ui,
            docks.content_width,
            pane_height,
            panes.len(),
        );
        for rect in &mut pane_rects {
            rect.x = rect.x.saturating_add(docks.content_x);
        }
        Ok(panes
            .iter()
            .zip(pane_rects.iter())
            .find_map(|(pane, rect)| {
                pixel_rect_contains_point(*rect, x, y).then_some((
                    pane.pane_id,
                    pane.buffer_id,
                    *rect,
                ))
            }))
    }

    fn focus_runtime_pane(&mut self, pane_id: PaneId) -> Result<(), ShellError> {
        let workspace_id = self
            .runtime
            .model()
            .active_workspace_id()
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        self.runtime
            .model_mut()
            .focus_pane(workspace_id, pane_id)
            .map_err(|error| ShellError::Runtime(error.to_string()))?;
        let ui = self.ui_mut()?;
        ui.set_popup_focus(false);
        ui.set_workspace_dock_focus(false);
        ui.set_acp_dock_focus(false);
        ui.focus_pane(pane_id);
        Ok(())
    }

    fn begin_mouse_selection(
        &mut self,
        buffer_id: BufferId,
        rect: PixelRect,
        mouse: MouseClick,
        metrics: CellMetrics,
    ) -> Result<(), ShellError> {
        let MouseClick {
            x: mouse_x,
            y: mouse_y,
            clicks,
        } = mouse;
        let CellMetrics {
            cell_width,
            line_height,
        } = metrics;
        {
            let has_sections = shell_buffer(&self.runtime, buffer_id)
                .map_err(ShellError::Runtime)?
                .has_plugin_sections();
            if has_sections {
                let sdl_rect = PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height);
                let index = {
                    let buffer =
                        shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?;
                    let command_line_visible = self.command_line_visible().unwrap_or(false);
                    let layout = buffer_footer_layout_with_command_line(
                        buffer,
                        sdl_rect,
                        line_height,
                        cell_width,
                        command_line_visible,
                    );
                    buffer.plugin_section_index_at_point(
                        sdl_rect,
                        layout,
                        cell_width,
                        line_height,
                        mouse_x,
                        mouse_y,
                    )
                };
                if let Some(index) = index {
                    shell_buffer_mut(&mut self.runtime, buffer_id)
                        .map_err(ShellError::Runtime)?
                        .plugin_focus_section_index(index);
                }
            }
        }
        let point = {
            let theme_registry = self.runtime.services().get::<ThemeRegistry>();
            let buffer = shell_buffer(&self.runtime, buffer_id).map_err(ShellError::Runtime)?;
            buffer_point_at_screen(
                buffer,
                PixelRectToRect::rect(rect.x, rect.y, rect.width, rect.height),
                &*shell_user_library(&self.runtime),
                theme_registry,
                ScreenHit {
                    x: mouse_x,
                    y: mouse_y,
                    clamp_body: false,
                    typing_active: false,
                },
                CellMetrics {
                    cell_width,
                    line_height,
                },
            )
        };
        let Some(point) = point else {
            self.mouse_drag = None;
            return Ok(());
        };

        shell_buffer_mut(&mut self.runtime, buffer_id)
            .map_err(ShellError::Runtime)?
            .set_cursor(point);

        let kind = if clicks >= 2 {
            VisualSelectionKind::Line
        } else {
            if self.ui()?.input_mode() == InputMode::Visual {
                self.ui_mut()?.enter_normal_mode();
            }
            VisualSelectionKind::Character
        };
        if kind == VisualSelectionKind::Line {
            self.ui_mut()?.enter_visual_mode(point, kind);
        }
        self.mouse_drag = Some(MouseDragState {
            buffer_id,
            rect,
            anchor: point,
            kind,
        });
        Ok(())
    }

    fn update_mouse_selection(
        &mut self,
        mouse_x: i32,
        mouse_y: i32,
        cell_width: i32,
        line_height: i32,
    ) -> Result<(), ShellError> {
        let Some(drag) = self.mouse_drag else {
            return Ok(());
        };
        let point = {
            let theme_registry = self.runtime.services().get::<ThemeRegistry>();
            let buffer =
                shell_buffer(&self.runtime, drag.buffer_id).map_err(ShellError::Runtime)?;
            buffer_point_at_screen(
                buffer,
                PixelRectToRect::rect(drag.rect.x, drag.rect.y, drag.rect.width, drag.rect.height),
                &*shell_user_library(&self.runtime),
                theme_registry,
                ScreenHit {
                    x: mouse_x,
                    y: mouse_y,
                    clamp_body: true,
                    typing_active: false,
                },
                CellMetrics {
                    cell_width,
                    line_height,
                },
            )
        };
        let Some(point) = point else {
            return Ok(());
        };
        let (input_mode, visual_anchor, visual_kind) = {
            let ui = self.ui()?;
            (
                ui.input_mode(),
                ui.vim().visual_anchor,
                ui.vim().visual_kind,
            )
        };
        if drag.kind == VisualSelectionKind::Character
            && point == drag.anchor
            && input_mode != InputMode::Visual
        {
            return Ok(());
        }
        if input_mode != InputMode::Visual
            || visual_anchor != Some(drag.anchor)
            || visual_kind != drag.kind
        {
            self.ui_mut()?.enter_visual_mode(drag.anchor, drag.kind);
        }
        shell_buffer_mut(&mut self.runtime, drag.buffer_id)
            .map_err(ShellError::Runtime)?
            .set_cursor(point);
        Ok(())
    }

    fn finish_mouse_selection(&mut self) -> Result<(), ShellError> {
        let Some(drag) = self.mouse_drag.take() else {
            return Ok(());
        };
        if drag.kind != VisualSelectionKind::Character {
            return Ok(());
        }
        let should_exit_visual = {
            let ui = self.ui()?;
            let buffer =
                shell_buffer(&self.runtime, drag.buffer_id).map_err(ShellError::Runtime)?;
            ui.input_mode() == InputMode::Visual
                && ui.vim().visual_anchor == Some(drag.anchor)
                && buffer.cursor_point() == drag.anchor
        };
        if should_exit_visual {
            self.ui_mut()?.enter_normal_mode();
        }
        Ok(())
    }

    fn refresh_picker_preview_syntax(&mut self) {
        if let Ok(ui) = self.ui_mut()
            && let Some(picker) = ui.picker_mut()
        {
            picker.ensure_selected_workspace_file_preview();
        }
        let preview = self
            .ui()
            .ok()
            .and_then(|ui| ui.picker())
            .filter(|picker| picker.show_preview())
            .and_then(|picker| picker.session().selected())
            .and_then(|selected| {
                let item = selected.item();
                item.preview()
                    .map(|preview| (item.id().to_owned(), preview.to_owned()))
            });
        let Some((item_id, preview)) = preview else {
            if let Ok(ui) = self.ui_mut()
                && let Some(picker) = ui.picker_mut()
            {
                picker.set_preview_syntax(None, IndexedSyntaxLines::new());
            }
            return;
        };
        let key = format!("{item_id}\n{preview}");
        if self
            .ui()
            .ok()
            .and_then(|ui| ui.picker())
            .and_then(PickerOverlay::preview_syntax_key)
            == Some(key.as_str())
        {
            return;
        }
        let syntax_lines =
            picker_preview_syntax_lines(&mut self.runtime, &preview).unwrap_or_default();
        if let Ok(ui) = self.ui_mut()
            && let Some(picker) = ui.picker_mut()
        {
            picker.set_preview_syntax(Some(key), syntax_lines);
        }
    }
}
