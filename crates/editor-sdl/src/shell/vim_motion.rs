fn exact_match_positions_in_chars(
    chars: &[char],
    pattern: &[char],
    case_sensitive: bool,
) -> Vec<usize> {
    if pattern.is_empty() || pattern.len() > chars.len() {
        return Vec::new();
    }

    let max_start = chars.len().saturating_sub(pattern.len());
    let mut matches = Vec::new();
    for start in 0..=max_start {
        if pattern.iter().enumerate().all(|(offset, expected)| {
            normalize_search_char(chars[start + offset], case_sensitive) == *expected
        }) {
            matches.push(start);
        }
    }
    matches
}

fn fuzzy_match_end_in_chars(
    chars: &[char],
    start: usize,
    pattern: &[char],
    case_sensitive: bool,
) -> Option<usize> {
    if pattern.is_empty()
        || chars
            .get(start)
            .copied()
            .map(|ch| normalize_search_char(ch, case_sensitive))
            != Some(pattern[0])
    {
        return None;
    }

    let mut last_index = start;
    let mut next_index = start.saturating_add(1);
    for target in pattern.iter().skip(1) {
        let found = chars
            .get(next_index..)
            .and_then(|slice| {
                slice
                    .iter()
                    .position(|ch| normalize_search_char(*ch, case_sensitive) == *target)
            })
            .map(|offset| next_index + offset)?;
        last_index = found;
        next_index = found.saturating_add(1);
    }
    Some(last_index)
}

fn fuzzy_match_positions_in_chars(
    chars: &[char],
    pattern: &[char],
    case_sensitive: bool,
) -> Vec<(usize, usize)> {
    if pattern.is_empty() || pattern.len() > chars.len() {
        return Vec::new();
    }

    let mut matches = Vec::new();
    for start in 0..chars.len() {
        if let Some(end) = fuzzy_match_end_in_chars(chars, start, pattern, case_sensitive) {
            matches.push((start, end.saturating_sub(start)));
        }
    }
    matches
}

fn search_start_char(
    buffer: &TextSnapshot,
    direction: VimSearchDirection,
    pattern_len: usize,
) -> usize {
    let cursor = buffer.cursor();
    match direction {
        VimSearchDirection::Forward => buffer
            .point_after(cursor)
            .map(|point| buffer.point_to_char_index(point))
            .unwrap_or(buffer.char_count()),
        VimSearchDirection::Backward => buffer
            .point_before(cursor)
            .map(|point| buffer.point_to_char_index(point))
            .unwrap_or_else(|| buffer.char_count().saturating_sub(pattern_len)),
    }
}

