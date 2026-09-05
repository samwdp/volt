/// Overlay drop-shadow offset in pixels.
pub const OVERLAY_SHADOW_OFFSET: i32 = 8;
/// Left accent bar width used by overlay cards and dock rows.
pub const OVERLAY_ACCENT_BAR_WIDTH: u32 = 5;

/// Truncates `text` to fit `max_width` using a monospace cell width, appending `...`.
pub fn truncate_text_to_width(text: &str, max_width: u32, cell_width: i32) -> String {
    if text.is_empty() || max_width == 0 {
        return String::new();
    }

    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    if text.chars().count() <= max_cells {
        return text.to_owned();
    }

    let ellipsis = "...";
    let ellipsis_cells = ellipsis.chars().count();
    if max_cells <= ellipsis_cells {
        return "...".to_owned();
    }

    let mut truncated = String::new();
    let available_cells = max_cells.saturating_sub(ellipsis_cells);
    for character in text.chars() {
        if truncated.chars().count() >= available_cells {
            break;
        }
        truncated.push(character);
    }

    truncated.push_str(ellipsis);
    truncated
}

/// Truncates `text` to fit `max_width`, keeping the end of the string.
pub fn truncate_text_to_width_preserving_end(
    text: &str,
    max_width: u32,
    cell_width: i32,
) -> String {
    if text.is_empty() || max_width == 0 {
        return String::new();
    }

    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    if text.chars().count() <= max_cells {
        return text.to_owned();
    }

    let ellipsis = "...";
    let ellipsis_cells = ellipsis.chars().count();
    if max_cells <= ellipsis_cells {
        return "...".to_owned();
    }

    let available_cells = max_cells.saturating_sub(ellipsis_cells);
    let suffix = text
        .chars()
        .rev()
        .take(available_cells)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<String>();
    format!("{ellipsis}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_start_appends_ellipsis() {
        assert_eq!(truncate_text_to_width("abcdef", 5, 1), "ab...");
    }

    #[test]
    fn truncate_end_keeps_tail() {
        assert_eq!(
            truncate_text_to_width_preserving_end("abcdef", 5, 1),
            "...ef"
        );
    }

    #[test]
    fn truncate_is_identity_when_text_fits() {
        assert_eq!(truncate_text_to_width("abc", 10, 1), "abc");
        assert_eq!(truncate_text_to_width_preserving_end("abc", 10, 1), "abc");
    }
}
