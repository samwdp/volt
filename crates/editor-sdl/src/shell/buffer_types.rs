#[derive(Debug, Clone)]
pub(crate) struct ShellBuffer {
    id: BufferId,
    pub(crate) name: String,
    pub(crate) kind: BufferKind,
    read_only: bool,
    input: Option<InputField>,
    section_state: Option<SectionedBufferState>,
    plugin_section_state: Option<PluginSectionBufferState>,
    image_state: Option<ImageBufferState>,
    pdf_state: Option<PdfBufferState>,
    acp_state: Option<AcpBufferState>,
    git_snapshot: Option<GitStatusSnapshot>,
    git_status_root: Option<PathBuf>,
    git_status_probe_revision: Option<u64>,
    git_view: Option<GitViewState>,
    git_fringe: Option<GitFringeState>,
    git_fringe_dirty: bool,
    git_fringe_last_edit_at: Option<Instant>,
    dap_fringe_live: bool,
    dap_fringe_markers: BTreeMap<usize, BreakpointState>,
    dap_execution_line: Option<usize>,
    browser_state: Option<BrowserBufferState>,
    directory_state: Option<DirectoryViewState>,
    terminal_render: Option<TerminalRenderSnapshot>,
    pub(crate) text: TextBuffer,
    lsp_path: Option<PathBuf>,
    backing_file_fingerprint: Option<BackingFileFingerprint>,
    backing_file_reload_pending: bool,
    backing_file_check_in_flight: bool,
    undo_tree: UndoTree,
    language_id: Option<String>,
    /// When true, `language_id` is Forced Language and must not be cleared by syntax refresh.
    forced_language: bool,
    /// Per-buffer Markdown Pretty override (`None` = use user config default).
    markdown_pretty_enabled: Option<bool>,
    /// Per-buffer rainbow delimiter override (`None` = use user config default).
    rainbow_parens_enabled: Option<bool>,
    /// Per-buffer show-paren override (`None` = use user config default).
    show_paren_enabled: Option<bool>,
    pub(crate) scroll_row: usize,
    scroll_col: usize,
    line_wrap: bool,
    viewport_lines: usize,
    content_viewport_lines: usize,
    scroll_wrap_cols: usize,
    scroll_indent_size: usize,
    wrap_cache: Option<WrapRowCache>,
    pretty_display_rows: BTreeMap<usize, usize>,
    markdown_pretty_plan_cache: Arc<Mutex<editor_markdown::MarkdownPrettyPlanCache>>,
    context_overlay_cache: Arc<Mutex<Option<Arc<BufferContextOverlaySnapshot>>>>,
    syntax_error: Option<String>,
    syntax_lines: BTreeMap<usize, Vec<LineSyntaxSpan>>,
    syntax_dirty: bool,
    syntax_requested_revision: Option<u64>,
    syntax_requested_window: Option<SyntaxLineWindow>,
    syntax_requested_at: Option<Instant>,
    syntax_applied_window: Option<SyntaxLineWindow>,
    lsp_enabled: bool,
    lsp_diagnostics: Vec<LspDiagnostic>,
    lsp_diagnostic_lines: BTreeMap<usize, Box<[DiagnosticLineSpan]>>,
    lsp_diagnostics_revision: u64,
    inline_completion: Option<InlineCompletionState>,
    last_edit_at: Option<Instant>,
    vim_buffer_state: VimBufferState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcpPane {
    Plan,
    Output,
    Input,
    Footer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BackingFileFingerprint {
    modified_at: Option<SystemTime>,
    len: u64,
}

impl BackingFileFingerprint {
    fn read(path: &Path) -> io::Result<Self> {
        let metadata = fs::metadata(path)?;
        Ok(Self {
            modified_at: metadata.modified().ok(),
            len: metadata.len(),
        })
    }
}

#[derive(Debug, Clone)]
struct PluginSectionBufferState {
    base_title: String,
    base_writable: bool,
    base_min_rows: Option<usize>,
    base_update: PluginBufferSectionUpdate,
    base_browser_kind: Option<DbBrowserKind>,
    active_section: usize,
    evaluate_target_section: usize,
    layout: Option<PluginBufferLayout>,
    attached_sections: Vec<PluginTextPaneState>,
}

#[derive(Debug, Clone)]
struct PluginTextPaneState {
    title: String,
    writable: bool,
    min_rows: Option<usize>,
    update: PluginBufferSectionUpdate,
    browser_kind: Option<DbBrowserKind>,
    text: TextBuffer,
    syntax_lines: IndexedSyntaxLines,
    scroll_row: usize,
    viewport_rows: usize,
    wrap_cols: usize,
}

#[derive(Debug, Clone)]
struct AcpBufferState {
    session_title: Option<String>,
    active_pane: AcpPane,
    plan_entries: Vec<PlanEntry>,
    output_items: Vec<AcpOutputItem>,
    tool_item_indices: BTreeMap<String, usize>,
    plan_pane: AcpPaneState,
    output_pane: AcpPaneState,
    input: InputField,
    footer_pane: PluginTextPaneState,
    pasted_images: Vec<AcpPastedImage>,
    next_image_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AcpPastedImage {
    id: u64,
    name: String,
    mime_type: String,
    data: String,
}

#[derive(Debug, Clone)]
struct AcpPaneState {
    text: TextBuffer,
    render_lines: Vec<AcpRenderedLine>,
    scroll_visual_row: usize,
    viewport_rows: usize,
    wrap_cols: usize,
}

#[derive(Debug, Clone)]
enum AcpOutputItem {
    UserPrompt(String),
    AgentBlocks(Vec<ContentBlock>),
    ToolCall(ToolCall),
    SystemMessage(String),
}

#[derive(Debug, Clone)]
enum AcpRenderedLine {
    Text(AcpRenderedTextLine),
    Image(AcpRenderedImageLine),
    Spacer,
}

#[derive(Debug, Clone)]
struct AcpRenderedTextLine {
    prefix: Vec<AcpRenderedSegment>,
    text: String,
    text_role: AcpColorRole,
    /// Markdown Pipeline spans for agent text (empty = solid `text_role` color).
    syntax_spans: Vec<LineSyntaxSpan>,
    row_fill: Option<AcpColorRole>,
    gutter: bool,
    align: AcpChatAlign,
    bubble: bool,
    bubble_group: u32,
}

#[derive(Debug, Clone)]
struct AcpRenderedSegment {
    text: String,
    role: AcpColorRole,
    animate: bool,
}

#[derive(Debug, Clone)]
struct AcpRenderedImageLine {
    label: String,
    image: Option<AcpDecodedImage>,
    rows: usize,
}

#[derive(Debug, Clone)]
struct AcpDecodedImage {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

type DecodedImage = AcpDecodedImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageBufferFormat {
    Raster,
    Svg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ImageBufferMode {
    Rendered,
    Source,
}

#[derive(Debug, Clone)]
struct ImageBufferState {
    format: ImageBufferFormat,
    mode: ImageBufferMode,
    decoded: DecodedImage,
    zoom: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcpChatAlign {
    Full,
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AcpColorRole {
    Default,
    Muted,
    Accent,
    Success,
    Warning,
    Error,
    PriorityHigh,
    PriorityMedium,
    PriorityLow,
}

const ACP_IMAGE_ROWS: usize = 12;
const ACP_DIFF_MAX_LINES: usize = 48;
const ACP_TOOL_NEST_PAD: &str = "  ";
const ACP_CHAT_BUBBLE_NUM: usize = 3;
const ACP_CHAT_BUBBLE_DEN: usize = 4;

impl Default for PluginTextPaneState {
    fn default() -> Self {
        Self {
            title: String::new(),
            writable: false,
            min_rows: None,
            update: PluginBufferSectionUpdate::Replace,
            browser_kind: None,
            text: TextBuffer::new(),
            syntax_lines: IndexedSyntaxLines::new(),
            scroll_row: 0,
            viewport_rows: 1,
            wrap_cols: 1,
        }
    }
}

impl PluginSectionBufferState {
    fn new(config: PluginBufferSections, evaluate_target_section: Option<&str>) -> Option<Self> {
        let mut sections = config.items().iter();
        let base = sections.next()?;
        let attached_sections = sections
            .map(|section| {
                let mut pane = PluginTextPaneState {
                    title: section.name().to_owned(),
                    writable: section.writable(),
                    min_rows: section.min_lines(),
                    update: section.update(),
                    browser_kind: section.browser_kind(),
                    ..PluginTextPaneState::default()
                };
                pane.replace_lines(
                    section
                        .initial_lines()
                        .iter()
                        .map(|line| line.to_string())
                        .collect(),
                    true,
                );
                pane
            })
            .collect::<Vec<_>>();
        let evaluate_target_section = evaluate_target_section
            .and_then(|name| {
                config
                    .items()
                    .iter()
                    .position(|section| section.name() == name)
            })
            .or_else(|| {
                config
                    .items()
                    .iter()
                    .enumerate()
                    .rev()
                    .find_map(|(index, section)| (!section.writable()).then_some(index))
            })
            .unwrap_or_else(|| config.items().len().saturating_sub(1));
        Some(Self {
            base_title: base.name().to_owned(),
            base_writable: base.writable(),
            base_min_rows: base.min_lines(),
            base_update: base.update(),
            base_browser_kind: base.browser_kind(),
            active_section: 0,
            evaluate_target_section,
            layout: config.layout().cloned(),
            attached_sections,
        })
    }

    fn section_count(&self) -> usize {
        self.attached_sections.len().saturating_add(1)
    }

    fn active_section_writable(&self) -> bool {
        if self.active_section == 0 {
            self.base_writable
        } else {
            self.attached_sections
                .get(self.active_section.saturating_sub(1))
                .map(|pane| pane.writable)
                .unwrap_or(false)
        }
    }

    fn active_attached_section(&self) -> Option<&PluginTextPaneState> {
        self.active_section
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get(index))
    }

    fn has_active_attached_section(&self) -> bool {
        self.active_section > 0
    }

    fn active_attached_section_mut(&mut self) -> Option<&mut PluginTextPaneState> {
        self.active_section
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get_mut(index))
    }

    fn attached_section(&self, section_index: usize) -> Option<&PluginTextPaneState> {
        section_index
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get(index))
    }

    fn attached_section_mut(&mut self, section_index: usize) -> Option<&mut PluginTextPaneState> {
        section_index
            .checked_sub(1)
            .and_then(|index| self.attached_sections.get_mut(index))
    }

    fn section_title(&self, index: usize) -> &str {
        if index == 0 {
            self.base_title.as_str()
        } else {
            self.attached_sections
                .get(index.saturating_sub(1))
                .map(|pane| pane.title.as_str())
                .unwrap_or("")
        }
    }

    fn section_min_rows(&self, index: usize) -> Option<usize> {
        if index == 0 {
            self.base_min_rows
        } else {
            self.attached_sections
                .get(index.saturating_sub(1))
                .and_then(|pane| pane.min_rows)
        }
    }

    fn section_index_by_name(&self, name: &str) -> Option<usize> {
        if self.base_title == name {
            return Some(0);
        }
        self.attached_sections
            .iter()
            .position(|pane| pane.title == name)
            .map(|index| index.saturating_add(1))
    }

    fn active_section_name(&self) -> &str {
        self.section_title(self.active_section)
    }

    fn focus_section_named(&mut self, name: &str) -> bool {
        let Some(index) = self.section_index_by_name(name) else {
            return false;
        };
        self.active_section = index;
        true
    }

    fn browser_kind_for_section(&self, index: usize) -> Option<DbBrowserKind> {
        if index == 0 {
            self.base_browser_kind
        } else {
            self.attached_sections
                .get(index.saturating_sub(1))
                .and_then(|pane| pane.browser_kind)
        }
    }
}

impl Default for AcpPaneState {
    fn default() -> Self {
        Self {
            text: TextBuffer::new(),
            render_lines: Vec::new(),
            scroll_visual_row: 0,
            viewport_rows: 1,
            wrap_cols: 1,
        }
    }
}

impl AcpBufferState {
    fn new(client_label: String) -> Self {
        let _ = client_label;
        let mut input = InputField::new("> ");
        input.set_placeholder(Some(ACP_INPUT_PLACEHOLDER.to_owned()));
        Self {
            session_title: None,
            active_pane: AcpPane::Output,
            plan_entries: Vec::new(),
            output_items: Vec::new(),
            tool_item_indices: BTreeMap::new(),
            plan_pane: AcpPaneState::default(),
            output_pane: AcpPaneState::default(),
            input,
            footer_pane: PluginTextPaneState {
                min_rows: Some(1),
                ..PluginTextPaneState::default()
            },
            pasted_images: Vec::new(),
            next_image_id: 1,
        }
    }
}

fn acp_pane_total_visual_rows(pane: &AcpPaneState) -> usize {
    pane.render_lines
        .iter()
        .map(|line| acp_rendered_line_row_count(line, pane.wrap_cols()))
        .sum()
}

fn acp_pane_max_scroll_visual_row(pane: &AcpPaneState) -> usize {
    acp_pane_total_visual_rows(pane).saturating_sub(pane.visible_rows())
}

fn acp_pane_cursor_visual_row(pane: &AcpPaneState) -> usize {
    if pane.render_lines.is_empty() {
        return 0;
    }
    let cursor = pane.cursor();
    let mut visual_row = 0usize;
    for (line_index, rendered_line) in pane.render_lines.iter().enumerate() {
        if line_index == cursor.line {
            let extra = match rendered_line {
                AcpRenderedLine::Text(line) => acp_segment_index_for_column(
                    &acp_rendered_text_segments(line, pane.wrap_cols()),
                    cursor.column,
                ),
                _ => 0,
            };
            return visual_row.saturating_add(extra);
        }
        visual_row =
            visual_row.saturating_add(acp_rendered_line_row_count(rendered_line, pane.wrap_cols()));
    }
    visual_row
}

fn acp_pane_line_index_for_visual_row(pane: &AcpPaneState, target_visual_row: usize) -> usize {
    if pane.render_lines.is_empty() {
        return 0;
    }
    let mut visual_row = 0usize;
    for (line_index, rendered_line) in pane.render_lines.iter().enumerate() {
        let row_count = acp_rendered_line_row_count(rendered_line, pane.wrap_cols());
        if visual_row.saturating_add(row_count) > target_visual_row {
            return line_index;
        }
        visual_row = visual_row.saturating_add(row_count);
    }
    pane.render_lines.len().saturating_sub(1)
}

fn acp_pane_point_for_visual_row(pane: &AcpPaneState, target_visual_row: usize) -> TextPoint {
    if pane.render_lines.is_empty() {
        return TextPoint::default();
    }
    let target_visual_row =
        target_visual_row.min(acp_pane_total_visual_rows(pane).saturating_sub(1));
    let mut visual_row = 0usize;
    for (line_index, rendered_line) in pane.render_lines.iter().enumerate() {
        match rendered_line {
            AcpRenderedLine::Text(line) => {
                let segments = acp_rendered_text_segments(line, pane.wrap_cols());
                for segment in segments.iter() {
                    if visual_row == target_visual_row {
                        return TextPoint::new(line_index, segment.start_col);
                    }
                    visual_row = visual_row.saturating_add(1);
                }
            }
            AcpRenderedLine::Image(_) | AcpRenderedLine::Spacer => {
                if visual_row == target_visual_row {
                    return TextPoint::new(line_index, 0);
                }
                visual_row = visual_row.saturating_add(1);
            }
        }
    }
    let line_index = pane.render_lines.len().saturating_sub(1);
    TextPoint::new(line_index, pane.line_len_chars(line_index))
}

impl AcpPaneState {
    fn line_count(&self) -> usize {
        self.text.line_count()
    }

    fn line_len_chars(&self, line_index: usize) -> usize {
        self.text.line_len_chars(line_index).unwrap_or(0)
    }

    fn cursor(&self) -> TextPoint {
        self.text.cursor()
    }

    fn set_cursor(&mut self, point: TextPoint) {
        self.text.set_cursor(point);
    }

    fn visible_rows(&self) -> usize {
        self.viewport_rows.max(1)
    }

    fn wrap_cols(&self) -> usize {
        self.wrap_cols.max(1)
    }

    fn viewport_scroll_top(&self) -> usize {
        self.scroll_visual_row
    }

    fn set_view_metrics(&mut self, visible_rows: usize, wrap_cols: usize) {
        self.viewport_rows = visible_rows.max(1);
        self.wrap_cols = wrap_cols.max(1);
        self.scroll_visual_row = self
            .scroll_visual_row
            .min(acp_pane_max_scroll_visual_row(self));
    }

    fn max_scroll_row(&self) -> usize {
        acp_pane_max_scroll_visual_row(self)
    }

    fn should_follow_output(&self, visible_rows: usize) -> bool {
        if self.render_lines.is_empty() {
            return true;
        }
        let max_scroll = acp_pane_total_visual_rows(self).saturating_sub(visible_rows.max(1));
        self.scroll_visual_row >= max_scroll
    }

    fn replace_render_lines(
        &mut self,
        render_lines: Vec<AcpRenderedLine>,
        follow_output: bool,
        visible_rows: usize,
    ) {
        let cursor_offset = self.text.point_to_char_index(self.cursor());
        let scroll_visual_row = self.scroll_visual_row;
        let lines = render_lines
            .iter()
            .map(AcpRenderedLine::plain_text)
            .collect::<Vec<_>>();
        let text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.viewport_rows = visible_rows.max(1);
        self.text = text;
        self.text.mark_clean();
        self.render_lines = render_lines;
        let line_count = self.line_count();
        if line_count == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_visual_row = 0;
            return;
        }
        let char_count = self.text.char_count();
        self.text.set_cursor(
            self.text
                .point_from_char_index(cursor_offset.min(char_count)),
        );
        if follow_output {
            self.scroll_visual_row = acp_pane_max_scroll_visual_row(self);
        } else {
            self.scroll_visual_row = scroll_visual_row.min(acp_pane_max_scroll_visual_row(self));
        }
    }

    fn move_visual_row(&mut self, delta: i32) -> bool {
        if self.render_lines.is_empty() {
            return false;
        }
        let before = self.cursor();
        let current = acp_pane_cursor_visual_row(self);
        let max_row = acp_pane_total_visual_rows(self).saturating_sub(1) as i32;
        let target = (current as i32 + delta).clamp(0, max_row) as usize;
        let point = acp_pane_point_for_visual_row(self, target);
        self.set_cursor(point);
        self.ensure_cursor_visible();
        self.cursor() != before
    }

    fn line_at_viewport_offset(&self, offset: usize) -> usize {
        acp_pane_line_index_for_visual_row(self, self.scroll_visual_row.saturating_add(offset))
    }

    fn cursor_viewport_offset(&self) -> usize {
        acp_pane_cursor_visual_row(self).saturating_sub(self.scroll_visual_row)
    }

    fn ensure_cursor_visible(&mut self) {
        if self.render_lines.is_empty() {
            self.scroll_visual_row = 0;
            return;
        }
        let cursor_visual = acp_pane_cursor_visual_row(self);
        let visible_rows = self.visible_rows();
        if cursor_visual < self.scroll_visual_row {
            self.scroll_visual_row = cursor_visual;
            return;
        }
        if cursor_visual < self.scroll_visual_row.saturating_add(visible_rows) {
            return;
        }
        self.scroll_visual_row = cursor_visual
            .saturating_add(1)
            .saturating_sub(visible_rows)
            .min(acp_pane_max_scroll_visual_row(self));
    }
}

impl PluginTextPaneState {
    fn line_count(&self) -> usize {
        self.text.line_count()
    }

    fn line_len_chars(&self, line_index: usize) -> usize {
        self.text.line_len_chars(line_index).unwrap_or(0)
    }

    fn cursor(&self) -> TextPoint {
        self.text.cursor()
    }

    fn set_cursor(&mut self, point: TextPoint) {
        self.text.set_cursor(point);
    }

    fn visible_rows(&self) -> usize {
        self.viewport_rows.max(1)
    }

    fn wrap_cols(&self) -> usize {
        self.wrap_cols.max(1)
    }

    fn set_view_metrics(&mut self, visible_rows: usize, wrap_cols: usize) {
        self.viewport_rows = visible_rows.max(1);
        self.wrap_cols = wrap_cols.max(1);
        self.scroll_row = self.scroll_row.min(self.max_scroll_row());
    }

    fn row_count_for_line(&self, line_index: usize) -> usize {
        let line = self.text.line(line_index).unwrap_or_default();
        wrap_line_segments(&LineCharMap::new(&line), self.wrap_cols(), self.wrap_cols())
            .len()
            .max(1)
    }

    fn max_scroll_row_for(&self, visible_rows: usize) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let visible_rows = visible_rows.max(1);
        let mut rows = 0usize;
        for line_index in (0..line_count).rev() {
            let row_count = self.row_count_for_line(line_index);
            if rows.saturating_add(row_count) > visible_rows {
                return if rows == 0 {
                    line_index
                } else {
                    line_index.saturating_add(1)
                };
            }
            rows = rows.saturating_add(row_count);
        }
        0
    }

    fn max_scroll_row(&self) -> usize {
        self.max_scroll_row_for(self.visible_rows())
    }

    fn should_follow_output(&self) -> bool {
        self.scroll_row >= self.max_scroll_row()
    }

    fn replace_lines(&mut self, lines: Vec<String>, follow_output: bool) {
        let cursor = self.cursor();
        let scroll_row = self.scroll_row;
        self.syntax_lines.clear();
        self.text = if lines.is_empty() {
            TextBuffer::new()
        } else {
            TextBuffer::from_text(lines.join("\n"))
        };
        self.text.mark_clean();
        if self.line_count() == 0 {
            self.text.set_cursor(TextPoint::default());
            self.scroll_row = 0;
            return;
        }
        let line = cursor.line.min(self.line_count().saturating_sub(1));
        let column = cursor.column.min(self.line_len_chars(line));
        self.text.set_cursor(TextPoint::new(line, column));
        if follow_output {
            self.scroll_row = self.max_scroll_row();
        } else {
            self.scroll_row = scroll_row.min(self.max_scroll_row());
        }
    }

    fn set_indexed_syntax_lines(&mut self, syntax_lines: IndexedSyntaxLines) {
        self.syntax_lines = syntax_lines;
    }

    fn append_lines(&mut self, mut lines: Vec<String>, follow_output: bool) {
        if lines.is_empty() {
            return;
        }
        let mut existing = (0..self.line_count())
            .map(|line_index| self.text.line(line_index).unwrap_or_default().to_owned())
            .collect::<Vec<_>>();
        existing.append(&mut lines);
        self.replace_lines(existing, follow_output);
    }

    fn line_at_viewport_offset(&self, offset: usize) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let mut line_index = self.scroll_row.min(line_count.saturating_sub(1));
        let mut remaining = offset;
        while line_index + 1 < line_count {
            let row_count = self.row_count_for_line(line_index);
            if remaining < row_count {
                return line_index;
            }
            remaining = remaining.saturating_sub(row_count);
            line_index = line_index.saturating_add(1);
        }
        line_index
    }

    fn cursor_viewport_offset(&self) -> usize {
        let line_count = self.line_count();
        if line_count == 0 {
            return 0;
        }
        let cursor = self.cursor();
        if cursor.line < self.scroll_row {
            return 0;
        }
        let mut offset = 0usize;
        for line_index in self.scroll_row..cursor.line {
            offset = offset.saturating_add(self.row_count_for_line(line_index));
        }
        let line = self.text.line(cursor.line).unwrap_or_default();
        let segments =
            wrap_line_segments(&LineCharMap::new(&line), self.wrap_cols(), self.wrap_cols());
        offset.saturating_add(segment_index_for_column(&segments, cursor.column))
    }

    fn ensure_cursor_visible(&mut self) {
        let line_count = self.line_count();
        if line_count == 0 {
            self.scroll_row = 0;
            return;
        }
        let cursor = self.cursor();
        if cursor.line < self.scroll_row {
            self.scroll_row = cursor.line;
            return;
        }
        let visible_rows = self.visible_rows();
        let mut offset = self.cursor_viewport_offset();
        if offset < visible_rows {
            return;
        }
        let mut new_scroll = self.scroll_row;
        while offset >= visible_rows && new_scroll < cursor.line {
            offset = offset.saturating_sub(self.row_count_for_line(new_scroll));
            new_scroll = new_scroll.saturating_add(1);
        }
        self.scroll_row = new_scroll.min(self.max_scroll_row());
    }
}

impl AcpRenderedLine {
    fn plain_text(&self) -> String {
        match self {
            Self::Text(line) => line.text.clone(),
            Self::Image(line) => line.label.clone(),
            Self::Spacer => String::new(),
        }
    }
}

fn acp_chat_bubble_cols(wrap_cols: usize) -> usize {
    wrap_cols
        .saturating_mul(ACP_CHAT_BUBBLE_NUM)
        .saturating_div(ACP_CHAT_BUBBLE_DEN)
        .max(8)
        .min(wrap_cols.max(1))
}

#[derive(Debug, Clone, Copy)]
struct AcpWrapSegment {
    start_col: usize,
    end_col: usize,
}

fn acp_rendered_text_wrap_cols(line: &AcpRenderedTextLine, wrap_cols: usize) -> usize {
    let available = match line.align {
        AcpChatAlign::Full => wrap_cols,
        AcpChatAlign::Start | AcpChatAlign::End => acp_chat_bubble_cols(wrap_cols),
    };
    available
        .saturating_sub(acp_prefix_columns(&line.prefix, acp_spinner_frame()))
        .max(1)
}

fn acp_rendered_text_segments(line: &AcpRenderedTextLine, wrap_cols: usize) -> Vec<AcpWrapSegment> {
    let text_wrap_cols = acp_rendered_text_wrap_cols(line, wrap_cols);
    acp_wrap_line_segments(&LineCharMap::new(&line.text), text_wrap_cols)
}

fn acp_wrap_line_segments(map: &LineCharMap, wrap_cols: usize) -> Vec<AcpWrapSegment> {
    let wrap_cols = wrap_cols.max(1);
    let len = map.len();
    if len == 0 {
        return vec![AcpWrapSegment {
            start_col: 0,
            end_col: 0,
        }];
    }

    let mut segments = Vec::new();
    let mut start = 0usize;
    while start < len {
        let remaining = map.display_cols_between(start, len);
        if remaining <= wrap_cols {
            segments.push(AcpWrapSegment {
                start_col: start,
                end_col: len,
            });
            break;
        }

        let mut wrap_limit = start;
        while wrap_limit < len && map.display_cols_between(start, wrap_limit + 1) <= wrap_cols {
            wrap_limit = wrap_limit.saturating_add(1);
        }
        if wrap_limit == start {
            wrap_limit = (start + 1).min(len);
        }
        let content_start = (start..wrap_limit)
            .find(|&idx| !map.whitespace.get(idx).copied().unwrap_or(false))
            .unwrap_or(wrap_limit);
        let end = (content_start..wrap_limit)
            .rev()
            .find(|&idx| map.whitespace.get(idx).copied().unwrap_or(false))
            .map(|idx| idx + 1)
            .unwrap_or(wrap_limit);
        let mut end = end;
        if end == wrap_limit && acp_wrap_window_all_whitespace(map, end, wrap_cols) {
            let mut rebalance = end;
            while rebalance > start.saturating_add(1) {
                rebalance = rebalance.saturating_sub(1);
                if !acp_wrap_window_all_whitespace(map, rebalance, wrap_cols) {
                    end = rebalance;
                    break;
                }
            }
        }
        segments.push(AcpWrapSegment {
            start_col: start,
            end_col: end,
        });
        start = end;
    }

    if segments.is_empty() {
        segments.push(AcpWrapSegment {
            start_col: 0,
            end_col: 0,
        });
    }

    segments
}

fn acp_wrap_window_all_whitespace(map: &LineCharMap, start: usize, wrap_cols: usize) -> bool {
    let end = start.saturating_add(wrap_cols).min(map.len());
    start < end && (start..end).all(|idx| map.whitespace.get(idx).copied().unwrap_or(false))
}

fn acp_segment_index_for_column(segments: &[AcpWrapSegment], column: usize) -> usize {
    if segments.is_empty() {
        return 0;
    }
    for (index, segment) in segments.iter().enumerate() {
        if column < segment.end_col || index == segments.len().saturating_sub(1) {
            return index;
        }
    }
    segments.len().saturating_sub(1)
}

fn acp_rendered_line_row_count(line: &AcpRenderedLine, wrap_cols: usize) -> usize {
    match line {
        AcpRenderedLine::Text(line) => acp_rendered_text_segments(line, wrap_cols).len().max(1),
        AcpRenderedLine::Image(image) => image.rows.max(1),
        AcpRenderedLine::Spacer => 1,
    }
}

fn acp_pane_content_rows(pane: &AcpPaneState, wrap_cols: usize) -> usize {
    pane.render_lines
        .iter()
        .map(|line| acp_rendered_line_row_count(line, wrap_cols))
        .sum()
}

fn acp_text_segment(text: impl Into<String>, role: AcpColorRole) -> AcpRenderedSegment {
    AcpRenderedSegment {
        text: text.into(),
        role,
        animate: false,
    }
}

fn acp_spinner_segment(role: AcpColorRole) -> AcpRenderedSegment {
    AcpRenderedSegment {
        text: editor_icons::symbols::fa::FA_SPINNER.to_owned(),
        role,
        animate: true,
    }
}

#[derive(Debug, Clone)]
struct WrapRowCache {
    wrap_cols: usize,
    indent_size: usize,
    line_count: usize,
    prefix_rows: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
struct WrapCacheInlineEdit {
    line_index: usize,
    old_row_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct WrapCacheLineSplice {
    start_line: usize,
    old_span: usize,
    old_line_count: usize,
}

#[derive(Debug, Clone, Copy)]
struct WrapCacheInsertPlan {
    inline: Option<WrapCacheInlineEdit>,
    splice: Option<WrapCacheLineSplice>,
    had_cache: bool,
    has_newline: bool,
}

const LARGE_BUFFER_WRAP_CACHE_LINE_THRESHOLD: usize = 2_048;
const LARGE_BUFFER_SYNC_INDENT_LINE_THRESHOLD: usize = 2_048;
const MAX_WRAP_CACHE_SPLICE_LINES: usize = 256;

impl WrapRowCache {
    fn build(buffer: &ShellBuffer, wrap_cols: usize, indent_size: usize) -> Self {
        let line_count = buffer.line_count();
        let mut prefix_rows: Vec<usize> = Vec::with_capacity(line_count + 1);
        prefix_rows.push(0);
        for line_index in 0..line_count {
            let rows = buffer.line_visual_row_count(line_index, wrap_cols, indent_size);
            let next = prefix_rows
                .last()
                .copied()
                .unwrap_or(0)
                .saturating_add(rows);
            prefix_rows.push(next);
        }
        Self {
            wrap_cols,
            indent_size,
            line_count,
            prefix_rows,
        }
    }

