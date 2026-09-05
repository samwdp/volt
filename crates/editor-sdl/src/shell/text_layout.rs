#[derive(Debug, Clone, PartialEq, Eq)]
struct LineSyntaxSpan {
    start: usize,
    end: usize,
    capture_name: Arc<str>,
    theme_token: Arc<str>,
}

type IndexedSyntaxLines = BTreeMap<usize, Vec<LineSyntaxSpan>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SyntaxLineWindow {
    start_line: usize,
    line_count: usize,
}

impl SyntaxLineWindow {
    fn new(start_line: usize, line_count: usize) -> Option<Self> {
        (line_count > 0).then_some(Self {
            start_line,
            line_count,
        })
    }

    fn end_line_exclusive(self) -> usize {
        self.start_line.saturating_add(self.line_count)
    }

    fn contains(self, other: Self) -> bool {
        self.start_line <= other.start_line
            && self.end_line_exclusive() >= other.end_line_exclusive()
    }

    fn to_highlight_window(self) -> HighlightWindow {
        HighlightWindow::new(self.start_line, self.line_count)
    }
}

#[derive(Debug, Clone, Copy)]
struct LineWrapSegment {
    start_col: usize,
    end_col: usize,
}

#[derive(Debug, Clone)]
struct LineCharMap {
    bytes: Vec<usize>,
    whitespace: Vec<bool>,
    display_columns: Vec<usize>,
}

impl LineCharMap {
    fn new(line: &str) -> Self {
        Self::with_tab_width(line, 4)
    }

    fn with_tab_width(line: &str, tab_width: usize) -> Self {
        let tab_width = resolved_tab_width(tab_width);
        let mut bytes = Vec::new();
        let mut whitespace = Vec::new();
        let mut display_columns = Vec::new();
        let mut display_col = 0usize;
        for (byte_index, character) in line.char_indices() {
            bytes.push(byte_index);
            whitespace.push(character.is_whitespace());
            display_columns.push(display_col);
            display_col = display_col.saturating_add(display_columns_for_character(
                character,
                display_col,
                tab_width,
            ));
        }
        bytes.push(line.len());
        display_columns.push(display_col);
        Self {
            bytes,
            whitespace,
            display_columns,
        }
    }

    fn len(&self) -> usize {
        self.whitespace.len()
    }

    fn slice<'a>(&self, line: &'a str, start_col: usize, end_col: usize) -> &'a str {
        if start_col >= end_col {
            return "";
        }
        let len = self.len();
        let start = start_col.min(len);
        let end = end_col.min(len);
        let start_byte = self.bytes.get(start).copied().unwrap_or(line.len());
        let end_byte = self.bytes.get(end).copied().unwrap_or(line.len());
        &line[start_byte..end_byte]
    }

    fn display_col_at(&self, column: usize) -> usize {
        let index = column.min(self.len());
        self.display_columns
            .get(index)
            .copied()
            .unwrap_or_else(|| self.display_columns.last().copied().unwrap_or(0))
    }

    fn display_cols_between(&self, start_col: usize, end_col: usize) -> usize {
        self.display_col_at(end_col)
            .saturating_sub(self.display_col_at(start_col))
    }

    fn char_col_for_display_col(&self, display_col: usize) -> usize {
        if self.len() == 0 {
            return 0;
        }
        for (index, window) in self.display_columns.windows(2).enumerate() {
            if display_col < window[1] {
                return index;
            }
        }
        self.len().saturating_sub(1)
    }

    fn is_zero_width_col(&self, column: usize) -> bool {
        column < self.len() && self.display_cols_between(column, column.saturating_add(1)) == 0
    }

    fn cursor_anchor_col(&self, column: usize) -> usize {
        if column >= self.len() {
            return self.len();
        }
        let mut column = column;
        while column > 0 && self.is_zero_width_col(column) {
            column = column.saturating_sub(1);
        }
        column
    }

    fn display_text_for_range(&self, line: &str, start_col: usize, end_col: usize) -> String {
        if start_col >= end_col {
            return String::new();
        }
        let len = self.len();
        let start = start_col.min(len);
        let end = end_col.min(len);
        let mut rendered = String::new();
        for column in start..end {
            let start_byte = self.bytes.get(column).copied().unwrap_or(line.len());
            let end_byte = self
                .bytes
                .get(column.saturating_add(1))
                .copied()
                .unwrap_or(line.len());
            let text = &line[start_byte..end_byte];
            let Some(character) = text.chars().next() else {
                continue;
            };
            if character == '\t' {
                rendered.push_str(&" ".repeat(self.display_cols_between(column, column + 1)));
            } else if is_zero_width_display_character(character) {
                continue;
            } else if let Some(caret) = ascii_control_caret_notation(character) {
                rendered.push('^');
                rendered.push(caret);
            } else {
                rendered.push_str(text);
            }
        }
        rendered
    }
}

