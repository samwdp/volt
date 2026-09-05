fn reverse_find_kind(kind: VimFindKind) -> VimFindKind {
    match kind {
        VimFindKind::ForwardTo => VimFindKind::BackwardTo,
        VimFindKind::BackwardTo => VimFindKind::ForwardTo,
        VimFindKind::ForwardBefore => VimFindKind::BackwardAfter,
        VimFindKind::BackwardAfter => VimFindKind::ForwardBefore,
    }
}

fn reverse_search_direction(direction: VimSearchDirection) -> VimSearchDirection {
    match direction {
        VimSearchDirection::Forward => VimSearchDirection::Backward,
        VimSearchDirection::Backward => VimSearchDirection::Forward,
    }
}

fn vim_delimited_text_object(chord: &str) -> Option<(char, char)> {
    match chord {
        "(" | ")" | "b" => Some(('(', ')')),
        "[" | "]" => Some(('[', ']')),
        "{" | "}" | "B" => Some(('{', '}')),
        ">" => Some(('<', '>')),
        "\"" => Some(('"', '"')),
        "'" => Some(('\'', '\'')),
        "`" => Some(('`', '`')),
        _ => None,
    }
}

fn vim_text_object_kind(chord: &str) -> Option<VimTextObjectKind> {
    match chord {
        "w" => Some(VimTextObjectKind::Word),
        "W" => Some(VimTextObjectKind::BigWord),
        "s" => Some(VimTextObjectKind::Sentence),
        "p" => Some(VimTextObjectKind::Paragraph),
        "t" => Some(VimTextObjectKind::Tag),
        _ => vim_delimited_text_object(chord)
            .map(|(open, close)| VimTextObjectKind::Delimited { open, close }),
    }
}

fn search_is_case_sensitive(_query: &str) -> bool {
    false
}

fn normalize_search_char(ch: char, case_sensitive: bool) -> char {
    if case_sensitive {
        ch
    } else {
        ch.to_ascii_lowercase()
    }
}

fn normalize_search_pattern(query: &str, case_sensitive: bool) -> Vec<char> {
    query
        .chars()
        .map(|ch| normalize_search_char(ch, case_sensitive))
        .collect()
}

fn matches_pattern_at(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
) -> bool {
    pattern.iter().enumerate().all(|(offset, expected)| {
        buffer
            .text
            .char_at_point(buffer.text.point_from_char_index(start_char + offset))
            .map(|ch| normalize_search_char(ch, case_sensitive))
            == Some(*expected)
    })
}

fn search_forward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    if pattern.is_empty() || pattern.len() > buffer.text.char_count() {
        return None;
    }

    let max_start = buffer.text.char_count().saturating_sub(pattern.len());
    let first_pass_start = start_char.min(max_start.saturating_add(1));
    for char_index in first_pass_start..=max_start {
        if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
            return Some(buffer.text.point_from_char_index(char_index));
        }
    }

    if wrap {
        for char_index in 0..first_pass_start.min(max_start.saturating_add(1)) {
            if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
                return Some(buffer.text.point_from_char_index(char_index));
            }
        }
    }

    None
}

fn search_backward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    if pattern.is_empty() || pattern.len() > buffer.text.char_count() {
        return None;
    }

    let max_start = buffer.text.char_count().saturating_sub(pattern.len());
    let first_pass_start = start_char.min(max_start);
    for char_index in (0..=first_pass_start).rev() {
        if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
            return Some(buffer.text.point_from_char_index(char_index));
        }
    }

    if wrap && first_pass_start < max_start {
        for char_index in ((first_pass_start + 1)..=max_start).rev() {
            if matches_pattern_at(buffer, char_index, pattern, case_sensitive) {
                return Some(buffer.text.point_from_char_index(char_index));
            }
        }
    }

    None
}

fn char_at_index(buffer: &ShellBuffer, char_index: usize) -> Option<char> {
    buffer
        .text
        .char_at_point(buffer.text.point_from_char_index(char_index))
}

fn find_char_forward(
    buffer: &ShellBuffer,
    start_char: usize,
    target: char,
    case_sensitive: bool,
) -> Option<usize> {
    let char_count = buffer.text.char_count();
    (start_char..char_count).find(|&char_index| {
        char_at_index(buffer, char_index).map(|ch| normalize_search_char(ch, case_sensitive))
            == Some(target)
    })
}

