fn execute_vim_command_line(runtime: &mut EditorRuntime, command: &str) -> Result<(), String> {
    let command = command.trim();
    if command.is_empty() {
        return Ok(());
    }
    if let Some(shell_command) = command.strip_prefix('!') {
        return run_shell_command_from_vim_command_line(runtime, shell_command.trim());
    }
    if let Some((scope, pattern, replacement, flags)) = parse_vim_substitute_command(command)? {
        return apply_vim_substitute_command(runtime, scope, &pattern, &replacement, &flags);
    }
    if runtime.commands().contains(command) {
        runtime
            .execute_command(command)
            .map_err(|error| error.to_string())?;
        sync_active_buffer(runtime)?;
        return Ok(());
    }
    if let Some(spec_id) = command.strip_prefix("lsp.install-server ") {
        return tool_install::handle_lsp_install_hook(runtime, Some(spec_id.trim()));
    }
    if let Some(spec_id) = command.strip_prefix("dap.install-server ") {
        return tool_install::handle_dap_install_hook(runtime, Some(spec_id.trim()));
    }
    let matches = runtime
        .commands()
        .command_names()
        .into_iter()
        .filter(|name| name.starts_with(command))
        .map(str::to_owned)
        .collect::<Vec<_>>();
    match matches.as_slice() {
        [matched] => {
            runtime
                .execute_command(matched)
                .map_err(|error| error.to_string())?;
            sync_active_buffer(runtime)?;
            Ok(())
        }
        [] => Err(format!("unknown command `{command}`")),
        _ => Err(format!("ambiguous command `{command}`")),
    }
}

fn apply_vim_substitute_command(
    runtime: &mut EditorRuntime,
    scope: VimSubstituteScope,
    pattern: &str,
    replacement: &str,
    flags: &str,
) -> Result<(), String> {
    if pattern.is_empty() {
        return Err(":s requires a search pattern".to_owned());
    }
    let replace_all = flags.contains('g');
    if flags.chars().any(|flag| flag != 'g') {
        return Err(format!("unsupported :s flags `{flags}`"));
    }
    let (original_cursor, range, replaced, replacements) = {
        let buffer = active_shell_buffer_mut(runtime)?;
        if buffer.is_read_only() {
            return Err(":s is blocked for read-only buffers".to_owned());
        }
        let original_cursor = buffer.cursor_point();
        let range = match scope {
            VimSubstituteScope::CurrentLine => buffer
                .line_range(buffer.cursor_row())
                .ok_or_else(|| "current line is unavailable for :s".to_owned())?,
            VimSubstituteScope::FullBuffer => buffer.full_range(),
            VimSubstituteScope::LineRange {
                start_line,
                end_line,
            } => buffer
                .line_span_range(
                    start_line,
                    end_line.saturating_sub(start_line).saturating_add(1),
                )
                .ok_or_else(|| "line range is unavailable for :s".to_owned())?,
        };
        let (replaced, replacements) =
            substitute_buffer_text(&buffer.slice(range), pattern, replacement, replace_all);
        (original_cursor, range, replaced, replacements)
    };
    if replacements == 0 {
        return Err(format!("no matches found for `{pattern}`"));
    }
    let buffer = active_shell_buffer_mut(runtime)?;
    buffer.replace_range(range, &replaced);
    if matches!(scope, VimSubstituteScope::FullBuffer) {
        buffer.invalidate_wrap_cache();
    }
    buffer.set_cursor(original_cursor);
    buffer.mark_syntax_dirty();
    Ok(())
}

fn parse_vim_substitute_command(
    command: &str,
) -> Result<Option<(VimSubstituteScope, String, String, String)>, String> {
    let Some((scope, rest)) = parse_vim_substitute_scope(command)? else {
        return Ok(None);
    };
    let Some(delimiter) = rest.chars().next() else {
        return Err(":s requires a delimiter".to_owned());
    };
    let mut remaining = &rest[delimiter.len_utf8()..];
    let (pattern, next) = split_vim_substitute_segment(remaining, delimiter)?;
    remaining = next;
    let (replacement, next) = split_vim_substitute_segment(remaining, delimiter)?;
    remaining = next;
    Ok(Some((
        scope,
        pattern,
        replacement,
        remaining.trim().to_owned(),
    )))
}

