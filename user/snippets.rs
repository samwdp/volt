//! Builtin Plugin: Snippet Definitions (`.vsnip` load, DSL parse/expand, Autocomplete Provider).

use crate::{autocomplete::AutocompleteProviderConfig, icon_font::symbols::cod};
use editor_plugin_api::{
    AutocompleteProviderItem, PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope,
    PluginPackage, PluginVimMode, SnippetAutocompleteContext, SnippetStructureNode, snippet_hooks,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const PROVIDER_ID: &str = "snippets";
pub const PROVIDER_LABEL: &str = "Snippets";

const PROVIDER_ICON: &str = cod::COD_SYMBOL_SNIPPET;
const ITEM_ICON: &str = cod::COD_SYMBOL_SNIPPET;

const NEXT_CHORD: &str = "Tab";
const PREVIOUS_CHORD: &str = "Shift+Tab";
const CANCEL_CHORD: &str = "Escape";

/// Returns the snippets Builtin Plugin package (Snippet Mode commands + keybinds).
pub fn package() -> PluginPackage {
    PluginPackage::new(
        PROVIDER_ID,
        true,
        "Snippet Mode navigation and Snippet Definition Autocomplete Provider.",
    )
    .with_commands(vec![
        hook_command(
            "snippet.next",
            "Moves to the next Insert Point in Snippet Mode.",
            snippet_hooks::NEXT,
        ),
        hook_command(
            "snippet.previous",
            "Moves to the previous Insert Point in Snippet Mode.",
            snippet_hooks::PREVIOUS,
        ),
        hook_command(
            "snippet.cancel",
            "Leaves Snippet Mode and returns to Insert.",
            snippet_hooks::CANCEL,
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new(NEXT_CHORD, "snippet.next", PluginKeymapScope::Snippet)
            .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new(
            PREVIOUS_CHORD,
            "snippet.previous",
            PluginKeymapScope::Snippet,
        )
        .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new(CANCEL_CHORD, "snippet.cancel", PluginKeymapScope::Snippet)
            .with_vim_mode(PluginVimMode::Insert),
    ])
}

/// Autocomplete Provider registration: empty static items; host fills via `snippet_autocomplete_items`.
pub fn autocomplete_provider() -> AutocompleteProviderConfig {
    AutocompleteProviderConfig::new(PROVIDER_ID, PROVIDER_LABEL, PROVIDER_ICON, ITEM_ICON)
}

pub const VSNIP_LANGUAGE_ID: &str = "vsnip";

/// Loads, filters, and expands Snippet Definitions for the given autocomplete context.
pub fn autocomplete_items(
    context: &SnippetAutocompleteContext<'_>,
) -> Vec<AutocompleteProviderItem> {
    if context.multicursor_active {
        return Vec::new();
    }

    if context.language_id == VSNIP_LANGUAGE_ID {
        return authoring_autocomplete_items(context.prefix);
    }

    let definitions = load_definitions(context.language_id);
    let cursor_byte = effective_cursor_byte(context);
    let mut items = Vec::new();

    for definition in definitions {
        if !trigger_matches(context.prefix, &definition.trigger) {
            continue;
        }
        // Offer as soon as the typed prefix matches the trigger (e.g. `/` → `///`).
        // When a Snippet Target exists, projections fill params/returns; otherwise
        // static body only (projection holes omitted).
        let expanded = match find_target(
            &definition.target,
            context.nodes,
            context.buffer_text,
            context.cursor_line,
            cursor_byte,
        ) {
            Some(target) => {
                match expand_definition(&definition, context.buffer_text, context.nodes, target) {
                    Some(text) => text,
                    None => continue,
                }
            }
            None => expand_definition_static(&definition),
        };
        items.push(AutocompleteProviderItem {
            label: definition.name.clone(),
            replacement: expanded.clone(),
            detail: Some(definition.trigger.clone()),
            documentation: Some(expanded),
        });
    }

    items
}

fn hook_command(name: &str, description: &str, hook_name: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, None::<&str>)],
    )
}

