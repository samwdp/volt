//! Shared Markdown Pipeline: plan Markdown Pretty overlays from source text.
//!
//! Uses tree-sitter structure nodes when a [`SyntaxRegistry`] can parse
//! `markdown`; otherwise falls back to a line scanner that emits the same
//! node-kind keys so the user icon map stays stable.

#![doc = "Markdown Pretty planning shared by file buffers, hover, and ACP."]

use std::collections::BTreeMap;
use std::ops::Range;
use std::path::{Path, PathBuf};

use editor_buffer::TextBuffer;
use editor_syntax::{SyntaxPoint, SyntaxRegistry, SyntaxStructureNode};

/// User-tunable Markdown Pretty settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownPrettyConfig {
    /// Master enable (filetype still must be markdown unless forced).
    pub enabled: bool,
    /// When true, oversized sources skip Pretty (default off).
    pub kill_switch_enabled: bool,
    /// Line count that trips the kill-switch when enabled.
    pub kill_switch_max_lines: usize,
    /// Byte count that trips the kill-switch when enabled.
    pub kill_switch_max_bytes: usize,
    /// Max decoded image bytes (https / data / local).
    pub image_max_bytes: usize,
    /// Max visual rows an inline image may occupy.
    pub image_max_rows: usize,
    /// treesitter node kind → icon glyph.
    pub icons: BTreeMap<String, String>,
}

impl Default for MarkdownPrettyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            kill_switch_enabled: false,
            kill_switch_max_lines: 20_000,
            kill_switch_max_bytes: 2_000_000,
            image_max_bytes: 10_000_000,
            image_max_rows: 24,
            icons: default_icon_map(),
        }
    }
}

/// Default icon map keyed by tree-sitter-markdown node kinds.
pub fn default_icon_map() -> BTreeMap<String, String> {
    use editor_icons::symbols::{md, oct};
    let mut map = BTreeMap::new();
    map.insert("atx_h1_marker".into(), md::MD_FORMAT_HEADER_1.into());
    map.insert("atx_h2_marker".into(), md::MD_FORMAT_HEADER_2.into());
    map.insert("atx_h3_marker".into(), md::MD_FORMAT_HEADER_3.into());
    map.insert("atx_h4_marker".into(), md::MD_FORMAT_HEADER_4.into());
    map.insert("atx_h5_marker".into(), md::MD_FORMAT_HEADER_5.into());
    map.insert("atx_h6_marker".into(), md::MD_FORMAT_HEADER_6.into());
    map.insert(
        "list_marker_minus".into(),
        md::MD_FORMAT_LIST_BULLETED.into(),
    );
    map.insert(
        "list_marker_plus".into(),
        md::MD_FORMAT_LIST_BULLETED.into(),
    );
    map.insert(
        "list_marker_star".into(),
        md::MD_FORMAT_LIST_BULLETED.into(),
    );
    map.insert("list_marker_dot".into(), md::MD_FORMAT_LIST_NUMBERED.into());
    map.insert(
        "list_marker_parenthesis".into(),
        md::MD_FORMAT_LIST_NUMBERED.into(),
    );
    map.insert(
        "task_list_marker_unchecked".into(),
        md::MD_CHECKBOX_BLANK_OUTLINE.into(),
    );
    map.insert(
        "task_list_marker_checked".into(),
        md::MD_CHECKBOX_MARKED.into(),
    );
    map.insert("thematic_break".into(), oct::OCT_HORIZONTAL_RULE.into());
    map.insert("image".into(), md::MD_IMAGE.into());
    map.insert("inline_link".into(), md::MD_LINK.into());
    map.insert("full_reference_link".into(), md::MD_LINK.into());
    map.insert("collapsed_reference_link".into(), md::MD_LINK.into());
    map.insert("shortcut_link".into(), md::MD_LINK.into());
    map.insert("uri_autolink".into(), md::MD_LINK.into());
    map.insert("email_autolink".into(), md::MD_LINK.into());
    map
}

