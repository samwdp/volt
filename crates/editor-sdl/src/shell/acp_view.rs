fn acp_tool_call_from_partial_update(update: &ToolCallUpdate) -> ToolCall {
    let mut tool_call = ToolCall::new(
        update.tool_call_id.clone(),
        update
            .fields
            .title
            .clone()
            .unwrap_or_else(|| "Tool call".to_owned()),
    );
    if let Some(kind) = update.fields.kind {
        tool_call.kind = kind;
    }
    if let Some(status) = update.fields.status {
        tool_call.status = status;
    }
    if let Some(content) = update.fields.content.clone() {
        tool_call.content = content;
    }
    if let Some(locations) = update.fields.locations.clone() {
        tool_call.locations = locations;
    }
    if let Some(raw_input) = update.fields.raw_input.clone() {
        tool_call.raw_input = Some(raw_input);
    }
    if let Some(raw_output) = update.fields.raw_output.clone() {
        tool_call.raw_output = Some(raw_output);
    }
    tool_call
}

fn acp_build_plan_lines(entries: &[PlanEntry]) -> Vec<AcpRenderedLine> {
    if entries.is_empty() {
        return vec![AcpRenderedLine::Text(acp_text_line(
            vec![acp_icon_segment(
                editor_icons::symbols::cod::COD_NOTEBOOK,
                AcpColorRole::Muted,
            )],
            " Waiting for plan updates...",
            AcpColorRole::Muted,
        ))];
    }
    entries
        .iter()
        .map(|entry| {
            let mut prefix = acp_plan_status_segments(entry.status.clone(), entry.priority.clone());
            prefix.push(acp_text_segment(" ", AcpColorRole::Default));
            AcpRenderedLine::Text(acp_text_line(
                prefix,
                entry.content.clone(),
                AcpColorRole::Default,
            ))
        })
        .collect()
}

struct AcpMarkdownPaint<'a> {
    registry: &'a mut SyntaxRegistry,
    config: &'a editor_markdown::MarkdownPrettyConfig,
}

fn normalize_acp_plan_entries(entries: &mut [PlanEntry]) {
    let active_index = entries
        .iter()
        .position(|entry| matches!(entry.status, PlanEntryStatus::InProgress));
    if let Some(active_index) = active_index {
        for entry in &mut entries[..active_index] {
            entry.status = PlanEntryStatus::Completed;
        }
        return;
    }

    let completed_prefix_len = entries
        .iter()
        .rposition(|entry| matches!(entry.status, PlanEntryStatus::Completed))
        .map(|index| index + 1)
        .unwrap_or(0);
    for entry in &mut entries[..completed_prefix_len] {
        entry.status = PlanEntryStatus::Completed;
    }
}