fn pick_search_selection_index(
    matches: &[VimSearchMatch],
    direction: VimSearchDirection,
    start_char: usize,
) -> usize {
    if matches.is_empty() {
        return 0;
    }

    let mut candidates: Vec<(usize, &VimSearchMatch)> = matches
        .iter()
        .enumerate()
        .filter(|(_, matched)| match direction {
            VimSearchDirection::Forward => matched.char_index >= start_char,
            VimSearchDirection::Backward => matched.char_index <= start_char,
        })
        .collect();

    if candidates.is_empty() {
        candidates = matches.iter().enumerate().collect();
    }

    candidates
        .into_iter()
        .min_by(|(_, left), (_, right)| {
            let span_order = left.span.cmp(&right.span);
            if span_order != std::cmp::Ordering::Equal {
                return span_order;
            }
            match direction {
                VimSearchDirection::Forward => left.char_index.cmp(&right.char_index),
                VimSearchDirection::Backward => right.char_index.cmp(&left.char_index),
            }
        })
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn vim_search_entries(
    buffer: &TextSnapshot,
    direction: VimSearchDirection,
    query: &str,
) -> SearchPickerData {
    let query = query.trim();
    if query.is_empty() {
        return SearchPickerData {
            entries: Vec::new(),
            selected_index: 0,
        };
    }

    let case_sensitive = search_is_case_sensitive(query);
    let pattern = normalize_search_pattern(query, case_sensitive);
    let line_count = buffer.line_count();
    let mut matches = Vec::new();

    for line_index in 0..line_count {
        let Some(line) = buffer.line(line_index) else {
            continue;
        };
        let chars: Vec<char> = line.chars().collect();
        let positions = exact_match_positions_in_chars(&chars, &pattern, case_sensitive);
        for start in positions {
            let point = TextPoint::new(line_index, start);
            let char_index = buffer.point_to_char_index(point);
            matches.push(VimSearchMatch {
                point,
                char_index,
                span: pattern.len().saturating_sub(1),
                line_text: line.clone(),
            });
        }
    }

    if matches.is_empty() {
        for line_index in 0..line_count {
            let Some(line) = buffer.line(line_index) else {
                continue;
            };
            let chars: Vec<char> = line.chars().collect();
            let positions = fuzzy_match_positions_in_chars(&chars, &pattern, case_sensitive);
            for (start, span) in positions {
                let point = TextPoint::new(line_index, start);
                let char_index = buffer.point_to_char_index(point);
                matches.push(VimSearchMatch {
                    point,
                    char_index,
                    span,
                    line_text: line.clone(),
                });
            }
        }
    }

    matches.sort_by_key(|matched| (matched.point.line, matched.point.column));

    if matches.len() > SEARCH_PICKER_ITEM_LIMIT {
        matches.truncate(SEARCH_PICKER_ITEM_LIMIT);
    }

    let start_char = search_start_char(buffer, direction, pattern.len());
    let selected_index = pick_search_selection_index(&matches, direction, start_char);

    let entries = matches
        .into_iter()
        .map(|matched| {
            let detail = format!(
                "Ln {}, Col {}",
                matched.point.line + 1,
                matched.point.column + 1
            );
            PickerEntry {
                item: PickerItem::new(
                    format!("{}:{}", matched.point.line, matched.point.column),
                    matched.line_text.trim().to_owned(),
                    detail,
                    None::<String>,
                ),
                action: PickerAction::VimSearchResult {
                    direction,
                    target: matched.point,
                },
                quickfix: None,
            }
        })
        .collect();

    SearchPickerData {
        entries,
        selected_index,
    }
}

fn move_buffer_with_motion(
    buffer: &mut ShellBuffer,
    motion: ShellMotion,
    count: Option<usize>,
) -> bool {
    let repeat = count.unwrap_or(1);
    match motion {
        ShellMotion::Left => (0..repeat).fold(false, |moved, _| buffer.move_left() || moved),
        ShellMotion::Down => (0..repeat).fold(false, |moved, _| buffer.move_down() || moved),
        ShellMotion::Up => (0..repeat).fold(false, |moved, _| buffer.move_up() || moved),
        ShellMotion::Right => (0..repeat).fold(false, |moved, _| buffer.move_right() || moved),
        ShellMotion::WordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_forward() || moved)
        }
        ShellMotion::BigWordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_forward() || moved)
        }
        ShellMotion::WordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_backward() || moved)
        }
        ShellMotion::BigWordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_backward() || moved)
        }
        ShellMotion::WordEnd => (0..repeat).fold(false, |moved, _| buffer.move_word_end() || moved),
        ShellMotion::BigWordEnd => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_end() || moved)
        }
        ShellMotion::SentenceForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_sentence_forward() || moved)
        }
        ShellMotion::SentenceBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_sentence_backward() || moved)
        }
        ShellMotion::ParagraphForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_paragraph_forward() || moved)
        }
        ShellMotion::ParagraphBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_paragraph_backward() || moved)
        }
        ShellMotion::WordEndBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_backward() || moved)
        }
        ShellMotion::BigWordEndBackward => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_backward() || moved
        }),
        ShellMotion::MatchPair => buffer.move_matching_delimiter(),
        ShellMotion::LineStart => buffer.move_line_start(),
        ShellMotion::LineFirstNonBlank => buffer.move_line_first_non_blank(),
        ShellMotion::LineEnd => {
            let line_repeat = repeat.saturating_sub(1);
            let moved_line = if line_repeat == 0 {
                false
            } else {
                (0..line_repeat).fold(false, |moved, _| buffer.move_down() || moved)
            };
            buffer.move_line_end() || moved_line
        }
        ShellMotion::ScreenTop => buffer.move_to_viewport_offset(repeat.saturating_sub(1)),
        ShellMotion::ScreenMiddle => buffer.move_to_viewport_middle(),
        ShellMotion::ScreenBottom => {
            let viewport = buffer.viewport_lines();
            let offset = viewport.saturating_sub(repeat.min(viewport));
            buffer.move_to_viewport_offset(offset)
        }
        ShellMotion::FirstLine => {
            if let Some(line) = count {
                buffer.goto_line(line.saturating_sub(1))
            } else {
                buffer.goto_first_line()
            }
        }
        ShellMotion::LastLine => {
            if let Some(line) = count {
                buffer.goto_line(line.saturating_sub(1))
            } else {
                buffer.goto_last_line()
            }
        }
    }
}

