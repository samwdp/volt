fn apply_linewise_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    line_count: usize,
) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        return apply_input_linewise_operator(runtime, operator, line_count);
    }
    let (range, original_cursor, flash_selection) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let original_cursor = buffer.cursor_point();
        let range = buffer
            .line_span_range(buffer.cursor_row(), line_count.max(1))
            .ok_or_else(|| "linewise Vim range could not be resolved".to_owned())?;
        (range, original_cursor, Some(VisualSelection::Range(range)))
    };
    apply_operator_to_range(
        runtime,
        operator,
        range,
        true,
        original_cursor,
        flash_selection,
    )
}

fn apply_text_object_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    kind: VimTextObjectKind,
    around: bool,
    count: usize,
) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        return apply_input_text_object_operator(runtime, operator, kind, around, count);
    }
    if shell_ui(runtime)?.vim().multicursor.is_some()
        && apply_multicursor_text_object_operator(runtime, operator, kind)?
    {
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
        // Multicursor text objects intentionally operate on the linked token set as a whole, so
        // the per-command around/count modifiers do not change that mirrored scope yet.
        let _ = around;
        let _ = count;
        return Ok(());
    }
    let (range, original_cursor, flash_selection) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let original_cursor = buffer.cursor_point();
        let range = buffer
            .text_object_range(kind, around, count.max(1))
            .ok_or_else(|| "text object is unavailable at the current cursor".to_owned())?;
        (
            range,
            original_cursor,
            line_flash_selection_for_range(buffer, range),
        )
    };
    apply_operator_to_range(
        runtime,
        operator,
        range,
        false,
        original_cursor,
        flash_selection,
    )
}

fn apply_motion_alias(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    motion: ShellMotion,
) -> Result<(), String> {
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    apply_operator_motion(runtime, operator, count, motion, None)
}

fn apply_visual_text_object(
    runtime: &mut EditorRuntime,
    kind: VimTextObjectKind,
    around: bool,
    count: usize,
) -> Result<(), String> {
    let (anchor, head) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let range = buffer
            .text_object_range(kind, around, count.max(1))
            .ok_or_else(|| "text object is unavailable at the current cursor".to_owned())?;
        let head = buffer
            .text
            .point_before(range.end())
            .unwrap_or(range.start());
        (range.start(), head)
    };
    active_shell_buffer_mut(runtime)?.set_cursor(head);
    shell_ui_mut(runtime)?.enter_visual_mode(anchor, VisualSelectionKind::Character);
    Ok(())
}

fn delete_chars(runtime: &mut EditorRuntime, backward: bool) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    let motion = if backward {
        ShellMotion::Left
    } else {
        ShellMotion::Right
    };
    apply_operator_motion(runtime, VimOperator::Delete, count, motion, Some(1))
}

fn substitute_chars(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    apply_operator_motion(
        runtime,
        VimOperator::Change,
        count,
        ShellMotion::Right,
        Some(1),
    )
}

fn start_replace_char(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::ReplaceChar { count });
    Ok(())
}

fn toggle_case_chars(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    let (range, end_point) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let Some(range) = range_for_char_count(buffer, count) else {
            return Ok(());
        };
        range
    };
    let buffer = active_shell_buffer_mut(runtime)?;
    let removed = buffer.slice(range);
    let replaced = transform_case_text(&removed, VimOperator::ToggleCase);
    buffer.replace_range(range, &replaced);
    buffer.set_cursor(end_point);
    buffer.mark_syntax_dirty();
    shell_ui_mut(runtime)?.enter_normal_mode();
    apply_directory_edit_queue_if_needed(runtime)?;
    schedule_finish_change(runtime)?;
    Ok(())
}

fn range_for_char_count(buffer: &ShellBuffer, count: usize) -> Option<(TextRange, TextPoint)> {
    let start = buffer.cursor_point();
    let mut end = start;
    for _ in 0..count.max(1) {
        let next = buffer.point_after(end)?;
        if buffer.slice(TextRange::new(end, next)) == "\n" {
            break;
        }
        end = next;
    }
    (end != start).then_some((TextRange::new(start, end), end))
}

