use std::collections::BTreeSet;

use editor_plugin_api::{GhostTextContext, treesitter};

use crate::treesittercontext_shared::format_context_label_from_header;

const MAX_HEADERLINE_CONTEXTS: usize = 4;
const HEADERLINE_SEPARATOR: &str = "  >  ";

/// Returns sticky headerline breadcrumbs derived from tree-sitter contexts.
pub fn headerline_lines(context: &GhostTextContext<'_>) -> Vec<String> {
    let contexts = treesitter::ancestor_contexts_for_cursor(
        &crate::syntax_languages(),
        context.language_id,
        context.buffer_text,
        context.buffer_id,
        context.buffer_revision,
        context.cursor_line,
        context.cursor_column,
    );
    build_headerline_lines(
        context.buffer_text,
        context.buffer_id,
        context.buffer_revision,
        context.viewport_top_line,
        &contexts,
    )
}

fn build_headerline_lines(
    buffer_text: &str,
    buffer_id: u64,
    buffer_revision: u64,
    viewport_top_line: usize,
    contexts: &[editor_plugin_api::SyntaxNodeContext],
) -> Vec<String> {
    if viewport_top_line == 0 {
        return Vec::new();
    }
    let mut seen = BTreeSet::new();
    let mut breadcrumbs = Vec::new();
    for context in contexts.iter().rev() {
        if context.start_position.line >= viewport_top_line {
            continue;
        }
        let Some(source_line) = treesitter::buffer_line_text(
            buffer_text,
            buffer_id,
            buffer_revision,
            context.start_position.line,
        ) else {
            continue;
        };
        let Some(text) = format_context_label_from_header(&source_line, &context.kind) else {
            continue;
        };
        if seen.insert(text.clone()) {
            breadcrumbs.push(text);
        }
    }
    if breadcrumbs.is_empty() {
        return Vec::new();
    }
    let start = breadcrumbs.len().saturating_sub(MAX_HEADERLINE_CONTEXTS);
    vec![breadcrumbs[start..].join(HEADERLINE_SEPARATOR)]
}

#[cfg(test)]
mod tests {
    use super::{HEADERLINE_SEPARATOR, build_headerline_lines};
    use crate::{icon_font, treesittercontext_shared::summarize_context};
    use editor_syntax::{SyntaxNodeContext, SyntaxPoint};

    #[test]
    fn build_headerline_lines_orders_contexts_outermost_first() {
        let buffer =
            "impl Demo {\n    fn render(value: usize) {\n        let current = value;\n    }\n}\n";
        let contexts = vec![
            SyntaxNodeContext {
                kind: "function_item".to_owned(),
                start_position: SyntaxPoint::new(1, 4),
                end_position: SyntaxPoint::new(3, 5),
            },
            SyntaxNodeContext {
                kind: "impl_item".to_owned(),
                start_position: SyntaxPoint::new(0, 0),
                end_position: SyntaxPoint::new(4, 1),
            },
        ];

        assert_eq!(
            build_headerline_lines(buffer, 1, 1, 2, &contexts),
            vec![format!(
                "{} impl Demo{}{} render(value: usize)",
                icon_font::symbols::cod::COD_SYMBOL_STRUCTURE,
                HEADERLINE_SEPARATOR,
                icon_font::symbols::cod::COD_SYMBOL_METHOD,
            )]
        );
    }

    #[test]
    fn build_headerline_lines_skips_contexts_visible_in_viewport() {
        let buffer =
            "impl Demo {\n    fn render(value: usize) {\n        let current = value;\n    }\n}\n";
        let contexts = vec![
            SyntaxNodeContext {
                kind: "function_item".to_owned(),
                start_position: SyntaxPoint::new(1, 4),
                end_position: SyntaxPoint::new(3, 5),
            },
            SyntaxNodeContext {
                kind: "impl_item".to_owned(),
                start_position: SyntaxPoint::new(0, 0),
                end_position: SyntaxPoint::new(4, 1),
            },
        ];

        assert_eq!(
            build_headerline_lines(buffer, 1, 1, 1, &contexts),
            vec![format!(
                "{} impl Demo",
                icon_font::symbols::cod::COD_SYMBOL_STRUCTURE
            )]
        );
        assert!(build_headerline_lines(buffer, 1, 1, 0, &contexts).is_empty());
    }

    #[test]
    fn summarize_context_handles_unknown_named_nodes() {
        assert_eq!(
            summarize_context("component Dashboard {", "component_declaration"),
            Some("component Dashboard".to_owned())
        );
        assert_eq!(summarize_context("{", "block"), None);
    }
}
