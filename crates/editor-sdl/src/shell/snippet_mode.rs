#[derive(Debug, Clone, PartialEq, Eq)]
struct SnippetModeState {
    buffer_id: BufferId,
    /// Absolute buffer ranges for Insert Points in order `$0`, `$1`, ...
    stops: Vec<TextRange>,
    index: usize,
}

/// Expands a Snippet template: strips `$N` markers and records clean-text byte ranges.
///
/// Returns `(clean_text, stops)` where each stop is `(tabstop_index, byte_range_in_clean_text)`.
/// First occurrence of each index wins. Stops are sorted ascending by index.
fn expand_snippet_template(template: &str) -> (String, Vec<(usize, std::ops::Range<usize>)>) {
    let mut clean = String::with_capacity(template.len());
    let mut stops = BTreeMap::new();
    let mut chars = template.char_indices().peekable();
    while let Some((byte_index, ch)) = chars.next() {
        if ch == '$' {
            let digit_start = byte_index + ch.len_utf8();
            let mut digit_end = digit_start;
            while let Some(&(next_index, next_ch)) = chars.peek() {
                if next_ch.is_ascii_digit() {
                    digit_end = next_index + next_ch.len_utf8();
                    chars.next();
                } else {
                    break;
                }
            }
            if digit_end > digit_start
                && let Ok(index) = template[digit_start..digit_end].parse::<usize>()
            {
                let pos = clean.len();
                stops.entry(index).or_insert(pos..pos);
                continue;
            }
        }
        clean.push(ch);
    }
    let stops = stops.into_iter().collect();
    (clean, stops)
}

/// Indents every line after the first with `indent_prefix` (keeps `$N` markers intact).
fn indent_snippet_body(body: &str, indent_prefix: &str) -> String {
    if indent_prefix.is_empty() {
        return body.to_owned();
    }
    let mut lines = body.split('\n');
    let Some(first) = lines.next() else {
        return String::new();
    };
    let mut out = String::with_capacity(body.len() + indent_prefix.len() * 4);
    out.push_str(first);
    for line in lines {
        out.push('\n');
        out.push_str(indent_prefix);
        out.push_str(line);
    }
    out
}

fn line_prefix_before_point(text: &TextSnapshot, point: TextPoint) -> String {
    text.line(point.line)
        .map(|line| line.chars().take(point.column).collect())
        .unwrap_or_default()
}

fn point_to_byte_offset(text: &TextBuffer, point: TextPoint) -> usize {
    let line_start = text.line_start_byte(point.line).unwrap_or(0);
    let line = text.line(point.line).unwrap_or_default();
    let prefix: String = line.chars().take(point.column).collect();
    line_start + prefix.len()
}

fn byte_offset_to_point(text: &TextBuffer, byte_offset: usize) -> TextPoint {
    let full = text.text();
    let mut byte_offset = byte_offset.min(full.len());
    while byte_offset > 0 && !full.is_char_boundary(byte_offset) {
        byte_offset -= 1;
    }
    let mut line = 0usize;
    let mut column = 0usize;
    let mut index = 0usize;
    for ch in full.chars() {
        if index >= byte_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 0;
        } else {
            column += 1;
        }
        index += ch.len_utf8();
    }
    TextPoint::new(line, column)
}

fn absolute_ranges_from_snippet_stops(
    text: &TextBuffer,
    base_byte: usize,
    stops: &[(usize, std::ops::Range<usize>)],
) -> Vec<TextRange> {
    stops
        .iter()
        .map(|(_, range)| {
            let start = byte_offset_to_point(text, base_byte.saturating_add(range.start));
            let end = byte_offset_to_point(text, base_byte.saturating_add(range.end));
            TextRange::new(start, end)
        })
        .collect()
}

#[cfg(test)]
mod snippet_mode_tests {
    use super::*;

    #[test]
    fn expand_snippet_template_strips_markers_and_orders_stops() {
        let (clean, stops) = expand_snippet_template("fn $1($2) {\n    $0\n}");
        assert_eq!(clean, "fn () {\n    \n}");
        assert_eq!(stops.len(), 3);
        assert_eq!(stops[0].0, 0);
        assert_eq!(stops[1].0, 1);
        assert_eq!(stops[2].0, 2);
        assert_eq!(&clean[stops[1].1.clone()], "");
        assert_eq!(stops[1].1.start, 3);
        assert_eq!(stops[2].1.start, 4);
        assert_eq!(stops[0].1.start, clean.find('\n').map(|i| i + 5).unwrap_or(0));
    }

    #[test]
    fn expand_snippet_template_keeps_literal_dollar_without_digits() {
        let (clean, stops) = expand_snippet_template("price is $not a stop $1");
        assert_eq!(clean, "price is $not a stop ");
        assert_eq!(stops, vec![(1, clean.len()..clean.len())]);
    }

    #[test]
    fn indent_snippet_body_prefixes_continuation_lines() {
        let indented = indent_snippet_body("one\ntwo\nthree", "    ");
        assert_eq!(indented, "one\n    two\n    three");
    }
}