fn move_input_with_motion(
    input: &mut InputField,
    motion: ShellMotion,
    count: Option<usize>,
) -> bool {
    let repeat = count.unwrap_or(1).max(1);
    let original_anchor = input.selection_anchor;
    let original_cursor = input.cursor_char();
    let original_point = input.cursor_point();
    let mut buffer = TextBuffer::from_text(input.text());
    buffer.set_cursor(input.cursor_point());
    let moved = match motion {
        ShellMotion::Left => (0..repeat).fold(false, |moved, _| buffer.move_left() || moved),
        ShellMotion::Down => (0..repeat).fold(false, |moved, _| buffer.move_down() || moved),
        ShellMotion::Up => (0..repeat).fold(false, |moved, _| buffer.move_up() || moved),
        ShellMotion::Right => (0..repeat).fold(false, |moved, _| buffer.move_right() || moved),
        ShellMotion::WordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_forward() || moved)
        }
        ShellMotion::BigWordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_forward() || moved)
        }
        ShellMotion::WordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_backward() || moved)
        }
        ShellMotion::BigWordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_backward() || moved)
        }
        ShellMotion::WordEnd => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_forward() || moved)
        }
        ShellMotion::BigWordEnd => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_forward() || moved
        }),
        ShellMotion::WordEndBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_backward() || moved)
        }
        ShellMotion::BigWordEndBackward => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_backward() || moved
        }),
        ShellMotion::LineStart => {
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, 0));
            buffer.cursor() != original_point
        }
        ShellMotion::LineFirstNonBlank => {
            let line = buffer.line(buffer.cursor().line).unwrap_or_default();
            let column = line
                .chars()
                .take_while(|character| character.is_whitespace())
                .count();
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, column));
            buffer.cursor() != original_point
        }
        ShellMotion::LineEnd => {
            let line = buffer.cursor().line;
            let line_repeat = repeat.saturating_sub(1);
            let moved_line = if line_repeat == 0 {
                false
            } else {
                (0..line_repeat).fold(false, |moved, _| buffer.move_down() || moved)
            };
            let line_len = buffer.line_len_chars(buffer.cursor().line).unwrap_or(0);
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, line_len));
            moved_line || buffer.cursor().line != line || line_len != original_point.column
        }
        ShellMotion::FirstLine => {
            let line = count.unwrap_or(1).saturating_sub(1);
            buffer.set_cursor(TextPoint::new(line, 0));
            buffer.cursor() != original_point
        }
        ShellMotion::LastLine => {
            let line = count
                .map(|value| value.saturating_sub(1))
                .unwrap_or_else(|| buffer.line_count().saturating_sub(1));
            buffer.set_cursor(TextPoint::new(line, 0));
            buffer.cursor() != original_point
        }
        ShellMotion::SentenceForward
        | ShellMotion::SentenceBackward
        | ShellMotion::ParagraphForward
        | ShellMotion::ParagraphBackward
        | ShellMotion::MatchPair
        | ShellMotion::ScreenTop
        | ShellMotion::ScreenMiddle
        | ShellMotion::ScreenBottom => false,
    };
    input.cursor = buffer.point_to_char_index(buffer.cursor());
    if original_anchor.is_none() {
        input.selection_anchor = None;
    } else {
        input.selection_anchor = original_anchor;
    }
    moved || input.cursor_char() != original_cursor
}

fn advance_point_by_text(mut point: TextPoint, text: &str) -> TextPoint {
    for character in text.chars() {
        if character == '\n' {
            point.line = point.line.saturating_add(1);
            point.column = 0;
        } else {
            point.column = point.column.saturating_add(1);
        }
    }
    point
}

fn statusline_mode_label(input_mode: InputMode, multicursor: bool) -> &'static str {
    if multicursor {
        match input_mode {
            InputMode::Normal => "MC NORMAL",
            InputMode::Insert => "MC INSERT",
            InputMode::Replace => "MC REPLACE",
            InputMode::Visual => "MC VISUAL",
        }
    } else {
        input_mode.label()
    }
}

fn move_text_buffer_with_motion(
    buffer: &mut TextBuffer,
    motion: ShellMotion,
    count: Option<usize>,
    language_id: Option<&str>,
) -> bool {
    let repeat = count.unwrap_or(1).max(1);
    match motion {
        ShellMotion::Left => (0..repeat).fold(false, |moved, _| buffer.move_left() || moved),
        ShellMotion::Down => (0..repeat).fold(false, |moved, _| buffer.move_down() || moved),
        ShellMotion::Up => (0..repeat).fold(false, |moved, _| buffer.move_up() || moved),
        ShellMotion::Right => (0..repeat).fold(false, |moved, _| buffer.move_right() || moved),
        ShellMotion::WordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_forward() || moved)
        }
        ShellMotion::BigWordForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_forward() || moved)
        }
        ShellMotion::WordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_backward() || moved)
        }
        ShellMotion::BigWordBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_big_word_backward() || moved)
        }
        ShellMotion::WordEnd => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_forward() || moved)
        }
        ShellMotion::BigWordEnd => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_forward() || moved
        }),
        ShellMotion::SentenceForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_sentence_forward() || moved)
        }
        ShellMotion::SentenceBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_sentence_backward() || moved)
        }
        ShellMotion::ParagraphForward => {
            (0..repeat).fold(false, |moved, _| buffer.move_paragraph_forward() || moved)
        }
        ShellMotion::ParagraphBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_paragraph_backward() || moved)
        }
        ShellMotion::WordEndBackward => {
            (0..repeat).fold(false, |moved, _| buffer.move_word_end_backward() || moved)
        }
        ShellMotion::BigWordEndBackward => (0..repeat).fold(false, |moved, _| {
            buffer.move_big_word_end_backward() || moved
        }),
        ShellMotion::MatchPair => buffer.move_matching_delimiter(language_id),
        ShellMotion::LineStart => {
            buffer.set_cursor(TextPoint::new(buffer.cursor().line, 0));
            true
        }
        ShellMotion::LineFirstNonBlank => {
            let point = buffer
                .first_non_blank_in_line(buffer.cursor().line)
                .unwrap_or(TextPoint::new(buffer.cursor().line, 0));
            let moved = point != buffer.cursor();
            buffer.set_cursor(point);
            moved
        }
        ShellMotion::LineEnd => {
            let line_repeat = repeat.saturating_sub(1);
            let moved_line = if line_repeat == 0 {
                false
            } else {
                (0..line_repeat).fold(false, |moved, _| buffer.move_down() || moved)
            };
            let line = buffer.cursor().line;
            let column = buffer.line_len_chars(line).unwrap_or(0);
            let moved = buffer.cursor().column != column;
            buffer.set_cursor(TextPoint::new(line, column));
            moved || moved_line
        }
        ShellMotion::FirstLine => {
            let line = count.unwrap_or(1).saturating_sub(1);
            let point = buffer
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            let moved = point != buffer.cursor();
            buffer.set_cursor(point);
            moved
        }
        ShellMotion::LastLine => {
            let line = count
                .map(|value| value.saturating_sub(1))
                .unwrap_or_else(|| buffer.line_count().saturating_sub(1));
            let point = buffer
                .first_non_blank_in_line(line)
                .unwrap_or(TextPoint::new(line, 0));
            let moved = point != buffer.cursor();
            buffer.set_cursor(point);
            moved
        }
        ShellMotion::ScreenTop | ShellMotion::ScreenMiddle | ShellMotion::ScreenBottom => false,
    }
}

