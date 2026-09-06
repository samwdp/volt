impl ShellBuffer {
    fn image_state(&self) -> Option<&ImageBufferState> {
        self.image_state.as_ref()
    }

    fn image_state_mut(&mut self) -> Option<&mut ImageBufferState> {
        self.image_state.as_mut()
    }

    fn pdf_state(&self) -> Option<&PdfBufferState> {
        self.pdf_state.as_ref()
    }

    fn pdf_state_mut(&mut self) -> Option<&mut PdfBufferState> {
        self.pdf_state.as_mut()
    }

    fn is_pdf_buffer(&self) -> bool {
        self.pdf_state.is_some()
            || matches!(&self.kind, BufferKind::Plugin(kind) if kind == PDF_BUFFER_KIND)
    }

    fn pdf_preview_url(&self) -> Option<String> {
        self.pdf_state()
            .and_then(|state| state.preview_url.as_ref().cloned())
    }

    fn has_pdf_preview_surface(&self) -> bool {
        self.pdf_state()
            .is_some_and(|state| state.preview_url.is_some())
    }

    fn uses_browser_host_surface(&self) -> bool {
        buffer_is_browser(&self.kind) || self.has_pdf_preview_surface()
    }

    fn is_rendered_image_buffer(&self) -> bool {
        self.image_state()
            .is_some_and(|state| state.mode == ImageBufferMode::Rendered)
    }

    fn is_svg_source_mode(&self) -> bool {
        self.image_state().is_some_and(|state| {
            state.format == ImageBufferFormat::Svg && state.mode == ImageBufferMode::Source
        })
    }

    fn supports_text_file_actions(&self) -> bool {
        self.kind == BufferKind::File || self.is_svg_source_mode() || buffer_is_db_query(&self.kind)
    }

    fn set_image_state(&mut self, state: ImageBufferState) {
        self.image_state = Some(state);
    }

    fn set_pdf_state(&mut self, state: PdfBufferState) {
        self.pdf_state = Some(state);
    }

    fn image_zoom_in(&mut self) -> bool {
        let Some(state) = self.image_state_mut() else {
            return false;
        };
        if state.mode != ImageBufferMode::Rendered {
            return false;
        }
        let next = (state.zoom * IMAGE_ZOOM_STEP).clamp(IMAGE_ZOOM_MIN, IMAGE_ZOOM_MAX);
        if (next - state.zoom).abs() < f32::EPSILON {
            return false;
        }
        state.zoom = next;
        let zoom = state.zoom;
        if let Some(pdf_state) = self.pdf_state_mut() {
            pdf_state.zoom_percent = pdf_zoom_percent_from_scale(zoom);
        }
        true
    }

    fn image_zoom_out(&mut self) -> bool {
        let Some(state) = self.image_state_mut() else {
            return false;
        };
        if state.mode != ImageBufferMode::Rendered {
            return false;
        }
        let next = (state.zoom / IMAGE_ZOOM_STEP).clamp(IMAGE_ZOOM_MIN, IMAGE_ZOOM_MAX);
        if (next - state.zoom).abs() < f32::EPSILON {
            return false;
        }
        state.zoom = next;
        let zoom = state.zoom;
        if let Some(pdf_state) = self.pdf_state_mut() {
            pdf_state.zoom_percent = pdf_zoom_percent_from_scale(zoom);
        }
        true
    }

    fn reset_image_zoom(&mut self) -> bool {
        let Some(state) = self.image_state_mut() else {
            return false;
        };
        if state.mode != ImageBufferMode::Rendered || (state.zoom - 1.0).abs() < f32::EPSILON {
            return false;
        }
        state.zoom = 1.0;
        if let Some(pdf_state) = self.pdf_state_mut() {
            pdf_state.zoom_percent = 100;
        }
        true
    }

    fn refresh_pdf_text_snapshot(&mut self) {
        let display_name = self.display_name().to_owned();
        let path = self.path().map(Path::to_path_buf);
        let (lines, language_id, anchor_line) = {
            let Some(state) = self.pdf_state() else {
                return;
            };
            let lines = pdf_buffer_lines(display_name.as_str(), path.as_deref(), state);
            let language_id = pdf_language_id(state.open_mode);
            let anchor_line =
                pdf_navigation_anchor_line(&lines, state.open_mode, state.current_page);
            (lines, language_id, anchor_line)
        };
        self.replace_with_lines_preserve_view(lines);
        self.set_language_id(language_id);
        if let Some(path) = path {
            self.text.set_path(path);
        }
        self.text.mark_clean();
        if let Some(anchor_line) = anchor_line {
            let line = anchor_line.min(self.line_count().saturating_sub(1));
            self.text.set_cursor(TextPoint::new(line, 0));
            self.scroll_row = line;
        }
    }

    fn apply_pdf_location_update(&mut self, url: &str) {
        if let Some(state) = self.pdf_state_mut() {
            state.preview_url = Some(url.to_owned());
            if let Some(page) = pdf_preview_page_from_url(url) {
                state.current_page = page.clamp(1, state.page_count().max(1));
            }
        }
        self.refresh_pdf_text_snapshot();
    }

    fn refresh_pdf_view(&mut self, write_preview_file: bool) {
        let buffer_id = self.id;
        let zoom = self
            .image_state()
            .map(|state| state.zoom)
            .or_else(|| {
                self.pdf_state()
                    .map(|state| pdf_zoom_scale(state.zoom_percent))
            })
            .unwrap_or(1.0);
        let rendered = {
            let Some(state) = self.pdf_state_mut() else {
                return;
            };
            state.zoom_percent = pdf_zoom_percent_from_scale(zoom);
            match state.open_mode {
                PdfOpenMode::Rendered => {
                    match render_pdf_page_image(state, buffer_id, write_preview_file) {
                        Ok(decoded) => {
                            state.render_error = None;
                            Some(decoded)
                        }
                        Err(error) => {
                            state.render_error = Some(error);
                            None
                        }
                    }
                }
                PdfOpenMode::Markdown | PdfOpenMode::Latex => {
                    state.render_error = None;
                    None
                }
            }
        };
        self.image_state = rendered.map(|decoded| ImageBufferState {
            format: ImageBufferFormat::Raster,
            mode: ImageBufferMode::Rendered,
            decoded,
            zoom,
        });
        self.refresh_pdf_text_snapshot();
    }

    fn pdf_next_page(&mut self) -> Result<bool, String> {
        let Some(state) = self.pdf_state_mut() else {
            return Ok(false);
        };
        let page_count = state.page_count();
        if state.current_page >= page_count {
            return Ok(false);
        }
        state.current_page += 1;
        self.refresh_pdf_view(false);
        Ok(true)
    }

    fn pdf_previous_page(&mut self) -> Result<bool, String> {
        let Some(state) = self.pdf_state_mut() else {
            return Ok(false);
        };
        if state.current_page <= 1 {
            return Ok(false);
        }
        state.current_page -= 1;
        self.refresh_pdf_view(false);
        Ok(true)
    }

    fn pdf_rotate_clockwise(&mut self) -> Result<bool, String> {
        let Some(state) = self.pdf_state_mut() else {
            return Ok(false);
        };
        let page_id = state
            .document
            .get_pages()
            .get(&state.current_page)
            .copied()
            .ok_or_else(|| format!("missing page {}", state.current_page))?;
        let page = state
            .document
            .get_dictionary_mut(page_id)
            .map_err(|error| error.to_string())?;
        let next_rotation = page
            .get(b"Rotate")
            .ok()
            .and_then(|rotation| rotation.as_i64().ok())
            .unwrap_or(0)
            + 90;
        page.set("Rotate", next_rotation.rem_euclid(PDF_ROTATION_FULL_CIRCLE));
        state.dirty = true;
        self.refresh_pdf_view(true);
        Ok(true)
    }

    fn pdf_delete_current_page(&mut self) -> Result<bool, String> {
        let Some(state) = self.pdf_state_mut() else {
            return Ok(false);
        };
        if state.page_count() <= 1 {
            return Err("cannot delete the last remaining PDF page".to_owned());
        }
        state.document.delete_pages(&[state.current_page]);
        state.document.prune_objects();
        if state.current_page > state.page_count() {
            state.current_page = state.page_count().max(1);
        }
        state.metadata.page_count = state.page_count();
        state.dirty = true;
        self.refresh_pdf_view(true);
        Ok(true)
    }

    fn toggle_svg_image_mode(&mut self) -> Result<bool, String> {
        let Some(state) = self.image_state.as_mut() else {
            return Ok(false);
        };
        if state.format != ImageBufferFormat::Svg {
            return Ok(false);
        }
        match state.mode {
            ImageBufferMode::Rendered => {
                state.mode = ImageBufferMode::Source;
                Ok(true)
            }
            ImageBufferMode::Source => {
                let path = self.text.path().map(Path::to_path_buf);
                let decoded = rasterize_svg_text(&self.text.text(), path.as_deref())?;
                state.decoded = decoded;
                state.mode = ImageBufferMode::Rendered;
                Ok(true)
            }
        }
    }

    fn set_plugin_output_lines(&mut self, lines: Vec<String>) {
        let Some((target_section, base_update)) = self
            .plugin_section_state
            .as_ref()
            .map(|state| (state.evaluate_target_section, state.base_update))
        else {
            return;
        };
        if target_section == 0 {
            let follow_output = self.should_follow_output();
            match base_update {
                PluginBufferSectionUpdate::Replace => self.replace_with_lines(lines),
                PluginBufferSectionUpdate::Append => self.append_output_lines(&lines),
            }
            if follow_output {
                self.scroll_output_to_end();
            }
            return;
        }
        let Some(state) = self.plugin_section_state.as_mut() else {
            return;
        };
        let Some(pane) = state.attached_section_mut(target_section) else {
            return;
        };
        let follow_output = pane.should_follow_output();
        match pane.update {
            PluginBufferSectionUpdate::Replace => pane.replace_lines(lines, follow_output),
            PluginBufferSectionUpdate::Append => pane.append_lines(lines, follow_output),
        }
    }
}