fn apply_input_operator_to_char_range(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    start: usize,
    end: usize,
    linewise: bool,
) -> Result<(), String> {
    let removed = {
        let input = active_shell_buffer_mut(runtime)?
            .input_field_mut()
            .ok_or_else(|| "input field is missing".to_owned())?;
        input.slice_char_range(start, end)
    };
    if removed.is_empty() {
        shell_ui_mut(runtime)?.enter_normal_mode();
        return Ok(());
    }

    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        let yank = if linewise {
            YankRegister::Line(removed.clone())
        } else {
            YankRegister::Character(removed.clone())
        };
        store_yank_register(runtime, yank, true)?;
    }

    match operator {
        VimOperator::Delete => {
            active_shell_buffer_mut(runtime)?
                .input_field_mut()
                .ok_or_else(|| "input field is missing".to_owned())?
                .delete_char_range(start, end);
            shell_ui_mut(runtime)?.enter_normal_mode();
            schedule_finish_change(runtime)?;
        }
        VimOperator::Change => {
            let input = active_shell_buffer_mut(runtime)?
                .input_field_mut()
                .ok_or_else(|| "input field is missing".to_owned())?;
            if linewise && removed.ends_with('\n') {
                input.replace_char_range(start, end, "\n");
                input.set_cursor_char(start);
            } else {
                input.delete_char_range(start, end);
            }
            shell_ui_mut(runtime)?.enter_insert_mode();
            mark_change_finish_on_normal(runtime)?;
        }
        VimOperator::Yank => {
            if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                input.set_cursor_char(start);
            }
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let replaced = transform_case_text(&removed, operator);
            let input = active_shell_buffer_mut(runtime)?
                .input_field_mut()
                .ok_or_else(|| "input field is missing".to_owned())?;
            input.replace_char_range(start, end, &replaced);
            input.set_cursor_char(start);
            shell_ui_mut(runtime)?.enter_normal_mode();
            schedule_finish_change(runtime)?;
        }
    }

    Ok(())
}

fn apply_input_linewise_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    line_count: usize,
) -> Result<(), String> {
    let (start, end) = {
        let input = active_shell_buffer_mut(runtime)?
            .input_field_mut()
            .ok_or_else(|| "input field is missing".to_owned())?;
        let line = input.cursor_point().line;
        input
            .line_span_range_chars(line, line_count.max(1))
            .ok_or_else(|| "linewise input Vim range could not be resolved".to_owned())?
    };
    apply_input_operator_to_char_range(runtime, operator, start, end, true)
}

fn apply_input_text_object_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    kind: VimTextObjectKind,
    around: bool,
    count: usize,
) -> Result<(), String> {
    let (start, end) = {
        let input = active_shell_buffer_mut(runtime)?
            .input_field_mut()
            .ok_or_else(|| "input field is missing".to_owned())?;
        input
            .text_object_range_chars(kind, around, count.max(1))
            .ok_or_else(|| "text object is unavailable at current input cursor".to_owned())?
    };
    apply_input_operator_to_char_range(runtime, operator, start, end, false)
}