/// Byte/char range to hide while Pretty is active on a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConcealRange {
    /// Inclusive start column (chars).
    pub start_col: usize,
    /// Exclusive end column (chars).
    pub end_col: usize,
}

/// Icon drawn at the start of a line (or before a link label).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrettyIcon {
    pub kind: String,
    pub glyph: String,
    /// Column where the icon replaces/prefixes content.
    pub at_col: usize,
}

/// Inline image reference discovered in source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrettyImage {
    pub line_index: usize,
    pub alt: String,
    pub destination: ImageDestination,
    /// Source columns covering `![...](...)`.
    pub source_cols: Range<usize>,
}

/// Where an embedded image should be loaded from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImageDestination {
    Local(PathBuf),
    Https(String),
    DataUrl(String),
    Unsupported(String),
}

/// Per-line Pretty decorations.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PrettyLinePlan {
    pub icons: Vec<PrettyIcon>,
    pub conceal: Vec<ConcealRange>,
    pub image: Option<PrettyImage>,
    /// When set, the whole line is replaced by a thematic-break glyph run.
    pub horizontal_rule: bool,
}

/// Full-buffer / string Pretty plan.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarkdownPrettyPlan {
    pub lines: BTreeMap<usize, PrettyLinePlan>,
    pub skipped_by_kill_switch: bool,
}

/// Context for resolving relative image paths and anti-conceal.
#[derive(Debug, Clone)]
pub struct MarkdownPrettyRequest<'a> {
    pub text: &'a str,
    pub config: &'a MarkdownPrettyConfig,
    /// Sticky/per-buffer override; `None` means use config.enabled.
    pub buffer_enabled: Option<bool>,
    pub buffer_path: Option<&'a Path>,
    pub workspace_root: Option<&'a Path>,
    /// Cursor line for Anti-conceal (raw).
    pub cursor_line: Option<usize>,
    /// Visual selection line range inclusive, when active.
    pub visual_lines: Option<Range<usize>>,
    /// Visible window for planning (files/ACP); `None` = whole string (hover).
    pub visible_lines: Option<Range<usize>>,
}

impl MarkdownPrettyPlan {
    /// Returns whether Pretty chrome should paint on `line_index`.
    pub fn line_is_anti_concealed(
        &self,
        request: &MarkdownPrettyRequest<'_>,
        line_index: usize,
    ) -> bool {
        if let Some(cursor) = request.cursor_line
            && cursor == line_index
        {
            return true;
        }
        if let Some(range) = &request.visual_lines
            && line_index >= range.start
            && line_index <= range.end
        {
            return true;
        }
        false
    }

    pub fn line(&self, line_index: usize) -> Option<&PrettyLinePlan> {
        self.lines.get(&line_index)
    }
}

/// Plan Pretty decorations for markdown source.
pub fn plan_markdown_pretty(
    request: &MarkdownPrettyRequest<'_>,
    registry: Option<&mut SyntaxRegistry>,
) -> MarkdownPrettyPlan {
    if !request.buffer_enabled.unwrap_or(request.config.enabled) {
        return MarkdownPrettyPlan::default();
    }

    let line_count = request.text.lines().count();
    let byte_count = request.text.len();
    if request.config.kill_switch_enabled
        && (line_count > request.config.kill_switch_max_lines
            || byte_count > request.config.kill_switch_max_bytes)
    {
        return MarkdownPrettyPlan {
            lines: BTreeMap::new(),
            skipped_by_kill_switch: true,
        };
    }

    let window = request
        .visible_lines
        .clone()
        .unwrap_or(0..line_count.saturating_add(1));

    if let Some(registry) = registry {
        let buffer = TextBuffer::from_text(request.text);
        if let Ok(nodes) = registry.structure_nodes_for_language("markdown", &buffer) {
            return plan_from_structure_nodes(request, &nodes, window);
        }
    }

    plan_from_line_scanner(request, window)
}