fn find_multicursor_seed_range(buffer: &ShellBuffer) -> Option<(String, TextRange)> {
    let point = buffer.cursor_point();
    let range = buffer.text.word_range_at(point, false, 1).or_else(|| {
        buffer
            .text
            .word_range_at_kind(point, WordKind::BigWord, false, 1)
    })?;
    let text = buffer.slice(range);
    (!text.is_empty()).then_some((text, range))
}

fn find_next_multicursor_match(
    buffer: &ShellBuffer,
    needle: &str,
    after_char_index: usize,
    existing: &[TextRange],
) -> Option<TextRange> {
    if needle.is_empty() {
        return None;
    }
    let haystack = buffer.text.text().chars().collect::<Vec<_>>();
    let needle_chars = needle.chars().collect::<Vec<_>>();
    if needle_chars.is_empty() || haystack.len() < needle_chars.len() {
        return None;
    }
    let existing = existing
        .iter()
        .map(|range| {
            (
                buffer.text.point_to_char_index(range.start()),
                buffer.text.point_to_char_index(range.end()),
            )
        })
        .collect::<Vec<_>>();
    let search_range = |start: usize, end: usize| {
        (start..end).find_map(|candidate| {
            let candidate_end = candidate.saturating_add(needle_chars.len());
            if candidate_end > haystack.len()
                || haystack[candidate..candidate_end] != needle_chars[..]
                || existing.iter().any(|&(existing_start, existing_end)| {
                    existing_start == candidate && existing_end == candidate_end
                })
            {
                return None;
            }
            let start_point = buffer.text.point_from_char_index(candidate);
            let end_point = buffer.text.point_from_char_index(candidate_end);
            let range = TextRange::new(start_point, end_point);
            let exact = buffer
                .text
                .word_range_at(start_point, false, 1)
                .or_else(|| {
                    buffer
                        .text
                        .word_range_at_kind(start_point, WordKind::BigWord, false, 1)
                });
            (exact == Some(range)).then_some(range)
        })
    };
    let search_end = haystack
        .len()
        .saturating_sub(needle_chars.len())
        .saturating_add(1);
    search_range(after_char_index.min(search_end), search_end)
        .or_else(|| search_range(0, after_char_index.min(search_end)))
}

fn find_previous_multicursor_match(
    buffer: &ShellBuffer,
    needle: &str,
    before_char_index: usize,
    existing: &[TextRange],
) -> Option<TextRange> {
    if needle.is_empty() {
        return None;
    }
    let haystack = buffer.text.text().chars().collect::<Vec<_>>();
    let needle_chars = needle.chars().collect::<Vec<_>>();
    if needle_chars.is_empty() || haystack.len() < needle_chars.len() {
        return None;
    }
    let existing = existing
        .iter()
        .map(|range| {
            (
                buffer.text.point_to_char_index(range.start()),
                buffer.text.point_to_char_index(range.end()),
            )
        })
        .collect::<Vec<_>>();
    let search_end = haystack
        .len()
        .saturating_sub(needle_chars.len())
        .saturating_add(1);
    let search_range_rev = |start: usize, end: usize| {
        (start..end).rev().find_map(|candidate| {
            let candidate_end = candidate.saturating_add(needle_chars.len());
            if candidate_end > haystack.len()
                || haystack[candidate..candidate_end] != needle_chars[..]
                || existing.iter().any(|&(existing_start, existing_end)| {
                    existing_start == candidate && existing_end == candidate_end
                })
            {
                return None;
            }
            let start_point = buffer.text.point_from_char_index(candidate);
            let end_point = buffer.text.point_from_char_index(candidate_end);
            let range = TextRange::new(start_point, end_point);
            let exact = buffer
                .text
                .word_range_at(start_point, false, 1)
                .or_else(|| {
                    buffer
                        .text
                        .word_range_at_kind(start_point, WordKind::BigWord, false, 1)
                });
            (exact == Some(range)).then_some(range)
        })
    };
    let before = before_char_index.min(search_end);
    search_range_rev(0, before).or_else(|| search_range_rev(before, search_end))
}