fn parse_vim_substitute_scope(command: &str) -> Result<Option<(VimSubstituteScope, &str)>, String> {
    if let Some(rest) = command.strip_prefix("%s") {
        return Ok(Some((VimSubstituteScope::FullBuffer, rest)));
    }
    if let Some(rest) = command.strip_prefix('s') {
        if starts_with_vim_substitute_delimiter(rest) {
            return Ok(Some((VimSubstituteScope::CurrentLine, rest)));
        }
        return Ok(None);
    }
    let Some(s_index) = command.find('s') else {
        return Ok(None);
    };
    let (range_text, tail) = command.split_at(s_index);
    if range_text.is_empty()
        || !range_text
            .chars()
            .all(|character| character.is_ascii_digit() || character == ',')
    {
        return Ok(None);
    }
    let rest = &tail['s'.len_utf8()..];
    if !starts_with_vim_substitute_delimiter(rest) {
        return Ok(None);
    }
    let (start_line, end_line) = parse_vim_substitute_line_range(range_text)?;
    Ok(Some((
        VimSubstituteScope::LineRange {
            start_line,
            end_line,
        },
        rest,
    )))
}

fn starts_with_vim_substitute_delimiter(rest: &str) -> bool {
    rest.chars()
        .next()
        .is_some_and(|character| !character.is_ascii_alphanumeric() && !character.is_whitespace())
}

fn parse_vim_substitute_line_range(range_text: &str) -> Result<(usize, usize), String> {
    let mut parts = range_text.split(',');
    let start_line = parts
        .next()
        .ok_or_else(|| "missing :s range start".to_owned())?
        .parse::<usize>()
        .map_err(|_| format!("invalid :s range `{range_text}`"))?;
    let end_line = parts
        .next()
        .map(|part| {
            part.parse::<usize>()
                .map_err(|_| format!("invalid :s range `{range_text}`"))
        })
        .transpose()?
        .unwrap_or(start_line);
    if parts.next().is_some() || start_line == 0 || end_line == 0 || start_line > end_line {
        return Err(format!("invalid :s range `{range_text}`"));
    }
    Ok((start_line.saturating_sub(1), end_line.saturating_sub(1)))
}

fn split_vim_substitute_segment(input: &str, delimiter: char) -> Result<(String, &str), String> {
    let mut escaped = false;
    for (index, character) in input.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        if character == delimiter {
            let segment = unescape_vim_substitute_segment(&input[..index]);
            let remaining = &input[index + delimiter.len_utf8()..];
            return Ok((segment, remaining));
        }
    }
    Err(format!("missing closing `{delimiter}` in :%s command"))
}

fn unescape_vim_substitute_segment(segment: &str) -> String {
    let mut text = String::new();
    let mut escaped = false;
    for character in segment.chars() {
        if escaped {
            text.push(character);
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
            continue;
        }
        text.push(character);
    }
    if escaped {
        text.push('\\');
    }
    text
}

fn substitute_buffer_text(
    text: &str,
    pattern: &str,
    replacement: &str,
    replace_all: bool,
) -> (String, usize) {
    let mut replacements = 0usize;
    let lines = text
        .split('\n')
        .map(|line| {
            let (updated, count) = substitute_line_text(line, pattern, replacement, replace_all);
            replacements = replacements.saturating_add(count);
            updated
        })
        .collect::<Vec<_>>();
    (lines.join("\n"), replacements)
}

