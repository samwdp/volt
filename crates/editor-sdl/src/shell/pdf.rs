use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PdfFitMode {
    Page,
}

#[derive(Debug, Clone)]
pub(super) struct PdfBufferState {
    pub(super) document: PdfDocument,
    pub(super) metadata: PdfMetadata,
    pub(super) current_page: u32,
    pub(super) fit_mode: PdfFitMode,
    pub(super) zoom_percent: u16,
    pub(super) dirty: bool,
}

impl PdfBufferState {
    pub(super) fn new(document: PdfDocument, metadata: PdfMetadata) -> Self {
        Self {
            document,
            metadata,
            current_page: 1,
            fit_mode: PdfFitMode::Page,
            zoom_percent: 100,
            dirty: false,
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
}

pub(super) fn pdf_previous_page(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.pdf_previous_page();
    Ok(())
}

pub(super) fn pdf_next_page(runtime: &mut EditorRuntime) -> Result<(), String> {
    active_shell_buffer_mut(runtime)?.pdf_next_page();
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

pub(super) fn pdf_buffer_lines(
    display_name: &str,
    path: Option<&Path>,
    state: &PdfBufferState,
) -> Vec<String> {
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
        String::new(),
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
    ];
    let text = pdf_page_text(&state.document, current_page);
    if text.is_empty() {
        lines.push("(no extractable text on the current page)".to_owned());
    } else {
        lines.extend(text.lines().map(str::to_owned));
    }
    lines
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
    let mut text =
        TextBuffer::from_text(pdf_buffer_lines(display_name, Some(path), &pdf_state).join("\n"));
    text.set_path(path.to_path_buf());
    text.mark_clean();
    let mut shell_buffer = ShellBuffer::from_text_buffer(buffer, text, &*user_library);
    pdf_state.clamp_current_page();
    shell_buffer.set_pdf_state(pdf_state);
    shell_buffer.refresh_pdf_preview();

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