fn apply_operator_motion(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    operator_count: usize,
    motion: ShellMotion,
    motion_count: Option<usize>,
) -> Result<(), String> {
    let motion = change_operator_word_motion(operator, motion);
    if active_shell_buffer_vim_targets_input(runtime)? {
        return apply_input_operator_motion(
            runtime,
            operator,
            operator_count,
            motion,
            motion_count,
        );
    }
    let (range, linewise, original_cursor, flash_selection) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let original_cursor = buffer.cursor_point();
        let range = match motion {
            ShellMotion::Down => {
                let line_count = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .saturating_add(1);
                buffer.line_span_range(buffer.cursor_row(), line_count)
            }
            ShellMotion::Up => {
                let line_count = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .saturating_add(1);
                let start_line = buffer
                    .cursor_row()
                    .saturating_sub(line_count.saturating_sub(1));
                Some(TextRange::new(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "up motion start line is unavailable".to_owned())?
                        .start(),
                    buffer
                        .line_range(buffer.cursor_row())
                        .ok_or_else(|| "up motion end line is unavailable".to_owned())?
                        .end(),
                ))
            }
            ShellMotion::FirstLine => {
                let target_line = motion_count.unwrap_or(1).saturating_sub(1);
                let start_line = target_line.min(buffer.cursor_row());
                let end_line = target_line.max(buffer.cursor_row());
                Some(TextRange::new(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "first-line range start is unavailable".to_owned())?
                        .start(),
                    buffer
                        .line_range(end_line)
                        .ok_or_else(|| "first-line range end is unavailable".to_owned())?
                        .end(),
                ))
            }
            ShellMotion::LastLine => {
                let target_line = motion_count
                    .map(|line| line.saturating_sub(1))
                    .unwrap_or(buffer.line_count().saturating_sub(1));
                let start_line = target_line.min(buffer.cursor_row());
                let end_line = target_line.max(buffer.cursor_row());
                Some(TextRange::new(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "last-line range start is unavailable".to_owned())?
                        .start(),
                    buffer
                        .line_range(end_line)
                        .ok_or_else(|| "last-line range end is unavailable".to_owned())?
                        .end(),
                ))
            }
            _ => {
                let repeat = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .max(1);
                if !move_buffer_with_motion(buffer, motion, Some(repeat)) {
                    None
                } else {
                    let target = buffer.cursor_point();
                    let range = charwise_motion_range(
                        buffer,
                        original_cursor,
                        target,
                        motion_is_inclusive(motion),
                    )
                    .map(|range| {
                        trim_word_forward_operator_range(
                            buffer,
                            motion,
                            original_cursor,
                            target,
                            range,
                            repeat,
                        )
                    });
                    buffer.set_cursor(original_cursor);
                    range
                }
            }
        };
        let range =
            range.ok_or_else(|| "Vim operator motion did not resolve a range".to_owned())?;
        (
            range,
            matches!(
                motion,
                ShellMotion::Down
                    | ShellMotion::Up
                    | ShellMotion::FirstLine
                    | ShellMotion::LastLine
            ),
            original_cursor,
            line_flash_selection_for_range(buffer, range),
        )
    };

    apply_operator_to_range(
        runtime,
        operator,
        range,
        linewise,
        original_cursor,
        flash_selection,
    )
}

