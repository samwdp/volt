fn visual_selection(
    buffer: &ShellBuffer,
    anchor: TextPoint,
    kind: VisualSelectionKind,
) -> Option<VisualSelection> {
    let head = buffer.cursor_point();
    match kind {
        VisualSelectionKind::Character => {
            let range = if head >= anchor {
                TextRange::new(anchor, buffer.point_after(head).unwrap_or(head))
            } else {
                TextRange::new(head, buffer.point_after(anchor).unwrap_or(anchor))
            };
            (range.start() != range.end()).then_some(VisualSelection::Range(range.normalized()))
        }
        VisualSelectionKind::Line => {
            let start_line = anchor.line.min(head.line);
            let line_count = anchor.line.max(head.line).saturating_sub(start_line) + 1;
            buffer
                .line_span_range(start_line, line_count)
                .map(VisualSelection::Range)
        }
        VisualSelectionKind::Block => {
            let end_col = anchor.column.max(head.column).saturating_add(1);
            Some(VisualSelection::Block(BlockSelection {
                start_line: anchor.line.min(head.line),
                end_line: anchor.line.max(head.line),
                start_col: anchor.column.min(head.column),
                end_col,
            }))
        }
    }
}

fn block_selection_ranges(buffer: &ShellBuffer, selection: BlockSelection) -> Vec<TextRange> {
    (selection.start_line..=selection.end_line)
        .filter_map(|line_index| {
            let line_len = buffer.line_len_chars(line_index);
            let start = selection.start_col.min(line_len);
            let end = selection.end_col.min(line_len);
            (start < end).then(|| {
                TextRange::new(
                    TextPoint::new(line_index, start),
                    TextPoint::new(line_index, end),
                )
            })
        })
        .collect()
}

fn line_text_without_newline(buffer: &ShellBuffer, line_index: usize) -> Option<String> {
    if line_index >= buffer.line_count() {
        return None;
    }
    let line_len = buffer.line_len_chars(line_index);
    Some(buffer.slice(TextRange::new(
        TextPoint::new(line_index, 0),
        TextPoint::new(line_index, line_len),
    )))
}

fn current_visual_state(
    runtime: &EditorRuntime,
) -> Result<(TextPoint, TextPoint, VisualSelectionKind), String> {
    let ui = shell_ui(runtime)?;
    let anchor = ui
        .vim()
        .visual_anchor
        .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
    let buffer = ui
        .buffer(active_shell_buffer_id(runtime)?)
        .ok_or_else(|| "active visual buffer is missing".to_owned())?;
    Ok((anchor, buffer.cursor_point(), ui.vim().visual_kind))
}

fn current_visual_line_span(
    runtime: &EditorRuntime,
) -> Result<(usize, usize, TextPoint, TextPoint, VisualSelectionKind), String> {
    let (anchor, cursor, kind) = current_visual_state(runtime)?;
    Ok((
        anchor.line.min(cursor.line),
        anchor.line.max(cursor.line),
        cursor,
        anchor,
        kind,
    ))
}

fn join_vim_lines_text(text: &str) -> (String, usize) {
    let trailing_newline = text.ends_with('\n');
    let mut lines = text.split('\n').collect::<Vec<_>>();
    if trailing_newline {
        let _ = lines.pop();
    }
    let Some((first, rest)) = lines.split_first() else {
        return (String::new(), 0);
    };
    let first_segment = first.trim_end_matches([' ', '\t']);
    let cursor_column = first_segment.chars().count();
    let mut joined = first_segment.to_owned();
    for line in rest {
        let segment = line.trim_start_matches([' ', '\t']);
        let needs_space = !joined.is_empty()
            && !joined.chars().last().is_some_and(char::is_whitespace)
            && !segment.is_empty();
        if needs_space {
            joined.push(' ');
        }
        joined.push_str(segment);
    }
    if trailing_newline {
        joined.push('\n');
    }
    (joined, cursor_column)
}

