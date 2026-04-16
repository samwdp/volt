use super::*;
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PdfFitMode {
    Page,
}

const PDF_RENDER_PROGRAM: &str = "pdftocairo";
const PDF_MARKDOWN_LANGUAGE_ID: &str = "markdown";
const PDF_LATEX_LANGUAGE_ID: &str = "latex";

#[derive(Debug, Clone)]
pub(super) struct PdfBufferState {
    pub(super) document: PdfDocument,
    pub(super) metadata: PdfMetadata,
    pub(super) current_page: u32,
    pub(super) fit_mode: PdfFitMode,
    pub(super) zoom_percent: u16,
    pub(super) open_mode: PdfOpenMode,
    pub(super) render_error: Option<String>,
    pub(super) dirty: bool,
    pub(super) preview_revision: u64,
    pub(super) preview_path: Option<PathBuf>,
    pub(super) preview_url: Option<String>,
}

impl PdfBufferState {
    pub(super) fn new(document: PdfDocument, metadata: PdfMetadata) -> Self {
        Self {
            document,
            metadata,
            current_page: 1,
            fit_mode: PdfFitMode::Page,
            zoom_percent: 100,
            open_mode: PdfOpenMode::Rendered,
            render_error: None,
            dirty: false,
            preview_revision: 0,
            preview_path: None,
            preview_url: None,
        }
    }

    pub(super) fn page_count(&self) -> u32 {
        self.document.get_pages().len() as u32
    }

    pub(super) fn clamp_current_page(&mut self) {
        let page_count = self.page_count().max(1);
        self.current_page = self.current_page.clamp(1, page_count);
        self.metadata.page_count = page_count;
    }

    pub(super) fn sync_preview(
        &mut self,
        buffer_id: BufferId,
        write_document: bool,
    ) -> Result<(), String> {
        if write_document || self.preview_path.is_none() {
            let preview_dir = pdf_preview_dir();
            fs::create_dir_all(&preview_dir).map_err(|error| {
                format!(
                    "failed to create PDF preview directory `{}`: {error}",
                    preview_dir.display()
                )
            })?;
            let next_revision = self.preview_revision.saturating_add(1);
            let preview_path = pdf_preview_file_path(buffer_id, next_revision);
            self.document.save(&preview_path).map_err(|error| {
                format!(
                    "failed to write PDF preview `{}`: {error}",
                    preview_path.display()
                )
            })?;
            let previous_path = self.preview_path.replace(preview_path.clone());
            self.preview_revision = next_revision;
            if let Some(previous_path) = previous_path
                && previous_path != preview_path
            {
                let _ = fs::remove_file(previous_path);
            }
        }
        self.preview_url = None;
        Ok(())
    }
}

pub(super) fn pdf_previous_page(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.pdf_previous_page()?;
    Ok(())
}

pub(super) fn pdf_next_page(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.pdf_next_page()?;
    Ok(())
}

pub(super) fn pdf_rotate_clockwise(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.pdf_rotate_clockwise()?;
    Ok(())
}

pub(super) fn pdf_delete_page(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.pdf_delete_current_page()?;
    Ok(())
}

/// Returns whether a path has a case-insensitive `.pdf` extension.
pub(super) fn is_pdf_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

pub(super) fn load_pdf_buffer_state(path: &Path) -> Result<PdfBufferState, String> {
    let document = PdfDocument::load(path)
        .map_err(|error| format!("failed to open `{}`: {error}", path.display()))?;
    let metadata = PdfDocument::load_metadata(path).map_err(|error| {
        format!(
            "failed to read PDF metadata for `{}`: {error}",
            path.display()
        )
    })?;
    let mut state = PdfBufferState::new(document, metadata);
    state.clamp_current_page();
    Ok(state)
}

fn pdf_preview_dir() -> PathBuf {
    env::temp_dir().join("volt").join("pdf-preview")
}

fn pdf_preview_file_path(buffer_id: BufferId, revision: u64) -> PathBuf {
    pdf_preview_dir().join(format!(
        "buffer-{}-{}-{revision}.pdf",
        std::process::id(),
        buffer_id
    ))
}