fn acp_build_output_lines(
    items: &[AcpOutputItem],
    mut markdown: Option<AcpMarkdownPaint<'_>>,
    buffer_enabled: Option<bool>,
) -> Vec<AcpRenderedLine> {
    let mut lines = Vec::new();
    let mut bubble_group = 1u32;
    for (index, item) in items.iter().enumerate() {
        if index > 0 {
            lines.push(AcpRenderedLine::Spacer);
        }
        match item {
            AcpOutputItem::UserPrompt(prompt) => {
                let prefix = vec![
                    acp_icon_segment(editor_icons::symbols::cod::COD_PERSON, AcpColorRole::Accent),
                    acp_text_segment(" ", AcpColorRole::Default),
                ];
                lines.extend(acp_mark_chat(
                    acp_multiline_text_lines(prefix, prompt, AcpColorRole::Accent),
                    AcpChatAlign::End,
                    bubble_group,
                ));
            }
            AcpOutputItem::SystemMessage(message) => {
                let prefix = vec![
                    acp_icon_segment(editor_icons::symbols::cod::COD_INFO, AcpColorRole::Muted),
                    acp_text_segment(" ", AcpColorRole::Default),
                ];
                lines.extend(acp_mark_chat(
                    acp_multiline_text_lines(prefix, message, AcpColorRole::Muted),
                    AcpChatAlign::Start,
                    bubble_group,
                ));
            }
            AcpOutputItem::AgentBlocks(blocks) => {
                let mut agent_lines = Vec::new();
                for block in blocks {
                    let prefix = vec![
                        acp_icon_segment(
                            editor_icons::symbols::cod::COD_COMMENT,
                            AcpColorRole::Accent,
                        ),
                        acp_text_segment(" ", AcpColorRole::Default),
                    ];
                    if let (Some(paint), ContentBlock::Text(text)) = (markdown.as_mut(), block) {
                        agent_lines.extend(acp_render_markdown_text_block(
                            prefix,
                            &text.text,
                            AcpColorRole::Default,
                            paint,
                            buffer_enabled,
                        ));
                    } else {
                        agent_lines.extend(acp_render_content_block(
                            block,
                            prefix,
                            AcpColorRole::Default,
                        ));
                    }
                }
                lines.extend(acp_mark_chat(
                    agent_lines,
                    AcpChatAlign::Start,
                    bubble_group,
                ));
            }
            AcpOutputItem::ToolCall(tool_call) => {
                let mut tool_lines = Vec::new();
                let mut prefix = acp_status_segments(tool_call.status);
                prefix.push(acp_text_segment(" ", AcpColorRole::Default));
                prefix.push(acp_icon_segment(
                    acp_tool_kind_icon(tool_call.kind),
                    AcpColorRole::Accent,
                ));
                prefix.push(acp_text_segment(" ", AcpColorRole::Default));
                let title_role = if matches!(tool_call.status, ToolCallStatus::Completed) {
                    AcpColorRole::Muted
                } else {
                    AcpColorRole::Default
                };
                tool_lines.push(AcpRenderedLine::Text(
                    acp_text_line(prefix, tool_call.title.clone(), title_role)
                        .with_row_fill(acp_tool_status_row_fill(tool_call.status)),
                ));
                for content in &tool_call.content {
                    tool_lines.extend(acp_render_tool_content(content));
                }
                for location in &tool_call.locations {
                    let line = location
                        .line
                        .map(|line| format!("{}:{line}", location.path.display()))
                        .unwrap_or_else(|| location.path.display().to_string());
                    tool_lines.push(AcpRenderedLine::Text(acp_text_line(
                        vec![
                            acp_icon_segment(
                                editor_icons::symbols::cod::COD_SEARCH,
                                AcpColorRole::Muted,
                            ),
                            acp_text_segment(" ", AcpColorRole::Default),
                        ],
                        line,
                        AcpColorRole::Muted,
                    )));
                }
                lines.extend(acp_mark_chat(tool_lines, AcpChatAlign::Start, bubble_group));
            }
        }
        bubble_group = bubble_group.saturating_add(1);
    }
    if lines.is_empty() {
        lines.extend(acp_mark_chat(
            vec![AcpRenderedLine::Text(acp_text_line(
                vec![acp_icon_segment(
                    editor_icons::symbols::cod::COD_HISTORY,
                    AcpColorRole::Muted,
                )],
                " Waiting for session output...",
                AcpColorRole::Muted,
            ))],
            AcpChatAlign::Start,
            1,
        ));
    }
    lines
}