fn apply_input_operator_motion(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    operator_count: usize,
    motion: ShellMotion,
    motion_count: Option<usize>,
) -> Result<(), String> {
    let (range, linewise) = {
        let input = active_shell_buffer_mut(runtime)?
            .input_field_mut()
            .ok_or_else(|| "input field is missing".to_owned())?;
        let original_cursor = input.cursor_char();
        let original_point = input.cursor_point();
        let range = match motion {
            ShellMotion::Down => {
                let line_count = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .saturating_add(1);
                input.line_span_range_chars(original_point.line, line_count)
            }
            ShellMotion::Up => {
                let line_count = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .saturating_add(1);
                let start_line = original_point
                    .line
                    .saturating_sub(line_count.saturating_sub(1));
                let end_line = original_point.line;
                let (start, _) = input
                    .line_range_chars(start_line)
                    .ok_or_else(|| "up input motion start line unavailable".to_owned())?;
                let (_, end) = input
                    .line_range_chars(end_line)
                    .ok_or_else(|| "up input motion end line unavailable".to_owned())?;
                Some((start, end))
            }
            ShellMotion::FirstLine => {
                let buffer = input.text_buffer();
                let target_line = motion_count.unwrap_or(1).saturating_sub(1);
                let start_line = target_line.min(original_point.line);
                let end_line = target_line.max(original_point.line);
                let start = buffer.point_to_char_index(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "first-line input range start unavailable".to_owned())?
                        .start(),
                );
                let end = buffer.point_to_char_index(
                    buffer
                        .line_range(end_line)
                        .ok_or_else(|| "first-line input range end unavailable".to_owned())?
                        .end(),
                );
                Some((start, end))
            }
            ShellMotion::LastLine => {
                let buffer = input.text_buffer();
                let target_line = motion_count
                    .map(|line| line.saturating_sub(1))
                    .unwrap_or(buffer.line_count().saturating_sub(1));
                let start_line = target_line.min(original_point.line);
                let end_line = target_line.max(original_point.line);
                let start = buffer.point_to_char_index(
                    buffer
                        .line_range(start_line)
                        .ok_or_else(|| "last-line input range start unavailable".to_owned())?
                        .start(),
                );
                let end = buffer.point_to_char_index(
                    buffer
                        .line_range(end_line)
                        .ok_or_else(|| "last-line input range end unavailable".to_owned())?
                        .end(),
                );
                Some((start, end))
            }
            _ => {
                let repeat = operator_count
                    .saturating_mul(motion_count.unwrap_or(1))
                    .max(1);
                if !move_input_with_motion(input, motion, Some(repeat)) {
                    None
                } else {
                    let target = input.cursor_char();
                    let target_point = input.cursor_point();
                    let range = input_charwise_motion_range(
                        input,
                        original_cursor,
                        target,
                        motion_is_inclusive(motion),
                    )
                    .map(|range| {
                        trim_word_forward_input_operator_range(
                            input,
                            motion,
                            original_point,
                            target_point,
                            range,
                            repeat,
                        )
                    });
                    input.set_cursor_char(original_cursor);
                    range
                }
            }
        };
        (
            range.ok_or_else(|| "input Vim operator motion did not resolve a range".to_owned())?,
            matches!(
                motion,
                ShellMotion::Down
                    | ShellMotion::Up
                    | ShellMotion::FirstLine
                    | ShellMotion::LastLine
            ),
        )
    };

    apply_input_operator_to_char_range(runtime, operator, range.0, range.1, linewise)
}

fn apply_motion_command(runtime: &mut EditorRuntime, motion: ShellMotion) -> Result<(), String> {
    let pending_operator = match shell_ui(runtime)?.vim().pending {
        Some(VimPending::Operator { operator, count }) => Some((operator, count)),
        _ => None,
    };

    if let Some((operator, count)) = pending_operator {
        let motion_count = shell_ui_mut(runtime)?.vim_mut().take_count();
        if shell_ui(runtime)?.vim().multicursor.is_some()
            && !active_shell_buffer_vim_targets_input(runtime)?
            && apply_multicursor_operator_motion(runtime, operator, count, motion, motion_count)?
        {
            return Ok(());
        }
        return apply_operator_motion(runtime, operator, count, motion, motion_count);
    }

    if shell_ui(runtime)?.vim().multicursor.is_some()
        && !active_shell_buffer_vim_targets_input(runtime)?
    {
        let _ = apply_multicursor_motion(runtime, motion)?;
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
        return Ok(());
    }

    let count = shell_ui_mut(runtime)?.vim_mut().take_count();
    let visual_mode = shell_ui(runtime)?.input_mode() == InputMode::Visual;
    if !visual_mode
        && let Some(scroll) = terminal_scroll_for_motion(motion, count)
        && scroll_active_terminal_view(runtime, scroll)?
    {
        return Ok(());
    }
    let input_mode = shell_ui(runtime)?.input_mode();
    let target_input = active_shell_buffer_vim_targets_input(runtime)?;
    let handled_input = {
        let buffer = active_shell_buffer_mut(runtime)?;
        if target_input {
            if let Some(input) = buffer.input_field_mut() {
                if matches!(input_mode, InputMode::Visual) && input.selection_anchor.is_none() {
                    input.start_selection();
                }
                Some(move_input_with_motion(input, motion, count))
            } else {
                None
            }
        } else {
            None
        }
    };
    if handled_input.is_none() {
        move_buffer_with_motion(active_shell_buffer_mut(runtime)?, motion, count);
    }
    Ok(())
}