fn plan_from_structure_nodes(
    request: &MarkdownPrettyRequest<'_>,
    nodes: &[SyntaxStructureNode],
    window: Range<usize>,
) -> MarkdownPrettyPlan {
    let mut plan = MarkdownPrettyPlan::default();
    let lines: Vec<&str> = request.text.lines().collect();

    for node in nodes {
        let line_index = node.start_position.line;
        if line_index < window.start || line_index >= window.end {
            continue;
        }
        let Some(line) = lines.get(line_index).copied() else {
            continue;
        };
        let line_plan = plan.lines.entry(line_index).or_default();
        apply_structure_node(request, line_plan, node, line);
    }
    plan
}

fn apply_structure_node(
    request: &MarkdownPrettyRequest<'_>,
    line_plan: &mut PrettyLinePlan,
    node: &SyntaxStructureNode,
    line: &str,
) {
    let kind = node.kind.as_str();
    let start_col = node.start_position.column;
    let end_col = if node.end_position.line == node.start_position.line {
        node.end_position.column.min(line.chars().count())
    } else {
        line.chars().count()
    };

    if let Some(glyph) = request.config.icons.get(kind) {
        match kind {
            "thematic_break" => {
                line_plan.horizontal_rule = true;
                line_plan.conceal.push(ConcealRange {
                    start_col: 0,
                    end_col: line.chars().count(),
                });
                line_plan.icons.push(PrettyIcon {
                    kind: kind.to_owned(),
                    glyph: glyph.clone(),
                    at_col: 0,
                });
            }
            "image" => {
                if let Some(image) = parse_image_from_line(request, node.start_position.line, line)
                {
                    line_plan.image = Some(image);
                    line_plan.conceal.push(ConcealRange { start_col, end_col });
                    line_plan.icons.push(PrettyIcon {
                        kind: kind.to_owned(),
                        glyph: glyph.clone(),
                        at_col: start_col,
                    });
                }
            }
            k if k.starts_with("atx_h") && k.ends_with("_marker") => {
                line_plan.conceal.push(ConcealRange { start_col, end_col });
                // Also conceal trailing spaces after marker up to content.
                let after = end_col;
                let rest: String = line.chars().skip(after).collect();
                let spaces = rest.chars().take_while(|c| *c == ' ').count();
                if spaces > 0 {
                    line_plan.conceal.push(ConcealRange {
                        start_col: after,
                        end_col: after + spaces,
                    });
                }
                line_plan.icons.push(PrettyIcon {
                    kind: kind.to_owned(),
                    glyph: glyph.clone(),
                    at_col: start_col,
                });
            }
            k if k.starts_with("list_marker_") || k.starts_with("task_list_marker_") => {
                line_plan.conceal.push(ConcealRange { start_col, end_col });
                line_plan.icons.push(PrettyIcon {
                    kind: kind.to_owned(),
                    glyph: glyph.clone(),
                    at_col: start_col,
                });
            }
            "inline_link"
            | "full_reference_link"
            | "collapsed_reference_link"
            | "shortcut_link"
            | "uri_autolink"
            | "email_autolink" => {
                apply_link_pretty(line_plan, kind, glyph, line, start_col, end_col);
            }
            _ => {}
        }
    }
}

fn apply_link_pretty(
    line_plan: &mut PrettyLinePlan,
    kind: &str,
    glyph: &str,
    line: &str,
    start_col: usize,
    end_col: usize,
) {
    let segment: String = line
        .chars()
        .skip(start_col)
        .take(end_col.saturating_sub(start_col))
        .collect();
    line_plan.icons.push(PrettyIcon {
        kind: kind.to_owned(),
        glyph: glyph.to_owned(),
        at_col: start_col,
    });
    if let Some((label_end, dest_start)) = link_label_and_dest_cols(&segment) {
        // Conceal `[` `]` `(` url `)` around label; keep label text.
        // segment-relative → line cols.
        if label_end > 1 {
            line_plan.conceal.push(ConcealRange {
                start_col,
                end_col: start_col + 1,
            });
            line_plan.conceal.push(ConcealRange {
                start_col: start_col + label_end.saturating_sub(1),
                end_col: start_col + label_end,
            });
        }
        if dest_start < segment.chars().count() {
            line_plan.conceal.push(ConcealRange {
                start_col: start_col + dest_start,
                end_col,
            });
        }
    } else {
        // Autolink / bare: keep short prefix, conceal rest lightly by icon only.
    }
}