fn acp_render_tool_content(content: &ToolCallContent) -> Vec<AcpRenderedLine> {
    match content {
        ToolCallContent::Content(content) => acp_mark_gutter(acp_render_content_block(
            &content.content,
            vec![
                acp_text_segment(ACP_TOOL_NEST_PAD, AcpColorRole::Muted),
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_CHEVRON_RIGHT,
                    AcpColorRole::Muted,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            AcpColorRole::Default,
        )),
        ToolCallContent::Diff(diff) => acp_render_diff(diff),
        ToolCallContent::Terminal(terminal) => {
            acp_mark_gutter(vec![AcpRenderedLine::Text(acp_text_line(
                vec![
                    acp_text_segment(ACP_TOOL_NEST_PAD, AcpColorRole::Muted),
                    acp_icon_segment(
                        editor_icons::symbols::cod::COD_TERMINAL,
                        AcpColorRole::Accent,
                    ),
                    acp_text_segment(" ", AcpColorRole::Default),
                ],
                format!("terminal {}", terminal.terminal_id),
                AcpColorRole::Default,
            ))])
        }
        _ => acp_mark_gutter(vec![AcpRenderedLine::Text(acp_text_line(
            vec![
                acp_text_segment(ACP_TOOL_NEST_PAD, AcpColorRole::Muted),
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_WARNING,
                    AcpColorRole::Warning,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            "Unsupported tool content",
            AcpColorRole::Warning,
        ))]),
    }
}

fn acp_render_content_block(
    block: &ContentBlock,
    prefix: Vec<AcpRenderedSegment>,
    text_role: AcpColorRole,
) -> Vec<AcpRenderedLine> {
    match block {
        ContentBlock::Text(text) => acp_multiline_text_lines(prefix, &text.text, text_role),
        ContentBlock::Image(image) => match acp_decode_image(image) {
            Ok(decoded) => vec![AcpRenderedLine::Image(AcpRenderedImageLine {
                label: format!(
                    "{} {}",
                    editor_icons::symbols::fa::FA_IMAGE,
                    image.mime_type
                ),
                image: Some(decoded),
                rows: ACP_IMAGE_ROWS,
            })],
            Err(error) => acp_multiline_text_lines(
                vec![
                    acp_icon_segment(
                        editor_icons::symbols::cod::COD_WARNING,
                        AcpColorRole::Warning,
                    ),
                    acp_text_segment(" ", AcpColorRole::Default),
                ],
                format!("Image decode failed: {error}"),
                AcpColorRole::Warning,
            ),
        },
        ContentBlock::Audio(_) | ContentBlock::ResourceLink(_) | ContentBlock::Resource(_) => {
            acp_multiline_text_lines(
                vec![
                    acp_icon_segment(
                        editor_icons::symbols::cod::COD_WARNING,
                        AcpColorRole::Warning,
                    ),
                    acp_text_segment(" ", AcpColorRole::Default),
                ],
                "Unsupported ACP content block",
                AcpColorRole::Warning,
            )
        }
        _ => acp_multiline_text_lines(
            vec![
                acp_icon_segment(
                    editor_icons::symbols::cod::COD_WARNING,
                    AcpColorRole::Warning,
                ),
                acp_text_segment(" ", AcpColorRole::Default),
            ],
            "Unsupported ACP content block",
            AcpColorRole::Warning,
        ),
    }
}

fn acp_text_line(
    prefix: Vec<AcpRenderedSegment>,
    text: impl Into<String>,
    text_role: AcpColorRole,
) -> AcpRenderedTextLine {
    AcpRenderedTextLine {
        prefix,
        text: text.into(),
        text_role,
        syntax_spans: Vec::new(),
        row_fill: None,
        gutter: false,
        align: AcpChatAlign::Full,
        bubble: false,
        bubble_group: 0,
    }
}

impl AcpRenderedTextLine {
    fn with_row_fill(mut self, role: AcpColorRole) -> Self {
        self.row_fill = Some(role);
        self
    }

    fn with_gutter(mut self) -> Self {
        self.gutter = true;
        self
    }
}

fn acp_mark_chat(
    mut lines: Vec<AcpRenderedLine>,
    align: AcpChatAlign,
    bubble_group: u32,
) -> Vec<AcpRenderedLine> {
    for line in &mut lines {
        if let AcpRenderedLine::Text(text) = line {
            text.align = align;
            text.bubble = true;
            text.bubble_group = bubble_group;
        }
    }
    lines
}

fn acp_mark_gutter(mut lines: Vec<AcpRenderedLine>) -> Vec<AcpRenderedLine> {
    for line in &mut lines {
        if let AcpRenderedLine::Text(text) = line {
            text.gutter = true;
        }
    }
    lines
}

fn acp_tool_status_row_fill(status: ToolCallStatus) -> AcpColorRole {
    match status {
        ToolCallStatus::InProgress => AcpColorRole::Accent,
        ToolCallStatus::Completed => AcpColorRole::Success,
        ToolCallStatus::Failed => AcpColorRole::Error,
        _ => AcpColorRole::Muted,
    }
}

fn acp_output_header_title(state: &AcpBufferState) -> String {
    let live = state.output_items.iter().any(|item| {
        matches!(
            item,
            AcpOutputItem::ToolCall(tool_call)
                if matches!(
                    tool_call.status,
                    ToolCallStatus::Pending | ToolCallStatus::InProgress
                )
        )
    });
    let follow = state
        .output_pane
        .should_follow_output(state.output_pane.visible_rows());
    let count = state.output_items.len();
    if live {
        "Output · live".to_owned()
    } else if !follow {
        "Output · paused".to_owned()
    } else if count > 0 {
        format!("Output · {count}")
    } else {
        "Output".to_owned()
    }
}

fn acp_render_diff(diff: &Diff) -> Vec<AcpRenderedLine> {
    let mut lines = acp_mark_gutter(vec![AcpRenderedLine::Text(acp_text_line(
        vec![
            acp_text_segment(ACP_TOOL_NEST_PAD, AcpColorRole::Muted),
            acp_icon_segment(
                editor_icons::symbols::cod::COD_DIFF_MODIFIED,
                AcpColorRole::Warning,
            ),
            acp_text_segment(" ", AcpColorRole::Default),
        ],
        diff.path.display().to_string(),
        AcpColorRole::Muted,
    ))]);
    let hunks = acp_diff_display_lines(diff.old_text.as_deref(), &diff.new_text);
    let truncated = hunks.len() > ACP_DIFF_MAX_LINES;
    for (role, marker, text) in hunks.into_iter().take(ACP_DIFF_MAX_LINES) {
        lines.push(AcpRenderedLine::Text(
            acp_text_line(
                vec![
                    acp_text_segment(ACP_TOOL_NEST_PAD, AcpColorRole::Muted),
                    acp_text_segment(marker, role),
                    acp_text_segment(" ", AcpColorRole::Default),
                ],
                text,
                role,
            )
            .with_gutter(),
        ));
    }
    if truncated {
        lines.push(AcpRenderedLine::Text(
            acp_text_line(
                vec![acp_text_segment(ACP_TOOL_NEST_PAD, AcpColorRole::Muted)],
                "… truncated diff".to_owned(),
                AcpColorRole::Muted,
            )
            .with_gutter(),
        ));
    }
    lines
}

fn acp_diff_display_lines(
    old_text: Option<&str>,
    new_text: &str,
) -> Vec<(AcpColorRole, &'static str, String)> {
    let old_lines = old_text
        .unwrap_or("")
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect::<Vec<_>>();
    let new_lines = new_text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line))
        .collect::<Vec<_>>();
    if old_text.is_none() || old_lines.iter().all(|line| line.is_empty()) {
        return new_lines
            .into_iter()
            .map(|line| (AcpColorRole::Success, "+", line.to_owned()))
            .collect();
    }
    let mut prefix = 0usize;
    while prefix < old_lines.len()
        && prefix < new_lines.len()
        && old_lines[prefix] == new_lines[prefix]
    {
        prefix = prefix.saturating_add(1);
    }
    let mut old_end = old_lines.len();
    let mut new_end = new_lines.len();
    while old_end > prefix && new_end > prefix && old_lines[old_end - 1] == new_lines[new_end - 1] {
        old_end = old_end.saturating_sub(1);
        new_end = new_end.saturating_sub(1);
    }
    const CONTEXT: usize = 2;
    let mut out = Vec::new();
    let context_start = prefix.saturating_sub(CONTEXT);
    for line in &old_lines[context_start..prefix] {
        out.push((AcpColorRole::Muted, " ", (*line).to_owned()));
    }
    for line in &old_lines[prefix..old_end] {
        out.push((AcpColorRole::Error, "-", (*line).to_owned()));
    }
    for line in &new_lines[prefix..new_end] {
        out.push((AcpColorRole::Success, "+", (*line).to_owned()));
    }
    let context_end = (old_end.saturating_add(CONTEXT)).min(old_lines.len());
    for line in &old_lines[old_end..context_end] {
        out.push((AcpColorRole::Muted, " ", (*line).to_owned()));
    }
    out
}

