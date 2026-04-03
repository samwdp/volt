use super::*;

#[derive(Default)]
pub(super) struct TerminalBufferState {
    sessions: BTreeMap<BufferId, LiveTerminalSession>,
}

impl TerminalBufferState {
    pub(super) fn contains(&self, buffer_id: BufferId) -> bool {
        self.sessions.contains_key(&buffer_id)
    }

    pub(super) fn session_mut(&mut self, buffer_id: BufferId) -> Option<&mut LiveTerminalSession> {
        self.sessions.get_mut(&buffer_id)
    }

    pub(super) fn insert(&mut self, buffer_id: BufferId, session: LiveTerminalSession) {
        self.sessions.insert(buffer_id, session);
    }

    pub(super) fn remove(&mut self, buffer_id: BufferId) -> Option<LiveTerminalSession> {
        self.sessions.remove(&buffer_id)
    }

    pub(super) fn buffer_ids(&self) -> Vec<BufferId> {
        self.sessions.keys().copied().collect()
    }
}

pub(super) fn terminal_buffer_state(
    runtime: &EditorRuntime,
) -> Result<std::sync::MutexGuard<'_, TerminalBufferState>, String> {
    runtime
        .services()
        .get::<Mutex<TerminalBufferState>>()
        .ok_or_else(|| "terminal buffer state service missing".to_owned())?
        .lock()
        .map_err(|_| "terminal buffer state mutex poisoned".to_owned())
}

pub(super) fn terminal_buffer_state_mut(
    runtime: &mut EditorRuntime,
) -> Result<&mut TerminalBufferState, String> {
    runtime
        .services_mut()
        .get_mut::<Mutex<TerminalBufferState>>()
        .ok_or_else(|| "terminal buffer state service missing".to_owned())?
        .get_mut()
        .map_err(|_| "terminal buffer state mutex poisoned".to_owned())
}

pub(super) fn buffer_is_terminal(kind: &BufferKind) -> bool {
    matches!(kind, BufferKind::Terminal)
}

pub(super) fn close_terminal_buffers_for_workspace(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let buffer_ids = {
        let ui = shell_ui(runtime)?;
        ui.workspace_views
            .get(&workspace_id)
            .map(|view| view.buffer_ids.clone())
            .unwrap_or_default()
    };
    for buffer_id in buffer_ids {
        close_terminal_buffer(runtime, buffer_id)?;
    }
    Ok(())
}

pub(super) fn terminal_scroll_for_motion(
    motion: ShellMotion,
    count: Option<usize>,
) -> Option<TerminalViewportScroll> {
    let count = count.unwrap_or(1).max(1);
    match motion {
        ShellMotion::Down => Some(TerminalViewportScroll::LineDelta(-(count as i32))),
        ShellMotion::Up => Some(TerminalViewportScroll::LineDelta(count as i32)),
        ShellMotion::FirstLine => Some(TerminalViewportScroll::Top),
        ShellMotion::LastLine => Some(TerminalViewportScroll::Bottom),
        _ => None,
    }
}