fn link_label_and_dest_cols(segment: &str) -> Option<(usize, usize)> {
    // [label](dest) → (end_of_label_incl_bracket, start_of_( )
    if !segment.starts_with('[') {
        return None;
    }
    let mut chars = segment.char_indices();
    let _ = chars.next()?;
    let mut depth = 1usize;
    let mut label_end = None;
    for (idx, ch) in chars.by_ref() {
        match ch {
            '[' => depth = depth.saturating_add(1),
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    label_end = Some(segment[..idx + ch.len_utf8()].chars().count());
                    break;
                }
            }
            _ => {}
        }
    }
    let label_end = label_end?;
    let _after: String = segment.chars().skip(label_end).collect();
    Some((label_end, label_end))
}

fn plan_from_line_scanner(
    request: &MarkdownPrettyRequest<'_>,
    window: Range<usize>,
) -> MarkdownPrettyPlan {
    let mut plan = MarkdownPrettyPlan::default();
    for (line_index, line) in request.text.lines().enumerate() {
        if line_index < window.start || line_index >= window.end {
            continue;
        }
        let mut line_plan = PrettyLinePlan::default();
        scan_line(request, line_index, line, &mut line_plan);
        if line_plan != PrettyLinePlan::default() {
            plan.lines.insert(line_index, line_plan);
        }
    }
    plan
}

fn scan_line(
    request: &MarkdownPrettyRequest<'_>,
    line_index: usize,
    line: &str,
    line_plan: &mut PrettyLinePlan,
) {
    let trimmed = line.trim_start();
    let indent = line.chars().count() - trimmed.chars().count();

    if is_thematic_break(trimmed) {
        if let Some(glyph) = request.config.icons.get("thematic_break") {
            line_plan.horizontal_rule = true;
            line_plan.conceal.push(ConcealRange {
                start_col: 0,
                end_col: line.chars().count(),
            });
            line_plan.icons.push(PrettyIcon {
                kind: "thematic_break".into(),
                glyph: glyph.clone(),
                at_col: 0,
            });
        }
        return;
    }

    if let Some((level, marker_len)) = atx_heading_marker(trimmed) {
        let kind = format!("atx_h{level}_marker");
        if let Some(glyph) = request.config.icons.get(&kind) {
            let spaces = trimmed
                .chars()
                .skip(marker_len)
                .take_while(|c| *c == ' ')
                .count();
            line_plan.conceal.push(ConcealRange {
                start_col: indent,
                end_col: indent + marker_len + spaces,
            });
            line_plan.icons.push(PrettyIcon {
                kind,
                glyph: glyph.clone(),
                at_col: indent,
            });
        }
    }

    if let Some((kind, marker_len)) = list_marker(trimmed) {
        if let Some(glyph) = request.config.icons.get(kind) {
            line_plan.conceal.push(ConcealRange {
                start_col: indent,
                end_col: indent + marker_len,
            });
            line_plan.icons.push(PrettyIcon {
                kind: kind.to_owned(),
                glyph: glyph.clone(),
                at_col: indent,
            });
        }
        let after_marker: String = trimmed.chars().skip(marker_len).collect();
        let task = after_marker.trim_start();
        let task_indent = after_marker.chars().count() - task.chars().count();
        if let Some((task_kind, task_len)) = task_marker(task)
            && let Some(glyph) = request.config.icons.get(task_kind)
        {
            let at = indent + marker_len + task_indent;
            line_plan.conceal.push(ConcealRange {
                start_col: at,
                end_col: at + task_len,
            });
            line_plan.icons.push(PrettyIcon {
                kind: task_kind.to_owned(),
                glyph: glyph.clone(),
                at_col: at,
            });
        }
    }

    if let Some(image) = parse_image_from_line(request, line_index, line) {
        let cols = image.source_cols.clone();
        if let Some(glyph) = request.config.icons.get("image") {
            line_plan.icons.push(PrettyIcon {
                kind: "image".into(),
                glyph: glyph.clone(),
                at_col: cols.start,
            });
        }
        line_plan.conceal.push(ConcealRange {
            start_col: cols.start,
            end_col: cols.end,
        });
        line_plan.image = Some(image);
    }

    scan_inline_links(request, line, line_plan);
}

