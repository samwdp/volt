fn store_yank_register(
    runtime: &mut EditorRuntime,
    yank: YankRegister,
    sync_to_system_clipboard: bool,
) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    vim.yank = Some(yank.clone());
    if let Some(register) = vim.active_register.take() {
        vim.registers.insert(register, yank.clone());
    }
    if sync_to_system_clipboard {
        let text = yank_to_clipboard_text(&yank);
        write_system_clipboard(text.as_ref());
    }
    Ok(())
}

fn start_change_recording(runtime: &mut EditorRuntime) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    if vim.replaying {
        return Ok(());
    }
    if !vim.recording_change {
        vim.recording_change = true;
        vim.change_buffer.clear();
    }
    Ok(())
}

fn start_change_recording_with_prefix(
    runtime: &mut EditorRuntime,
    prefix: Option<VimRecordedInput>,
) -> Result<(), String> {
    start_change_recording(runtime)?;
    if let Some(input) = prefix {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        if vim.change_buffer.is_empty() {
            vim.change_buffer.push(input);
        }
    }
    Ok(())
}

fn mark_change_finish_on_normal(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.vim_mut().finish_change_on_normal = true;
    Ok(())
}

fn schedule_finish_change(runtime: &mut EditorRuntime) -> Result<(), String> {
    shell_ui_mut(runtime)?.vim_mut().finish_change_after_input = true;
    Ok(())
}

fn finish_change_recording(runtime: &mut EditorRuntime) -> Result<(), String> {
    let record_snapshot = {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        if vim.recording_change {
            if !vim.change_buffer.is_empty() {
                vim.last_change = vim.change_buffer.clone();
            }
            vim.change_buffer.clear();
            vim.recording_change = false;
            vim.finish_change_on_normal = false;
            vim.finish_change_after_input = false;
            true
        } else {
            false
        }
    };
    if record_snapshot {
        record_undo_tree_snapshot(runtime)?;
    }
    Ok(())
}

fn record_undo_tree_snapshot(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer = active_shell_buffer_mut(runtime)?;
    buffer.record_undo_snapshot();
    Ok(())
}

fn start_macro_record(runtime: &mut EditorRuntime, register: char) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    vim.recording_macro = Some(register);
    vim.macro_buffer.clear();
    vim.skip_next_macro_input = true;
    vim.clear_transient();
    Ok(())
}

fn stop_macro_record(runtime: &mut EditorRuntime) -> Result<(), String> {
    let vim = shell_ui_mut(runtime)?.vim_mut();
    if let Some(register) = vim.recording_macro.take() {
        let recorded = std::mem::take(&mut vim.macro_buffer);
        vim.macros.insert(register, recorded);
    }
    vim.clear_transient();
    Ok(())
}

fn store_last_visual_selection(
    runtime: &mut EditorRuntime,
    anchor: TextPoint,
    head: TextPoint,
    kind: VisualSelectionKind,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    shell_ui_mut(runtime)?.vim_mut().last_visual = Some(VimVisualSnapshot {
        buffer_id,
        anchor,
        head,
        kind,
    });
    Ok(())
}

fn restore_last_visual_selection(runtime: &mut EditorRuntime) -> Result<(), String> {
    let snapshot = shell_ui(runtime)?.vim().last_visual;
    let Some(snapshot) = snapshot else {
        return Ok(());
    };
    let ui = shell_ui_mut(runtime)?;
    ui.focus_buffer(snapshot.buffer_id);
    let buffer = ui
        .buffer_mut(snapshot.buffer_id)
        .ok_or_else(|| "visual buffer is missing".to_owned())?;
    buffer.set_cursor(snapshot.head);
    ui.enter_visual_mode(snapshot.anchor, snapshot.kind);
    Ok(())
}

fn jump_to_mark(runtime: &mut EditorRuntime, mark: char, linewise: bool) -> Result<(), String> {
    let snapshot = shell_ui(runtime)?.vim().marks.get(&mark).copied();
    let Some(snapshot) = snapshot else {
        return Ok(());
    };
    let ui = shell_ui_mut(runtime)?;
    ui.focus_buffer(snapshot.buffer_id);
    let buffer = ui
        .buffer_mut(snapshot.buffer_id)
        .ok_or_else(|| "mark buffer is missing".to_owned())?;
    if linewise {
        buffer.goto_line(snapshot.point.line);
    } else {
        buffer.set_cursor(snapshot.point);
    }
    ui.vim_mut().clear_transient();
    Ok(())
}