fn substitute_line_text(
    line: &str,
    pattern: &str,
    replacement: &str,
    replace_all: bool,
) -> (String, usize) {
    if pattern.is_empty() {
        return (line.to_owned(), 0);
    }
    if !replace_all {
        if let Some(index) = line.find(pattern) {
            let mut updated = String::new();
            updated.push_str(&line[..index]);
            updated.push_str(replacement);
            updated.push_str(&line[index + pattern.len()..]);
            return (updated, 1);
        }
        return (line.to_owned(), 0);
    }
    let mut remaining = line;
    let mut updated = String::new();
    let mut replacements = 0usize;
    while let Some(index) = remaining.find(pattern) {
        updated.push_str(&remaining[..index]);
        updated.push_str(replacement);
        remaining = &remaining[index + pattern.len()..];
        replacements = replacements.saturating_add(1);
    }
    if replacements == 0 {
        return (line.to_owned(), 0);
    }
    updated.push_str(remaining);
    (updated, replacements)
}

fn open_vim_search_prompt(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
) -> Result<(), String> {
    let title = match direction {
        VimSearchDirection::Forward => "Search /",
        VimSearchDirection::Backward => "Search ?",
    };
    shell_ui_mut(runtime)?.set_picker(PickerOverlay::search(title, direction, Vec::new()));
    Ok(())
}

fn run_vim_search(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
    query: &str,
) -> Result<(), String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(());
    }

    let target = {
        let buffer = active_shell_buffer_mut(runtime)?;
        search_buffer(buffer, direction, query)
            .ok_or_else(|| format!("no matches found for `{query}`"))?
    };
    active_shell_buffer_mut(runtime)?.set_cursor(target);
    shell_ui_mut(runtime)?.vim_mut().last_search = Some(LastSearch {
        direction,
        query: query.to_owned(),
    });
    shell_ui_mut(runtime)?.vim_mut().clear_transient();
    Ok(())
}

fn apply_vim_search_result(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
    target: TextPoint,
    query: &str,
) -> Result<(), String> {
    let query = query.trim();
    if query.is_empty() {
        return Ok(());
    }

    active_shell_buffer_mut(runtime)?.set_cursor(target);
    shell_ui_mut(runtime)?.vim_mut().last_search = Some(LastSearch {
        direction,
        query: query.to_owned(),
    });
    shell_ui_mut(runtime)?.vim_mut().clear_transient();
    Ok(())
}

fn search_word_under_cursor(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
) -> Result<(), String> {
    let query = {
        let buffer = active_shell_buffer_mut(runtime)?;
        let range = buffer
            .text_object_range(VimTextObjectKind::Word, false, 1)
            .ok_or_else(|| "no Vim word is available at the current cursor".to_owned())?;
        buffer.slice(range)
    };
    run_vim_search(runtime, direction, &query)
}

fn submit_vim_search(
    runtime: &mut EditorRuntime,
    direction: VimSearchDirection,
    query: &str,
) -> Result<(), String> {
    if !query.trim().is_empty() {
        return run_vim_search(runtime, direction, query);
    }

    let last_search = shell_ui(runtime)?
        .vim()
        .last_search
        .clone()
        .ok_or_else(|| "no previous Vim search is available".to_owned())?;
    run_vim_search(runtime, direction, &last_search.query)
}

fn repeat_vim_search(runtime: &mut EditorRuntime, reverse: bool) -> Result<(), String> {
    let last_search = shell_ui(runtime)?
        .vim()
        .last_search
        .clone()
        .ok_or_else(|| "no previous Vim search is available".to_owned())?;
    let direction = if reverse {
        reverse_search_direction(last_search.direction)
    } else {
        last_search.direction
    };
    run_vim_search(runtime, direction, &last_search.query)?;
    shell_ui_mut(runtime)?.vim_mut().last_search = Some(last_search);
    Ok(())
}