fn scan_inline_links(
    request: &MarkdownPrettyRequest<'_>,
    line: &str,
    line_plan: &mut PrettyLinePlan,
) {
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '[' {
            // image — handled elsewhere
            i += 1;
            continue;
        }
        if chars[i] == '['
            && let Some((end, label_end, dest_start)) = match_inline_link(&chars, i)
        {
            if let Some(glyph) = request.config.icons.get("inline_link") {
                line_plan.icons.push(PrettyIcon {
                    kind: "inline_link".into(),
                    glyph: glyph.clone(),
                    at_col: i,
                });
            }
            line_plan.conceal.push(ConcealRange {
                start_col: i,
                end_col: i + 1,
            });
            if label_end > i + 1 {
                line_plan.conceal.push(ConcealRange {
                    start_col: label_end.saturating_sub(1),
                    end_col: label_end,
                });
            }
            line_plan.conceal.push(ConcealRange {
                start_col: dest_start,
                end_col: end,
            });
            i = end;
            continue;
        }
        i += 1;
    }
}

fn match_inline_link(chars: &[char], start: usize) -> Option<(usize, usize, usize)> {
    if chars.get(start) != Some(&'[') {
        return None;
    }
    let mut depth = 1usize;
    let mut j = start + 1;
    let mut label_end = None;
    while j < chars.len() {
        match chars[j] {
            '[' => depth += 1,
            ']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    label_end = Some(j + 1);
                    break;
                }
            }
            _ => {}
        }
        j += 1;
    }
    let label_end = label_end?;
    if chars.get(label_end) != Some(&'(') {
        return None;
    }
    let mut k = label_end + 1;
    let mut paren = 1usize;
    while k < chars.len() {
        match chars[k] {
            '(' => paren += 1,
            ')' => {
                paren = paren.saturating_sub(1);
                if paren == 0 {
                    return Some((k + 1, label_end, label_end));
                }
            }
            _ => {}
        }
        k += 1;
    }
    None
}

fn parse_image_from_line(
    request: &MarkdownPrettyRequest<'_>,
    line_index: usize,
    line: &str,
) -> Option<PrettyImage> {
    let chars: Vec<char> = line.chars().collect();
    let start = chars.iter().position(|c| *c == '!')?;
    if chars.get(start + 1) != Some(&'[') {
        return None;
    }
    let (end, label_end, dest_start) = match_inline_link(&chars, start + 1)?;
    // match_inline_link expects `[` at start; we passed start+1 which is `[`
    let alt: String = chars[start + 2..label_end.saturating_sub(1)]
        .iter()
        .collect();
    let dest: String = chars[dest_start + 1..end.saturating_sub(1)]
        .iter()
        .collect();
    let destination =
        resolve_image_destination(dest.trim(), request.buffer_path, request.workspace_root);
    Some(PrettyImage {
        line_index,
        alt,
        destination,
        source_cols: start..end,
    })
}

