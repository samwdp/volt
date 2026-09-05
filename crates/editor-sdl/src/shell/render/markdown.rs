#[derive(Debug, Clone)]
pub(super) struct MarkdownInlineImageDraw {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
    rows: usize,
    alt: String,
}

impl MarkdownInlineImageDraw {
    pub(super) fn rows(&self) -> usize {
        self.rows
    }
}

#[derive(Debug, Default)]
pub(super) struct MarkdownPrettyPaintPlan {
    pub(super) text_overrides: BTreeMap<usize, String>,
    pub(super) images: BTreeMap<usize, MarkdownInlineImageDraw>,
}

fn markdown_inline_image_rows(
    image_width: u32,
    image_height: u32,
    pane_width_px: u32,
    line_height: i32,
    max_rows: usize,
) -> usize {
    let line_height = line_height.max(1) as u32;
    let pane_width_px = pane_width_px.max(1);
    let scaled_height = if image_width == 0 {
        image_height
    } else {
        let scaled = (u64::from(image_height) * u64::from(pane_width_px)) / u64::from(image_width);
        scaled.min(u64::from(u32::MAX)) as u32
    };
    let rows = scaled_height.div_ceil(line_height).max(1) as usize;
    rows.min(max_rows.max(1))
}

pub(super) fn markdown_pretty_paint_plan(
    buffer: &ShellBuffer,
    user_library: &dyn UserLibrary,
    args: MarkdownPrettyPaintArgs,
) -> MarkdownPrettyPaintPlan {
    let MarkdownPrettyPaintArgs {
        visible_start,
        visible_end,
        visual_selection,
        input_mode,
        pane_width_px,
        line_height,
    } = args;
    let mut paint = MarkdownPrettyPaintPlan::default();
    if buffer.language_id() != Some("markdown") {
        return paint;
    }
    let config = markdown_pretty::user_library_pretty_config(user_library);
    let enabled = buffer.markdown_pretty_enabled().unwrap_or(config.enabled);
    let plan = markdown_pretty::cached_plan_for_buffer(buffer, &config, enabled, None);
    if !enabled || plan.skipped_by_kill_switch || plan.lines.is_empty() {
        return paint;
    }
    let cursor_line = buffer.cursor_row();
    let visual_range = if matches!(input_mode, InputMode::Visual) {
        visual_selection.map(|selection| match selection {
            VisualSelection::Range(range) => {
                let start = range.start().line.min(range.end().line);
                let end = range.start().line.max(range.end().line);
                start..end.saturating_add(1)
            }
            VisualSelection::Block(block) => {
                let start = block.start_line.min(block.end_line);
                let end = block.start_line.max(block.end_line);
                start..end.saturating_add(1)
            }
        })
    } else {
        None
    };
    let line_count = buffer.line_count();
    for line_index in visible_start..visible_end.min(line_count) {
        let source = buffer.text.line(line_index).unwrap_or_default();
        let anti = editor_markdown::line_is_anti_concealed(
            Some(cursor_line),
            visual_range.as_ref(),
            line_index,
        );
        if !anti
            && let Some(image) =
                markdown_pretty::line_plan(&plan, line_index).and_then(|line| line.image.as_ref())
        {
            let entry =
                markdown_pretty::ensure_image_loaded(&image.destination, config.image_max_bytes);
            match entry {
                markdown_pretty::MarkdownImageCacheEntry::Ready(decoded) => {
                    let rows = markdown_inline_image_rows(
                        decoded.width,
                        decoded.height,
                        pane_width_px,
                        line_height,
                        config.image_max_rows,
                    );
                    paint.images.insert(
                        line_index,
                        MarkdownInlineImageDraw {
                            width: decoded.width,
                            height: decoded.height,
                            pixels: Arc::clone(&decoded.pixels),
                            rows,
                            alt: image.alt.clone(),
                        },
                    );
                    // Empty display text — image occupies the visual rows.
                    paint.text_overrides.insert(line_index, String::new());
                }
                markdown_pretty::MarkdownImageCacheEntry::Loading => {
                    paint.text_overrides.insert(
                        line_index,
                        format!("{} loading…", editor_icons::symbols::md::MD_IMAGE),
                    );
                }
                markdown_pretty::MarkdownImageCacheEntry::Failed(error) => {
                    paint.text_overrides.insert(
                        line_index,
                        format!("{} {}", editor_icons::symbols::md::MD_IMAGE_BROKEN, error),
                    );
                }
            }
            continue;
        }
        let display = markdown_pretty::pretty_display_line(&plan, anti, line_index, &source);
        if display != source {
            paint.text_overrides.insert(line_index, display);
        }
    }
    paint
}
