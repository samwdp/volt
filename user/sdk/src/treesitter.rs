use editor_buffer::{TextBuffer, TextPoint};
use editor_syntax::{LanguageConfiguration, SyntaxNodeContext, SyntaxRegistry};

/// Returns named tree-sitter ancestor contexts for the provided cursor position.
pub fn ancestor_contexts_for_cursor(
    languages: &[LanguageConfiguration],
    language_id: Option<&str>,
    buffer_text: &str,
    cursor_line: usize,
    cursor_column: usize,
) -> Vec<SyntaxNodeContext> {
    let Some(language_id) = language_id else {
        return Vec::new();
    };
    if buffer_text.is_empty() {
        return Vec::new();
    }
    let mut registry = SyntaxRegistry::new();
    if registry.register_all(languages.iter().cloned()).is_err() {
        return Vec::new();
    }
    let buffer = TextBuffer::from_text(buffer_text);
    registry
        .ancestor_contexts_for_language(
            language_id,
            &buffer,
            TextPoint::new(cursor_line, cursor_column),
        )
        .unwrap_or_default()
}