fn resolved_tab_width(tab_width: usize) -> usize {
    if tab_width == 0 { 4 } else { tab_width }
}

fn is_zero_width_display_character(character: char) -> bool {
    matches!(
        character as u32,
        0x200C..=0x200D | 0xFEFF | 0xFE00..=0xFE0F | 0xE0100..=0xE01EF
    )
}

fn is_wide_display_character(character: char) -> bool {
    matches!(
        character as u32,
        0x1F000..=0x1FAFF | 0x2600..=0x27BF
    )
}

fn ascii_control_caret_notation(character: char) -> Option<char> {
    match character {
        '\0'..='\u{1f}' if character != '\t' && character != '\n' => {
            char::from_u32(character as u32 + 0x40)
        }
        '\u{7f}' => Some('?'),
        _ => None,
    }
}

fn display_columns_for_character(character: char, display_col: usize, tab_width: usize) -> usize {
    if is_zero_width_display_character(character) {
        0
    } else if character == '\t' {
        let remainder = display_col % tab_width;
        if remainder == 0 {
            tab_width
        } else {
            tab_width - remainder
        }
    } else if ascii_control_caret_notation(character).is_some()
        || is_wide_display_character(character)
    {
        2
    } else {
        1
    }
}

fn theme_lang_indent(theme_registry: Option<&ThemeRegistry>, language_id: Option<&str>) -> usize {
    let Some(registry) = theme_registry else {
        return 0;
    };
    let Some(language_id) = language_id else {
        return 0;
    };
    let key = format!("langs.{language_id}.indent");
    registry
        .resolve_number(&key)
        .map(|value| value.max(0.0).round() as usize)
        .unwrap_or(0)
}

fn theme_lang_format_on_save(
    theme_registry: Option<&ThemeRegistry>,
    language_id: Option<&str>,
) -> bool {
    let Some(registry) = theme_registry else {
        return false;
    };
    let Some(language_id) = language_id else {
        return false;
    };
    let key = format!("langs.{language_id}.format_on_save");
    registry.resolve_bool(&key).unwrap_or(false)
}

fn theme_lang_use_tabs(theme_registry: Option<&ThemeRegistry>, language_id: Option<&str>) -> bool {
    let Some(registry) = theme_registry else {
        return false;
    };
    let Some(language_id) = language_id else {
        return false;
    };
    let key = format!("langs.{language_id}.use_tabs");
    registry.resolve_bool(&key).unwrap_or(false)
}

fn theme_scrolloff(theme_registry: Option<&ThemeRegistry>) -> usize {
    theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_SCROLL_OFF))
        .map(|value| value.max(0.0).round() as usize)
        .unwrap_or(0)
}

fn cached_context_overlay_snapshot(
    snapshot: Option<&Arc<BufferContextOverlaySnapshot>>,
    key: &BufferContextOverlayCacheKey,
    typing_active: bool,
) -> Option<Arc<BufferContextOverlaySnapshot>> {
    snapshot
        .filter(|snapshot| {
            snapshot.key == *key
                || (typing_active
                    && snapshot.key.buffer_name == key.buffer_name
                    && snapshot.key.language_id == key.language_id)
        })
        .cloned()
}

fn buffer_context_overlay_snapshot(
    buffer: &ShellBuffer,
    active: bool,
    typing_active: bool,
    user_library: &dyn UserLibrary,
) -> Option<Arc<BufferContextOverlaySnapshot>> {
    if buffer_is_db_query(&buffer.kind) {
        return None;
    }
    active.then(|| buffer.context_overlay_snapshot(user_library, typing_active))
}

fn lsp_formatting_options(
    runtime: &EditorRuntime,
    language_id: Option<&str>,
) -> LspFormattingOptions {
    let theme_registry = runtime.services().get::<ThemeRegistry>();
    let indent_size = theme_lang_indent(theme_registry, language_id);
    let tab_size = if indent_size == 0 { 4 } else { indent_size } as u32;
    let insert_spaces = !theme_lang_use_tabs(theme_registry, language_id);
    LspFormattingOptions::new(tab_size, insert_spaces)
}