fn acp_render_markdown_text_block(
    prefix: Vec<AcpRenderedSegment>,
    text: &str,
    text_role: AcpColorRole,
    paint: &mut AcpMarkdownPaint<'_>,
    buffer_enabled: Option<bool>,
) -> Vec<AcpRenderedLine> {
    let rendered =
        render_markdown_ephemeral_content(text, paint.config, buffer_enabled, Some(paint.registry));
    if rendered.lines.is_empty() {
        return acp_multiline_text_lines(prefix, text, text_role);
    }
    let continuation_prefix = acp_padding_prefix(&prefix);
    let syntax_lines = rendered.syntax_lines;
    rendered
        .lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let syntax_spans = syntax_lines.get(&index).cloned().unwrap_or_default();
            AcpRenderedLine::Text(AcpRenderedTextLine {
                prefix: if index == 0 {
                    prefix.clone()
                } else {
                    continuation_prefix.clone()
                },
                text: line,
                text_role,
                syntax_spans,
                row_fill: None,
                gutter: false,
                align: AcpChatAlign::Full,
                bubble: false,
                bubble_group: 0,
            })
        })
        .collect()
}

fn acp_multiline_text_lines(
    prefix: Vec<AcpRenderedSegment>,
    text: impl AsRef<str>,
    text_role: AcpColorRole,
) -> Vec<AcpRenderedLine> {
    let text = text.as_ref();
    let mut lines = Vec::new();
    let continuation_prefix = acp_padding_prefix(&prefix);
    let parts = text
        .split('\n')
        .map(|line| line.strip_suffix('\r').unwrap_or(line).to_owned())
        .collect::<Vec<_>>();
    for (index, line) in parts.into_iter().enumerate() {
        lines.push(AcpRenderedLine::Text(acp_text_line(
            if index == 0 {
                prefix.clone()
            } else {
                continuation_prefix.clone()
            },
            line,
            text_role,
        )));
    }
    lines
}