    fn max_scroll_row(&self, visible_rows: usize) -> usize {
        if self.line_count == 0 {
            return 0;
        }
        let visible_rows = visible_rows.max(1);
        let total_rows = self.prefix_rows.last().copied().unwrap_or(0);
        if total_rows <= visible_rows {
            return 0;
        }
        let threshold = total_rows.saturating_sub(visible_rows);
        self.prefix_rows
            .partition_point(|&value| value <= threshold)
            .saturating_sub(1)
            .min(self.line_count.saturating_sub(1))
    }

    fn adjust_for_line_row_delta(
        &mut self,
        line_index: usize,
        old_row_count: usize,
        new_row_count: usize,
    ) {
        self.apply_row_count_delta_from(line_index.saturating_add(1), old_row_count, new_row_count);
    }

    fn apply_row_count_delta_from(
        &mut self,
        from_index: usize,
        old_row_count: usize,
        new_row_count: usize,
    ) {
        if old_row_count == new_row_count {
            return;
        }
        let affected = self.prefix_rows.iter_mut().skip(from_index);
        if new_row_count > old_row_count {
            let delta = new_row_count.saturating_sub(old_row_count);
            for prefix in affected {
                *prefix = prefix.saturating_add(delta);
            }
        } else {
            let delta = old_row_count.saturating_sub(new_row_count);
            for prefix in affected {
                *prefix = prefix.saturating_sub(delta);
            }
        }
    }