fn sync_multicursor_primary_cursor(runtime: &mut EditorRuntime) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let Some(primary_range) = state.ranges.get(state.primary).copied() else {
        return Ok(());
    };
    let prefix = state
        .match_text
        .chars()
        .take(state.cursor_offset.min(state.match_text.chars().count()))
        .collect::<String>();
    let point = advance_point_by_text(primary_range.start(), &prefix);
    active_shell_buffer_mut(runtime)?.set_cursor(point);
    Ok(())
}

fn replace_multicursor_ranges(
    runtime: &mut EditorRuntime,
    text: &str,
    cursor_offset: usize,
    visual_anchor_offset: Option<usize>,
) -> Result<(), String> {
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let new_text_len = text.chars().count();
    let original_ranges = {
        let buffer = active_shell_buffer_mut(runtime)?;
        state
            .ranges
            .iter()
            .map(|range| {
                let start = buffer.text.point_to_char_index(range.start());
                let end = buffer.text.point_to_char_index(range.end());
                (start, end)
            })
            .collect::<Vec<_>>()
    };
    let mut adjusted_starts = Vec::with_capacity(original_ranges.len());
    let mut delta = 0isize;
    for (start, end) in &original_ranges {
        adjusted_starts.push(start.saturating_add_signed(delta));
        delta += new_text_len as isize - end.saturating_sub(*start) as isize;
    }
    {
        let buffer = active_shell_buffer_mut(runtime)?;
        for range in state.ranges.iter().rev().copied() {
            buffer.replace_range(range, text);
        }
        buffer.mark_syntax_dirty();
        state.ranges = adjusted_starts
            .into_iter()
            .map(|start| {
                TextRange::new(
                    buffer.text.point_from_char_index(start),
                    buffer
                        .text
                        .point_from_char_index(start.saturating_add(new_text_len)),
                )
            })
            .collect();
    }
    let text_len = text.chars().count();
    state.match_text = text.to_owned();
    state.cursor_offset = cursor_offset.min(text_len);
    state.visual_anchor_offset = visual_anchor_offset.map(|offset| offset.min(text_len));
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state);
    sync_multicursor_primary_cursor(runtime)?;
    Ok(())
}

fn apply_multicursor_motion(
    runtime: &mut EditorRuntime,
    motion: ShellMotion,
) -> Result<bool, String> {
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let language_id = {
        let buffer_id = active_shell_buffer_id(runtime)?;
        shell_buffer(runtime, buffer_id)?
            .language_id()
            .map(str::to_owned)
    };
    let mut buffer = TextBuffer::from_text(&state.match_text);
    buffer.set_cursor(buffer.point_from_char_index(state.cursor_offset));
    let moved = move_text_buffer_with_motion(
        &mut buffer,
        motion,
        shell_ui_mut(runtime)?.vim_mut().take_count(),
        language_id.as_deref(),
    );
    state.cursor_offset = buffer.point_to_char_index(buffer.cursor());
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state);
    sync_multicursor_primary_cursor(runtime)?;
    Ok(moved)
}

fn set_multicursor_cursor_offset(runtime: &mut EditorRuntime, offset: usize) -> Result<(), String> {
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    state.cursor_offset = offset.min(state.match_text.chars().count());
    state.visual_anchor_offset = None;
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state);
    sync_multicursor_primary_cursor(runtime)
}

fn add_next_multicursor_match(runtime: &mut EditorRuntime) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? || active_shell_buffer_is_terminal(runtime)?
    {
        return Ok(());
    }
    let mut state = if let Some(existing) = shell_ui(runtime)?.vim().multicursor.clone() {
        existing
    } else {
        let buffer = active_shell_buffer_mut(runtime)?;
        let Some((match_text, range)) = find_multicursor_seed_range(buffer) else {
            return Ok(());
        };
        MulticursorState {
            match_text,
            ranges: vec![range],
            primary: 0,
            cursor_offset: buffer
                .text
                .point_to_char_index(buffer.cursor_point())
                .saturating_sub(buffer.text.point_to_char_index(range.start())),
            visual_anchor_offset: None,
        }
    };
    let after_char = active_shell_buffer_mut(runtime)?
        .text
        .point_to_char_index(state.ranges[state.primary].end());
    let next = {
        let buffer = active_shell_buffer_mut(runtime)?;
        find_next_multicursor_match(buffer, &state.match_text, after_char, &state.ranges)
    };
    if shell_ui(runtime)?.vim().multicursor.is_none() {
        shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state.clone());
        sync_multicursor_primary_cursor(runtime)?;
        return Ok(());
    }
    let Some(next) = next else {
        return Ok(());
    };
    push_multicursor_range(runtime, &mut state, next)
}