fn acp_padding_prefix(prefix: &[AcpRenderedSegment]) -> Vec<AcpRenderedSegment> {
    let width = prefix
        .iter()
        .map(|segment| segment.text.chars().count())
        .sum();
    if width == 0 {
        return Vec::new();
    }
    vec![acp_text_segment(" ".repeat(width), AcpColorRole::Muted)]
}

fn acp_icon_segment(icon: &str, role: AcpColorRole) -> AcpRenderedSegment {
    acp_text_segment(icon, role)
}

fn acp_status_segments(status: ToolCallStatus) -> Vec<AcpRenderedSegment> {
    match status {
        ToolCallStatus::Pending => vec![acp_icon_segment(
            editor_icons::symbols::dev::DEV_CIRCLECI,
            AcpColorRole::Muted,
        )],
        ToolCallStatus::InProgress => vec![acp_spinner_segment(AcpColorRole::Accent)],
        ToolCallStatus::Completed => vec![acp_icon_segment(
            editor_icons::symbols::fa::FA_CHECK,
            AcpColorRole::Success,
        )],
        ToolCallStatus::Failed => vec![acp_icon_segment(
            editor_icons::symbols::cod::COD_ERROR,
            AcpColorRole::Error,
        )],
        _ => vec![acp_icon_segment(
            editor_icons::symbols::cod::COD_WARNING,
            AcpColorRole::Warning,
        )],
    }
}