fn theme_color(theme_registry: Option<&ThemeRegistry>, token: &str, fallback: Color) -> Color {
    theme_registry
        .and_then(|registry| registry.resolve(token))
        .map(to_sdl_color)
        .unwrap_or(fallback)
}

fn is_dark_color(color: Color) -> bool {
    let luminance =
        0.2126 * f32::from(color.r) + 0.7152 * f32::from(color.g) + 0.0722 * f32::from(color.b);
    luminance < 128.0
}

fn adjust_color(color: Color, delta: i16) -> Color {
    let adjust = |channel: u8| -> u8 { (i16::from(channel) + delta).clamp(0, 255) as u8 };
    Color::RGBA(adjust(color.r), adjust(color.g), adjust(color.b), color.a)
}

fn blend_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    let blend = |a: u8, b: u8| -> u8 {
        (f32::from(a) * inv + f32::from(b) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Color::RGBA(
        blend(a.r, b.r),
        blend(a.g, b.g),
        blend(a.b, b.b),
        blend(a.a, b.a),
    )
}

fn normalize_tabs<'a>(text: &'a str, indent_size: usize, use_tabs: bool) -> Cow<'a, str> {
    if use_tabs || !text.contains('\t') {
        return Cow::Borrowed(text);
    }
    let indent_size = indent_size.max(1);
    Cow::Owned(text.replace('\t', &" ".repeat(indent_size)))
}

fn leading_whitespace_info(line: &str, tab_width: usize) -> (usize, usize) {
    let tab_width = tab_width.max(1);
    let mut columns = 0usize;
    let mut end = 0usize;
    for (index, character) in line.char_indices() {
        if !character.is_whitespace() {
            end = index;
            return (columns, end);
        }
        if character == '\t' {
            columns = columns.saturating_add(tab_width);
        } else {
            columns = columns.saturating_add(1);
        }
        end = index + character.len_utf8();
    }
    (columns, end)
}

fn leading_indent_string(line: &str, indent_size: usize) -> String {
    let (_, end) = leading_whitespace_info(line, indent_size);
    line[..end].to_owned()
}

fn indent_string_from_columns(columns: usize, indent_size: usize, use_tabs: bool) -> String {
    if columns == 0 {
        return String::new();
    }
    if !use_tabs || indent_size == 0 {
        return " ".repeat(columns);
    }
    let tabs = columns / indent_size;
    let spaces = columns % indent_size;
    format!("{}{}", "\t".repeat(tabs), " ".repeat(spaces))
}

fn tab_insert_string(theme_registry: Option<&ThemeRegistry>, language_id: Option<&str>) -> String {
    let indent_size = theme_lang_indent(theme_registry, language_id);
    let indent_columns = if indent_size == 0 { 4 } else { indent_size };
    indent_string_from_columns(
        indent_columns,
        indent_columns,
        theme_lang_use_tabs(theme_registry, language_id),
    )
}

fn desired_indent_columns_for_text(
    buffer: &TextBuffer,
    line_index: usize,
    indent_size: usize,
) -> usize {
    let mut base_line = buffer.line(line_index).unwrap_or_default();
    let mut search_index = line_index;
    while search_index > 0 && base_line.trim().is_empty() {
        search_index = search_index.saturating_sub(1);
        let Some(line) = buffer.line(search_index) else {
            continue;
        };
        if !line.trim().is_empty() {
            base_line = line;
            break;
        }
    }
    let (mut indent_columns, _) = leading_whitespace_info(&base_line, indent_size);
    let base_trimmed = base_line.trim_end();
    if indent_size > 0
        && (matches!(base_trimmed.chars().last(), Some('{') | Some('['))
            || opening_tag_name_before_cursor(base_trimmed).is_some())
    {
        indent_columns = indent_columns.saturating_add(indent_size);
    }
    let current_line = buffer.line(line_index).unwrap_or_default();
    let current_trimmed = current_line.trim_start();
    if matches!(current_trimmed.chars().next(), Some('}') | Some(']'))
        || current_trimmed.starts_with("</")
    {
        indent_columns = indent_columns.saturating_sub(indent_size);
    }
    indent_columns
}

