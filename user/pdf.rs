use editor_plugin_api::{
    PdfOpenMode, PluginAction, PluginBuffer, PluginCommand, PluginKeyBinding, PluginKeymapScope,
    PluginPackage, buffer_kinds, pdf_hooks,
};

pub const PDF_BUFFER_KIND: &str = buffer_kinds::PDF;
// pub const OPEN_MODE: PdfOpenMode = PdfOpenMode::Rendered;
pub const OPEN_MODE: PdfOpenMode = PdfOpenMode::Latex;
// pub const OPEN_MODE: PdfOpenMode = PdfOpenMode::Markdown;

/// Returns the preferred mode for newly opened PDF buffers.
pub fn open_mode() -> PdfOpenMode {
    OPEN_MODE
}

/// Returns the metadata for native PDF commands.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "pdf",
        true,
        "Native PDF buffer navigation and structural editing commands.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "pdf.next-page",
            "Moves the active PDF buffer to the next page.",
            vec![PluginAction::emit_hook(pdf_hooks::NEXT_PAGE, None::<&str>)],
        ),
        PluginCommand::new(
            "pdf.previous-page",
            "Moves the active PDF buffer to the previous page.",
            vec![PluginAction::emit_hook(
                pdf_hooks::PREVIOUS_PAGE,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "pdf.rotate-clockwise",
            "Rotates the current PDF page clockwise.",
            vec![PluginAction::emit_hook(
                pdf_hooks::ROTATE_CLOCKWISE,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "pdf.delete-page",
            "Deletes the current page from the active PDF buffer.",
            vec![PluginAction::emit_hook(
                pdf_hooks::DELETE_PAGE,
                None::<&str>,
            )],
        ),
    ])
    .with_buffers(vec![
        // Native PDF buffers render their live contents in the host, so they do not
        // need package-defined initial lines here.
        PluginBuffer::new(PDF_BUFFER_KIND, Vec::<String>::new()).with_key_bindings(vec![
            PluginKeyBinding::new("PageDown", "pdf.next-page", PluginKeymapScope::Workspace),
            PluginKeyBinding::new("PageUp", "pdf.previous-page", PluginKeymapScope::Workspace),
            PluginKeyBinding::new(
                "Ctrl+r",
                "pdf.rotate-clockwise",
                PluginKeymapScope::Workspace,
            ),
            PluginKeyBinding::new("D", "pdf.delete-page", PluginKeymapScope::Workspace),
            PluginKeyBinding::new("S", "buffer.save", PluginKeymapScope::Workspace),
        ]),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_pdf_commands() {
        let package = package();
        let command_names = package
            .commands()
            .iter()
            .map(|command| command.name())
            .collect::<Vec<_>>();

        assert!(command_names.contains(&"pdf.next-page"));
        assert!(command_names.contains(&"pdf.previous-page"));
        assert!(command_names.contains(&"pdf.rotate-clockwise"));
        assert!(command_names.contains(&"pdf.delete-page"));
    }

    #[test]
    fn package_exports_pdf_buffer_keybindings() {
        let package = package();
        let buffer = package
            .buffer(PDF_BUFFER_KIND)
            .expect("pdf buffer registration should exist");

        assert!(
            buffer
                .key_bindings()
                .iter()
                .any(|binding| binding.command_name() == "pdf.next-page")
        );
        assert!(
            buffer
                .key_bindings()
                .iter()
                .any(|binding| binding.command_name() == "buffer.save")
        );
    }
}