    fn matches(&self, wrap_cols: usize, indent_size: usize, line_count: usize) -> bool {
        self.wrap_cols == wrap_cols
            && self.indent_size == indent_size
            && self.line_count == line_count
    }

    fn splice_lines(
        &mut self,
        start_line: usize,
        old_span: usize,
        new_row_counts: &[usize],
    ) -> bool {
        let old_end = start_line.saturating_add(old_span);
        if old_span == 0
            || old_end >= self.prefix_rows.len()
            || self.prefix_rows.len() != self.line_count.saturating_add(1)
        {
            return false;
        }
        let base = self.prefix_rows[start_line];
        let old_region_rows = self.prefix_rows[old_end].saturating_sub(base);
        let new_region_rows: usize = new_row_counts.iter().copied().sum();
        let mut new_prefixes = Vec::with_capacity(new_row_counts.len());
        let mut acc = base;
        for &rows in new_row_counts {
            acc = acc.saturating_add(rows);
            new_prefixes.push(acc);
        }
        let rest_start = start_line
            .saturating_add(1)
            .saturating_add(new_row_counts.len());
        self.prefix_rows.splice(
            start_line.saturating_add(1)..old_end.saturating_add(1),
            new_prefixes,
        );
        self.apply_row_count_delta_from(rest_start, old_region_rows, new_region_rows);
        self.line_count = self
            .line_count
            .saturating_sub(old_span)
            .saturating_add(new_row_counts.len());
        self.prefix_rows.len() == self.line_count.saturating_add(1)
    }
}