fn add_previous_multicursor_match(runtime: &mut EditorRuntime) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? || active_shell_buffer_is_terminal(runtime)?
    {
        return Ok(());
    }
    let Some(mut state) = shell_ui(runtime)?.vim().multicursor.clone() else {
        return Ok(());
    };
    let before_char = active_shell_buffer_mut(runtime)?
        .text
        .point_to_char_index(state.ranges[state.primary].start());
    let previous = {
        let buffer = active_shell_buffer_mut(runtime)?;
        find_previous_multicursor_match(buffer, &state.match_text, before_char, &state.ranges)
    };
    let Some(previous) = previous else {
        return Ok(());
    };
    push_multicursor_range(runtime, &mut state, previous)
}

fn push_multicursor_range(
    runtime: &mut EditorRuntime,
    state: &mut MulticursorState,
    range: TextRange,
) -> Result<(), String> {
    state.ranges.push(range);
    let buffer = active_shell_buffer_mut(runtime)?;
    state
        .ranges
        .sort_by_key(|candidate| buffer.text.point_to_char_index(candidate.start()));
    state.primary = state
        .ranges
        .iter()
        .position(|candidate| *candidate == range)
        .unwrap_or(state.primary);
    shell_ui_mut(runtime)?.vim_mut().multicursor = Some(state.clone());
    sync_multicursor_primary_cursor(runtime)?;
    Ok(())
}

fn multicursor_selection_offsets(
    state: &MulticursorState,
    input_mode: InputMode,
) -> Option<(usize, usize)> {
    if input_mode == InputMode::Visual {
        state.visual_anchor_offset.map(|anchor| {
            let start = anchor.min(state.cursor_offset);
            let end = anchor.max(state.cursor_offset);
            (start, end)
        })
    } else {
        Some((0, state.match_text.chars().count()))
    }
}

fn apply_multicursor_insert_text(
    runtime: &mut EditorRuntime,
    text: &str,
    replace: bool,
) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let mut buffer = TextBuffer::from_text(&state.match_text);
    buffer.set_cursor(buffer.point_from_char_index(state.cursor_offset));
    if replace {
        let cursor = buffer.cursor();
        let next = buffer.point_after(cursor).unwrap_or(cursor);
        if next != cursor {
            buffer.replace(TextRange::new(cursor, next), text);
        } else {
            buffer.insert_text(text);
        }
    } else {
        buffer.insert_text(text);
    }
    let new_text = buffer.text();
    let new_offset = buffer.point_to_char_index(buffer.cursor());
    replace_multicursor_ranges(runtime, &new_text, new_offset, None)
}

fn apply_multicursor_delete(runtime: &mut EditorRuntime, backward: bool) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let mut buffer = TextBuffer::from_text(&state.match_text);
    buffer.set_cursor(buffer.point_from_char_index(state.cursor_offset));
    let changed = if backward {
        buffer.backspace()
    } else {
        buffer.delete_forward()
    };
    if !changed {
        return Ok(());
    }
    let new_text = buffer.text();
    let new_offset = buffer.point_to_char_index(buffer.cursor());
    replace_multicursor_ranges(runtime, &new_text, new_offset, None)
}

fn toggle_multicursor_visual_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let input_mode = shell_ui(runtime)?.input_mode();
    let mut state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    if input_mode == InputMode::Visual {
        state.visual_anchor_offset = None;
        let ui = shell_ui_mut(runtime)?;
        ui.input_mode = InputMode::Normal;
        ui.vim_mut().multicursor = Some(state);
        ui.vim_mut().clear_transient();
        return Ok(());
    }
    state.visual_anchor_offset = Some(state.cursor_offset);
    let ui = shell_ui_mut(runtime)?;
    ui.input_mode = InputMode::Visual;
    ui.vim_mut().multicursor = Some(state);
    ui.vim_mut().clear_transient();
    Ok(())
}

fn apply_multicursor_visual_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
) -> Result<(), String> {
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let Some((start, end)) = multicursor_selection_offsets(&state, InputMode::Visual) else {
        return Ok(());
    };
    if start == end {
        let ui = shell_ui_mut(runtime)?;
        ui.input_mode = InputMode::Normal;
        ui.vim_mut().multicursor = Some(state);
        return Ok(());
    }
    let selected = state
        .match_text
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect::<String>();
    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        store_yank_register(runtime, YankRegister::Character(selected.clone()), true)?;
    }
    match operator {
        VimOperator::Delete | VimOperator::Change => {
            let prefix = state.match_text.chars().take(start).collect::<String>();
            let suffix = state.match_text.chars().skip(end).collect::<String>();
            let replacement = format!("{prefix}{suffix}");
            replace_multicursor_ranges(runtime, &replacement, start, None)?;
            if operator == VimOperator::Change {
                let ui = shell_ui_mut(runtime)?;
                ui.input_mode = InputMode::Insert;
                ui.vim_mut().clear_transient();
            } else {
                let ui = shell_ui_mut(runtime)?;
                ui.input_mode = InputMode::Normal;
                ui.vim_mut().clear_transient();
            }
        }
        VimOperator::Yank => {
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Normal;
            ui.vim_mut().clear_transient();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let prefix = state.match_text.chars().take(start).collect::<String>();
            let middle = transform_case_text(&selected, operator);
            let suffix = state.match_text.chars().skip(end).collect::<String>();
            let replacement = format!("{prefix}{middle}{suffix}");
            replace_multicursor_ranges(runtime, &replacement, end, None)?;
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Normal;
            ui.vim_mut().clear_transient();
        }
    }
    Ok(())
}

