use editor_plugin_api::SyntaxNodeContext;

use crate::icon_font;

pub fn format_context_label(lines: &[&str], context: &SyntaxNodeContext) -> Option<String> {
    let context_line = lines.get(context.start_position.line)?.trim();
    if context_line.is_empty() {
        return None;
    }
    let summary = summarize_context(context_line, &context.kind)?;
    let icon = context_icon(&context.kind, &summary);
    Some(format!("{icon} {summary}"))
}

pub fn summarize_context(header: &str, kind: &str) -> Option<String> {
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

pub fn extract_signature(header: &str) -> Option<String> {
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

pub fn extract_control_flow_header(header: &str, keywords: &[&str]) -> Option<String> {
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

pub fn context_icon(kind: &str, summary: &str) -> &'static str {
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
        icon_font::symbols::cod::COD_QUESTION
    } else {
        icon_font::symbols::cod::COD_SYMBOL_METHOD
    }
}

fn trim_context_header(header: &str) -> &str {
    header.split('{').next().unwrap_or(header)
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