pub(super) fn refresh_pending_terminal(
    runtime: &mut EditorRuntime,
    render_width: u32,
    render_height: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<bool, String> {
    let resized_buffer = resize_active_terminal_session(
        runtime,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?;
    let active_buffer_id = active_shell_buffer_id(runtime).ok();
    let terminal_buffer_ids = terminal_buffer_state(runtime)?.buffer_ids();
    if terminal_buffer_ids.is_empty() {
        return Ok(false);
    }
    let mut updates = Vec::new();
    {
        let state = terminal_buffer_state_mut(runtime)?;
        for buffer_id in terminal_buffer_ids {
            let Some(session) = state.session_mut(buffer_id) else {
                continue;
            };
            let changed = session.poll().map_err(|error| error.to_string())?;
            if changed || resized_buffer == Some(buffer_id) {
                updates.push((
                    buffer_id,
                    session.snapshot().lines().to_vec(),
                    session.render_snapshot(),
                ));
            }
        }
    }
    let changed = !updates.is_empty();
    for (buffer_id, lines, render) in updates {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.set_terminal_render(render);
        if active_buffer_id == Some(buffer_id) {
            buffer.replace_with_lines_follow_output(lines);
        } else {
            buffer.replace_with_lines_preserve_view(lines);
        }
    }
    Ok(changed)
}

pub(super) fn ensure_terminal_session(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<bool, String> {
    if !buffer_is_terminal(&shell_buffer(runtime, buffer_id)?.kind) {
        return Ok(false);
    }
    if terminal_buffer_state(runtime)?.contains(buffer_id) {
        return Ok(false);
    }
    let config = terminal_spawn_config(runtime, buffer_id, 24, 80)?;
    let session = LiveTerminalSession::spawn(config).map_err(|error| error.to_string())?;
    let lines = session.snapshot().lines().to_vec();
    let render = session.render_snapshot();
    terminal_buffer_state_mut(runtime)?.insert(buffer_id, session);
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.set_terminal_render(render);
    buffer.replace_with_lines_follow_output(lines);
    Ok(true)
}

pub(super) fn resize_active_terminal_session(
    runtime: &mut EditorRuntime,
    render_width: u32,
    render_height: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<Option<BufferId>, String> {
    let Some((buffer_id, rows, cols)) = active_terminal_dimensions(
        runtime,
        render_width,
        render_height,
        cell_width,
        line_height,
    )?
    else {
        return Ok(None);
    };
    ensure_terminal_session(runtime, buffer_id)?;
    let resized = terminal_buffer_state_mut(runtime)?
        .session_mut(buffer_id)
        .ok_or_else(|| format!("terminal session for buffer `{buffer_id}` is missing"))?
        .resize(rows, cols)
        .map_err(|error| error.to_string())?;
    Ok(resized.then_some(buffer_id))
}

pub(super) fn active_terminal_dimensions(
    runtime: &EditorRuntime,
    render_width: u32,
    render_height: u32,
    cell_width: i32,
    line_height: i32,
) -> Result<Option<(BufferId, u16, u16)>, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_terminal(&buffer.kind) {
        return Ok(None);
    }

    let popup = active_runtime_popup(runtime)?;
    let (width, height) = if popup
        .as_ref()
        .is_some_and(|popup| popup.active_buffer == buffer_id)
    {
        (
            render_width,
            popup_window_height(render_height, line_height).max(1),
        )
    } else {
        let popup_height = popup
            .as_ref()
            .map(|_| popup_window_height(render_height, line_height))
            .unwrap_or(0);
        let pane_height = render_height.saturating_sub(popup_height);
        let ui = shell_ui(runtime)?;
        let panes = ui
            .panes()
            .ok_or_else(|| "active workspace view is missing".to_owned())?;
        let pane_rects = match ui.pane_split_direction() {
            PaneSplitDirection::Vertical => {
                vertical_pane_rects(render_width, pane_height, panes.len())
            }
            PaneSplitDirection::Horizontal => {
                horizontal_pane_rects(render_width, pane_height, panes.len())
            }
        };
        let rect = pane_rects
            .get(ui.active_pane_index())
            .ok_or_else(|| "active pane rect is missing".to_owned())?;
        (rect.width, rect.height)
    };

    let rows = buffer_visible_rows_for_height(buffer, height, line_height).max(1);
    let cols = wrap_columns_for_width(width, cell_width).max(1);
    Ok(Some((
        buffer_id,
        rows.min(u16::MAX as usize) as u16,
        cols.min(u16::MAX as usize) as u16,
    )))
}

pub(super) fn terminal_spawn_config(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    rows: u16,
    cols: u16,
) -> Result<LiveTerminalConfig, String> {
    let title = shell_buffer(runtime, buffer_id)?.display_name().to_owned();
    let terminal_config = shell_user_library(runtime).terminal_config();
    let mut config = LiveTerminalConfig::new(title, terminal_config.program, terminal_config.args)
        .with_size(rows, cols);
    if let Some(cwd) = terminal_working_dir(runtime)? {
        config = config.with_cwd(cwd);
    }
    Ok(config)
}

pub(super) fn terminal_working_dir(runtime: &EditorRuntime) -> Result<Option<PathBuf>, String> {
    if let Some(root) = active_workspace_root(runtime)? {
        return Ok(Some(root));
    }
    env::current_dir()
        .map(Some)
        .map_err(|error| format!("failed to determine terminal working directory: {error}"))
}

pub(super) fn scroll_active_terminal_view(
    runtime: &mut EditorRuntime,
    scroll: TerminalViewportScroll,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    if !buffer_is_terminal(&shell_buffer(runtime, buffer_id)?.kind) {
        return Ok(false);
    }
    ensure_terminal_session(runtime, buffer_id)?;
    let (changed, lines, render) = {
        let state = terminal_buffer_state_mut(runtime)?;
        let session = state
            .session_mut(buffer_id)
            .ok_or_else(|| format!("terminal session for buffer `{buffer_id}` is missing"))?;
        let changed = session.scroll_viewport(scroll);
        let lines = changed.then(|| session.snapshot().lines().to_vec());
        let render = changed.then(|| session.render_snapshot());
        (changed, lines, render)
    };
    if !changed {
        return Ok(false);
    }
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    buffer.set_terminal_render(
        render.ok_or_else(|| "terminal render snapshot missing after scroll".to_owned())?,
    );
    buffer.replace_with_lines_preserve_view(
        lines.ok_or_else(|| "terminal lines missing after scroll".to_owned())?,
    );
    Ok(true)
}

pub(super) fn write_active_terminal_text(
    runtime: &mut EditorRuntime,
    text: &str,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    ensure_terminal_session(runtime, buffer_id)?;
    terminal_buffer_state_mut(runtime)?
        .session_mut(buffer_id)
        .ok_or_else(|| format!("terminal session for buffer `{buffer_id}` is missing"))?
        .write_text(text)
        .map_err(|error| error.to_string())
}

pub(super) fn write_active_terminal_key(
    runtime: &mut EditorRuntime,
    key: TerminalKey,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    ensure_terminal_session(runtime, buffer_id)?;
    terminal_buffer_state_mut(runtime)?
        .session_mut(buffer_id)
        .ok_or_else(|| format!("terminal session for buffer `{buffer_id}` is missing"))?
        .write_key(key)
        .map_err(|error| error.to_string())
}

pub(super) fn close_terminal_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
        buffer.clear_terminal_render();
    }
    let Some(mut session) = terminal_buffer_state_mut(runtime)?.remove(buffer_id) else {
        return Ok(());
    };
    session.kill().map_err(|error| {
        format!(
            "failed to terminate terminal session `{}` for buffer `{buffer_id}`: {error}",
            session.title()
        )
    })
}

pub(super) fn terminal_key_for_event(keycode: Keycode, keymod: Mod) -> Option<TerminalKey> {
    if keymod.intersects(ctrl_mod()) && keycode == Keycode::C {
        return Some(TerminalKey::CtrlC);
    }
    match keycode {
        Keycode::Return | Keycode::KpEnter => Some(TerminalKey::Enter),
        Keycode::Backspace => Some(TerminalKey::Backspace),
        Keycode::Delete => Some(TerminalKey::Delete),
        Keycode::Left => Some(TerminalKey::Left),
        Keycode::Right => Some(TerminalKey::Right),
        Keycode::Up => Some(TerminalKey::Up),
        Keycode::Down => Some(TerminalKey::Down),
        Keycode::Home => Some(TerminalKey::Home),
        Keycode::End => Some(TerminalKey::End),
        Keycode::PageUp => Some(TerminalKey::PageUp),
        Keycode::PageDown => Some(TerminalKey::PageDown),
        Keycode::Tab if keymod.intersects(shift_mod()) => Some(TerminalKey::BackTab),
        Keycode::Tab => Some(TerminalKey::Tab),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn render_terminal_buffer(
    target: &mut DrawTarget<'_>,
    buffer: &ShellBuffer,
    terminal_render: &TerminalRenderSnapshot,
    rect: Rect,
    layout: BufferFooterLayout,
    active: bool,
    input_mode: InputMode,
    visual_selection: Option<VisualSelection>,
    yank_flash: Option<VisualSelection>,
    _theme_registry: Option<&ThemeRegistry>,
    base_background: Color,
    cursor_color: Color,
    text_color: Color,
    border_color: Color,
    statusline: String,
    statusline_active: Color,
    statusline_inactive: Color,
    selection_color: Color,
    yank_flash_color: Color,
    cell_width: i32,
    line_height: i32,
) -> Result<(), ShellError> {
    let text_x = rect.x() + 12;
    for (row_index, line) in terminal_render
        .lines()
        .iter()
        .enumerate()
        .take(layout.visible_rows)
    {
        let y = layout.body_y + row_index as i32 * line_height;
        for run in line.runs() {
            let run_x = text_x + run.col() as i32 * cell_width;
            let run_width = (run.width_cells() as i32 * cell_width).max(1) as u32;
            if let Some(background) = run.background() {
                fill_rect(
                    target,
                    PixelRectToRect::rect(run_x, y, run_width, line_height.max(1) as u32),
                    Color::RGB(background.r, background.g, background.b),
                )?;
            }
        }
        let line_len = buffer
            .text
            .line(row_index)
            .map(|line| line.chars().count())
            .unwrap_or_default();
        if let Some((selection_start, selection_end)) = visual_selection
            .and_then(|selection| selection_columns_for_visual(selection, row_index, line_len))
        {
            fill_rect(
                target,
                PixelRectToRect::rect(
                    text_x + selection_start as i32 * cell_width,
                    y,
                    (selection_end.saturating_sub(selection_start) as i32 * cell_width) as u32,
                    line_height.max(1) as u32,
                ),
                selection_color,
            )?;
        }
        if let Some((selection_start, selection_end)) = yank_flash
            .and_then(|selection| selection_columns_for_visual(selection, row_index, line_len))
        {
            fill_rect(
                target,
                PixelRectToRect::rect(
                    text_x + selection_start as i32 * cell_width,
                    y,
                    (selection_end.saturating_sub(selection_start) as i32 * cell_width) as u32,
                    line_height.max(1) as u32,
                ),
                yank_flash_color,
            )?;
        }
        for run in line.runs() {
            let run_x = text_x + run.col() as i32 * cell_width;
            let run_width = (run.width_cells() as i32 * cell_width).max(1) as u32;
            if run.text().chars().any(|character| character != ' ') {
                draw_text(
                    target,
                    run_x,
                    y,
                    run.text(),
                    Color::RGB(run.foreground().r, run.foreground().g, run.foreground().b),
                )?;
            }
            if let Some(underline) = run.underline() {
                fill_rect(
                    target,
                    PixelRectToRect::rect(run_x, y + line_height.saturating_sub(2), run_width, 1),
                    Color::RGB(underline.r, underline.g, underline.b),
                )?;
            }
        }
    }

    let live_terminal_cursor = active
        .then(|| terminal_render.cursor())
        .flatten()
        .filter(|_| matches!(input_mode, InputMode::Insert | InputMode::Replace));
    let buffer_cursor = if matches!(input_mode, InputMode::Normal | InputMode::Visual) || !active {
        let row = buffer.cursor_row();
        let col = buffer.cursor_col();
        let text = buffer
            .text
            .line(row)
            .and_then(|line| line.chars().nth(col))
            .map(|character| character.to_string())
            .unwrap_or_else(|| " ".to_owned());
        Some(editor_terminal::TerminalCursorSnapshot::new(
            row.min(layout.visible_rows.saturating_sub(1)) as u16,
            col.min(terminal_render.cols() as usize) as u16,
            1,
            match input_mode {
                InputMode::Normal | InputMode::Visual => {
                    editor_terminal::TerminalCursorShape::Block
                }
                InputMode::Insert | InputMode::Replace => {
                    editor_terminal::TerminalCursorShape::Beam
                }
            },
            text,
        ))
        .filter(|_| row < layout.visible_rows)
    } else {
        None
    };
    if let Some(cursor) = live_terminal_cursor.or(buffer_cursor.as_ref()) {
        draw_terminal_cursor(
            target,
            text_x,
            layout.body_y,
            cursor,
            cursor.shape(),
            cursor_color,
            base_background,
            cell_width,
            line_height,
        )?;
    }

    fill_rect(
        target,
        PixelRectToRect::rect(
            rect.x() + 8,
            layout.statusline_y - 6,
            rect.width().saturating_sub(16),
            1,
        ),
        border_color,
    )?;
    draw_text(
        target,
        rect.x() + 12,
        layout.statusline_y,
        &statusline,
        if active {
            statusline_active
        } else {
            statusline_inactive
        },
    )?;
    let _ = text_color;
    fill_rect(
        target,
        PixelRectToRect::rect(
            rect.x(),
            rect.y() + rect.height() as i32 - 2,
            rect.width(),
            1,
        ),
        border_color,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_terminal_cursor(
    target: &mut DrawTarget<'_>,
    text_x: i32,
    body_y: i32,
    cursor: &editor_terminal::TerminalCursorSnapshot,
    shape: editor_terminal::TerminalCursorShape,
    cursor_color: Color,
    text_override_color: Color,
    cell_width: i32,
    line_height: i32,
) -> Result<(), ShellError> {
    let x = text_x + cursor.col() as i32 * cell_width;
    let y = body_y + cursor.row() as i32 * line_height;
    let width = (cursor.width_cells() as i32 * cell_width).max(1);
    match shape {
        editor_terminal::TerminalCursorShape::Hidden => {}
        editor_terminal::TerminalCursorShape::Block => {
            fill_rounded_rect(
                target,
                PixelRectToRect::rect(x, y, width as u32, line_height.max(2) as u32),
                2,
                cursor_color,
            )?;
            if !cursor.text().is_empty() {
                draw_text(target, x, y, cursor.text(), text_override_color)?;
            }
        }
        editor_terminal::TerminalCursorShape::Underline => {
            fill_rect(
                target,
                PixelRectToRect::rect(x, y + line_height.saturating_sub(2), width as u32, 2),
                cursor_color,
            )?;
        }
        editor_terminal::TerminalCursorShape::Beam => {
            fill_rect(
                target,
                PixelRectToRect::rect(
                    x,
                    y,
                    (cell_width / 4).max(2) as u32,
                    line_height.max(2) as u32,
                ),
                cursor_color,
            )?;
        }
        editor_terminal::TerminalCursorShape::HollowBlock => {
            fill_rect(
                target,
                PixelRectToRect::rect(x, y, width as u32, 1),
                cursor_color,
            )?;
            fill_rect(
                target,
                PixelRectToRect::rect(x, y + line_height.saturating_sub(1), width as u32, 1),
                cursor_color,
            )?;
            fill_rect(
                target,
                PixelRectToRect::rect(x, y, 1, line_height.max(1) as u32),
                cursor_color,
            )?;
            fill_rect(
                target,
                PixelRectToRect::rect(x + width.saturating_sub(1), y, 1, line_height.max(1) as u32),
                cursor_color,
            )?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CursorTextOverlay {
    draw_x: i32,
    text: String,
    color: Color,
}