fn directory_yank_for_range(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    range: TextRange,
) -> Result<Option<YankRegister>, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_directory(&buffer.kind) {
        return Ok(None);
    }
    let Some(state) = buffer.directory_state() else {
        return Ok(None);
    };
    let start_line = range.start().line.min(range.end().line);
    let end_line = range.start().line.max(range.end().line);
    let mut entries = Vec::new();
    for line in start_line..=end_line {
        let Some(action) = buffer
            .section_line_meta(line)
            .and_then(|meta| meta.action.as_ref())
        else {
            continue;
        };
        if action.id() != oil_protocol::ACTION_OIL_ENTRY {
            continue;
        }
        let Some(detail) = action.detail() else {
            continue;
        };
        let path = Path::new(detail);
        let Some(entry) = state.entries.iter().find(|entry| entry.path() == path) else {
            continue;
        };
        entries.push(DirectoryYankEntry {
            path: entry.path().to_path_buf(),
            label: directory_entry_label(entry),
            is_dir: matches!(entry.kind(), DirectoryEntryKind::Directory),
        });
    }
    Ok((!entries.is_empty()).then_some(YankRegister::Directory(entries)))
}

fn apply_directory_edit_queue_if_needed(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
    if buffer_is_directory(&kind) {
        apply_directory_edit_queue(runtime, buffer_id)?;
    }
    apply_dap_locals_edits(runtime, buffer_id)
}

fn apply_operator_to_range(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    range: TextRange,
    linewise: bool,
    original_cursor: TextPoint,
    flash_selection: Option<VisualSelection>,
) -> Result<(), String> {
    let removed = active_shell_buffer_mut(runtime)?.slice(range);
    if removed.is_empty() {
        shell_ui_mut(runtime)?.enter_normal_mode();
        return Ok(());
    }

    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        let yank = if linewise {
            let buffer_id = active_shell_buffer_id(runtime)?;
            directory_yank_for_range(runtime, buffer_id, range)?
                .unwrap_or_else(|| YankRegister::Line(removed.clone()))
        } else {
            YankRegister::Character(removed.clone())
        };
        store_yank_register(runtime, yank, true)?;
    }

    match operator {
        VimOperator::Delete => {
            let buffer = active_shell_buffer_mut(runtime)?;
            buffer.delete_range(range);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            apply_directory_edit_queue_if_needed(runtime)?;
            schedule_finish_change(runtime)?;
        }
        VimOperator::Change => {
            let buffer = active_shell_buffer_mut(runtime)?;
            if linewise && removed.ends_with('\n') {
                buffer.replace_range(range, "\n");
                buffer.set_cursor(range.start());
            } else {
                buffer.delete_range(range);
            }
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_insert_mode();
            mark_change_finish_on_normal(runtime)?;
        }
        VimOperator::Yank => {
            if let Some(selection) = flash_selection {
                let buffer_id = active_shell_buffer_id(runtime)?;
                shell_ui_mut(runtime)?.set_yank_flash(buffer_id, selection);
            }
            active_shell_buffer_mut(runtime)?.set_cursor(original_cursor);
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let buffer = active_shell_buffer_mut(runtime)?;
            let replaced = transform_case_text(&removed, operator);
            buffer.replace_range(range, &replaced);
            buffer.set_cursor(original_cursor);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            apply_directory_edit_queue_if_needed(runtime)?;
            schedule_finish_change(runtime)?;
        }
    }

    Ok(())
}