pub(super) fn pdf_preview_page_from_url(url: &str) -> Option<u32> {
    Url::parse(url)
        .ok()?
        .fragment()?
        .split('&')
        .find_map(|part| part.strip_prefix("page=")?.parse::<u32>().ok())
        .filter(|page| *page > 0)
}

pub(super) fn pdf_language_id(mode: PdfOpenMode) -> Option<String> {
    match mode {
        PdfOpenMode::Rendered => None,
        PdfOpenMode::Markdown => Some(PDF_MARKDOWN_LANGUAGE_ID.to_owned()),
        PdfOpenMode::Latex => Some(PDF_LATEX_LANGUAGE_ID.to_owned()),
    }
}

pub(super) fn pdf_zoom_scale(zoom_percent: u16) -> f32 {
    f32::from(zoom_percent.max(1)) / 100.0
}

pub(super) fn pdf_zoom_percent_from_scale(scale: f32) -> u16 {
    (scale * 100.0).round().clamp(1.0, f32::from(u16::MAX)) as u16
}

pub(super) fn pdf_navigation_anchor_line(
    lines: &[String],
    mode: PdfOpenMode,
    current_page: u32,
) -> Option<usize> {
    let anchor = match mode {
        PdfOpenMode::Rendered => return None,
        PdfOpenMode::Markdown => format!("## Page {current_page}"),
        PdfOpenMode::Latex => format!(r"\subsection*{{Page {current_page}}}"),
    };
    lines.iter().position(|line| line == &anchor)
}

pub(super) fn render_pdf_page_image(
    state: &mut PdfBufferState,
    buffer_id: BufferId,
    write_preview_file: bool,
) -> Result<DecodedImage, String> {
    state.sync_preview(buffer_id, write_preview_file)?;
    let preview_path = state
        .preview_path
        .as_deref()
        .ok_or_else(|| "missing PDF preview source".to_owned())?;
    render_pdf_page_with_pdftocairo(
        preview_path,
        buffer_id,
        state.preview_revision,
        state.current_page,
    )
}

pub(super) fn pdf_fit_mode_label(mode: PdfFitMode) -> &'static str {
    match mode {
        PdfFitMode::Page => "fit page",
    }
}

pub(super) fn pdf_inherited_page_value<'a>(
    document: &'a PdfDocument,
    page_number: u32,
    key: &[u8],
) -> Option<&'a lopdf::Object> {
    let mut page_id = document.get_pages().get(&page_number).copied()?;
    loop {
        let page = document.get_dictionary(page_id).ok()?;
        if let Ok(value) = page.get(key) {
            return Some(value);
        }
        page_id = page.get(b"Parent").ok()?.as_reference().ok()?;
    }
}

pub(super) fn pdf_page_rotation(document: &PdfDocument, page_number: u32) -> Option<i64> {
    pdf_inherited_page_value(document, page_number, b"Rotate")
        .and_then(|rotation| rotation.as_i64().ok())
}