/// Resolve an image destination string to a loadable target.
pub fn resolve_image_destination(
    raw: &str,
    buffer_path: Option<&Path>,
    workspace_root: Option<&Path>,
) -> ImageDestination {
    let raw = raw.trim();
    if raw.is_empty() {
        return ImageDestination::Unsupported(raw.to_owned());
    }
    if raw.starts_with("data:") {
        return ImageDestination::DataUrl(raw.to_owned());
    }
    if raw.starts_with("https://") || raw.starts_with("http://") {
        if raw.starts_with("https://") {
            return ImageDestination::Https(raw.to_owned());
        }
        return ImageDestination::Unsupported(raw.to_owned());
    }
    let path = PathBuf::from(raw.trim_start_matches("./").trim_start_matches(".\\"));
    if path.is_absolute() {
        return ImageDestination::Local(path);
    }
    if let Some(buffer_path) = buffer_path
        && let Some(parent) = buffer_path.parent()
    {
        return ImageDestination::Local(parent.join(&path));
    }
    if let Some(root) = workspace_root {
        return ImageDestination::Local(root.join(&path));
    }
    ImageDestination::Local(path)
}

fn is_thematic_break(trimmed: &str) -> bool {
    let t = trimmed.trim_end();
    if t.len() < 3 {
        return false;
    }
    let chars: Vec<char> = t.chars().collect();
    let first = chars[0];
    if !matches!(first, '-' | '_' | '*') {
        return false;
    }
    chars.iter().all(|c| *c == first || *c == ' ')
        && chars.iter().filter(|c| **c == first).count() >= 3
}

fn atx_heading_marker(trimmed: &str) -> Option<(u8, usize)> {
    let mut level = 0u8;
    for ch in trimmed.chars() {
        if ch == '#' && level < 6 {
            level += 1;
        } else {
            break;
        }
    }
    if level == 0 {
        return None;
    }
    let rest: String = trimmed.chars().skip(level as usize).collect();
    if rest.is_empty() || rest.starts_with(' ') {
        Some((level, level as usize))
    } else {
        None
    }
}

fn list_marker(trimmed: &str) -> Option<(&'static str, usize)> {
    let mut chars = trimmed.chars().peekable();
    match chars.peek().copied()? {
        '-' | '+' | '*' => {
            let marker = chars.next()?;
            if chars.peek() == Some(&' ') || chars.peek().is_none() {
                let kind = match marker {
                    '-' => "list_marker_minus",
                    '+' => "list_marker_plus",
                    _ => "list_marker_star",
                };
                return Some((kind, 2.min(trimmed.chars().count())));
            }
        }
        '0'..='9' => {
            let mut len = 0usize;
            while matches!(chars.peek(), Some('0'..='9')) {
                chars.next();
                len += 1;
            }
            match chars.next() {
                Some('.') if chars.peek() == Some(&' ') || chars.peek().is_none() => {
                    return Some(("list_marker_dot", len + 2));
                }
                Some(')') if chars.peek() == Some(&' ') || chars.peek().is_none() => {
                    return Some(("list_marker_parenthesis", len + 2));
                }
                _ => {}
            }
        }
        _ => {}
    }
    None
}

fn task_marker(trimmed: &str) -> Option<(&'static str, usize)> {
    let lower = trimmed.to_ascii_lowercase();
    if lower.starts_with("[ ]") {
        Some(("task_list_marker_unchecked", 3))
    } else if lower.starts_with("[x]") {
        Some(("task_list_marker_checked", 3))
    } else {
        None
    }
}

/// Display text for a pretty line with conceal applied (icons not inserted).
pub fn conceal_line_text(line: &str, conceal: &[ConcealRange]) -> String {
    if conceal.is_empty() {
        return line.to_owned();
    }
    let mut ranges = conceal.to_vec();
    ranges.sort_by_key(|r| r.start_col);
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::new();
    let mut col = 0usize;
    for range in ranges {
        let start = range.start_col.min(chars.len());
        let end = range.end_col.min(chars.len());
        if col < start {
            out.extend(chars[col..start].iter());
        }
        col = col.max(end);
    }
    if col < chars.len() {
        out.extend(chars[col..].iter());
    }
    out
}

