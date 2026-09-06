#[derive(Debug, Clone, PartialEq, Eq)]
enum HoverProviderFragment {
    PlainLines(Vec<String>),
    MarkdownText(String),
    SignatureHelpMarkdown {
        text: String,
        active_parameter: Option<editor_lsp::LspSignatureActiveParameter>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HoverProviderDraft {
    provider_label: String,
    provider_icon: String,
    fragments: Vec<HoverProviderFragment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HoverOverlayDraft {
    buffer_id: BufferId,
    anchor: TextPoint,
    token: String,
    providers: Vec<HoverProviderDraft>,
    line_limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct HoverRenderedContent {
    lines: Vec<String>,
    syntax_lines: IndexedSyntaxLines,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownCodeFenceBlock {
    language: Option<String>,
    code_start_line: usize,
    code_end_line_exclusive: usize,
}

fn hover_overlay_draft_for_buffer(
    buffer_id: BufferId,
    buffer: &ShellBuffer,
    registry: &HoverRegistry,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
    user_library: &dyn UserLibrary,
) -> Option<HoverOverlayDraft> {
    if registry.providers.is_empty() {
        return None;
    }
    let anchor = buffer.cursor_point();
    let token_info = completion_token_at_cursor(buffer);
    let token = token_info
        .as_ref()
        .map(|(_, token)| token.clone())
        .filter(|token| !token.is_empty())
        .unwrap_or_else(|| "Cursor".to_owned());
    let providers = registry
        .providers
        .iter()
        .filter_map(|provider| {
            let fragments = match provider.kind {
                HoverProviderKind::TestHover => vec![HoverProviderFragment::PlainLines(
                    hover_test_provider_lines(buffer, token_info.as_ref()),
                )],
                HoverProviderKind::Lsp => {
                    hover_lsp_provider_fragments(buffer, lsp_client, lsp_context)
                }
                HoverProviderKind::SignatureHelp => {
                    hover_signature_provider_fragments(buffer, lsp_client, lsp_context)
                }
                HoverProviderKind::Diagnostics => {
                    hover_diagnostic_provider_fragments(buffer, user_library)
                }
                HoverProviderKind::Manual => vec![HoverProviderFragment::PlainLines(
                    hover_manual_provider_lines(buffer, provider),
                )],
            };
            (!hover_provider_fragments_empty(&fragments)).then(|| HoverProviderDraft {
                provider_label: provider.label.clone(),
                provider_icon: provider.icon.clone(),
                fragments,
            })
        })
        .collect::<Vec<_>>();
    let providers = if providers.is_empty() {
        vec![HoverProviderDraft {
            provider_label: "Hover".to_owned(),
            provider_icon: editor_icons::symbols::md::MD_HELP_CIRCLE_OUTLINE.to_owned(),
            fragments: vec![HoverProviderFragment::PlainLines(
                hover_empty_provider_lines(buffer, token_info.as_ref()),
            )],
        }]
    } else {
        providers
    };
    Some(HoverOverlayDraft {
        buffer_id,
        anchor,
        token,
        providers,
        line_limit: registry.line_limit,
    })
}

fn hover_test_provider_lines(
    buffer: &ShellBuffer,
    token_info: Option<&(TextRange, String)>,
) -> Vec<String> {
    let mut lines = vec![
        format!("Buffer: {}", buffer.display_name()),
        format!(
            "Line: {}, Column: {}",
            buffer.cursor_row() + 1,
            buffer.cursor_col() + 1
        ),
    ];
    if let Some((range, token)) = token_info {
        lines.extend([
            format!("Token: {token}"),
            format!(
                "Range: {}:{}-{}:{}",
                range.start().line + 1,
                range.start().column + 1,
                range.end().line + 1,
                range.end().column + 1
            ),
            format!("Characters: {}", token.chars().count()),
            format!("Uppercase: {}", token.to_uppercase()),
            format!("Lowercase: {}", token.to_lowercase()),
        ]);
    } else {
        lines.extend([
            "No symbol under the cursor yet.".to_owned(),
            "Move onto an identifier to inspect token details.".to_owned(),
        ]);
    }
    if let Some(span) = hover_syntax_span_at_cursor(buffer, token_info) {
        let capture_name = if span.capture_name.starts_with('@') {
            span.capture_name.to_string()
        } else {
            format!("@{}", span.capture_name)
        };
        lines.extend([
            format!("Theme color: {}", span.theme_token),
            format!("Tree-sitter token: {capture_name}"),
        ]);
    }
    lines
}

fn hover_syntax_span_at_cursor<'a>(
    buffer: &'a ShellBuffer,
    token_info: Option<&(TextRange, String)>,
) -> Option<&'a LineSyntaxSpan> {
    let point = token_info
        .map(|(range, _)| range.start())
        .unwrap_or_else(|| buffer.cursor_point());
    buffer
        .line_syntax_spans(point.line)?
        .iter()
        .filter(|span| point.column >= span.start && point.column < span.end)
        .min_by_key(|span| span.end.saturating_sub(span.start))
}

fn hover_empty_provider_lines(
    buffer: &ShellBuffer,
    token_info: Option<&(TextRange, String)>,
) -> Vec<String> {
    let mut lines = vec![
        format!("Buffer: {}", buffer.display_name()),
        format!(
            "Line: {}, Column: {}",
            buffer.cursor_row() + 1,
            buffer.cursor_col() + 1
        ),
    ];
    if let Some((_, token)) = token_info {
        lines.push(format!("No hover details are available for `{token}` yet."));
    } else {
        lines.push("No symbol is under the cursor.".to_owned());
    }
    lines.push(
        "Try moving onto an identifier or waiting for LSP/diagnostics to refresh.".to_owned(),
    );
    lines
}

fn hover_provider_fragments_empty(fragments: &[HoverProviderFragment]) -> bool {
    fragments.iter().all(|fragment| match fragment {
        HoverProviderFragment::PlainLines(lines) => lines.is_empty(),
        HoverProviderFragment::MarkdownText(text) => text.trim().is_empty(),
        HoverProviderFragment::SignatureHelpMarkdown { text, .. } => text.trim().is_empty(),
    })
}

fn finalize_hover_overlay(runtime: &mut EditorRuntime, draft: HoverOverlayDraft) -> HoverOverlay {
    let providers = draft
        .providers
        .into_iter()
        .map(|provider| finalize_hover_provider_content(runtime, provider))
        .collect::<Vec<_>>();
    HoverOverlay {
        buffer_id: draft.buffer_id,
        anchor: draft.anchor,
        token: draft.token,
        providers,
        provider_index: 0,
        scroll_offset: 0,
        focused: false,
        line_limit: draft.line_limit,
        pending_g_prefix: false,
        count: None,
    }
}

fn finalize_hover_provider_content(
    runtime: &mut EditorRuntime,
    draft: HoverProviderDraft,
) -> HoverProviderContent {
    let mut content = HoverRenderedContent::default();
    for fragment in draft.fragments {
        let rendered = match fragment {
            HoverProviderFragment::PlainLines(lines) => plain_hover_rendered_content(lines),
            HoverProviderFragment::MarkdownText(text) => {
                render_markdown_hover_content(runtime, &text)
            }
            HoverProviderFragment::SignatureHelpMarkdown {
                text,
                active_parameter,
            } => {
                let mut rendered = render_markdown_hover_content(runtime, &text);
                if let Some(active_parameter) = active_parameter {
                    apply_signature_active_parameter_emphasis(&mut rendered, &active_parameter);
                }
                rendered
            }
        };
        append_hover_rendered_content(&mut content, rendered);
    }
    HoverProviderContent {
        provider_label: draft.provider_label,
        provider_icon: draft.provider_icon,
        lines: content.lines,
        syntax_lines: content.syntax_lines,
    }
}

fn plain_hover_rendered_content(lines: Vec<String>) -> HoverRenderedContent {
    HoverRenderedContent {
        lines,
        syntax_lines: BTreeMap::new(),
    }
}

fn append_hover_rendered_content(
    content: &mut HoverRenderedContent,
    rendered: HoverRenderedContent,
) {
    let line_offset = content.lines.len();
    content.lines.extend(rendered.lines);
    for (line_index, spans) in rendered.syntax_lines {
        content.syntax_lines.insert(line_index + line_offset, spans);
    }
}

fn render_markdown_hover_content(runtime: &mut EditorRuntime, text: &str) -> HoverRenderedContent {
    let config = markdown_pretty::user_library_pretty_config(&*shell_user_library(runtime));
    let registry = syntax_registry_mut(runtime).ok();
    render_markdown_ephemeral_content(text, &config, Some(config.enabled), registry)
}

fn render_markdown_ephemeral_content(
    text: &str,
    config: &editor_markdown::MarkdownPrettyConfig,
    buffer_enabled: Option<bool>,
    mut registry: Option<&mut SyntaxRegistry>,
) -> HoverRenderedContent {
    let normalized = normalize_hover_multiline_text(text);
    let mut rendered = plain_hover_rendered_content(hover_multiline_lines(&normalized));
    if rendered.lines.is_empty() {
        return rendered;
    }
    let plan = editor_markdown::plan_markdown_pretty_ephemeral(
        &normalized,
        config,
        buffer_enabled,
        registry.as_deref_mut(),
    );
    if !plan.skipped_by_kill_switch {
        for (line_index, line) in rendered.lines.iter_mut().enumerate() {
            *line = markdown_pretty::pretty_display_line(&plan, false, line_index, line);
        }
    }
    let Some(registry) = registry else {
        return rendered;
    };
    let markdown_buffer = TextBuffer::from_text(&normalized);
    let mut parse_session = None;
    let (_, syntax_result) = compute_buffer_syntax(
        registry,
        None,
        &markdown_buffer,
        Some("markdown"),
        None,
        &mut parse_session,
    );
    if let Some(Ok(snapshot)) = syntax_result {
        rendered.syntax_lines = index_syntax_lines(snapshot, &markdown_buffer);
    }
    apply_markdown_code_fence_syntax(&mut rendered, registry);
    rendered
}

pub(super) fn rebuild_acp_output_markdown(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    follow_output: bool,
) -> Result<(), String> {
    let config = markdown_pretty::user_library_pretty_config(&*shell_user_library(runtime));
    let (items, visible_rows, buffer_enabled) = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let Some(state) = buffer.acp_state.as_ref() else {
            return Ok(());
        };
        (
            state.output_items.clone(),
            buffer.acp_output_viewport_lines(),
            buffer.markdown_pretty_enabled(),
        )
    };
    let render_lines = {
        let markdown = match syntax_registry_mut(runtime) {
            Ok(registry) => Some(AcpMarkdownPaint {
                registry,
                config: &config,
            }),
            Err(_) => None,
        };
        acp_build_output_lines(&items, markdown, buffer_enabled)
    };
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    if let Some(state) = buffer.acp_state.as_mut() {
        state
            .output_pane
            .replace_render_lines(render_lines, follow_output, visible_rows);
    }
    Ok(())
}

fn apply_markdown_code_fence_syntax(
    rendered: &mut HoverRenderedContent,
    registry: &mut SyntaxRegistry,
) {
    for block in markdown_code_fence_blocks(&rendered.lines) {
        let Some(language_id) =
            resolve_markdown_code_fence_language_id(registry, block.language.as_deref())
        else {
            continue;
        };
        let code_lines =
            rendered.lines[block.code_start_line..block.code_end_line_exclusive].to_vec();
        let code_text = code_lines.join("\n");
        let code_buffer = TextBuffer::from_text(&code_text);
        let mut parse_session = None;
        let (_, syntax_result) = compute_buffer_syntax(
            registry,
            None,
            &code_buffer,
            Some(&language_id),
            None,
            &mut parse_session,
        );
        let Some(Ok(snapshot)) = syntax_result else {
            continue;
        };
        for line_index in block.code_start_line..block.code_end_line_exclusive {
            rendered.syntax_lines.remove(&line_index);
        }
        for (line_index, spans) in index_syntax_lines(snapshot, &code_buffer) {
            rendered
                .syntax_lines
                .insert(block.code_start_line + line_index, spans);
        }
    }
}

pub(super) const HOVER_SIGNATURE_ACTIVE_PARAMETER_TOKEN: &str =
    "ui.hover.signature.active_parameter";

fn apply_signature_active_parameter_emphasis(
    rendered: &mut HoverRenderedContent,
    emphasis: &editor_lsp::LspSignatureActiveParameter,
) {
    if emphasis.start >= emphasis.end {
        return;
    }
    let blocks = markdown_code_fence_blocks(&rendered.lines);
    let Some(block) = blocks.get(emphasis.signature_index) else {
        return;
    };
    let (start_line, start_col) = char_offset_to_line_col(&emphasis.label, emphasis.start);
    let (end_line, end_col) = char_offset_to_line_col(&emphasis.label, emphasis.end);
    if start_line == end_line {
        insert_signature_active_parameter_span(
            rendered,
            block.code_start_line + start_line,
            start_col,
            end_col,
        );
        return;
    }
    let first_line = block.code_start_line + start_line;
    let first_line_len = rendered
        .lines
        .get(first_line)
        .map(|line| line.chars().count())
        .unwrap_or(0);
    insert_signature_active_parameter_span(rendered, first_line, start_col, first_line_len);
    for line in (start_line + 1)..end_line {
        let rendered_line = block.code_start_line + line;
        let line_len = rendered
            .lines
            .get(rendered_line)
            .map(|line| line.chars().count())
            .unwrap_or(0);
        if line_len > 0 {
            insert_signature_active_parameter_span(rendered, rendered_line, 0, line_len);
        }
    }
    if end_col > 0 {
        insert_signature_active_parameter_span(
            rendered,
            block.code_start_line + end_line,
            0,
            end_col,
        );
    }
}

fn char_offset_to_line_col(text: &str, char_offset: usize) -> (usize, usize) {
    let mut line = 0usize;
    let mut col = 0usize;
    for (index, character) in text.chars().enumerate() {
        if index >= char_offset {
            break;
        }
        if character == '\n' {
            line += 1;
            col = 0;
        } else {
            col += 1;
        }
    }
    (line, col)
}

fn insert_signature_active_parameter_span(
    rendered: &mut HoverRenderedContent,
    line_index: usize,
    start_col: usize,
    end_col: usize,
) {
    if start_col >= end_col {
        return;
    }
    rendered
        .syntax_lines
        .entry(line_index)
        .or_default()
        .push(LineSyntaxSpan {
            start: start_col,
            end: end_col,
            capture_name: Arc::from("signature.active_parameter"),
            theme_token: Arc::from(HOVER_SIGNATURE_ACTIVE_PARAMETER_TOKEN),
        });
}

fn resolve_markdown_code_fence_language_id(
    registry: &SyntaxRegistry,
    language: Option<&str>,
) -> Option<String> {
    let candidate = normalize_markdown_code_fence_language(language?)?;
    if registry.language(&candidate).is_some() {
        return Some(candidate);
    }
    registry
        .language_for_extension(&candidate)
        .map(|language| language.id().to_owned())
}

fn normalize_markdown_code_fence_language(language: &str) -> Option<String> {
    let token = language
        .trim()
        .trim_matches(|character| character == '{' || character == '}')
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .split(',')
        .next()
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_ascii_lowercase();
    (!token.is_empty()).then_some(token)
}

fn markdown_code_fence_blocks(lines: &[String]) -> Vec<MarkdownCodeFenceBlock> {
    let mut blocks = Vec::new();
    let mut open_fence: Option<(char, usize, usize, Option<String>)> = None;

    for (line_index, line) in lines.iter().enumerate() {
        let Some((marker, count, rest)) = parse_markdown_fence(line) else {
            continue;
        };
        if let Some((open_marker, open_count, open_line, language)) = &open_fence {
            if marker == *open_marker && count >= *open_count && rest.trim().is_empty() {
                let code_start_line = open_line.saturating_add(1);
                if code_start_line <= line_index {
                    blocks.push(MarkdownCodeFenceBlock {
                        language: language.clone(),
                        code_start_line,
                        code_end_line_exclusive: line_index,
                    });
                }
                open_fence = None;
            }
            continue;
        }
        open_fence = Some((
            marker,
            count,
            line_index,
            normalize_markdown_code_fence_language(&rest),
        ));
    }

    if let Some((_, _, open_line, language)) = open_fence {
        blocks.push(MarkdownCodeFenceBlock {
            language,
            code_start_line: open_line.saturating_add(1),
            code_end_line_exclusive: lines.len(),
        });
    }

    blocks
}

fn parse_markdown_fence(line: &str) -> Option<(char, usize, String)> {
    let trimmed = line.trim_start();
    let marker = trimmed.chars().next()?;
    if marker != '`' && marker != '~' {
        return None;
    }
    let count = trimmed
        .chars()
        .take_while(|character| *character == marker)
        .count();
    if count < 3 {
        return None;
    }
    Some((
        marker,
        count,
        trimmed.get(count..).unwrap_or_default().trim().to_owned(),
    ))
}

fn hover_multiline_lines(text: &str) -> Vec<String> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut lines = text
        .split('\n')
        .map(str::trim_end)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines
}

fn normalize_hover_multiline_text(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn hover_manual_provider_lines(buffer: &ShellBuffer, provider: &HoverProviderSpec) -> Vec<String> {
    let plugin_kind = match &buffer.kind {
        BufferKind::Plugin(kind) => Some(kind.as_str()),
        _ => None,
    };
    if provider.buffer_kind.as_deref() != plugin_kind {
        return Vec::new();
    }
    completion_token_at_cursor(buffer)
        .and_then(|(_, token)| {
            provider
                .topics
                .iter()
                .find(|topic| topic.token == token)
                .map(|topic| topic.lines.clone())
        })
        .unwrap_or_default()
}

fn hover_lsp_provider_fragments(
    buffer: &ShellBuffer,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
) -> Vec<HoverProviderFragment> {
    let hovers = synced_hover_lsp_request(buffer, lsp_client, lsp_context, LspClientManager::hover);
    let show_server_labels = hovers.len() > 1;
    let mut fragments = Vec::new();
    for hover in hovers {
        if show_server_labels {
            fragments.push(HoverProviderFragment::PlainLines(vec![format!(
                "{} {}",
                editor_icons::symbols::cod::COD_INFO,
                hover.server_id()
            )]));
        }
        if hover.is_markdown() {
            fragments.push(HoverProviderFragment::MarkdownText(hover.text().to_owned()));
        } else {
            fragments.push(HoverProviderFragment::PlainLines(hover.lines().to_vec()));
        }
    }
    fragments
}

fn hover_signature_provider_fragments(
    buffer: &ShellBuffer,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
) -> Vec<HoverProviderFragment> {
    let signatures = synced_hover_lsp_request_at_point(
        lsp_client,
        lsp_context,
        hover_signature_request_point(buffer),
        LspClientManager::signature_help,
    );
    let language = buffer.language_id();
    let show_server_labels = signatures.len() > 1;
    let mut fragments = Vec::new();
    for signature in signatures {
        if show_server_labels {
            fragments.push(HoverProviderFragment::PlainLines(vec![format!(
                "{} {}",
                editor_icons::symbols::md::MD_SIGNATURE,
                signature.server_id()
            )]));
        }
        let text = signature.markdown_text(language);
        if !text.trim().is_empty() {
            fragments.push(HoverProviderFragment::SignatureHelpMarkdown {
                text,
                active_parameter: signature.active_parameter_range(),
            });
        }
    }
    fragments
}

fn synced_hover_lsp_request<T>(
    buffer: &ShellBuffer,
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
    request: fn(&LspClientManager, &Path, TextPoint) -> Result<Vec<T>, LspClientError>,
) -> Vec<T> {
    synced_hover_lsp_request_at_point(lsp_client, lsp_context, buffer.cursor_point(), request)
}

fn synced_hover_lsp_request_at_point<T>(
    lsp_client: Option<&Arc<LspClientManager>>,
    lsp_context: Option<&ActiveLspBufferContext>,
    position: TextPoint,
    request: fn(&LspClientManager, &Path, TextPoint) -> Result<Vec<T>, LspClientError>,
) -> Vec<T> {
    let Some(lsp_client) = lsp_client else {
        return Vec::new();
    };
    let Some(context) = lsp_context else {
        return Vec::new();
    };
    lsp_client
        .sync_buffer(
            &context.path,
            &context.text,
            context.revision,
            context.root.as_deref(),
        )
        .ok()
        .and_then(|_| request(lsp_client, &context.path, position).ok())
        .unwrap_or_default()
}

fn hover_diagnostic_provider_fragments(
    buffer: &ShellBuffer,
    user_library: &dyn UserLibrary,
) -> Vec<HoverProviderFragment> {
    let cursor = buffer.cursor_point();
    let diagnostic_icon = user_library.lsp_diagnostic_icon();
    let diagnostic_line_limit = user_library.lsp_diagnostic_line_limit();
    let matching = buffer
        .lsp_diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic_matches_cursor_line(diagnostic, cursor))
        .take(diagnostic_line_limit)
        .flat_map(|diagnostic| {
            hover_diagnostic_fragments_for_diagnostic(diagnostic, diagnostic_icon, true)
        })
        .collect::<Vec<_>>();
    if !matching.is_empty() {
        return matching;
    }
    buffer
        .lsp_diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.range().start().line == cursor.line)
        .take(diagnostic_line_limit)
        .flat_map(|diagnostic| {
            hover_diagnostic_fragments_for_diagnostic(diagnostic, diagnostic_icon, false)
        })
        .collect()
}