fn apply_scroll_command(runtime: &mut EditorRuntime, command: ScrollCommand) -> Result<(), String> {
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    let visual_mode = shell_ui(runtime)?.input_mode() == InputMode::Visual;
    let terminal_scroll = match command {
        ScrollCommand::HalfPageDown => Some(TerminalViewportScroll::HalfPageDown),
        ScrollCommand::HalfPageUp => Some(TerminalViewportScroll::HalfPageUp),
        ScrollCommand::PageDown => Some(TerminalViewportScroll::PageDown),
        ScrollCommand::PageUp => Some(TerminalViewportScroll::PageUp),
        ScrollCommand::LineDown => Some(TerminalViewportScroll::LineDelta(-(count as i32))),
        ScrollCommand::LineUp => Some(TerminalViewportScroll::LineDelta(count as i32)),
    };
    if !visual_mode
        && let Some(scroll) = terminal_scroll
        && scroll_active_terminal_view(runtime, scroll)?
    {
        return Ok(());
    }
    let buffer = active_shell_buffer_mut(runtime)?;
    let viewport = buffer.viewport_lines().max(1);
    match command {
        ScrollCommand::HalfPageDown => {
            scroll_buffer_with_cursor(buffer, ((viewport / 2).max(1) * count) as i32);
            Ok(())
        }
        ScrollCommand::HalfPageUp => {
            scroll_buffer_with_cursor(buffer, -(((viewport / 2).max(1) * count) as i32));
            Ok(())
        }
        ScrollCommand::PageDown => {
            scroll_buffer_with_cursor(buffer, (viewport * count) as i32);
            Ok(())
        }
        ScrollCommand::PageUp => {
            scroll_buffer_with_cursor(buffer, -((viewport * count) as i32));
            Ok(())
        }
        ScrollCommand::LineDown => {
            scroll_buffer_viewport_only(buffer, count as i32);
            Ok(())
        }
        ScrollCommand::LineUp => {
            scroll_buffer_viewport_only(buffer, -(count as i32));
            Ok(())
        }
    }
}

fn scroll_buffer_with_cursor(buffer: &mut ShellBuffer, delta: i32) {
    if let Some(pane) = buffer.acp_active_pane_state_mut() {
        let screen_offset = pane.cursor_viewport_offset();
        let max_scroll = pane.max_scroll_row() as i32;
        pane.scroll_visual_row =
            ((pane.scroll_visual_row as i32) + delta).clamp(0, max_scroll) as usize;
        let target_visual = pane
            .scroll_visual_row
            .saturating_add(screen_offset)
            .min(acp_pane_total_visual_rows(pane).saturating_sub(1));
        pane.set_cursor(acp_pane_point_for_visual_row(pane, target_visual));
        return;
    }
    let screen_offset = buffer.cursor_viewport_offset();
    buffer.scroll_by(delta);
    let target_line = buffer.line_at_viewport_offset(screen_offset);
    let _ = buffer.goto_line(target_line);
}

fn scroll_buffer_viewport_only(buffer: &mut ShellBuffer, delta: i32) {
    if let Some(pane) = buffer.acp_active_pane_state_mut() {
        let screen_offset = pane.cursor_viewport_offset();
        let max_scroll = pane.max_scroll_row() as i32;
        pane.scroll_visual_row =
            ((pane.scroll_visual_row as i32) + delta).clamp(0, max_scroll) as usize;
        let target_visual = pane
            .scroll_visual_row
            .saturating_add(screen_offset)
            .min(acp_pane_total_visual_rows(pane).saturating_sub(1));
        let point = acp_pane_point_for_visual_row(pane, target_visual);
        pane.set_cursor(point);
        return;
    }
    buffer.scroll_by(delta);
    // Always resolve viewport edges through line_at_viewport_offset. ACP panes scroll
    // by visual row, so comparing cursor_row() to current_scroll_row() wrongly treats a
    // visual offset as a line index and teleports the cursor (and any visual selection).
    let top = buffer.line_at_viewport_offset(0);
    let bottom = buffer.line_at_viewport_offset(buffer.viewport_lines().saturating_sub(1));
    if buffer.cursor_row() < top {
        let _ = buffer.goto_line(top);
    } else if buffer.cursor_row() > bottom {
        let _ = buffer.goto_line(bottom);
    }
}