const AUTHORING_COMPLETIONS: &[(&str, &str, &str)] = &[
    ("name", "name: ", "Snippet Definition name field"),
    (
        "trigger",
        "trigger: ",
        "Typed prefix that offers this Snippet",
    ),
    (
        "target",
        "target: next ",
        "Snippet Target relation + node kind",
    ),
    ("each", "each ", "Repeat projection for each capture"),
    ("maybe", "maybe ", "Optional zero-or-one projection"),
    ("path", "path: ", "Capture path under the Snippet Target"),
    ("line", "line: ", "Projection line template"),
    ("body", "body:", "Static Snippet body block"),
    (
        "next",
        "next ",
        "Target relation: first matching node at/after cursor",
    ),
    (
        "enclosing",
        "enclosing ",
        "Target relation: innermost matching node around cursor",
    ),
    ("${}", "${}", "Projection hole in the body"),
    (
        "$${i}",
        "$${i}",
        "Auto-numbered Insert Point in a projection line",
    ),
    ("$0", "$0", "First Insert Point after accept"),
];

fn authoring_autocomplete_items(prefix: &str) -> Vec<AutocompleteProviderItem> {
    let prefix_lower = prefix.to_ascii_lowercase();
    AUTHORING_COMPLETIONS
        .iter()
        .filter_map(|(label, replacement, documentation)| {
            if !prefix_lower.is_empty() && !label.to_ascii_lowercase().starts_with(&prefix_lower) {
                return None;
            }
            Some(AutocompleteProviderItem {
                label: (*label).to_owned(),
                replacement: (*replacement).to_owned(),
                detail: Some("vsnip".to_owned()),
                documentation: Some((*documentation).to_owned()),
            })
        })
        .collect()
}

fn snippets_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("snippets")
}