fn apply_multicursor_text_object_operator(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    kind: VimTextObjectKind,
) -> Result<bool, String> {
    if !matches!(kind, VimTextObjectKind::Word | VimTextObjectKind::BigWord) {
        return Ok(false);
    }
    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let selected = state.match_text.clone();
    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        store_yank_register(runtime, YankRegister::Character(selected), true)?;
    }
    match operator {
        VimOperator::Delete => {
            replace_multicursor_ranges(runtime, "", 0, None)?;
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Normal;
            ui.vim_mut().clear_transient();
        }
        VimOperator::Change => {
            replace_multicursor_ranges(runtime, "", 0, None)?;
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Insert;
            ui.vim_mut().clear_transient();
            mark_change_finish_on_normal(runtime)?;
        }
        VimOperator::Yank => {
            let ui = shell_ui_mut(runtime)?;
            ui.vim_mut().clear_transient();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {}
    }
    Ok(true)
}

fn apply_multicursor_operator_motion(
    runtime: &mut EditorRuntime,
    operator: VimOperator,
    operator_count: usize,
    motion: ShellMotion,
    motion_count: Option<usize>,
) -> Result<bool, String> {
    let motion = change_operator_word_motion(operator, motion);
    if matches!(
        motion,
        ShellMotion::Down | ShellMotion::Up | ShellMotion::FirstLine | ShellMotion::LastLine
    ) {
        return Ok(false);
    }

    let state = shell_ui(runtime)?
        .vim()
        .multicursor
        .clone()
        .ok_or_else(|| "multicursor state is missing".to_owned())?;
    let language_id = {
        let buffer_id = active_shell_buffer_id(runtime)?;
        shell_buffer(runtime, buffer_id)?
            .language_id()
            .map(str::to_owned)
    };
    let total = state.match_text.chars().count();
    let original_offset = state.cursor_offset.min(total);
    let mut buffer = TextBuffer::from_text(&state.match_text);
    buffer.set_cursor(buffer.point_from_char_index(original_offset));
    let repeat = operator_count
        .saturating_mul(motion_count.unwrap_or(1))
        .max(1);
    if !move_text_buffer_with_motion(&mut buffer, motion, Some(repeat), language_id.as_deref()) {
        return Ok(false);
    }
    let target_offset = buffer.point_to_char_index(buffer.cursor()).min(total);
    let inclusive = motion_is_inclusive(motion);
    let (start, end) = if target_offset >= original_offset {
        let end = if inclusive {
            target_offset.saturating_add(1).min(total)
        } else {
            target_offset
        };
        (original_offset, end)
    } else {
        let end = if inclusive {
            original_offset.saturating_add(1).min(total)
        } else {
            original_offset
        };
        (target_offset, end)
    };
    if start >= end {
        return Ok(false);
    }

    let selected = state
        .match_text
        .chars()
        .skip(start)
        .take(end.saturating_sub(start))
        .collect::<String>();
    if matches!(
        operator,
        VimOperator::Delete | VimOperator::Change | VimOperator::Yank
    ) {
        store_yank_register(runtime, YankRegister::Character(selected.clone()), true)?;
    }
    match operator {
        VimOperator::Delete | VimOperator::Change => {
            let prefix = state.match_text.chars().take(start).collect::<String>();
            let suffix = state.match_text.chars().skip(end).collect::<String>();
            let replacement = format!("{prefix}{suffix}");
            replace_multicursor_ranges(runtime, &replacement, start, None)?;
            if operator == VimOperator::Change {
                let ui = shell_ui_mut(runtime)?;
                ui.input_mode = InputMode::Insert;
                ui.vim_mut().clear_transient();
                mark_change_finish_on_normal(runtime)?;
            } else {
                let ui = shell_ui_mut(runtime)?;
                ui.input_mode = InputMode::Normal;
                ui.vim_mut().clear_transient();
            }
        }
        VimOperator::Yank => {
            let ui = shell_ui_mut(runtime)?;
            ui.vim_mut().clear_transient();
        }
        VimOperator::ToggleCase | VimOperator::Lowercase | VimOperator::Uppercase => {
            let prefix = state.match_text.chars().take(start).collect::<String>();
            let middle = transform_case_text(&selected, operator);
            let suffix = state.match_text.chars().skip(end).collect::<String>();
            let replacement = format!("{prefix}{middle}{suffix}");
            replace_multicursor_ranges(runtime, &replacement, end, None)?;
            let ui = shell_ui_mut(runtime)?;
            ui.input_mode = InputMode::Normal;
            ui.vim_mut().clear_transient();
        }
    }
    Ok(true)
}