fn position_current_line_in_viewport(
    runtime: &mut EditorRuntime,
    viewport_offset: usize,
) -> Result<(), String> {
    let buffer = active_shell_buffer_mut(runtime)?;
    let max_scroll = buffer.line_count().saturating_sub(buffer.viewport_lines());
    let target_scroll = buffer.cursor_row().saturating_sub(viewport_offset);
    buffer.scroll_row = target_scroll.min(max_scroll);
    Ok(())
}

fn resolve_find_target(
    runtime: &mut EditorRuntime,
    operator: Option<VimOperator>,
    kind: VimFindKind,
    count: usize,
    target: char,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.vim_mut().last_find = Some(LastFind { kind, target });

    if let Some(operator) = operator {
        let (range, original_cursor, flash_selection) = {
            let buffer = active_shell_buffer_mut(runtime)?;
            let original_cursor = buffer.cursor_point();
            if !buffer.move_find(kind, target, count.max(1)) {
                shell_ui_mut(runtime)?.enter_normal_mode();
                return Ok(());
            }
            let moved_to = buffer.cursor_point();
            let range = charwise_motion_range(
                buffer,
                original_cursor,
                moved_to,
                matches!(kind, VimFindKind::ForwardTo | VimFindKind::BackwardTo),
            )
            .ok_or_else(|| "find motion did not resolve a Vim range".to_owned())?;
            buffer.set_cursor(original_cursor);
            (
                range,
                original_cursor,
                line_flash_selection_for_range(buffer, range),
            )
        };
        apply_operator_to_range(
            runtime,
            operator,
            range,
            false,
            original_cursor,
            flash_selection,
        )?;
    } else {
        active_shell_buffer_mut(runtime)?.move_find(kind, target, count.max(1));
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
    }

    Ok(())
}

fn repeat_last_find(runtime: &mut EditorRuntime, reverse: bool) -> Result<(), String> {
    let last_find = shell_ui(runtime)?
        .vim()
        .last_find
        .ok_or_else(|| "no previous Vim find motion is available".to_owned())?;
    let kind = if reverse {
        reverse_find_kind(last_find.kind)
    } else {
        last_find.kind
    };

    let pending_operator = match shell_ui(runtime)?.vim().pending {
        Some(VimPending::Operator { operator, count }) => Some((operator, count)),
        _ => None,
    };
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    if let Some((operator, operator_count)) = pending_operator {
        resolve_find_target(
            runtime,
            Some(operator),
            kind,
            operator_count.saturating_mul(count).max(1),
            last_find.target,
        )
    } else {
        resolve_find_target(runtime, None, kind, count, last_find.target)
    }
}

fn open_vim_command_line(runtime: &mut EditorRuntime) -> Result<(), String> {
    if !shell_user_library(runtime).commandline_enabled() {
        let picker = picker::picker_overlay(runtime, "commands")?;
        shell_ui_mut(runtime)?.set_picker(picker);
        return Ok(());
    }
    clear_key_sequence(runtime)?;
    shell_ui_mut(runtime)?.set_command_line(CommandLineOverlay::new());
    Ok(())
}

/// Called when the user confirms an [`InputPromptOverlay`] with Enter.
fn dispatch_input_prompt_confirm(
    runtime: &mut EditorRuntime,
    id: &str,
    text: &str,
) -> Result<(), String> {
    match id {
        COMPILE_PROMPT_ID => run_compile_command_streamed(runtime, text),
        DAP_PROGRAM_PROMPT_ID => confirm_dap_program_prompt(runtime, text),
        DAP_PROCESS_PROMPT_ID => confirm_dap_process_prompt(runtime, text),
        DAP_EXPRESSION_ADD_PROMPT_ID => add_dap_expression(runtime, text),
        DAP_EVAL_PROMPT_ID => show_dap_eval_result(runtime, text, DapEvaluateContext::Repl),
        DAP_REPL_PROMPT_ID => submit_dap_repl_expression(runtime, text),
        DAP_BP_CONDITION_PROMPT_ID => {
            apply_dap_breakpoint_extra(runtime, DapBreakpointExtraKind::Condition, text)
        }
        DAP_BP_HIT_CONDITION_PROMPT_ID => {
            apply_dap_breakpoint_extra(runtime, DapBreakpointExtraKind::HitCondition, text)
        }
        DAP_BP_LOG_MESSAGE_PROMPT_ID => {
            apply_dap_breakpoint_extra(runtime, DapBreakpointExtraKind::LogMessage, text)
        }
        _ => Ok(()),
    }
}

