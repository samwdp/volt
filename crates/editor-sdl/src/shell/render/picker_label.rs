const PICKER_PATH_TAIL_KEEP: usize = 3;

fn text_fits_width(text: &str, max_width: u32, cell_width: i32) -> bool {
    if text.is_empty() || max_width == 0 {
        return true;
    }
    let cell_width = cell_width.max(1) as u32;
    let max_cells = (max_width / cell_width) as usize;
    text.chars().count() <= max_cells
}

fn path_separator_for(path: &str) -> char {
    if path.contains('\\') && !path.contains('/') {
        '\\'
    } else {
        '/'
    }
}

fn is_path_like(text: &str) -> bool {
    text.contains('/') || text.contains('\\')
}

fn shrink_directory_name(name: &str) -> String {
    name.chars()
        .next()
        .map(|character| character.to_string())
        .unwrap_or_default()
}

fn path_segments(path_str: &str) -> Option<Vec<String>> {
    if !is_path_like(path_str) {
        return None;
    }
    let path = Path::new(path_str);
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => {
                segments.push(prefix.as_os_str().to_string_lossy().into_owned());
            }
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => segments.push("..".to_owned()),
            Component::Normal(name) => segments.push(name.to_string_lossy().into_owned()),
        }
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments)
    }
}

fn join_path_segments(segments: &[String], sep: char) -> String {
    if segments.is_empty() {
        return String::new();
    }
    let mut result = segments[0].clone();
    for segment in segments.iter().skip(1) {
        if !result.is_empty() {
            result.push(sep);
        }
        result.push_str(segment);
    }
    result
}

fn shrink_path_directories(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    if segments.len() == 1 {
        return Some(segments[0].clone());
    }
    let sep = path_separator_for(path_str);
    let file_name = segments.last()?.clone();
    let parents = segments
        .iter()
        .take(segments.len().saturating_sub(1))
        .map(|segment| {
            if segment.ends_with(':') || segment == ".." {
                segment.clone()
            } else {
                shrink_directory_name(segment)
            }
        })
        .collect::<Vec<_>>();
    let mut joined = join_path_segments(&parents, sep);
    if joined.is_empty() {
        return Some(file_name);
    }
    if !joined.ends_with(':') {
        joined.push(sep);
    }
    joined.push_str(&file_name);
    Some(joined)
}

fn split_file_stem_ext(file_name: &str) -> (String, String) {
    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| file_name.to_owned());
    let extension = path
        .extension()
        .map(|extension| format!(".{}", extension.to_string_lossy()))
        .unwrap_or_default();
    (stem, extension)
}

fn shrink_path_all(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    if segments.len() == 1 {
        let (stem, extension) = split_file_stem_ext(&segments[0]);
        return Some(format!("{}{extension}", shrink_directory_name(&stem)));
    }
    let sep = path_separator_for(path_str);
    let file_name = segments.last()?.clone();
    let (stem, extension) = split_file_stem_ext(&file_name);
    let shrunk_file = format!("{}{extension}", shrink_directory_name(&stem));
    let mut parents = segments
        .iter()
        .take(segments.len().saturating_sub(1))
        .map(|segment| {
            if segment.ends_with(':') || segment == ".." {
                segment.clone()
            } else {
                shrink_directory_name(segment)
            }
        })
        .collect::<Vec<_>>();
    parents.push(shrunk_file);
    Some(join_path_segments(&parents, sep))
}

fn file_name_with_parent(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    let sep = path_separator_for(path_str);
    if segments.len() < 2 {
        return segments.last().cloned();
    }
    Some(format!(
        "{}{}{}",
        segments[segments.len() - 2],
        sep,
        segments[segments.len() - 1]
    ))
}

fn parent_initial_file_name(path_str: &str) -> Option<String> {
    let segments = path_segments(path_str)?;
    let sep = path_separator_for(path_str);
    if segments.len() < 2 {
        return segments.last().cloned();
    }
    Some(format!(
        "{}{}{}",
        shrink_directory_name(&segments[segments.len() - 2]),
        sep,
        segments[segments.len() - 1]
    ))
}