fn resolve_g_prefix(
    runtime: &mut EditorRuntime,
    operator: Option<VimOperator>,
    line_target: Option<usize>,
    chord: &str,
) -> Result<(), String> {
    match chord {
        "g" => {
            if let Some(operator) = operator {
                let (range, original_cursor, flash_selection) = {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    let original_cursor = buffer.cursor_point();
                    let target_line = line_target.unwrap_or(1).saturating_sub(1);
                    let start_line = target_line.min(buffer.cursor_row());
                    let end_line = target_line.max(buffer.cursor_row());
                    let range = TextRange::new(
                        buffer
                            .line_range(start_line)
                            .ok_or_else(|| "gg range start is unavailable".to_owned())?
                            .start(),
                        buffer
                            .line_range(end_line)
                            .ok_or_else(|| "gg range end is unavailable".to_owned())?
                            .end(),
                    );
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
            } else {
                let target_line = line_target.unwrap_or(1).saturating_sub(1);
                active_shell_buffer_mut(runtime)?.goto_line(target_line);
                shell_ui_mut(runtime)?.vim_mut().clear_transient();
                Ok(())
            }
        }
        "e" | "E" => {
            let motion = if chord == "e" {
                ShellMotion::WordEndBackward
            } else {
                ShellMotion::BigWordEndBackward
            };
            if let Some(operator) = operator {
                let motion_count = line_target;
                let operator_count = 1;
                apply_operator_motion(runtime, operator, operator_count, motion, motion_count)
            } else {
                move_buffer_with_motion(active_shell_buffer_mut(runtime)?, motion, line_target);
                shell_ui_mut(runtime)?.vim_mut().clear_transient();
                Ok(())
            }
        }
        _ => {
            shell_ui_mut(runtime)?.vim_mut().clear_transient();
            Ok(())
        }
    }
}

fn start_vim_operator(runtime: &mut EditorRuntime, operator: VimOperator) -> Result<(), String> {
    if matches!(
        operator,
        VimOperator::Delete
            | VimOperator::Change
            | VimOperator::ToggleCase
            | VimOperator::Lowercase
            | VimOperator::Uppercase
    ) {
        start_change_recording(runtime)?;
    }
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::Operator { operator, count });
    Ok(())
}

fn start_vim_format(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_change_recording(runtime)?;
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::Format { count });
    Ok(())
}

fn start_vim_find(runtime: &mut EditorRuntime, kind: VimFindKind) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    let pending_operator = match ui.vim().pending {
        Some(VimPending::Operator { operator, count }) => Some((operator, count)),
        _ => None,
    };
    let count = ui.vim_mut().take_count_or_one();
    ui.vim_mut().pending = Some(VimPending::FindTarget {
        operator: pending_operator.map(|(operator, _)| operator),
        kind,
        count: pending_operator
            .map(|(_, operator_count)| operator_count.saturating_mul(count))
            .unwrap_or(count),
    });
    Ok(())
}

fn start_vim_g_prefix(runtime: &mut EditorRuntime) -> Result<(), String> {
    let line_target = shell_ui_mut(runtime)?.vim_mut().take_count();
    let vim = shell_ui_mut(runtime)?.vim_mut();
    vim.pending_change_prefix = Some(VimRecordedInput::Text("g".to_owned()));
    vim.pending = Some(VimPending::GPrefix {
        operator: None,
        line_target,
    });
    Ok(())
}

fn start_visual_mode_with_kind(
    runtime: &mut EditorRuntime,
    kind: VisualSelectionKind,
) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        let cursor = {
            let buffer = active_shell_buffer_mut(runtime)?;
            let Some(input) = buffer.input_field_mut() else {
                return Ok(());
            };
            input.start_selection();
            input.cursor_point()
        };
        shell_ui_mut(runtime)?.enter_visual_mode(cursor, kind);
        return Ok(());
    }
    let cursor = active_shell_buffer_mut(runtime)?.cursor_point();
    shell_ui_mut(runtime)?.enter_visual_mode(cursor, kind);
    Ok(())
}

fn start_visual_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_visual_mode_with_kind(runtime, VisualSelectionKind::Character)
}

fn start_visual_line_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_visual_mode_with_kind(runtime, VisualSelectionKind::Line)
}

fn start_visual_block_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    start_visual_mode_with_kind(runtime, VisualSelectionKind::Block)
}

fn start_visual_text_object(runtime: &mut EditorRuntime, around: bool) -> Result<(), String> {
    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::VisualTextObject { around, count });
    Ok(())
}