fn desired_reindent_columns_for_line(
    buffer: &TextBuffer,
    line_index: usize,
    indent_size: usize,
) -> usize {
    let current_line = buffer.line(line_index).unwrap_or_default();
    let mut base_line = String::new();
    let mut search_index = line_index;
    while search_index > 0 {
        search_index = search_index.saturating_sub(1);
        let Some(line) = buffer.line(search_index) else {
            continue;
        };
        if !line.trim().is_empty() {
            base_line = line;
            break;
        }
    }
    let (mut indent_columns, _) = leading_whitespace_info(&base_line, indent_size);
    let base_trimmed = base_line.trim_end();
    if indent_size > 0
        && (matches!(base_trimmed.chars().last(), Some('{') | Some('['))
            || opening_tag_name_before_cursor(base_trimmed).is_some())
    {
        indent_columns = indent_columns.saturating_add(indent_size);
    }
    let current_trimmed = current_line.trim_start();
    if matches!(current_trimmed.chars().next(), Some('}') | Some(']'))
        || current_trimmed.starts_with("</")
    {
        indent_columns = indent_columns.saturating_sub(indent_size);
    }
    indent_columns
}

fn should_format_current_line_after_closing_delimiter(buffer: &ShellBuffer, text: &str) -> bool {
    matches!(text, "}" | "]")
        && matches!(
            buffer
                .text
                .line(buffer.cursor_row())
                .unwrap_or_default()
                .trim_start()
                .chars()
                .next(),
            Some('}') | Some(']')
        )
}

fn should_split_insert_newline_for_pair(buffer: &ShellBuffer) -> bool {
    let cursor = buffer.cursor_point();
    let Some(previous_point) = buffer.text.point_before(cursor) else {
        return false;
    };
    let Some(previous) = buffer.text.char_at_point(previous_point) else {
        return false;
    };
    let Some(next) = buffer.text.char_at_point(cursor) else {
        return false;
    };
    matches!((previous, next), ('{', '}') | ('[', ']'))
        || should_split_insert_newline_for_tag_pair(buffer)
}

fn should_split_insert_newline_for_tag_pair(buffer: &ShellBuffer) -> bool {
    let cursor = buffer.cursor_point();
    let Some(line) = buffer.text.line(cursor.line) else {
        return false;
    };
    let split_index = byte_index_for_char_column(&line, cursor.column);
    let before = &line[..split_index];
    let after = &line[split_index..];
    let Some(open_name) = opening_tag_name_before_cursor(before) else {
        return false;
    };
    let Some(close_name) = closing_tag_name_after_cursor(after) else {
        return false;
    };
    open_name == close_name
}

fn byte_index_for_char_column(line: &str, column: usize) -> usize {
    line.char_indices()
        .nth(column)
        .map(|(index, _)| index)
        .unwrap_or(line.len())
}

fn opening_tag_name_before_cursor(before: &str) -> Option<&str> {
    let trimmed = before.trim_end();
    let open_end = trimmed.strip_suffix('>')?;
    let open_start = open_end.rfind('<')?;
    let tag = open_end[open_start + 1..].trim_start();
    if tag.starts_with('/') || tag.starts_with('!') || tag.starts_with('?') || tag.ends_with('/') {
        return None;
    }
    tag_name(tag)
}

fn closing_tag_name_after_cursor(after: &str) -> Option<&str> {
    let tag = after.trim_start().strip_prefix("</")?;
    tag_name(tag)
}

fn tag_name(tag: &str) -> Option<&str> {
    let end = tag
        .char_indices()
        .find_map(|(index, character)| (!is_tag_name_character(character)).then_some(index))
        .unwrap_or(tag.len());
    (end > 0).then_some(&tag[..end])
}

fn is_tag_name_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | ':' | '.')
}

fn insert_newline_inside_pair(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    indent_size: usize,
    use_tabs: bool,
) -> Result<bool, String> {
    let should_split = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        should_split_insert_newline_for_pair(buffer)
    };
    if !should_split {
        return Ok(false);
    }

    let cursor = shell_buffer(runtime, buffer_id)?.cursor_point();
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.insert_text("\n\n");
        buffer.set_cursor(TextPoint::new(cursor.line + 1, 0));
    }
    format_buffer_line_indent(runtime, buffer_id, cursor.line + 1, indent_size, use_tabs)?;
    format_buffer_line_indent(runtime, buffer_id, cursor.line + 2, indent_size, use_tabs)?;
    Ok(true)
}

fn desired_indent_for_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    line_index: usize,
    indent_size: usize,
    use_tabs: bool,
) -> Result<String, String> {
    let syntax_indent =
        syntax_indent_for_buffer(runtime, buffer_id, line_index, indent_size, use_tabs)?;
    let text = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer.text.clone()
    };
    if let Some(indent) = syntax_indent {
        return Ok(adjust_tag_child_indent(
            &text,
            line_index,
            indent_size,
            use_tabs,
            &indent,
        ));
    }
    Ok(indent_string_from_columns(
        desired_indent_columns_for_text(&text, line_index, indent_size),
        indent_size,
        use_tabs,
    ))
}