fn resolve_block_insert_text(original: &str, current: &str, insert_col: usize) -> String {
    let original_chars: Vec<char> = original.chars().collect();
    let current_chars: Vec<char> = current.chars().collect();
    let prefix_len = insert_col
        .min(original_chars.len())
        .min(current_chars.len());
    if original_chars[..prefix_len] != current_chars[..prefix_len] {
        return current_chars[prefix_len..].iter().collect();
    }
    let suffix = &original_chars[prefix_len..];
    if current_chars.len() >= prefix_len + suffix.len() {
        let suffix_start = current_chars.len() - suffix.len();
        if current_chars[suffix_start..] == *suffix {
            return current_chars[prefix_len..suffix_start].iter().collect();
        }
    }
    current_chars[prefix_len..].iter().collect()
}

fn prepare_block_insert_state(
    runtime: &mut EditorRuntime,
    selection: BlockSelection,
    insert_col: usize,
    origin_line: usize,
) -> Result<(), String> {
    let original_line = {
        let buffer = active_shell_buffer_mut(runtime)?;
        line_text_without_newline(buffer, origin_line)
            .ok_or_else(|| "block insert origin line is missing".to_owned())?
    };
    shell_ui_mut(runtime)?.vim_mut().block_insert = Some(BlockInsertState {
        selection,
        insert_col,
        origin_line,
        original_line,
    });
    Ok(())
}

fn apply_pending_block_insert(runtime: &mut EditorRuntime) -> Result<(), String> {
    let pending = shell_ui_mut(runtime)?.vim_mut().block_insert.take();
    let Some(pending) = pending else {
        return Ok(());
    };
    let origin_line = pending.origin_line;
    let original_line = pending.original_line;
    let insert_col = pending.insert_col;
    let selection = pending.selection;
    let buffer = active_shell_buffer_mut(runtime)?;
    let Some(current_line) = line_text_without_newline(buffer, origin_line) else {
        return Err("block insert origin line is missing".to_owned());
    };
    let origin_col = insert_col.min(original_line.chars().count());
    let inserted = resolve_block_insert_text(&original_line, &current_line, origin_col);
    if inserted.is_empty() {
        return Ok(());
    }
    let cursor = buffer.cursor_point();
    for line in (selection.start_line..=selection.end_line).rev() {
        if line == origin_line || line >= buffer.line_count() {
            continue;
        }
        let target_col = insert_col.min(buffer.line_len_chars(line));
        buffer.insert_at(TextPoint::new(line, target_col), &inserted);
    }
    buffer.set_cursor(cursor);
    buffer.mark_syntax_dirty();
    Ok(())
}

fn start_visual_block_insert(runtime: &mut EditorRuntime, append: bool) -> Result<(), String> {
    let (selection, insert_col, origin_line) = {
        let ui = shell_ui(runtime)?;
        let anchor = ui
            .vim()
            .visual_anchor
            .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
        let buffer = ui
            .buffer(active_shell_buffer_id(runtime)?)
            .ok_or_else(|| "active visual buffer is missing".to_owned())?;
        let selection = match visual_selection(buffer, anchor, ui.vim().visual_kind) {
            Some(VisualSelection::Block(block)) => block,
            _ => return Err("visual block insert requires block selection".to_owned()),
        };
        let insert_col = if append {
            selection.end_col
        } else {
            selection.start_col
        };
        (selection, insert_col, selection.start_line)
    };
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        let line_len = buffer.line_len_chars(origin_line);
        let target_col = insert_col.min(line_len);
        buffer.set_cursor(TextPoint::new(origin_line, target_col));
    }
    prepare_block_insert_state(runtime, selection, insert_col, origin_line)?;
    shell_ui_mut(runtime)?.enter_insert_mode();
    Ok(())
}

fn line_flash_selection_for_range(
    buffer: &ShellBuffer,
    range: TextRange,
) -> Option<VisualSelection> {
    let range = range.normalized();
    let line_count = range.end().line.saturating_sub(range.start().line) + 1;
    buffer
        .line_span_range(range.start().line, line_count)
        .map(VisualSelection::Range)
}

fn transform_case_text(text: &str, operator: VimOperator) -> String {
    text.chars()
        .map(|character| match operator {
            VimOperator::ToggleCase => {
                if character.is_lowercase() {
                    character.to_uppercase().collect::<String>()
                } else if character.is_uppercase() {
                    character.to_lowercase().collect::<String>()
                } else {
                    character.to_string()
                }
            }
            VimOperator::Lowercase => character.to_lowercase().collect::<String>(),
            VimOperator::Uppercase => character.to_uppercase().collect::<String>(),
            _ => character.to_string(),
        })
        .collect()
}

