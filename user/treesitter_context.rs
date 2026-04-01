use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use editor_buffer::{TextBuffer, TextPoint};
use editor_plugin_api::{GhostTextContext, GhostTextLine};
use editor_syntax::{SyntaxNodeContext, SyntaxRegistry};

use crate::{icon_font, syntax_languages};

pub fn ghost_text_lines(context: &GhostTextContext<'_>) -> Vec<GhostTextLine> {
    let Some(language_id) = context.language_id else {
        return Vec::new();
    };
    if context.buffer_text.is_empty() {
        return Vec::new();
    }
    let Ok(mut registry) = syntax_registry().lock() else {
        return Vec::new();
    };
    let buffer = TextBuffer::from_text(context.buffer_text.to_owned());
    let Ok(contexts) = registry.ancestor_contexts_for_language(
        language_id,
        &buffer,
        TextPoint::new(context.cursor_line, context.cursor_column),
    ) else {
        return Vec::new();
    };
    build_ghost_text_lines(context.buffer_text, context.cursor_line, &contexts)
}

fn syntax_registry() -> &'static Mutex<SyntaxRegistry> {
    static REGISTRY: OnceLock<Mutex<SyntaxRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = SyntaxRegistry::new();
        if let Err(error) = registry.register_all(syntax_languages()) {
            panic!("failed to register user syntax languages for ghost text: {error:?}");
        }
        Mutex::new(registry)
    })
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
        if context.end_position.line <= cursor_line {
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

fn format_context_label(lines: &[&str], context: &SyntaxNodeContext) -> Option<String> {
    let header = lines.get(context.start_position.line)?.trim();
    if header.is_empty() {
        return None;
    }
    let summary = summarize_context(header, &context.kind)?;
    let icon = context_icon(&context.kind, &summary);
    Some(format!("{icon} {summary}"))
}

fn summarize_context(header: &str, kind: &str) -> Option<String> {
    let header = header.split('{').next().unwrap_or(header);
    let header = collapse_whitespace(header);
    if header.is_empty() {
        return None;
    }
    if is_function_kind(kind) {
        return extract_signature(&header).or_else(|| Some(header));
    }
    if let Some(summary) = extract_named_keyword(&header, &["class", "struct", "interface", "enum"])
    {
        return Some(summary);
    }
    if let Some(summary) = extract_named_keyword(&header, &["trait", "impl", "namespace", "module"])
    {
        return Some(summary);
    }
    if kind.contains("function") || kind.contains("method") || kind.contains("constructor") {
        return extract_signature(&header).or_else(|| Some(header));
    }
    None
}

fn extract_signature(header: &str) -> Option<String> {
    let open = header.find('(')?;
    let close = header[open..].find(')')?;
    let close = open + close;
    let prefix = header[..open].trim_end();
    let name_start = prefix
        .char_indices()
        .rev()
        .find(|(_, ch)| ch.is_whitespace() || matches!(ch, ':' | '.' | '>' | '<'))
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(0);
    let name = prefix[name_start..].trim();
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
    } else {
        icon_font::symbols::cod::COD_SYMBOL_METHOD
    }
}

#[cfg(test)]
mod tests {
    use super::{build_ghost_text_lines, extract_signature};
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
            "class Demo {\n    fn render(value: usize) {\n        let current = value;\n    }\n}\n";
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
        assert!(lines[1].text.contains("class Demo") || lines[1].text.contains("impl Demo"));
    }
}