fn adjust_tag_child_indent(
    buffer: &TextBuffer,
    line_index: usize,
    indent_size: usize,
    use_tabs: bool,
    syntax_indent: &str,
) -> String {
    let Some(previous_line) = previous_nonblank_line(buffer, line_index) else {
        return syntax_indent.to_owned();
    };
    if opening_tag_name_before_cursor(previous_line.trim_end()).is_none() {
        return syntax_indent.to_owned();
    }
    let (base_columns, _) = leading_whitespace_info(&previous_line, indent_size);
    let max_columns = base_columns.saturating_add(indent_size);
    let (syntax_columns, _) = leading_whitespace_info(syntax_indent, indent_size);
    if syntax_columns <= max_columns {
        return syntax_indent.to_owned();
    }
    indent_string_from_columns(max_columns, indent_size, use_tabs)
}

fn previous_nonblank_line(buffer: &TextBuffer, line_index: usize) -> Option<String> {
    let mut index = line_index;
    while index > 0 {
        index = index.saturating_sub(1);
        let line = buffer.line(index)?;
        if !line.trim().is_empty() {
            return Some(line);
        }
    }
    None
}

fn syntax_indent_for_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    line_index: usize,
    indent_size: usize,
    use_tabs: bool,
) -> Result<Option<String>, String> {
    let (text, language_id) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if buffer_is_db_query(&buffer.kind) {
            return Ok(None);
        }
        (buffer.text.clone(), buffer.language_id().map(str::to_owned))
    };
    let Some(language_id) = language_id else {
        return Ok(None);
    };
    if language_id == "tsx" {
        return Ok(None);
    }
    let mut parse_session = {
        let ui = shell_ui_mut(runtime)?;
        ui.take_indent_parse_session(buffer_id)
    };
    let has_parse_session = parse_session.is_some();
    if !has_parse_session && text.line_count() >= LARGE_BUFFER_SYNC_INDENT_LINE_THRESHOLD {
        return Ok(None);
    }
    let columns = syntax_registry_mut(runtime)
        .ok()
        .and_then(|registry| {
            registry
                .desired_indent_for_language_with_session(
                    &language_id,
                    &text,
                    line_index,
                    indent_size,
                    &mut parse_session,
                )
                .ok()
        })
        .flatten();
    shell_ui_mut(runtime)?.store_indent_parse_session(buffer_id, parse_session);
    let Some(columns) = columns else {
        return Ok(None);
    };
    Ok(Some(indent_string_from_columns(
        columns,
        indent_size,
        use_tabs,
    )))
}

fn format_buffer_line_indent(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    line_index: usize,
    indent_size: usize,
    use_tabs: bool,
) -> Result<(), String> {
    let indent = desired_indent_for_buffer(runtime, buffer_id, line_index, indent_size, use_tabs)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    apply_line_indent(buffer, line_index, indent_size, &indent);
    Ok(())
}

fn apply_line_indent(
    buffer: &mut ShellBuffer,
    line_index: usize,
    indent_size: usize,
    indent: &str,
) {
    let line = buffer.text.line(line_index).unwrap_or_default();
    let (_, end) = leading_whitespace_info(&line, indent_size);
    let current_indent = &line[..end];
    if current_indent == indent {
        return;
    }
    let end_col = current_indent.chars().count();
    buffer.replace_range(
        TextRange::new(
            TextPoint::new(line_index, 0),
            TextPoint::new(line_index, end_col),
        ),
        indent,
    );
    let cursor = buffer.cursor_point();
    if cursor.line == line_index {
        let new_indent_cols = indent.chars().count();
        let delta = new_indent_cols as isize - end_col as isize;
        let new_col = if cursor.column <= end_col {
            new_indent_cols
        } else {
            let adjusted = cursor.column as isize + delta;
            if adjusted < new_indent_cols as isize {
                new_indent_cols
            } else {
                adjusted as usize
            }
        };
        buffer.set_cursor(TextPoint::new(line_index, new_col));
    }
}

fn format_current_line_indent(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    indent_size: usize,
    use_tabs: bool,
) -> Result<(), String> {
    let line_index = shell_buffer(runtime, buffer_id)?.cursor_row();
    format_buffer_line_indent(runtime, buffer_id, line_index, indent_size, use_tabs)
}