fn replace_visual_text(text: &str, character: char) -> String {
    let replacement = character.to_string();
    text.chars()
        .map(|current| {
            if current == '\n' {
                "\n".to_owned()
            } else {
                replacement.clone()
            }
        })
        .collect()
}

fn shift_visual_selection(runtime: &mut EditorRuntime, indent: bool) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        shell_ui_mut(runtime)?.enter_normal_mode();
        schedule_finish_change(runtime)?;
        return Ok(());
    }
    let (start_line, end_line, cursor, anchor, kind) = current_visual_line_span(runtime)?;
    store_last_visual_selection(runtime, anchor, cursor, kind)?;
    let (indent_size, use_tabs) = {
        let ui = shell_ui(runtime)?;
        let buffer_id = active_shell_buffer_id(runtime)?;
        let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
        let theme_registry = runtime.services().get::<ThemeRegistry>();
        let indent_size = theme_lang_indent(theme_registry, language_id);
        (
            if indent_size == 0 { 4 } else { indent_size },
            theme_lang_use_tabs(theme_registry, language_id),
        )
    };
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        for line_index in start_line..=end_line {
            let line = buffer.text.line(line_index).unwrap_or_default();
            let (columns, _) = leading_whitespace_info(&line, indent_size);
            let target_columns = if indent {
                columns.saturating_add(indent_size)
            } else {
                columns.saturating_sub(indent_size)
            };
            let target_indent = indent_string_from_columns(target_columns, indent_size, use_tabs);
            apply_line_indent(buffer, line_index, indent_size, &target_indent);
        }
        let target = buffer
            .text
            .first_non_blank_in_line(start_line)
            .unwrap_or(TextPoint::new(start_line, 0));
        buffer.set_cursor(target);
        buffer.mark_syntax_dirty();
    }
    shell_ui_mut(runtime)?.enter_normal_mode();
    apply_directory_edit_queue_if_needed(runtime)?;
    schedule_finish_change(runtime)?;
    Ok(())
}

fn join_visual_selection_lines(runtime: &mut EditorRuntime) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        shell_ui_mut(runtime)?.enter_normal_mode();
        schedule_finish_change(runtime)?;
        return Ok(());
    }
    let (start_line, end_line, cursor, anchor, kind) = current_visual_line_span(runtime)?;
    store_last_visual_selection(runtime, anchor, cursor, kind)?;
    if start_line == end_line {
        shell_ui_mut(runtime)?.enter_normal_mode();
        schedule_finish_change(runtime)?;
        return Ok(());
    }
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        let range = buffer
            .line_span_range(
                start_line,
                end_line.saturating_sub(start_line).saturating_add(1),
            )
            .ok_or_else(|| "visual join range is unavailable".to_owned())?;
        let original = buffer.slice(range);
        let (joined, cursor_column) = join_vim_lines_text(&original);
        buffer.replace_range(range, &joined);
        buffer.set_cursor(TextPoint::new(start_line, cursor_column));
        buffer.mark_syntax_dirty();
    }
    shell_ui_mut(runtime)?.enter_normal_mode();
    apply_directory_edit_queue_if_needed(runtime)?;
    schedule_finish_change(runtime)?;
    Ok(())
}

