use std::collections::BTreeMap;

use editor_plugin_api::{GhostTextContext, GhostTextLine, treesitter};

use crate::treesittercontext_shared::format_context_label;

/// Returns inline ghost-text breadcrumbs derived from tree-sitter contexts.
pub fn ghost_text_lines(context: &GhostTextContext<'_>) -> Vec<GhostTextLine> {
    let contexts = treesitter::ancestor_contexts_for_cursor(
        &crate::syntax_languages(),
        context.language_id,
        context.buffer_text,
        context.buffer_id,
        context.buffer_revision,
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
        if !should_render_context(&lines, cursor_line, context) {
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

fn should_render_context(
    lines: &[&str],
    cursor_line: usize,
    context: &editor_plugin_api::SyntaxNodeContext,
) -> bool {
    if context.end_position.line < cursor_line {
        return false;
    }
    if context.end_position.line > cursor_line {
        return true;
    }
    context.start_position.line < context.end_position.line
        && is_block_closing_line(lines.get(cursor_line).copied().unwrap_or_default())
        && is_block_like_kind(&context.kind)
}

fn is_block_closing_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('}') || trimmed.starts_with("</")
}

fn is_block_like_kind(kind: &str) -> bool {
    kind.contains("function")
        || kind.contains("method")
        || kind.contains("constructor")
        || kind.contains("class")
        || kind.contains("struct")
        || kind.contains("interface")
        || kind.contains("enum")
        || kind.contains("trait")
        || kind.contains("impl")
        || kind.contains("namespace")
        || kind.contains("module")
        || kind.contains("for")
        || kind.contains("while")
        || kind.contains("loop")
        || kind.contains("do")
        || kind.contains("if")
        || kind.contains("else")
        || kind.contains("match")
        || kind.contains("switch")
        || kind.contains("case")
        || kind.contains("default")
        || kind.contains("try")
        || kind.contains("catch")
        || kind.contains("finally")
}

#[cfg(test)]
mod tests {
    use super::build_ghost_text_lines;
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
    fn build_ghost_text_lines_skips_current_line_for_single_line_contexts() {
        let buffer = "pub use buffer_kinds::CALCULATOR as CALCULATOR_KIND;\n";
        let contexts = vec![SyntaxNodeContext {
            kind: "use_declaration".to_owned(),
            start_position: SyntaxPoint::new(0, 0),
            end_position: SyntaxPoint::new(0, 52),
        }];

        let lines = build_ghost_text_lines(buffer, 0, &contexts);
        assert!(lines.is_empty());
    }

    #[test]
    fn build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts() {
        let buffer = "pub use buffer_kinds::{\n    CALCULATOR,\n};\n";
        let contexts = vec![SyntaxNodeContext {
            kind: "use_declaration".to_owned(),
            start_position: SyntaxPoint::new(0, 0),
            end_position: SyntaxPoint::new(2, 2),
        }];

        let lines = build_ghost_text_lines(buffer, 2, &contexts);
        assert!(lines.is_empty());
    }

    #[test]
    fn build_ghost_text_lines_keeps_current_line_for_block_end_contexts() {
        let buffer = "fn render() {\n    draw();\n}\n";
        let contexts = vec![SyntaxNodeContext {
            kind: "function_item".to_owned(),
            start_position: SyntaxPoint::new(0, 0),
            end_position: SyntaxPoint::new(2, 1),
        }];

        let lines = build_ghost_text_lines(buffer, 2, &contexts);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].line, 2);
        assert!(lines[0].text.contains("render()"));
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
}
