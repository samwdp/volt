use std::collections::BTreeMap;

use editor_plugin_api::{GhostTextContext, GhostTextLine, PluginPackage, treesitter};

use crate::treesittercontext_shared::format_context_label;

/// Returns the metadata for the tree-sitter ghost text context package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "treesittercontext_ghosttext",
        true,
        "Tree-sitter powered inline breadcrumb annotations.",
    )
}

pub fn ghost_text_lines(context: &GhostTextContext<'_>) -> Vec<GhostTextLine> {
    let contexts = treesitter::ancestor_contexts_for_cursor(
        &crate::syntax_languages(),
        context.language_id,
        context.buffer_text,
        context.cursor_line,
        context.cursor_column,
    );
    build_ghost_text_lines(context.buffer_text, context.cursor_line, &contexts)
}

fn build_ghost_text_lines(
    buffer_text: &str,
    cursor_line: usize,
    contexts: &[editor_plugin_api::SyntaxNodeContext],
) -> Vec<GhostTextLine> {
    let lines = buffer_text.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Vec::new();
    }
    let mut by_line = BTreeMap::new();
    for context in contexts {
        if context.end_position.line < cursor_line {
            continue;
        }
        let Some(text) = format_context_label(&lines, context) else {
            continue;
        };
        by_line.entry(context.end_position.line).or_insert(text);
    }
    by_line
        .into_iter()
        .map(|(line, text)| GhostTextLine { line, text })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{build_ghost_text_lines, package};
    use crate::icon_font;
    use crate::treesittercontext_shared::{
        context_icon, extract_control_flow_header, extract_signature, summarize_context,
    };
    use editor_syntax::{SyntaxNodeContext, SyntaxPoint};

    #[test]
    fn extract_signature_drops_modifiers_and_return_types() {
        assert_eq!(
            extract_signature("public void render(string value)"),
            Some("render(string value)".to_owned())
        );
        assert_eq!(
            extract_signature("fn test(input: &str)"),
            Some("test(input: &str)".to_owned())
        );
    }

    #[test]
    fn build_ghost_text_lines_prefers_inner_context_on_shared_closing_line() {
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
            SyntaxNodeContext {
                kind: "block".to_owned(),
                start_position: SyntaxPoint::new(1, 28),
                end_position: SyntaxPoint::new(3, 5),
            },
        ];

        let lines = build_ghost_text_lines(buffer, 2, &contexts);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].text.contains("render(value: usize)"));
        assert!(lines[1].text.contains("impl Demo"));
    }

    #[test]
    fn summarize_context_handles_loops_and_conditionals() {
        assert_eq!(
            summarize_context("for item in items {", "for_statement"),
            Some("for item in items".to_owned())
        );
        assert_eq!(
            summarize_context("if value > 0 {", "if_statement"),
            Some("if value > 0".to_owned())
        );
        assert_eq!(
            summarize_context("} else if value < 0 {", "else_if_clause"),
            Some("else if value < 0".to_owned())
        );
        assert_eq!(
            summarize_context("match value {", "match_expression"),
            Some("match value".to_owned())
        );
    }

    #[test]
    fn extract_control_flow_header_finds_embedded_keywords() {
        assert_eq!(
            extract_control_flow_header("} catch (error) {", &["catch", "finally"]),
            Some("catch (error)".to_owned())
        );
        assert_eq!(
            extract_control_flow_header("} finally {", &["catch", "finally"]),
            Some("finally".to_owned())
        );
    }

    #[test]
    fn build_ghost_text_lines_includes_loop_contexts() {
        let buffer = "fn render() {\n    for item in items {\n        draw(item);\n    }\n}\n";
        let contexts = vec![
            SyntaxNodeContext {
                kind: "for_statement".to_owned(),
                start_position: SyntaxPoint::new(1, 4),
                end_position: SyntaxPoint::new(3, 5),
            },
            SyntaxNodeContext {
                kind: "function_item".to_owned(),
                start_position: SyntaxPoint::new(0, 0),
                end_position: SyntaxPoint::new(4, 1),
            },
        ];

        let lines = build_ghost_text_lines(buffer, 2, &contexts);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].text.contains("for item in items"));
        assert!(lines[1].text.contains("render()"));
    }

    #[test]
    fn control_flow_contexts_use_distinct_icons() {
        assert_eq!(
            context_icon("for_statement", "for item in items"),
            icon_font::symbols::md::MD_REPEAT
        );
        assert_eq!(
            context_icon("if_statement", "if value > 0"),
            icon_font::symbols::md::MD_SOURCE_BRANCH
        );
    }

    #[test]
    fn package_is_auto_loaded() {
        assert!(package().auto_load());
    }
}