fn apply_block_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    selection: BlockSelection,
    original_cursor: TextPoint,
    flash_selection: Option<VisualSelection>,
) -> Result<(), String> {
    let (ranges, yanked) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let ranges = block_selection_ranges(buffer, selection);
        let yanked = ranges
            .iter()
            .map(|range| buffer.slice(*range))
            .collect::<Vec<_>>();
        (ranges, yanked)
    };
    if ranges.is_empty() {
        shell_ui_mut(runtime)?.enter_normal_mode();
        return Ok(());
    }

    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        store_yank_register(runtime, YankRegister::Block(yanked), true)?;
    }
    let target_cursor = ranges[0].start();

    match operator {
        VimOperator::Delete => {
            let buffer = active_shell_buffer_mut(runtime)?;
            for range in ranges.iter().rev().copied() {
                buffer.delete_range(range);
            }
            buffer.set_cursor(target_cursor);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            apply_directory_edit_queue_if_needed(runtime)?;
            schedule_finish_change(runtime)?;
        }
        VimOperator::Change => {
            {
                let buffer = active_shell_buffer_mut(runtime)?;
                for range in ranges.iter().rev().copied() {
                    buffer.delete_range(range);
                }
                buffer.set_cursor(target_cursor);
                buffer.mark_syntax_dirty();
            }
            prepare_block_insert_state(
                runtime,
                selection,
                selection.start_col,
                target_cursor.line,
            )?;
            shell_ui_mut(runtime)?.enter_insert_mode();
            mark_change_finish_on_normal(runtime)?;
        }
        VimOperator::Yank => {
            if let Some(selection) = flash_selection {
                let buffer_id = active_shell_buffer_id(runtime)?;
                shell_ui_mut(runtime)?.set_yank_flash(buffer_id, selection);
            }
            active_shell_buffer_mut(runtime)?.set_cursor(original_cursor);
            shell_ui_mut(runtime)?.enter_normal_mode();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let buffer = active_shell_buffer_mut(runtime)?;
            for range in ranges.iter().copied() {
                let removed = buffer.slice(range);
                let replaced = transform_case_text(&removed, operator);
                buffer.replace_range(range, &replaced);
            }
            buffer.set_cursor(original_cursor);
            buffer.mark_syntax_dirty();
            shell_ui_mut(runtime)?.enter_normal_mode();
            schedule_finish_change(runtime)?;
        }
    }

    Ok(())
}

fn apply_visual_operator(runtime: &mut EditorRuntime, operator: VimOperator) -> Result<(), String> {
    if shell_ui(runtime)?.vim().multicursor.is_some()
        && !active_shell_buffer_vim_targets_input(runtime)?
    {
        return apply_multicursor_visual_operator(runtime, operator);
    }
    if active_shell_buffer_vim_targets_input(runtime)? {
        let kind = shell_ui(runtime)?.vim().visual_kind;
        let selected = {
            let buffer = active_shell_buffer_mut(runtime)?;
            let Some(input) = buffer.input_field_mut() else {
                return Ok(());
            };
            input.selected_text(kind)
        };
        let Some(selected) = selected else {
            return Ok(());
        };
        match operator {
            VimOperator::Yank => {
                let yank = match kind {
                    VisualSelectionKind::Line => YankRegister::Line(selected),
                    VisualSelectionKind::Character => YankRegister::Character(selected),
                    VisualSelectionKind::Block => return Ok(()),
                };
                store_yank_register(runtime, yank, true)?;
                if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                    input.clear_selection();
                }
                shell_ui_mut(runtime)?.enter_normal_mode();
                return Ok(());
            }
            VimOperator::Delete | VimOperator::Change => {
                if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                    input.delete_selection_kind(kind);
                }
                if operator == VimOperator::Change {
                    shell_ui_mut(runtime)?.enter_insert_mode();
                } else {
                    shell_ui_mut(runtime)?.enter_normal_mode();
                }
                return Ok(());
            }
            VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
                return Ok(());
            }
        }
    }
    let (selection, cursor, kind, anchor) = {
        let ui = shell_ui(runtime)?;
        let anchor = ui
            .vim()
            .visual_anchor
            .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
        let kind = ui.vim().visual_kind;
        let buffer = shell_ui(runtime)?
            .buffer(active_shell_buffer_id(runtime)?)
            .ok_or_else(|| "active visual buffer is missing".to_owned())?;
        (
            visual_selection(buffer, anchor, kind)
                .ok_or_else(|| "visual selection is empty".to_owned())?,
            buffer.cursor_point(),
            kind,
            anchor,
        )
    };
    store_last_visual_selection(runtime, anchor, cursor, kind)?;

    match selection {
        VisualSelection::Range(range) => apply_operator_to_range(
            runtime,
            operator,
            range,
            matches!(kind, VisualSelectionKind::Line),
            cursor,
            (operator == VimOperator::Yank).then_some(selection),
        ),
        VisualSelection::Block(block) => apply_block_operator(
            runtime,
            operator,
            block,
            cursor,
            (operator == VimOperator::Yank).then_some(selection),
        ),
    }
}