/// Prefix icons for a line in column order.
pub fn line_icon_prefix(plan: &PrettyLinePlan) -> String {
    let mut icons = plan.icons.clone();
    icons.sort_by_key(|icon| icon.at_col);
    let mut seen = std::collections::BTreeSet::new();
    let mut out = String::new();
    for icon in icons {
        if seen.insert(icon.at_col) {
            out.push_str(&icon.glyph);
            out.push(' ');
        }
    }
    out
}

/// Point helper re-export for callers mapping structure nodes.
pub type PrettyPoint = SyntaxPoint;

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> MarkdownPrettyConfig {
        MarkdownPrettyConfig::default()
    }

    #[test]
    fn plans_headings_lists_hr_links_and_images() {
        let text = "\
# Title
- item
- [ ] todo
---
![alt](./pic.png)
See [docs](https://example.com).
";
        let config = cfg();
        let request = MarkdownPrettyRequest {
            text,
            config: &config,
            buffer_enabled: None,
            buffer_path: Some(Path::new("P:/proj/README.md")),
            workspace_root: Some(Path::new("P:/proj")),
            cursor_line: None,
            visual_lines: None,
            visible_lines: None,
        };
        let plan = plan_markdown_pretty(&request, None);
        assert!(plan.line(0).is_some_and(|l| !l.icons.is_empty()));
        assert!(plan.line(1).is_some_and(|l| !l.icons.is_empty()));
        assert!(plan.line(2).is_some_and(|l| {
            l.icons
                .iter()
                .any(|i| i.kind == "task_list_marker_unchecked")
        }));
        assert!(plan.line(3).is_some_and(|l| l.horizontal_rule));
        let image = plan.line(4).and_then(|l| l.image.clone());
        assert!(matches!(
            image.map(|i| i.destination),
            Some(ImageDestination::Local(path)) if path.ends_with("pic.png")
        ));
        assert!(plan.line(5).is_some_and(|l| {
            l.icons.iter().any(|i| i.kind == "inline_link") && !l.conceal.is_empty()
        }));
    }

    #[test]
    fn kill_switch_skips_when_enabled() {
        let text = "# hi\n";
        let mut config = cfg();
        config.kill_switch_enabled = true;
        config.kill_switch_max_lines = 0;
        let request = MarkdownPrettyRequest {
            text,
            config: &config,
            buffer_enabled: None,
            buffer_path: None,
            workspace_root: None,
            cursor_line: None,
            visual_lines: None,
            visible_lines: None,
        };
        let plan = plan_markdown_pretty(&request, None);
        assert!(plan.skipped_by_kill_switch);
        assert!(plan.lines.is_empty());
    }

    #[test]
    fn anti_conceal_detects_cursor_and_visual() {
        let plan = MarkdownPrettyPlan::default();
        let config = cfg();
        let request = MarkdownPrettyRequest {
            text: "a\nb\nc\n",
            config: &config,
            buffer_enabled: None,
            buffer_path: None,
            workspace_root: None,
            cursor_line: Some(1),
            visual_lines: Some(2..2),
            visible_lines: None,
        };
        assert!(plan.line_is_anti_concealed(&request, 1));
        assert!(plan.line_is_anti_concealed(&request, 2));
        assert!(!plan.line_is_anti_concealed(&request, 0));
    }

    #[test]
    fn resolve_relative_dot_slash_from_buffer_dir() {
        let dest = resolve_image_destination(
            "./crates/volt/assets/banner.png",
            Some(Path::new("P:/volt/README.md")),
            None,
        );
        match dest {
            ImageDestination::Local(path) => {
                assert!(
                    path.ends_with("crates/volt/assets/banner.png")
                        || path.ends_with(r"crates\volt\assets\banner.png"),
                    "unexpected path: {}",
                    path.display()
                );
            }
            other => panic!("expected local path, got {other:?}"),
        }
    }
}
