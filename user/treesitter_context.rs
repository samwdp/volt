use std::collections::{BTreeMap, BTreeSet};

use editor_plugin_api::{
    GhostTextContext, GhostTextLine, PluginPackage, SyntaxNodeContext, treesitter,
};

use crate::icon_font;

/// Returns the metadata for the tree-sitter context package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "treesitter-context",
        true,
        "Tree-sitter powered sticky headerline context and inline breadcrumbs.",
    )
}

pub fn headerline_lines(context: &GhostTextContext<'_>) -> Vec<String> {
    let contexts = treesitter::ancestor_contexts_for_cursor(
        &crate::syntax_languages(),
        context.language_id,
        context.buffer_text,
        context.cursor_line,
        context.cursor_column,
    );
    build_headerline_lines(context.buffer_text, &contexts)
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
    contexts: &[SyntaxNodeContext],
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

fn build_headerline_lines(buffer_text: &str, contexts: &[SyntaxNodeContext]) -> Vec<String> {
    let lines = buffer_text.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Vec::new();
    }
    let mut seen = BTreeSet::new();
    let mut headerline = Vec::new();
    for context in contexts.iter().rev() {
        let Some(text) = format_context_label(&lines, context) else {
            continue;
        };
        if seen.insert(text.clone()) {
            headerline.push(text);
        }
    }
    headerline
}

fn format_context_label(lines: &[&str], context: &SyntaxNodeContext) -> Option<String> {
    let context_line = lines.get(context.start_position.line)?.trim();
    if context_line.is_empty() {
        return None;
    }
    let summary = summarize_context(context_line, &context.kind)?;
    let icon = context_icon(&context.kind, &summary);
    Some(format!("{icon} {summary}"))
}

fn summarize_context(header: &str, kind: &str) -> Option<String> {
    let header = trim_context_header(header);
    let header = collapse_whitespace(header);
    if header.is_empty() {
        return None;
    }
    if is_function_kind(kind) {
        return extract_signature(&header).or(Some(header));
    }
    if let Some(summary) = extract_named_keyword(&header, &["class", "struct", "interface", "enum"])
    {
        return Some(summary);
    }
    if let Some(summary) = extract_named_keyword(&header, &["trait", "impl", "namespace", "module"])
    {
        return Some(summary);
    }
    if is_loop_kind(kind) {
        return extract_control_flow_header(&header, &["for", "while", "loop", "do"])
            .or(Some(header));
    }
    if is_conditional_kind(kind) {
        return extract_control_flow_header(
            &header,
            &[
                "else if", "if", "else", "match", "switch", "case", "default", "try", "catch",
                "finally",
            ],
        )
        .or(Some(header));
    }
    if ignored_context_kind(kind) {
        return None;
    }
    Some(header)
}

fn trim_context_header(header: &str) -> &str {
    header.split('{').next().unwrap_or(header)
}

fn extract_signature(header: &str) -> Option<String> {
    let open = header.find('(')?;
    let close = header[open..].find(')')?;
    let close = open + close;
    let prefix = header[..open].trim_end();
    let name = prefix
        .split_whitespace()
        .last()
        .and_then(|token| token.rsplit("::").next())
        .and_then(|token| token.rsplit('.').next())
        .unwrap_or(prefix)
        .trim();
    if name.is_empty() {
        return None;
    }
    Some(format!("{name}{}", &header[open..=close]))
}

fn extract_named_keyword(header: &str, keywords: &[&str]) -> Option<String> {
    let lowercase = header.to_ascii_lowercase();
    for keyword in keywords {
        let needle = format!("{keyword} ");
        let Some(start) = lowercase.find(&needle) else {
            continue;
        };
        let value = header[start..].trim();
        let mut parts = value.split_whitespace();
        let keyword = parts.next()?.to_ascii_lowercase();
        let name = parts.next()?;
        return Some(format!("{keyword} {name}"));
    }
    None
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_function_kind(kind: &str) -> bool {
    kind.contains("function") || kind.contains("method") || kind.contains("constructor")
}

fn is_loop_kind(kind: &str) -> bool {
    kind.contains("for") || kind.contains("while") || kind.contains("loop") || kind.contains("do")
}

fn is_conditional_kind(kind: &str) -> bool {
    kind.contains("if")
        || kind.contains("else")
        || kind.contains("match")
        || kind.contains("switch")
        || kind.contains("case")
        || kind.contains("default")
        || kind.contains("try")
        || kind.contains("catch")
        || kind.contains("finally")
}

fn extract_control_flow_header(header: &str, keywords: &[&str]) -> Option<String> {
    let lowercase = header.to_ascii_lowercase();
    for keyword in keywords {
        let Some(start) = lowercase.find(keyword) else {
            continue;
        };
        let summary = collapse_whitespace(trim_context_header(header[start..].trim()));
        if !summary.is_empty() {
            return Some(summary);
        }
    }
    None
}

fn ignored_context_kind(kind: &str) -> bool {
    matches!(
        kind,
        "block"
            | "compound_statement"
            | "statement_block"
            | "declaration_list"
            | "source_file"
            | "program"
            | "document"
            | "body"
            | "parameters"
            | "parameter_list"
            | "arguments"
            | "argument_list"
            | "field_declaration_list"
    )
}

fn context_icon(kind: &str, summary: &str) -> &'static str {
    if kind.contains("class") || summary.starts_with("class ") {
        icon_font::symbols::cod::COD_SYMBOL_CLASS
    } else if kind.contains("interface") || summary.starts_with("interface ") {
        icon_font::symbols::cod::COD_SYMBOL_INTERFACE
    } else if kind.contains("enum") || summary.starts_with("enum ") {
        icon_font::symbols::cod::COD_SYMBOL_ENUM
    } else if kind.contains("struct")
        || summary.starts_with("struct ")
        || summary.starts_with("impl ")
    {
        icon_font::symbols::cod::COD_SYMBOL_STRUCTURE
    } else if kind.contains("namespace")
        || summary.starts_with("namespace ")
        || summary.starts_with("module ")
    {
        icon_font::symbols::cod::COD_SYMBOL_NAMESPACE
    } else if is_loop_kind(kind) {
        icon_font::symbols::md::MD_REPEAT
    } else if is_conditional_kind(kind) {
        icon_font::symbols::md::MD_SOURCE_BRANCH
    } else {
        icon_font::symbols::cod::COD_SYMBOL_METHOD
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_ghost_text_lines, build_headerline_lines, context_icon, extract_control_flow_header,
        extract_signature, package, summarize_context,
    };
    use crate::icon_font;
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
            build_headerline_lines(buffer, &contexts),
            vec![
                format!(
                    "{} impl Demo",
                    icon_font::symbols::cod::COD_SYMBOL_STRUCTURE
                ),
                format!(
                    "{} render(value: usize)",
                    icon_font::symbols::cod::COD_SYMBOL_METHOD
                ),
            ]
        );
    }

    #[test]
    fn summarize_context_handles_unknown_named_nodes() {
        assert_eq!(
            summarize_context("component Dashboard {", "component_declaration"),
            Some("component Dashboard".to_owned())
        );
        assert_eq!(summarize_context("{", "block"), None);
    }

    #[test]
    fn package_is_auto_loaded() {
        assert!(package().auto_load());
    }
}