fn toggle_visual_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.vim().multicursor.is_some()
        && !active_shell_buffer_vim_targets_input(runtime)?
    {
        return toggle_multicursor_visual_mode(runtime);
    }
    let mode = shell_ui(runtime)?.input_mode();
    if mode != InputMode::Visual {
        return start_visual_mode(runtime);
    }

    if shell_ui(runtime)?.vim().visual_kind == VisualSelectionKind::Character {
        shell_ui_mut(runtime)?.enter_normal_mode();
    } else {
        let ui = shell_ui_mut(runtime)?;
        ui.vim_mut().visual_kind = VisualSelectionKind::Character;
        ui.vim_mut().clear_transient();
    }

    Ok(())
}

fn toggle_visual_line_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let mode = shell_ui(runtime)?.input_mode();
    if mode != InputMode::Visual {
        return start_visual_line_mode(runtime);
    }

    if shell_ui(runtime)?.vim().visual_kind == VisualSelectionKind::Line {
        shell_ui_mut(runtime)?.enter_normal_mode();
    } else {
        let ui = shell_ui_mut(runtime)?;
        ui.vim_mut().visual_kind = VisualSelectionKind::Line;
        ui.vim_mut().clear_transient();
    }

    Ok(())
}

fn toggle_visual_block_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let mode = shell_ui(runtime)?.input_mode();
    if mode != InputMode::Visual {
        return start_visual_block_mode(runtime);
    }

    if shell_ui(runtime)?.vim().visual_kind == VisualSelectionKind::Block {
        shell_ui_mut(runtime)?.enter_normal_mode();
    } else {
        let ui = shell_ui_mut(runtime)?;
        ui.vim_mut().visual_kind = VisualSelectionKind::Block;
        ui.vim_mut().clear_transient();
    }

    Ok(())
}

fn swap_visual_anchor(runtime: &mut EditorRuntime) -> Result<(), String> {
    let current = active_shell_buffer_mut(runtime)?.cursor_point();
    let anchor = shell_ui(runtime)?
        .vim()
        .visual_anchor
        .ok_or_else(|| "visual selection anchor is missing".to_owned())?;
    active_shell_buffer_mut(runtime)?.set_cursor(anchor);
    let ui = shell_ui_mut(runtime)?;
    ui.vim_mut().visual_anchor = Some(current);
    ui.vim_mut().visual_anchor_char_offset = ui
        .focused_buffer_id()
        .and_then(|buffer_id| ui.buffer(buffer_id))
        .and_then(|buffer| buffer.char_offset_for_point(current));
    ui.vim_mut().clear_transient();
    Ok(())
}

fn remap_acp_output_visual_anchors(runtime: &mut EditorRuntime) -> Result<(), String> {
    if shell_ui(runtime)?.input_mode() != InputMode::Visual {
        return Ok(());
    }
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !buffer_is_acp(&buffer.kind) || buffer.acp_active_pane() != Some(AcpPane::Output) {
        return Ok(());
    }
    let Some(offset) = shell_ui(runtime)?.vim().visual_anchor_char_offset else {
        return Ok(());
    };
    let anchor = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let pane = buffer
            .acp_state
            .as_ref()
            .map(|state| &state.output_pane)
            .ok_or_else(|| "ACP output pane is missing".to_owned())?;
        pane.text
            .point_from_char_index(offset.min(pane.text.char_count()))
    };
    shell_ui_mut(runtime)?.vim_mut().visual_anchor = Some(anchor);
    Ok(())
}

fn resolve_put_yank(runtime: &mut EditorRuntime) -> Result<Option<YankRegister>, String> {
    let (active_register, fallback_yank) = {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        (vim.active_register.take(), vim.yank.clone())
    };
    let yank = if let Some(register) = active_register {
        let vim = shell_ui_mut(runtime)?.vim_mut();
        vim.registers.get(&register).cloned().or(fallback_yank)
    } else {
        let clipboard_text = read_system_clipboard();
        let clipboard_yank = clipboard_text.as_deref().and_then(yank_from_clipboard_text);
        // Prefer internal block yanks when the clipboard matches them, since block shapes
        // cannot be reconstructed from clipboard text alone.
        let prefer_internal_block = match (fallback_yank.as_ref(), clipboard_text.as_deref()) {
            (Some(block @ YankRegister::Block(_)), Some(text)) => {
                let block_text = yank_to_clipboard_text(block);
                text == block_text.as_ref()
            }
            (Some(YankRegister::Block(_)), None) => true,
            _ => false,
        };
        if prefer_internal_block {
            fallback_yank
        } else if let Some(clipboard) = clipboard_yank {
            shell_ui_mut(runtime)?.vim_mut().yank = Some(clipboard.clone());
            Some(clipboard)
        } else {
            fallback_yank
        }
    };
    Ok(yank)
}