fn confirm_dap_program_prompt(runtime: &mut EditorRuntime, text: &str) -> Result<(), String> {
    let program = text.trim();
    if program.is_empty() {
        return Err("Debug program path required".to_owned());
    }
    let pending = shell_ui_mut(runtime)?
        .pending_dap_start
        .take()
        .ok_or_else(|| "dap program prompt has no pending start".to_owned())?;
    let configuration = pending
        .configuration
        .with_target_program(PathBuf::from(program));
    continue_dap_start(
        runtime,
        &pending.adapter_id,
        configuration,
        pending.ask_heuristic_compile,
    )
}

fn confirm_dap_process_prompt(runtime: &mut EditorRuntime, text: &str) -> Result<(), String> {
    let process_id = text
        .trim()
        .parse::<u32>()
        .map_err(|_| "Attach process id must be a number".to_owned())?;
    let pending = shell_ui_mut(runtime)?
        .pending_dap_start
        .take()
        .ok_or_else(|| "dap process prompt has no pending start".to_owned())?;
    let configuration = pending.configuration.with_process_id(process_id);
    continue_dap_start(
        runtime,
        &pending.adapter_id,
        configuration,
        pending.ask_heuristic_compile,
    )
}

fn cycle_vim_command_line_completion(
    runtime: &mut EditorRuntime,
    reverse: bool,
) -> Result<(), String> {
    let (seed, is_vim_command) = {
        let Some(command_line) = shell_ui(runtime)?.command_line() else {
            return Ok(());
        };
        (
            command_line.text().to_owned(),
            matches!(command_line.purpose(), CommandLinePurpose::VimCommand),
        )
    };
    if !is_vim_command {
        return Ok(());
    }
    let matches = vim_command_line_completion_matches(runtime, &seed);
    if matches.is_empty() {
        return Ok(());
    }
    if let Some(command_line) = shell_ui_mut(runtime)?.command_line_mut() {
        command_line.cycle_completion(matches, reverse);
    }
    Ok(())
}

fn vim_command_line_completion_matches(runtime: &EditorRuntime, seed: &str) -> Vec<String> {
    let trimmed = seed.trim();
    if trimmed.starts_with('!') {
        return Vec::new();
    }
    if trimmed.starts_with('%') {
        let candidate = "%s///g";
        return candidate
            .starts_with(trimmed)
            .then_some(candidate.to_owned())
            .into_iter()
            .collect();
    }
    runtime
        .commands()
        .command_names()
        .into_iter()
        .filter(|name| name.starts_with(trimmed))
        .map(str::to_owned)
        .collect()
}

fn submit_vim_command_line(runtime: &mut EditorRuntime) -> Result<(), String> {
    let (text, purpose) = {
        let command_line = shell_ui(runtime)?
            .command_line()
            .ok_or_else(|| "command line is not open".to_owned())?;
        (
            command_line.text().trim().to_owned(),
            command_line.purpose().clone(),
        )
    };
    shell_ui_mut(runtime)?.close_command_line();
    match purpose {
        CommandLinePurpose::VimCommand => execute_vim_command_line(runtime, &text),
        CommandLinePurpose::GitWorktreeNewBranch { buffer_id } => {
            submit_git_worktree_new_branch_name(runtime, buffer_id, &text)
        }
        CommandLinePurpose::IssuesCreate => submit_issues_create(runtime, &text),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VimSubstituteScope {
    CurrentLine,
    FullBuffer,
    LineRange { start_line: usize, end_line: usize },
}