fn load_definitions(language_id: &str) -> Vec<SnippetDefinition> {
    let dir = snippets_root().join(language_id);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return Vec::new(),
    };

    let mut definitions = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("vsnip") {
            continue;
        }
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(definition) = parse_vsnip(&source) {
            definitions.push(definition);
        }
    }
    definitions
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TargetRelation {
    Next,
    Enclosing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnippetTarget {
    relation: TargetRelation,
    kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProjectionKind {
    Each,
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Projection {
    kind: ProjectionKind,
    id: String,
    path: String,
    line: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SnippetDefinition {
    name: String,
    trigger: String,
    target: SnippetTarget,
    projections: Vec<Projection>,
    body: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParseError {
    message: String,
}

impl ParseError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Parses one `.vsnip` Snippet Definition.
fn parse_vsnip(source: &str) -> Result<SnippetDefinition, ParseError> {
    let mut name: Option<String> = None;
    let mut trigger: Option<String> = None;
    let mut target: Option<SnippetTarget> = None;
    let mut projections: Vec<Projection> = Vec::new();
    let mut body: Option<Vec<String>> = None;

    let mut lines = source.lines().peekable();
    while let Some(raw) = lines.next() {
        let line = strip_trailing_cr(raw);
        if line.trim().is_empty() {
            continue;
        }

        if let Some(rest) = line.strip_prefix("name:") {
            name = Some(rest.trim().to_owned());
            continue;
        }
        if let Some(rest) = line.strip_prefix("trigger:") {
            trigger = Some(rest.trim().to_owned());
            continue;
        }
        if let Some(rest) = line.strip_prefix("target:") {
            target = Some(parse_target(rest.trim())?);
            continue;
        }
        if let Some(rest) = line.strip_prefix("each ") {
            let id = rest
                .strip_suffix(':')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| ParseError::new("each block missing id"))?
                .to_owned();
            let (path, template) = parse_projection_block(&mut lines)?;
            projections.push(Projection {
                kind: ProjectionKind::Each,
                id,
                path,
                line: template,
            });
            continue;
        }
        if let Some(rest) = line.strip_prefix("maybe ") {
            let id = rest
                .strip_suffix(':')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| ParseError::new("maybe block missing id"))?
                .to_owned();
            let (path, template) = parse_projection_block(&mut lines)?;
            projections.push(Projection {
                kind: ProjectionKind::Maybe,
                id,
                path,
                line: template,
            });
            continue;
        }
        if line.trim() == "body:" {
            let mut body_lines = Vec::new();
            for raw_body in lines.by_ref() {
                body_lines.push(strip_trailing_cr(raw_body).to_owned());
            }
            while body_lines
                .last()
                .is_some_and(|value| value.trim().is_empty())
            {
                body_lines.pop();
            }
            body = Some(body_lines);
            break;
        }

        return Err(ParseError::new(format!("unexpected vsnip line: {line}")));
    }

    Ok(SnippetDefinition {
        name: name.ok_or_else(|| ParseError::new("missing name"))?,
        trigger: trigger.ok_or_else(|| ParseError::new("missing trigger"))?,
        target: target.ok_or_else(|| ParseError::new("missing target"))?,
        projections,
        body: body.ok_or_else(|| ParseError::new("missing body"))?,
    })
}

fn parse_target(value: &str) -> Result<SnippetTarget, ParseError> {
    let mut parts = value.split_whitespace();
    let relation = match parts.next() {
        Some("next") => TargetRelation::Next,
        Some("enclosing") => TargetRelation::Enclosing,
        other => {
            return Err(ParseError::new(format!(
                "target relation must be next|enclosing, got {other:?}"
            )));
        }
    };
    let kind = parts
        .next()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ParseError::new("target missing kind"))?
        .to_owned();
    if parts.next().is_some() {
        return Err(ParseError::new("target has trailing tokens"));
    }
    Ok(SnippetTarget { relation, kind })
}

fn parse_projection_block<'a, I>(
    lines: &mut std::iter::Peekable<I>,
) -> Result<(String, String), ParseError>
where
    I: Iterator<Item = &'a str>,
{
    let mut path: Option<String> = None;
    let mut template: Option<String> = None;

    while let Some(raw) = lines.peek().copied() {
        let line = strip_trailing_cr(raw);
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if indent == 0 {
            break;
        }
        lines.next();
        if let Some(rest) = trimmed.strip_prefix("path:") {
            path = Some(rest.trim().to_owned());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("line:") {
            template = Some(rest.trim().to_owned());
            continue;
        }
        return Err(ParseError::new(format!(
            "unexpected projection field: {trimmed}"
        )));
    }

    Ok((
        path.ok_or_else(|| ParseError::new("projection missing path"))?,
        template.ok_or_else(|| ParseError::new("projection missing line"))?,
    ))
}

fn strip_trailing_cr(line: &str) -> &str {
    line.strip_suffix('\r').unwrap_or(line)
}

fn trigger_matches(prefix: &str, trigger: &str) -> bool {
    !prefix.is_empty() && trigger.starts_with(prefix)
}

fn effective_cursor_byte(context: &SnippetAutocompleteContext<'_>) -> usize {
    if context.cursor_byte > 0 {
        return context.cursor_byte.min(context.buffer_text.len());
    }
    line_column_to_byte(
        context.buffer_text,
        context.cursor_line,
        context.cursor_column,
    )
}

fn line_column_to_byte(text: &str, line: usize, column: usize) -> usize {
    let mut current_line = 0usize;
    let mut index = 0usize;
    for ch in text.chars() {
        if current_line == line {
            break;
        }
        index += ch.len_utf8();
        if ch == '\n' {
            current_line += 1;
        }
    }
    let mut col = 0usize;
    let bytes = text.as_bytes();
    while index < bytes.len() && col < column {
        let ch = text[index..].chars().next();
        let Some(ch) = ch else {
            break;
        };
        if ch == '\n' {
            break;
        }
        index += ch.len_utf8();
        col += 1;
    }
    index
}

fn byte_to_line(text: &str, byte: usize) -> usize {
    let end = byte.min(text.len());
    text[..end].bytes().filter(|&b| b == b'\n').count()
}

fn find_target<'a>(
    target: &SnippetTarget,
    nodes: &'a [SnippetStructureNode],
    buffer_text: &str,
    cursor_line: usize,
    cursor_byte: usize,
) -> Option<&'a SnippetStructureNode> {
    match target.relation {
        TargetRelation::Next => nodes
            .iter()
            .filter(|node| node.kind == target.kind)
            .filter(|node| byte_to_line(buffer_text, node.start_byte) >= cursor_line)
            .min_by_key(|node| node.start_byte),
        TargetRelation::Enclosing => nodes
            .iter()
            .filter(|node| node.kind == target.kind)
            .filter(|node| node.start_byte <= cursor_byte && cursor_byte < node.end_byte)
            .min_by_key(|node| node.end_byte.saturating_sub(node.start_byte)),
    }
}