fn fuzzy_match_end(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
) -> Option<usize> {
    if pattern.is_empty()
        || char_at_index(buffer, start_char).map(|ch| normalize_search_char(ch, case_sensitive))
            != Some(pattern[0])
    {
        return None;
    }

    let mut last_index = start_char;
    let mut next_index = start_char.saturating_add(1);
    for target in pattern.iter().skip(1) {
        let found = find_char_forward(buffer, next_index, *target, case_sensitive)?;
        last_index = found;
        next_index = found.saturating_add(1);
    }

    Some(last_index)
}

fn search_fuzzy_forward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    let char_count = buffer.text.char_count();
    if pattern.is_empty() || pattern.len() > char_count {
        return None;
    }

    let max_start = char_count.saturating_sub(1);
    let first_pass_start = start_char.min(max_start.saturating_add(1));
    let mut best: Option<(usize, usize)> = None;

    if first_pass_start <= max_start {
        for char_index in first_pass_start..=max_start {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive)
            else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.is_none_or(|(_, best_span)| span < best_span) {
                best = Some((char_index, span));
            }
        }
    }
    if best.is_some() {
        return best.map(|(start, _)| buffer.text.point_from_char_index(start));
    }

    if wrap {
        for char_index in 0..first_pass_start.min(max_start.saturating_add(1)) {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive)
            else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.is_none_or(|(_, best_span)| span < best_span) {
                best = Some((char_index, span));
            }
        }
    }

    best.map(|(start, _)| buffer.text.point_from_char_index(start))
}

fn search_fuzzy_backward(
    buffer: &ShellBuffer,
    start_char: usize,
    pattern: &[char],
    case_sensitive: bool,
    wrap: bool,
) -> Option<TextPoint> {
    let char_count = buffer.text.char_count();
    if pattern.is_empty() || pattern.len() > char_count {
        return None;
    }

    let max_start = char_count.saturating_sub(1);
    let first_pass_start = start_char.min(max_start);
    let mut best: Option<(usize, usize)> = None;

    for char_index in (0..=first_pass_start).rev() {
        let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive) else {
            continue;
        };
        let span = end_index.saturating_sub(char_index);
        if best.is_none_or(|(_, best_span)| span < best_span) {
            best = Some((char_index, span));
        }
    }
    if best.is_some() {
        return best.map(|(start, _)| buffer.text.point_from_char_index(start));
    }

    if wrap && first_pass_start < max_start {
        for char_index in ((first_pass_start + 1)..=max_start).rev() {
            let Some(end_index) = fuzzy_match_end(buffer, char_index, pattern, case_sensitive)
            else {
                continue;
            };
            let span = end_index.saturating_sub(char_index);
            if best.is_none_or(|(_, best_span)| span < best_span) {
                best = Some((char_index, span));
            }
        }
    }

    best.map(|(start, _)| buffer.text.point_from_char_index(start))
}

fn search_buffer(
    buffer: &ShellBuffer,
    direction: VimSearchDirection,
    query: &str,
) -> Option<TextPoint> {
    let case_sensitive = search_is_case_sensitive(query);
    let pattern = normalize_search_pattern(query, case_sensitive);
    if pattern.is_empty() {
        return None;
    }

    let cursor = buffer.cursor_point();
    let exact_match = match direction {
        VimSearchDirection::Forward => {
            let start_char = buffer
                .point_after(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or(buffer.text.char_count());
            search_forward(buffer, start_char, &pattern, case_sensitive, true)
        }
        VimSearchDirection::Backward => {
            let start_char = buffer
                .text
                .point_before(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or_else(|| buffer.text.char_count().saturating_sub(pattern.len()));
            search_backward(buffer, start_char, &pattern, case_sensitive, true)
        }
    };

    if exact_match.is_some() {
        return exact_match;
    }

    match direction {
        VimSearchDirection::Forward => {
            let start_char = buffer
                .point_after(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or(buffer.text.char_count());
            search_fuzzy_forward(buffer, start_char, &pattern, case_sensitive, true)
        }
        VimSearchDirection::Backward => {
            let start_char = buffer
                .text
                .point_before(cursor)
                .map(|point| buffer.text.point_to_char_index(point))
                .unwrap_or_else(|| buffer.text.char_count().saturating_sub(pattern.len()));
            search_fuzzy_backward(buffer, start_char, &pattern, case_sensitive, true)
        }
    }
}