fn hover_diagnostic_fragments_for_diagnostic(
    diagnostic: &LspDiagnostic,
    diagnostic_icon: &str,
    include_source_in_plain: bool,
) -> Vec<HoverProviderFragment> {
    if !markdown_code_fence_blocks(&hover_multiline_lines(diagnostic.message())).is_empty() {
        let source = diagnostic.source();
        let header = if source.is_empty() {
            format!("{diagnostic_icon} Diagnostic")
        } else {
            format!("{diagnostic_icon} {source}")
        };
        return vec![
            HoverProviderFragment::PlainLines(vec![header]),
            HoverProviderFragment::MarkdownText(diagnostic.message().to_owned()),
        ];
    }

    let source = diagnostic.source();
    let line = if include_source_in_plain && !source.is_empty() {
        format!("{diagnostic_icon} {} ({source})", diagnostic.message())
    } else {
        format!("{diagnostic_icon} {}", diagnostic.message())
    };
    vec![HoverProviderFragment::PlainLines(vec![line])]
}

fn diagnostic_matches_cursor_line(diagnostic: &LspDiagnostic, cursor: TextPoint) -> bool {
    let range = diagnostic.range().normalized();
    if cursor.line < range.start().line || cursor.line > range.end().line {
        return false;
    }
    if range.start().line == range.end().line {
        return cursor.column >= range.start().column && cursor.column <= range.end().column;
    }
    if cursor.line == range.start().line {
        return cursor.column >= range.start().column;
    }
    if cursor.line == range.end().line {
        return cursor.column <= range.end().column;
    }
    true
}