fn expand_definition(
    definition: &SnippetDefinition,
    buffer_text: &str,
    nodes: &[SnippetStructureNode],
    target: &SnippetStructureNode,
) -> Option<String> {
    let mut next_insert = max_explicit_insert_point(&definition.body) + 1;
    let mut hole_values: Vec<(String, String)> = Vec::new();

    for projection in &definition.projections {
        let lines = expand_projection(projection, buffer_text, nodes, target, &mut next_insert)?;
        hole_values.push((projection.id.clone(), lines.join("\n")));
    }

    Some(render_body(&definition.body, &hole_values))
}

/// Expand body with all projection holes empty (omit sole-hole lines).
fn expand_definition_static(definition: &SnippetDefinition) -> String {
    let hole_values = definition
        .projections
        .iter()
        .map(|projection| (projection.id.clone(), String::new()))
        .collect::<Vec<_>>();
    render_body(&definition.body, &hole_values)
}

fn render_body(body: &[String], hole_values: &[(String, String)]) -> String {
    let mut output_lines: Vec<String> = Vec::new();
    for body_line in body {
        let trimmed = body_line.trim();
        if let Some(id) = sole_hole_id(trimmed)
            && let Some((_, value)) = hole_values.iter().find(|(hole, _)| hole == id)
        {
            if value.is_empty() {
                continue;
            }
            for value_line in value.lines() {
                output_lines.push(value_line.to_owned());
            }
            continue;
        }

        let mut rendered = body_line.clone();
        for (id, value) in hole_values {
            let needle = format!("${{{id}}}");
            rendered = rendered.replace(&needle, value);
        }
        output_lines.push(rendered);
    }
    output_lines.join("\n")
}

fn sole_hole_id(trimmed: &str) -> Option<&str> {
    let rest = trimmed.strip_prefix("${")?;
    let id = rest.strip_suffix('}')?;
    if id.is_empty() || id.contains('{') || id.contains('}') || id.contains('$') {
        return None;
    }
    Some(id)
}

fn max_explicit_insert_point(body: &[String]) -> isize {
    let mut max = -1isize;
    for line in body {
        max = max.max(max_explicit_in_line(line));
    }
    max
}

fn max_explicit_in_line(line: &str) -> isize {
    let bytes = line.as_bytes();
    let mut max = -1isize;
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'$' {
            // Skip auto-marker `$${i}` and `${id}` holes.
            if index + 1 < bytes.len() && bytes[index + 1] == b'$' {
                index += 2;
                continue;
            }
            if index + 1 < bytes.len() && bytes[index + 1] == b'{' {
                index += 2;
                continue;
            }
            let mut end = index + 1;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            if end > index + 1 {
                if let Ok(digits) = std::str::from_utf8(&bytes[index + 1..end])
                    && let Ok(value) = digits.parse::<isize>()
                {
                    max = max.max(value);
                }
                index = end;
                continue;
            }
        }
        index += 1;
    }
    max
}