pub(super) fn pdf_page_media_box(document: &PdfDocument, page_number: u32) -> Option<String> {
    let media_box = pdf_inherited_page_value(document, page_number, b"MediaBox")
        .and_then(|value| value.as_array().ok())?;
    let numbers = media_box
        .iter()
        .map(|value| {
            value
                .as_float()
                .map(|number| format!("{number:.0}"))
                .or_else(|_| value.as_i64().map(|number| number.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    Some(numbers.join(" "))
}

/// Extracts trimmed text for one PDF page, returning an empty string on failure.
pub(super) fn pdf_page_text(document: &PdfDocument, page_number: u32) -> String {
    document
        .extract_text(&[page_number])
        .map(|text| text.trim().to_owned())
        .unwrap_or_default()
}

fn pdf_render_error_message(error: &io::Error) -> String {
    if error.kind() == io::ErrorKind::NotFound {
        return format!(
            "rendered PDF mode requires `{PDF_RENDER_PROGRAM}` on PATH; install Poppler or set `user::pdf::OPEN_MODE` to PdfOpenMode::Markdown or PdfOpenMode::Latex"
        );
    }
    format!("failed to start `{PDF_RENDER_PROGRAM}`: {error}")
}

fn render_pdf_page_with_pdftocairo(
    preview_path: &Path,
    buffer_id: BufferId,
    revision: u64,
    current_page: u32,
) -> Result<DecodedImage, String> {
    let render_dir = pdf_preview_dir();
    fs::create_dir_all(&render_dir).map_err(|error| {
        format!(
            "failed to create PDF render directory `{}`: {error}",
            render_dir.display()
        )
    })?;
    let render_base = render_dir.join(format!(
        "buffer-{}-{buffer_id}-{revision}-page-{current_page}",
        std::process::id()
    ));
    let render_path = render_base.with_extension("png");
    if render_path.exists() {
        let _ = fs::remove_file(&render_path);
    }
    let output = Command::new(PDF_RENDER_PROGRAM)
        .arg("-png")
        .arg("-singlefile")
        .arg("-f")
        .arg(current_page.to_string())
        .arg("-l")
        .arg(current_page.to_string())
        .arg(preview_path)
        .arg(&render_base)
        .output()
        .map_err(|error| pdf_render_error_message(&error))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let detail = if stderr.is_empty() {
            format!("`{PDF_RENDER_PROGRAM}` exited with {}", output.status)
        } else {
            stderr
        };
        return Err(format!(
            "failed to render page {current_page} from `{}`: {detail}",
            preview_path.display()
        ));
    }
    let decoded = decode_raster_image_path(&render_path)?;
    let _ = fs::remove_file(render_path);
    Ok(decoded)
}

fn pdf_header_lines(
    display_name: &str,
    path: Option<&Path>,
    state: &PdfBufferState,
) -> (u32, u32, String, i64, String, Vec<String>) {
    let page_count = state.page_count().max(1);
    let current_page = state.current_page.min(page_count);
    let path_label = path
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| display_name.to_owned());
    let page_rotation = pdf_page_rotation(&state.document, current_page).unwrap_or(0);
    let page_media_box =
        pdf_page_media_box(&state.document, current_page).unwrap_or_else(|| "unknown".to_owned());
    let mut lines = vec![
        format!("{display_name} is a native PDF buffer."),
        format!("File: {path_label}"),
        format!(
            "Page {current_page}/{page_count} · {} · {}% · rotation {}°",
            pdf_fit_mode_label(state.fit_mode),
            state.zoom_percent,
            page_rotation.rem_euclid(PDF_ROTATION_FULL_CIRCLE)
        ),
        format!(
            "Modified: {} · PDF version {}",
            if state.dirty { "yes" } else { "no" },
            state.metadata.version
        ),
        "Commands: PageUp previous · PageDown next · Ctrl+r rotate · D delete · S save".to_owned(),
    ];
    if let Some(error) = state.render_error.as_deref() {
        lines.push(format!("Rendered preview unavailable: {error}"));
    }
    lines.push(String::new());
    (
        current_page,
        page_count,
        path_label,
        page_rotation,
        page_media_box,
        lines,
    )
}

fn pdf_rendered_lines(
    display_name: &str,
    path: Option<&Path>,
    state: &PdfBufferState,
) -> Vec<String> {
    let (current_page, _page_count, _path_label, _page_rotation, page_media_box, mut lines) =
        pdf_header_lines(display_name, path, state);
    lines.extend([
        "Document metadata:".to_owned(),
        format!(
            "Title: {}",
            state.metadata.title.as_deref().unwrap_or("(none)")
        ),
        format!(
            "Author: {}",
            state.metadata.author.as_deref().unwrap_or("(none)")
        ),
        format!(
            "Creator: {}",
            state.metadata.creator.as_deref().unwrap_or("(none)")
        ),
        format!(
            "Producer: {}",
            state.metadata.producer.as_deref().unwrap_or("(none)")
        ),
        format!("MediaBox: {page_media_box}"),
        String::new(),
        "Current page text:".to_owned(),
    ]);
    let text = pdf_page_text(&state.document, current_page);
    if text.is_empty() {
        lines.push("(no extractable text on the current page)".to_owned());
    } else {
        lines.extend(text.lines().map(str::to_owned));
    }
    lines
}

fn pdf_markdown_lines(
    display_name: &str,
    path: Option<&Path>,
    state: &PdfBufferState,
) -> Vec<String> {
    let (current_page, page_count, path_label, _page_rotation, _page_media_box, _header) =
        pdf_header_lines(display_name, path, state);
    let mut lines = vec![
        format!("# {display_name}"),
        String::new(),
        format!("- File: `{path_label}`"),
        format!("- Current page: {current_page}/{page_count}"),
        format!("- Modified: {}", if state.dirty { "yes" } else { "no" }),
        format!("- PDF version: {}", state.metadata.version),
        String::new(),
    ];
    for page_number in 1..=page_count {
        lines.push(format!("## Page {page_number}"));
        lines.push(String::new());
        let text = pdf_page_text(&state.document, page_number);
        if text.is_empty() {
            lines.push("_No extractable text on this page._".to_owned());
        } else {
            lines.extend(text.lines().map(str::to_owned));
        }
        lines.push(String::new());
    }
    lines
}

fn latex_escape_text(text: &str) -> String {
    let mut escaped = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => escaped.push_str(r"\textbackslash{}"),
            '{' => escaped.push_str(r"\{"),
            '}' => escaped.push_str(r"\}"),
            '#' => escaped.push_str(r"\#"),
            '$' => escaped.push_str(r"\$"),
            '%' => escaped.push_str(r"\%"),
            '&' => escaped.push_str(r"\&"),
            '_' => escaped.push_str(r"\_"),
            '^' => escaped.push_str(r"\^{}"),
            '~' => escaped.push_str(r"\~{}"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn pdf_latex_lines(display_name: &str, path: Option<&Path>, state: &PdfBufferState) -> Vec<String> {
    let (current_page, page_count, path_label, _page_rotation, _page_media_box, _header) =
        pdf_header_lines(display_name, path, state);
    let mut lines = vec![
        format!(r"\section*{{{}}}", latex_escape_text(display_name)),
        format!(
            r"\textbf{{File}}: \texttt{{{}}}\\",
            latex_escape_text(&path_label)
        ),
        format!(r"\textbf{{Current page}}: {current_page}/{page_count}\\"),
        format!(
            r"\textbf{{Modified}}: {}\\",
            if state.dirty { "yes" } else { "no" }
        ),
        format!(
            r"\textbf{{PDF version}}: {}",
            latex_escape_text(&state.metadata.version)
        ),
        String::new(),
    ];
    for page_number in 1..=page_count {
        lines.push(format!(r"\subsection*{{Page {page_number}}}"));
        let text = pdf_page_text(&state.document, page_number);
        if text.is_empty() {
            lines.push(r"\emph{No extractable text on this page.}".to_owned());
        } else {
            lines.extend(text.lines().map(latex_escape_text));
        }
        lines.push(String::new());
    }
    lines
}

pub(super) fn pdf_buffer_lines(
    display_name: &str,
    path: Option<&Path>,
    state: &PdfBufferState,
) -> Vec<String> {
    match state.open_mode {
        PdfOpenMode::Rendered => pdf_rendered_lines(display_name, path, state),
        PdfOpenMode::Markdown => pdf_markdown_lines(display_name, path, state),
        PdfOpenMode::Latex => pdf_latex_lines(display_name, path, state),
    }
}

pub(super) fn open_pdf_workspace_file(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    display_name: &str,
    path: &Path,
) -> Result<BufferId, String> {
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            display_name,
            BufferKind::Plugin(PDF_BUFFER_KIND.to_owned()),
            Some(path.to_path_buf()),
        )
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("new pdf buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let mut pdf_state = load_pdf_buffer_state(path)?;
    pdf_state.open_mode = user_library.pdf_open_mode();
    let mut text =
        TextBuffer::from_text(pdf_buffer_lines(display_name, Some(path), &pdf_state).join("\n"));
    text.set_path(path.to_path_buf());
    text.mark_clean();
    let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
    pdf_state.clamp_current_page();
    shell_buffer.set_language_id(pdf_language_id(pdf_state.open_mode));
    shell_buffer.set_pdf_state(pdf_state);
    shell_buffer.refresh_pdf_view(true);

    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
    }

    if let Some(detail) = file_open_detail(path) {
        runtime
            .emit_hook(
                builtins::FILE_OPEN,
                HookEvent::new()
                    .with_workspace(workspace_id)
                    .with_buffer(buffer_id)
                    .with_detail(detail),
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(buffer_id)
}