fn input_field_paste_shortcut_requested(keycode: Keycode, keymod: Mod) -> bool {
    keycode == Keycode::V
        && keymod.intersects(ctrl_mod())
        && keymod.intersects(shift_mod())
        && !keymod.intersects(alt_mod() | gui_mod())
}

fn paste_into_active_input_buffer(runtime: &mut EditorRuntime) -> Result<bool, String> {
    match read_system_clipboard_paste() {
        ClipboardPaste::Empty => Ok(false),
        ClipboardPaste::Text(text) => paste_text_into_active_input_buffer(runtime, &text),
        ClipboardPaste::Image(image) => paste_image_into_active_input_buffer(runtime, image),
    }
}

fn paste_image_into_active_input_buffer(
    runtime: &mut EditorRuntime,
    image: ClipboardImage,
) -> Result<bool, String> {
    acp::paste_image_into_active_input(runtime, image)
}

fn paste_text_into_active_input_buffer(
    runtime: &mut EditorRuntime,
    text: &str,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer_is_acp(&buffer.kind)
    };
    close_acp_inline_picker_for(runtime, buffer_id, is_acp)?;
    let handled = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        if let Some(input) = buffer.input_field_mut() {
            input.insert_text(text);
            true
        } else {
            false
        }
    };
    if handled && is_acp {
        shell_ui_mut(runtime)?.close_picker();
        acp::maybe_open_acp_input_completion(runtime, buffer_id)?;
        acp::refresh_acp_input_hint(runtime, buffer_id)?;
    }
    Ok(handled)
}

fn close_acp_inline_picker_for(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    is_acp: bool,
) -> Result<(), String> {
    if is_acp
        && matches!(
            shell_ui(runtime)?.picker_kind(),
            Some(kind) if kind.acp_inline_buffer_id() == Some(buffer_id)
        )
    {
        shell_ui_mut(runtime)?.close_picker();
    }
    Ok(())
}

fn change_operator_word_motion(operator: VimOperator, motion: ShellMotion) -> ShellMotion {
    if operator != VimOperator::Change {
        return motion;
    }
    match motion {
        ShellMotion::WordForward => ShellMotion::WordEnd,
        ShellMotion::BigWordForward => ShellMotion::BigWordEnd,
        other => other,
    }
}

fn motion_is_inclusive(motion: ShellMotion) -> bool {
    matches!(
        motion,
        ShellMotion::WordEnd
            | ShellMotion::BigWordEnd
            | ShellMotion::WordEndBackward
            | ShellMotion::BigWordEndBackward
            | ShellMotion::MatchPair
            | ShellMotion::LineEnd
    )
}

fn trim_word_forward_operator_range(
    buffer: &ShellBuffer,
    motion: ShellMotion,
    original_cursor: TextPoint,
    target: TextPoint,
    range: TextRange,
    repeat: usize,
) -> TextRange {
    if repeat != 1
        || !matches!(
            motion,
            ShellMotion::WordForward | ShellMotion::BigWordForward
        )
        || target.line == original_cursor.line
    {
        return range;
    }

    let line_end = TextPoint::new(
        original_cursor.line,
        buffer.line_len_chars(original_cursor.line),
    );
    if line_end <= original_cursor {
        return range;
    }

    TextRange::new(original_cursor, line_end)
}

fn trim_word_forward_input_operator_range(
    input: &InputField,
    motion: ShellMotion,
    original_cursor: TextPoint,
    target: TextPoint,
    range: (usize, usize),
    repeat: usize,
) -> (usize, usize) {
    if repeat != 1
        || !matches!(
            motion,
            ShellMotion::WordForward | ShellMotion::BigWordForward
        )
        || target.line == original_cursor.line
    {
        return range;
    }

    let (_, line_end) = match input.line_range_chars(original_cursor.line) {
        Some(range) => range,
        None => return range,
    };
    let start = input.text_buffer().point_to_char_index(original_cursor);
    if line_end <= start {
        return range;
    }
    (start, line_end)
}

fn charwise_motion_range(
    buffer: &ShellBuffer,
    start: TextPoint,
    target: TextPoint,
    inclusive: bool,
) -> Option<TextRange> {
    let range = if target >= start {
        let end = if inclusive {
            buffer.point_after(target).unwrap_or(target)
        } else {
            target
        };
        TextRange::new(start, end)
    } else {
        let end = if inclusive {
            buffer.point_after(start).unwrap_or(start)
        } else {
            start
        };
        TextRange::new(target, end)
    };
    (range.start() != range.end()).then_some(range.normalized())
}

fn input_charwise_motion_range(
    input: &InputField,
    start: usize,
    target: usize,
    inclusive: bool,
) -> Option<(usize, usize)> {
    let total = input.char_count();
    let range = if target >= start {
        let end = if inclusive {
            target.saturating_add(1).min(total)
        } else {
            target
        };
        (start, end)
    } else {
        let end = if inclusive {
            start.saturating_add(1).min(total)
        } else {
            start
        };
        (target, end)
    };
    (range.0 < range.1).then_some(range)
}