fn expand_projection(
    projection: &Projection,
    buffer_text: &str,
    nodes: &[SnippetStructureNode],
    target: &SnippetStructureNode,
    next_insert: &mut isize,
) -> Option<Vec<String>> {
    let captures = match projection.kind {
        ProjectionKind::Each => capture_each(&projection.path, buffer_text, nodes, target),
        ProjectionKind::Maybe => {
            match capture_maybe(&projection.path, buffer_text, nodes, target) {
                Some(text) if should_emit_maybe(&projection.id, &text) => vec![text],
                _ => Vec::new(),
            }
        }
    };

    let field = last_path_segment(&projection.path);
    let mut lines = Vec::new();
    for capture in captures {
        let mut rendered = projection.line.replace(&format!("${{{field}}}"), &capture);
        while rendered.contains("$${i}") {
            let marker = format!("${}", *next_insert);
            rendered = rendered.replacen("$${i}", &marker, 1);
            *next_insert += 1;
        }
        lines.push(rendered);
    }
    Some(lines)
}

fn should_emit_maybe(id: &str, text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    if id == "returns" && trimmed == "void" {
        return false;
    }
    true
}

fn last_path_segment(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn capture_each(
    path: &str,
    buffer_text: &str,
    nodes: &[SnippetStructureNode],
    target: &SnippetStructureNode,
) -> Vec<String> {
    let segments: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();
    if segments.is_empty() {
        return Vec::new();
    }

    let under_target: Vec<&SnippetStructureNode> = nodes
        .iter()
        .filter(|node| node_under_target(node, target))
        .collect();

    let mut roots: Vec<&SnippetStructureNode> = under_target
        .iter()
        .copied()
        .filter(|node| segment_matches(node, segments[0]))
        .collect();
    roots.sort_by_key(|node| node.start_byte);

    let mut captures = Vec::new();
    for root in roots {
        let mut current = root;
        let mut ok = true;
        for segment in &segments[1..] {
            let Some(child) = under_target
                .iter()
                .copied()
                .filter(|node| node_under_exclusive(node, current))
                .find(|node| segment_matches(node, segment))
            else {
                ok = false;
                break;
            };
            current = child;
        }
        if ok {
            captures.push(node_text(buffer_text, current));
        }
    }
    captures
}

fn capture_maybe(
    path: &str,
    buffer_text: &str,
    nodes: &[SnippetStructureNode],
    target: &SnippetStructureNode,
) -> Option<String> {
    if path == "returns" {
        return capture_returns(buffer_text, nodes, target);
    }
    capture_each(path, buffer_text, nodes, target)
        .into_iter()
        .next()
}

fn capture_returns(
    buffer_text: &str,
    nodes: &[SnippetStructureNode],
    target: &SnippetStructureNode,
) -> Option<String> {
    let under_target: Vec<&SnippetStructureNode> = nodes
        .iter()
        .filter(|node| node_under_target(node, target))
        .collect();

    if let Some(node) = under_target
        .iter()
        .copied()
        .find(|node| node.field_name.as_deref() == Some("returns"))
    {
        return Some(node_text(buffer_text, node));
    }

    let parameter_list_start = under_target
        .iter()
        .copied()
        .filter(|node| node.kind == "parameter_list")
        .map(|node| node.start_byte)
        .min();

    under_target
        .iter()
        .copied()
        .filter(|node| node.field_name.as_deref() == Some("type"))
        .filter(|node| match parameter_list_start {
            Some(start) => node.start_byte < start,
            None => true,
        })
        .min_by_key(|node| node.start_byte)
        .map(|node| node_text(buffer_text, node))
}

fn node_under_target(node: &SnippetStructureNode, target: &SnippetStructureNode) -> bool {
    node.start_byte >= target.start_byte && node.end_byte <= target.end_byte
}

fn node_under_exclusive(node: &SnippetStructureNode, parent: &SnippetStructureNode) -> bool {
    node.start_byte >= parent.start_byte
        && node.end_byte <= parent.end_byte
        && !(node.start_byte == parent.start_byte && node.end_byte == parent.end_byte)
}

fn segment_matches(node: &SnippetStructureNode, segment: &str) -> bool {
    node.kind == segment || node.field_name.as_deref() == Some(segment)
}

fn node_text(buffer_text: &str, node: &SnippetStructureNode) -> String {
    let start = node.start_byte.min(buffer_text.len());
    let end = node.end_byte.min(buffer_text.len()).max(start);
    buffer_text[start..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xmldoc_source() -> String {
        fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("snippets/csharp/xmldoc.vsnip"),
        )
        .expect("xmldoc.vsnip should exist")
    }

    fn method_nodes_int_params() -> (String, Vec<SnippetStructureNode>, usize) {
        // public int foo(int a, int b)
        // Positions approximate a flat tree-sitter walk.
        let buffer = "public int foo(int a, int b)\n{\n}\n";
        let method = SnippetStructureNode {
            kind: "method_declaration".to_owned(),
            field_name: None,
            start_byte: 0,
            end_byte: buffer.find('{').unwrap_or(buffer.len()),
        };
        let type_node = SnippetStructureNode {
            kind: "predefined_type".to_owned(),
            field_name: Some("type".to_owned()),
            start_byte: 7,
            end_byte: 10,
        };
        let name = SnippetStructureNode {
            kind: "identifier".to_owned(),
            field_name: Some("name".to_owned()),
            start_byte: 11,
            end_byte: 14,
        };
        let parameter_list = SnippetStructureNode {
            kind: "parameter_list".to_owned(),
            field_name: Some("parameters".to_owned()),
            start_byte: 14,
            end_byte: 27,
        };
        let param_a = SnippetStructureNode {
            kind: "parameter".to_owned(),
            field_name: None,
            start_byte: 15,
            end_byte: 20,
        };
        let param_a_name = SnippetStructureNode {
            kind: "identifier".to_owned(),
            field_name: Some("name".to_owned()),
            start_byte: 19,
            end_byte: 20,
        };
        let param_b = SnippetStructureNode {
            kind: "parameter".to_owned(),
            field_name: None,
            start_byte: 22,
            end_byte: 27,
        };
        let param_b_name = SnippetStructureNode {
            kind: "identifier".to_owned(),
            field_name: Some("name".to_owned()),
            start_byte: 26,
            end_byte: 27,
        };
        let nodes = vec![
            method,
            type_node,
            name,
            parameter_list,
            param_a,
            param_a_name,
            param_b,
            param_b_name,
        ];
        (buffer.to_owned(), nodes, 0)
    }

    fn method_nodes_void() -> (String, Vec<SnippetStructureNode>) {
        let buffer = "public void foo(int a)\n{\n}\n";
        let method = SnippetStructureNode {
            kind: "method_declaration".to_owned(),
            field_name: None,
            start_byte: 0,
            end_byte: buffer.find('{').unwrap_or(buffer.len()),
        };
        let type_node = SnippetStructureNode {
            kind: "predefined_type".to_owned(),
            field_name: Some("type".to_owned()),
            start_byte: 7,
            end_byte: 11,
        };
        let parameter_list = SnippetStructureNode {
            kind: "parameter_list".to_owned(),
            field_name: Some("parameters".to_owned()),
            start_byte: 15,
            end_byte: 21,
        };
        let param_a = SnippetStructureNode {
            kind: "parameter".to_owned(),
            field_name: None,
            start_byte: 16,
            end_byte: 21,
        };
        let param_a_name = SnippetStructureNode {
            kind: "identifier".to_owned(),
            field_name: Some("name".to_owned()),
            start_byte: 20,
            end_byte: 21,
        };
        (
            buffer.to_owned(),
            vec![method, type_node, parameter_list, param_a, param_a_name],
        )
    }

    #[test]
    fn parses_csharp_xmldoc_vsnip() {
        let definition = parse_vsnip(&xmldoc_source()).expect("parse xmldoc");
        assert_eq!(definition.name, "csharp-xmldoc");
        assert_eq!(definition.trigger, "///");
        assert_eq!(definition.target.relation, TargetRelation::Next);
        assert_eq!(definition.target.kind, "method_declaration");
        assert_eq!(definition.projections.len(), 2);
        assert_eq!(definition.projections[0].id, "params");
        assert_eq!(definition.projections[0].path, "parameter/name");
        assert_eq!(definition.projections[1].id, "returns");
        assert!(definition.body.iter().any(|line| line.contains("$0")));
        assert!(
            definition
                .body
                .iter()
                .any(|line| line.trim() == "${params}")
        );
    }

    #[test]
    fn expands_xmldoc_with_params_and_returns() {
        let definition = parse_vsnip(&xmldoc_source()).expect("parse xmldoc");
        let (buffer, nodes, cursor_line) = method_nodes_int_params();
        let target = find_target(&definition.target, &nodes, &buffer, cursor_line, 0)
            .expect("method target");
        let expanded = expand_definition(&definition, &buffer, &nodes, target).expect("expand");
        assert!(expanded.contains("/// <summary>"));
        assert!(expanded.contains("/// $0"));
        assert!(expanded.contains("/// </summary>"));
        assert!(expanded.contains("/// <param name=\"a\">$1</param>"));
        assert!(expanded.contains("/// <param name=\"b\">$2</param>"));
        assert!(expanded.contains("/// <returns>$3</returns>"));
    }

    #[test]
    fn expands_xmldoc_omits_void_returns() {
        let definition = parse_vsnip(&xmldoc_source()).expect("parse xmldoc");
        let (buffer, nodes) = method_nodes_void();
        let target = find_target(&definition.target, &nodes, &buffer, 0, 0).expect("method target");
        let expanded = expand_definition(&definition, &buffer, &nodes, target).expect("expand");
        assert!(expanded.contains("/// <param name=\"a\">$1</param>"));
        assert!(!expanded.contains("<returns>"));
    }

    #[test]
    fn autocomplete_items_respect_prefix_target_and_multicursor() {
        let (buffer, nodes, _) = method_nodes_int_params();
        let context = SnippetAutocompleteContext {
            language_id: "csharp",
            prefix: "///",
            cursor_byte: 0,
            cursor_line: 0,
            cursor_column: 0,
            buffer_text: &buffer,
            nodes: &nodes,
            multicursor_active: false,
        };
        // Only works when csharp snippets dir is present (it is).
        let items = autocomplete_items(&context);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "csharp-xmldoc");
        assert_eq!(items[0].detail.as_deref(), Some("///"));
        assert!(
            items[0]
                .replacement
                .contains("/// <param name=\"a\">$1</param>")
        );

        let slash_only = SnippetAutocompleteContext {
            prefix: "/",
            ..context
        };
        let slash_items = autocomplete_items(&slash_only);
        assert_eq!(
            slash_items.len(),
            1,
            "typing `/` must offer /// docstring snippet"
        );

        let empty_prefix = SnippetAutocompleteContext {
            prefix: "",
            ..context
        };
        assert!(
            autocomplete_items(&empty_prefix).is_empty(),
            "empty autocomplete prefix must not match /// triggers"
        );

        let no_nodes = SnippetAutocompleteContext {
            prefix: "/",
            nodes: &[],
            ..context
        };
        let static_items = autocomplete_items(&no_nodes);
        assert_eq!(static_items.len(), 1);
        assert!(static_items[0].replacement.contains("/// <summary>"));
        assert!(
            !static_items[0].replacement.contains("<param"),
            "without Snippet Target, param projections stay omitted"
        );

        let blocked = SnippetAutocompleteContext {
            multicursor_active: true,
            ..context
        };
        assert!(autocomplete_items(&blocked).is_empty());
    }

    #[test]
    fn package_exports_snippet_mode_bindings() {
        let package = package();
        let names: Vec<_> = package.commands().iter().map(|c| c.name()).collect();
        assert!(names.contains(&"snippet.next"));
        assert!(names.contains(&"snippet.previous"));
        assert!(names.contains(&"snippet.cancel"));

        let bindings = package.key_bindings();
        assert!(bindings.iter().any(|b| {
            b.chord() == NEXT_CHORD
                && b.scope() == PluginKeymapScope::Snippet
                && b.vim_mode() == PluginVimMode::Insert
        }));
        assert!(bindings.iter().any(|b| b.chord() == PREVIOUS_CHORD));
        assert!(bindings.iter().any(|b| b.chord() == CANCEL_CHORD));
    }

    #[test]
    fn autocomplete_provider_has_no_or_group_and_empty_items() {
        let provider = autocomplete_provider();
        assert_eq!(provider.id, PROVIDER_ID);
        assert_eq!(provider.label, PROVIDER_LABEL);
        assert!(provider.or_group.is_none());
        assert!(provider.items.is_empty());
    }

    #[test]
    fn vsnip_authoring_offers_field_keywords() {
        let context = SnippetAutocompleteContext {
            language_id: VSNIP_LANGUAGE_ID,
            prefix: "tri",
            cursor_byte: 0,
            cursor_line: 0,
            cursor_column: 0,
            buffer_text: "tri",
            nodes: &[],
            multicursor_active: false,
        };
        let items = autocomplete_items(&context);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "trigger");
        assert_eq!(items[0].replacement, "trigger: ");
    }

    fn class_nodes_test() -> (String, Vec<SnippetStructureNode>, usize) {
        // public class Test\n{\n  \n}\n — cursor inside braces
        let buffer = "public class Test\n{\n  \n}\n";
        let class_start = 0usize;
        let class_end = buffer.len();
        let name_start = buffer.find("Test").expect("Test");
        let name_end = name_start + 4;
        let cursor_byte = buffer.find("{\n  ").expect("body") + 3;
        let class_node = SnippetStructureNode {
            kind: "class_declaration".to_owned(),
            field_name: None,
            start_byte: class_start,
            end_byte: class_end,
        };
        let name_node = SnippetStructureNode {
            kind: "identifier".to_owned(),
            field_name: Some("name".to_owned()),
            start_byte: name_start,
            end_byte: name_end,
        };
        (buffer.to_owned(), vec![class_node, name_node], cursor_byte)
    }

    #[test]
    fn expands_ctor_with_enclosing_class_name() {
        let source = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("snippets/csharp/ctor.vsnip"),
        )
        .expect("ctor.vsnip");
        let definition = parse_vsnip(&source).expect("parse ctor");
        let (buffer, nodes, cursor_byte) = class_nodes_test();
        let target = find_target(&definition.target, &nodes, &buffer, 2, cursor_byte)
            .expect("enclosing class");
        let expanded = expand_definition(&definition, &buffer, &nodes, target).expect("expand");
        assert_eq!(expanded, "public Test($0)\n{\n$1\n}");
    }

    #[test]
    fn autocomplete_ctor_matches_prefix() {
        let (buffer, nodes, cursor_byte) = class_nodes_test();
        let context = SnippetAutocompleteContext {
            language_id: "csharp",
            prefix: "ct",
            cursor_byte,
            cursor_line: 2,
            cursor_column: 2,
            buffer_text: &buffer,
            nodes: &nodes,
            multicursor_active: false,
        };
        let items = autocomplete_items(&context);
        let ctor = items
            .iter()
            .find(|item| item.label == "csharp-ctor")
            .expect("ctor snippet");
        assert_eq!(ctor.detail.as_deref(), Some("ctor"));
        assert_eq!(ctor.replacement, "public Test($0)\n{\n$1\n}");
    }
}