fn put_yank(runtime: &mut EditorRuntime, after: bool) -> Result<(), String> {
    let Some(yank) = resolve_put_yank(runtime)? else {
        return Ok(());
    };
    if active_shell_buffer_is_terminal(runtime)? {
        let text = yank_to_clipboard_text(&yank);
        write_active_terminal_text(runtime, text.as_ref())?;
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
        return Ok(());
    }
    if let YankRegister::Directory(entries) = &yank {
        let buffer_id = active_shell_buffer_id(runtime)?;
        if buffer_is_directory(&shell_buffer(runtime, buffer_id)?.kind) {
            let root = shell_buffer(runtime, buffer_id)?
                .directory_state()
                .ok_or_else(|| "directory state is missing".to_owned())?
                .root
                .clone();
            copy_directory_yank_entries(entries, &root)?;
            let copied_paths = entries
                .iter()
                .filter_map(|entry| entry.path.file_name().map(|name| root.join(name)))
                .collect::<Vec<_>>();
            if patch_directory_created_paths(runtime, buffer_id, &copied_paths).is_err() {
                refresh_directory_buffer(runtime, buffer_id)?;
            }
            shell_ui_mut(runtime)?.vim_mut().clear_transient();
            return Ok(());
        }
    }
    if active_shell_buffer_vim_targets_input(runtime)? {
        start_change_recording(runtime)?;
        {
            let input = active_shell_buffer_mut(runtime)?
                .input_field_mut()
                .ok_or_else(|| "input field is missing".to_owned())?;
            match &yank {
                YankRegister::Character(text) => {
                    let insertion = if after {
                        input
                            .cursor_char()
                            .saturating_add(1)
                            .min(input.char_count())
                    } else {
                        input.cursor_char()
                    };
                    input.set_cursor_char(insertion);
                    input.insert_text(text);
                }
                YankRegister::Line(text) => {
                    let buffer = input.text_buffer();
                    let line = buffer.cursor().line;
                    let current = input
                        .line_range_chars(line)
                        .ok_or_else(|| "current input line is unavailable".to_owned())?;
                    let insertion = if after { current.1 } else { current.0 };
                    let line_count = buffer.line_count();
                    let mut inserted = text.clone();
                    if !inserted.ends_with('\n') {
                        inserted.push('\n');
                    }
                    let cursor = if after && line + 1 >= line_count && input.char_count() > 0 {
                        inserted.insert(0, '\n');
                        insertion.saturating_add(1)
                    } else {
                        insertion
                    };
                    input.replace_char_range(insertion, insertion, &inserted);
                    input.set_cursor_char(cursor);
                }
                YankRegister::Block(lines) => {
                    let insertion = if after {
                        input
                            .cursor_char()
                            .saturating_add(1)
                            .min(input.char_count())
                    } else {
                        input.cursor_char()
                    };
                    input.set_cursor_char(insertion);
                    input.insert_text(&lines.join("\n"));
                }
                YankRegister::Directory(entries) => {
                    let insertion = if after {
                        input
                            .cursor_char()
                            .saturating_add(1)
                            .min(input.char_count())
                    } else {
                        input.cursor_char()
                    };
                    input.set_cursor_char(insertion);
                    input.insert_text(
                        &entries
                            .iter()
                            .map(|entry| entry.label.as_str())
                            .collect::<Vec<_>>()
                            .join("\n"),
                    );
                }
            }
        }
        shell_ui_mut(runtime)?.vim_mut().clear_transient();
        schedule_finish_change(runtime)?;
        return Ok(());
    }

    start_change_recording(runtime)?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (indent_size, use_tabs) = {
        let ui = shell_ui(runtime)?;
        let language_id = ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
        let theme_registry = runtime.services().get::<ThemeRegistry>();
        (
            theme_lang_indent(theme_registry, language_id),
            theme_lang_use_tabs(theme_registry, language_id),
        )
    };
    let should_format_indent = {
        let buffer = active_shell_buffer_mut(runtime)?;
        match yank {
            YankRegister::Character(text) => {
                let insertion_point = if after {
                    buffer
                        .point_after(buffer.cursor_point())
                        .unwrap_or_else(|| buffer.cursor_point())
                } else {
                    buffer.cursor_point()
                };
                buffer.insert_at(insertion_point, &text);
            }
            YankRegister::Line(mut text) => {
                if !text.ends_with('\n') {
                    text.push('\n');
                }
                let line = buffer.cursor_row();
                let insertion_point = if after {
                    buffer
                        .line_range(line)
                        .map(TextRange::end)
                        .unwrap_or_else(|| buffer.cursor_point())
                } else {
                    buffer
                        .line_range(line)
                        .map(TextRange::start)
                        .unwrap_or_else(|| buffer.cursor_point())
                };
                let text = if after && line + 1 >= buffer.line_count() {
                    format!("\n{text}")
                } else {
                    text
                };
                buffer.insert_at(insertion_point, &text);
                if after {
                    buffer.goto_line(line.saturating_add(1));
                } else {
                    buffer.goto_line(line);
                }
            }
            YankRegister::Block(lines) => {
                let origin = buffer.cursor_point();
                let insertion_col = if after {
                    origin.column.saturating_add(1)
                } else {
                    origin.column
                };
                ensure_buffer_has_line(
                    buffer,
                    origin.line.saturating_add(lines.len().saturating_sub(1)),
                );
                for (offset, segment) in lines.iter().enumerate().rev() {
                    let target_line = origin.line + offset;
                    let target_col = insertion_col.min(buffer.line_len_chars(target_line));
                    buffer.insert_at(TextPoint::new(target_line, target_col), segment);
                }
                let target_col = insertion_col.min(buffer.line_len_chars(origin.line));
                buffer.set_cursor(TextPoint::new(origin.line, target_col));
            }
            YankRegister::Directory(entries) => {
                let mut text = entries
                    .iter()
                    .map(|entry| entry.label.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                text.push('\n');
                let line = buffer.cursor_row();
                let insertion_point = if after {
                    buffer
                        .line_range(line)
                        .map(TextRange::end)
                        .unwrap_or_else(|| buffer.cursor_point())
                } else {
                    buffer
                        .line_range(line)
                        .map(TextRange::start)
                        .unwrap_or_else(|| buffer.cursor_point())
                };
                buffer.insert_at(insertion_point, &text);
                buffer.goto_line(if after { line.saturating_add(1) } else { line });
            }
        }
        buffer.supports_text_file_actions()
    };

    if should_format_indent {
        format_current_line_indent(runtime, buffer_id, indent_size, use_tabs)?;
    }
    shell_buffer_mut(runtime, buffer_id)?.mark_syntax_dirty();
    apply_directory_edit_queue_if_needed(runtime)?;

    shell_ui_mut(runtime)?.vim_mut().clear_transient();
    schedule_finish_change(runtime)?;
    Ok(())
}

fn put_yank_over_visual_selection(runtime: &mut EditorRuntime, after: bool) -> Result<(), String> {
    let Some(yank) = resolve_put_yank(runtime)? else {
        shell_ui_mut(runtime)?.enter_normal_mode();
        return Ok(());
    };
    if active_shell_buffer_is_terminal(runtime)? {
        return put_yank(runtime, after);
    }
    if let YankRegister::Directory(entries) = &yank {
        let buffer_id = active_shell_buffer_id(runtime)?;
        if buffer_is_directory(&shell_buffer(runtime, buffer_id)?.kind) {
            let root = shell_buffer(runtime, buffer_id)?
                .directory_state()
                .ok_or_else(|| "directory state is missing".to_owned())?
                .root
                .clone();
            copy_directory_yank_entries(entries, &root)?;
            let copied_paths = entries
                .iter()
                .filter_map(|entry| entry.path.file_name().map(|name| root.join(name)))
                .collect::<Vec<_>>();
            if patch_directory_created_paths(runtime, buffer_id, &copied_paths).is_err() {
                refresh_directory_buffer(runtime, buffer_id)?;
            }
            shell_ui_mut(runtime)?.vim_mut().clear_transient();
            shell_ui_mut(runtime)?.enter_normal_mode();
            return Ok(());
        }
    }

    start_change_recording(runtime)?;

    if active_shell_buffer_vim_targets_input(runtime)? {
        let kind = shell_ui(runtime)?.vim().visual_kind;
        let Some(replaced) = active_shell_buffer_mut(runtime)?
            .input_field_mut()
            .and_then(|input| input.selected_text(kind))
        else {
            shell_ui_mut(runtime)?.enter_normal_mode();
            return Ok(());
        };
        let inserted = yank_to_clipboard_text(&yank).into_owned();
        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
            input.delete_selection_kind(kind);
            input.insert_text(&inserted);
        }
        let replaced_yank = match kind {
            VisualSelectionKind::Line => YankRegister::Line(replaced),
            VisualSelectionKind::Character | VisualSelectionKind::Block => {
                YankRegister::Character(replaced)
            }
        };
        store_yank_register(runtime, replaced_yank, true)?;
        shell_ui_mut(runtime)?.enter_normal_mode();
        schedule_finish_change(runtime)?;
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

    let replaced_yank = match selection {
        VisualSelection::Range(range) => {
            let inserted = yank_to_clipboard_text(&yank).into_owned();
            let target_cursor = if after {
                advance_point_by_text(range.start(), &inserted)
            } else {
                range.start()
            };
            let removed = {
                let buffer = active_shell_buffer_mut(runtime)?;
                let removed = buffer.slice(range);
                if !removed.is_empty() {
                    buffer.replace_range(range, &inserted);
                    buffer.set_cursor(target_cursor);
                    buffer.mark_syntax_dirty();
                }
                removed
            };
            if removed.is_empty() {
                shell_ui_mut(runtime)?.enter_normal_mode();
                return Ok(());
            }
            if kind == VisualSelectionKind::Line {
                YankRegister::Line(removed)
            } else {
                YankRegister::Character(removed)
            }
        }
        VisualSelection::Block(block) => {
            let removed = {
                let buffer = active_shell_buffer_mut(runtime)?;
                let ranges = block_selection_ranges(buffer, block);
                let removed = ranges
                    .iter()
                    .copied()
                    .map(|range| buffer.slice(range))
                    .collect::<Vec<_>>();
                match &yank {
                    YankRegister::Block(lines) => {
                        for index in (0..ranges.len()).rev() {
                            let replacement = lines.get(index).map(String::as_str).unwrap_or("");
                            buffer.replace_range(ranges[index], replacement);
                        }
                    }
                    _ => {
                        let inserted = yank_to_clipboard_text(&yank).into_owned();
                        for range in ranges.iter().rev().copied() {
                            buffer.replace_range(range, &inserted);
                        }
                    }
                }
                if let Some(first) = ranges.first().copied() {
                    buffer.set_cursor(first.start());
                }
                buffer.mark_syntax_dirty();
                removed
            };
            YankRegister::Block(removed)
        }
    };

    store_yank_register(runtime, replaced_yank, true)?;
    shell_ui_mut(runtime)?.enter_normal_mode();
    apply_directory_edit_queue_if_needed(runtime)?;
    schedule_finish_change(runtime)?;
    Ok(())
}

fn ensure_buffer_has_line(buffer: &mut ShellBuffer, target_line: usize) {
    while buffer.line_count() <= target_line {
        let last_line = buffer.line_count().saturating_sub(1);
        let point = TextPoint::new(last_line, buffer.line_len_chars(last_line));
        buffer.insert_at(point, "\n");
    }
}