fn move_visual_selection_lines(runtime: &mut EditorRuntime, down: bool) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        shell_ui_mut(runtime)?.enter_normal_mode();
        schedule_finish_change(runtime)?;
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (start_line, end_line, cursor, anchor, kind) = current_visual_line_span(runtime)?;
    let line_count = shell_buffer(runtime, buffer_id)?.line_count();
    if (down && end_line.saturating_add(1) >= line_count) || (!down && start_line == 0) {
        return Ok(());
    }
    let (indent_size, use_tabs) = {
        let ui = shell_ui(runtime)?;
        let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
        let theme_registry = runtime.services().get::<ThemeRegistry>();
        let indent_size = theme_lang_indent(theme_registry, language_id);
        (
            if indent_size == 0 { 4 } else { indent_size },
            theme_lang_use_tabs(theme_registry, language_id),
        )
    };
    let (replacement_range, replacement_text, moved_start_line, moved_end_line) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let selected_range = buffer
            .line_span_range(
                start_line,
                end_line.saturating_sub(start_line).saturating_add(1),
            )
            .ok_or_else(|| "visual move range is unavailable".to_owned())?;
        let selected_text = buffer.slice(selected_range);
        if down {
            let adjacent_range = buffer
                .line_span_range(end_line.saturating_add(1), 1)
                .ok_or_else(|| "visual move adjacent range is unavailable".to_owned())?;
            let adjacent_text = buffer.slice(adjacent_range);
            (
                TextRange::new(selected_range.start(), adjacent_range.end()),
                format!("{adjacent_text}{selected_text}"),
                start_line.saturating_add(1),
                end_line.saturating_add(1),
            )
        } else {
            let adjacent_range = buffer
                .line_span_range(start_line.saturating_sub(1), 1)
                .ok_or_else(|| "visual move adjacent range is unavailable".to_owned())?;
            let adjacent_text = buffer.slice(adjacent_range);
            (
                TextRange::new(adjacent_range.start(), selected_range.end()),
                format!("{selected_text}{adjacent_text}"),
                start_line.saturating_sub(1),
                end_line.saturating_sub(1),
            )
        }
    };
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        buffer.replace_range(replacement_range, &replacement_text);
        buffer.mark_syntax_dirty();
    }
    for line_index in moved_start_line..=moved_end_line {
        let indent = {
            let text = shell_buffer(runtime, buffer_id)?.text.clone();
            indent_string_from_columns(
                desired_reindent_columns_for_line(&text, line_index, indent_size),
                indent_size,
                use_tabs,
            )
        };
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        apply_line_indent(buffer, line_index, indent_size, &indent);
    }
    let new_anchor = TextPoint::new(
        if down {
            anchor.line.saturating_add(1)
        } else {
            anchor.line.saturating_sub(1)
        },
        anchor.column,
    );
    let new_cursor = TextPoint::new(
        if down {
            cursor.line.saturating_add(1)
        } else {
            cursor.line.saturating_sub(1)
        },
        cursor.column,
    );
    let (new_anchor, new_cursor) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let anchor_column = new_anchor
            .column
            .min(buffer.line_len_chars(new_anchor.line));
        let cursor_column = new_cursor
            .column
            .min(buffer.line_len_chars(new_cursor.line));
        let new_anchor = TextPoint::new(new_anchor.line, anchor_column);
        let new_cursor = TextPoint::new(new_cursor.line, cursor_column);
        buffer.set_cursor(new_cursor);
        (new_anchor, new_cursor)
    };
    shell_ui_mut(runtime)?.enter_visual_mode(new_anchor, kind);
    store_last_visual_selection(runtime, new_anchor, new_cursor, kind)?;
    apply_directory_edit_queue_if_needed(runtime)?;
    schedule_finish_change(runtime)?;
    Ok(())
}

fn replace_visual_selection_chars(
    runtime: &mut EditorRuntime,
    character: char,
) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        shell_ui_mut(runtime)?.enter_normal_mode();
        return Ok(());
    }
    let (selection, cursor, kind, anchor) = {
        let ui = shell_ui(runtime)?;
        let anchor = ui
            .vim()
            .visual_anchor
            .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
        let kind = ui.vim().visual_kind;
        let buffer = ui
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
        VisualSelection::Range(range) => {
            let buffer = active_shell_buffer_mut(runtime)?;
            let replaced = replace_visual_text(&buffer.slice(range), character);
            buffer.replace_range(range, &replaced);
            buffer.set_cursor(range.start());
            buffer.mark_syntax_dirty();
        }
        VisualSelection::Block(block) => {
            let ranges = {
                let buffer = active_shell_buffer_mut(runtime)?;
                block_selection_ranges(buffer, block)
            };
            if ranges.is_empty() {
                shell_ui_mut(runtime)?.enter_normal_mode();
                return Ok(());
            }
            let buffer = active_shell_buffer_mut(runtime)?;
            for range in ranges.iter().rev().copied() {
                let replaced = replace_visual_text(&buffer.slice(range), character);
                buffer.replace_range(range, &replaced);
            }
            buffer.set_cursor(TextPoint::new(block.start_line, block.start_col));
            buffer.mark_syntax_dirty();
        }
    }
    shell_ui_mut(runtime)?.enter_normal_mode();
    apply_directory_edit_queue_if_needed(runtime)?;
    Ok(())
}