fn acp_plan_status_segments(
    status: PlanEntryStatus,
    priority: PlanEntryPriority,
) -> Vec<AcpRenderedSegment> {
    match status {
        PlanEntryStatus::Pending => vec![acp_icon_segment(
            editor_icons::symbols::dev::DEV_CIRCLECI,
            acp_priority_color_role(priority),
        )],
        PlanEntryStatus::InProgress => vec![acp_spinner_segment(AcpColorRole::Accent)],
        PlanEntryStatus::Completed => vec![acp_icon_segment(
            editor_icons::symbols::fa::FA_CHECK,
            AcpColorRole::Success,
        )],
        _ => vec![acp_icon_segment(
            editor_icons::symbols::cod::COD_WARNING,
            AcpColorRole::Warning,
        )],
    }
}

fn acp_priority_color_role(priority: PlanEntryPriority) -> AcpColorRole {
    match priority {
        PlanEntryPriority::High => AcpColorRole::PriorityHigh,
        PlanEntryPriority::Medium => AcpColorRole::PriorityMedium,
        PlanEntryPriority::Low => AcpColorRole::PriorityLow,
        _ => AcpColorRole::Muted,
    }
}

fn acp_tool_kind_icon(kind: ToolKind) -> &'static str {
    match kind {
        ToolKind::Read => editor_icons::symbols::cod::COD_NOTEBOOK,
        ToolKind::Edit => editor_icons::symbols::cod::COD_EDIT,
        ToolKind::Delete => editor_icons::symbols::cod::COD_DIFF_REMOVED,
        ToolKind::Move => editor_icons::symbols::cod::COD_ARROW_SWAP,
        ToolKind::Search => editor_icons::symbols::cod::COD_SEARCH,
        ToolKind::Execute => editor_icons::symbols::cod::COD_TERMINAL,
        ToolKind::Think => editor_icons::symbols::cod::COD_LIGHTBULB,
        ToolKind::Fetch => editor_icons::symbols::cod::COD_CLOUD_DOWNLOAD,
        ToolKind::SwitchMode => editor_icons::symbols::cod::COD_SYNC,
        ToolKind::Other => editor_icons::symbols::cod::COD_TOOLS,
        _ => editor_icons::symbols::cod::COD_TOOLS,
    }
}

fn acp_decode_image(image: &agent_client_protocol::ImageContent) -> Result<DecodedImage, String> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(image.data.as_bytes())
        .map_err(|error| error.to_string())?;
    decode_raster_image_bytes(&bytes)
}

fn decode_raster_image_bytes(bytes: &[u8]) -> Result<DecodedImage, String> {
    let decoded = image::load_from_memory(bytes).map_err(|error| error.to_string())?;
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    Ok(DecodedImage {
        width,
        height,
        pixels: Arc::<[u8]>::from(rgba.into_raw()),
    })
}

fn decode_raster_image_path(path: &Path) -> Result<DecodedImage, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    decode_raster_image_bytes(&bytes)
}

fn rasterize_svg_text(text: &str, path: Option<&Path>) -> Result<DecodedImage, String> {
    let mut options = resvg::usvg::Options {
        resources_dir: path.and_then(Path::parent).map(Path::to_path_buf),
        ..resvg::usvg::Options::default()
    };
    options.fontdb_mut().load_system_fonts();
    let tree = resvg::usvg::Tree::from_str(text, &options).map_err(|error| error.to_string())?;
    let size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or_else(|| "failed to allocate SVG render target".to_owned())?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    Ok(DecodedImage {
        width: pixmap.width(),
        height: pixmap.height(),
        pixels: Arc::<[u8]>::from(pixmap.take()),
    })
}