fn shrink_leading_keep_tail(path_str: &str, tail_keep: usize) -> Option<String> {
    let segments = path_segments(path_str)?;
    if segments.len() <= tail_keep {
        return shrink_path_directories(path_str);
    }
    let sep = path_separator_for(path_str);
    let split_at = segments.len() - tail_keep;
    let (leading, tail) = segments.split_at(split_at);
    let mut parts = leading
        .iter()
        .map(|segment| {
            if segment.ends_with(':') || segment == ".." {
                segment.clone()
            } else {
                shrink_directory_name(segment)
            }
        })
        .collect::<Vec<_>>();
    parts.extend_from_slice(tail);
    Some(join_path_segments(&parts, sep))
}

pub(super) fn truncate_text_to_width_middle(text: &str, max_width: u32, cell_width: i32) -> String {
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
        return ellipsis.to_owned();
    }

    let available_cells = max_cells.saturating_sub(ellipsis_cells);
    let head_cells = available_cells / 2;
    let tail_cells = available_cells - head_cells;
    let chars = text.chars().collect::<Vec<_>>();
    let head = chars.iter().take(head_cells).copied().collect::<String>();
    let tail = chars
        .iter()
        .skip(chars.len().saturating_sub(tail_cells))
        .copied()
        .collect::<String>();
    format!("{head}{ellipsis}{tail}")
}

fn fit_picker_label_after_transform(
    text: &str,
    max_width: u32,
    cell_width: i32,
    overflow: PickerTruncateStrategy,
) -> String {
    if text_fits_width(text, max_width, cell_width) {
        return text.to_owned();
    }
    match overflow {
        PickerTruncateStrategy::EndEllipsis => truncate_text_to_width(text, max_width, cell_width),
        PickerTruncateStrategy::MiddleEllipsis => {
            truncate_text_to_width_middle(text, max_width, cell_width)
        }
        _ => truncate_text_to_width_preserving_end(text, max_width, cell_width),
    }
}

pub(super) fn truncate_picker_label(
    text: &str,
    max_width: u32,
    cell_width: i32,
    strategy: PickerTruncateStrategy,
) -> String {
    match strategy {
        PickerTruncateStrategy::EndEllipsis => truncate_text_to_width(text, max_width, cell_width),
        PickerTruncateStrategy::StartEllipsis => {
            truncate_text_to_width_preserving_end(text, max_width, cell_width)
        }
        PickerTruncateStrategy::MiddleEllipsis => {
            truncate_text_to_width_middle(text, max_width, cell_width)
        }
        PickerTruncateStrategy::Full => {
            if text_fits_width(text, max_width, cell_width) {
                text.to_owned()
            } else {
                truncate_text_to_width_preserving_end(text, max_width, cell_width)
            }
        }
        PickerTruncateStrategy::Auto => {
            if text_fits_width(text, max_width, cell_width) {
                return text.to_owned();
            }
            if let Some(shrunk) = shrink_path_directories(text)
                && text_fits_width(&shrunk, max_width, cell_width)
            {
                return shrunk;
            }
            truncate_text_to_width_preserving_end(text, max_width, cell_width)
        }
        PickerTruncateStrategy::ShrinkDirectories => {
            let display = shrink_path_directories(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::ShrinkAll => {
            let display = shrink_path_all(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::FileName => {
            let display = Path::new(text)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::FileNameWithParent => {
            let display = file_name_with_parent(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::ParentInitialFileName => {
            let display = parent_initial_file_name(text).unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
        PickerTruncateStrategy::ShrinkLeadingKeepTail => {
            let display = shrink_leading_keep_tail(text, PICKER_PATH_TAIL_KEEP)
                .unwrap_or_else(|| text.to_owned());
            fit_picker_label_after_transform(
                &display,
                max_width,
                cell_width,
                PickerTruncateStrategy::StartEllipsis,
            )
        }
    }
}

pub(super) struct PixelRectToRect;

impl PixelRectToRect {
    pub(super) fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect::new(x, y, width, height)
    }

    pub(super) fn from_pixel_rect(rect: PixelRect) -> Rect {
        Self::rect(rect.x, rect.y, rect.width, rect.height)
    }
}
