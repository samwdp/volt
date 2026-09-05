#[derive(Debug, Clone)]
struct UndoSnapshot {
    text: TextSnapshot,
    cursor: TextPoint,
}

impl UndoSnapshot {
    fn from_buffer(buffer: &TextBuffer) -> Self {
        Self {
            text: buffer.snapshot(),
            cursor: buffer.cursor(),
        }
    }

    fn diff_preview(&self, parent: Option<&Self>) -> Option<String> {
        let Some(parent) = parent else {
            let cursor_line = self
                .text
                .line(self.cursor.line)
                .unwrap_or_default()
                .trim_end()
                .to_owned();
            return Some(format!(
                "(root snapshot)\nline {}, col {}\n{}",
                self.cursor.line + 1,
                self.cursor.column + 1,
                cursor_line
            ));
        };
        let before_lines = snapshot_lines(&parent.text);
        let after_lines = snapshot_lines(&self.text);
        if before_lines == after_lines {
            let cursor_line = self
                .text
                .line(self.cursor.line)
                .unwrap_or_default()
                .trim_end()
                .to_owned();
            return Some(format!(
                "cursor → line {}, col {}\n{}",
                self.cursor.line + 1,
                self.cursor.column + 1,
                cursor_line
            ));
        }
        Some(format_undo_snapshot_diff(&before_lines, &after_lines))
    }

    const fn cursor(&self) -> TextPoint {
        self.cursor
    }

    fn set_cursor(&mut self, cursor: TextPoint) {
        self.cursor = cursor;
    }
}

const UNDO_TREE_DIFF_PREVIEW_LINES: usize = 48;
const UNDO_TREE_DIFF_CONTEXT_LINES: usize = 2;

fn snapshot_lines(snapshot: &TextSnapshot) -> Vec<String> {
    (0..snapshot.line_count())
        .map(|index| snapshot.line(index).unwrap_or_default())
        .collect()
}

fn format_undo_snapshot_diff(before: &[String], after: &[String]) -> String {
    let prefix = before
        .iter()
        .zip(after.iter())
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = before
        .iter()
        .skip(prefix)
        .rev()
        .zip(after.iter().skip(prefix).rev())
        .take_while(|(left, right)| left == right)
        .count()
        .min(before.len().saturating_sub(prefix))
        .min(after.len().saturating_sub(prefix));
    let before_end = before.len().saturating_sub(suffix);
    let after_end = after.len().saturating_sub(suffix);
    let context_start = prefix.saturating_sub(UNDO_TREE_DIFF_CONTEXT_LINES);
    let context_end = (before_end + UNDO_TREE_DIFF_CONTEXT_LINES)
        .max(after_end + UNDO_TREE_DIFF_CONTEXT_LINES)
        .min(before.len().max(after.len()));

    let old_count = before_end.saturating_sub(prefix);
    let new_count = after_end.saturating_sub(prefix);
    let mut lines = vec![format!(
        "@@ -{},{} +{},{} @@",
        prefix + 1,
        old_count.max(1),
        prefix + 1,
        new_count.max(1)
    )];
    for line in &before[context_start..prefix] {
        lines.push(format!(" {line}"));
    }
    for line in &before[prefix..before_end] {
        lines.push(format!("-{line}"));
    }
    for line in &after[prefix..after_end] {
        lines.push(format!("+{line}"));
    }
    let trailing_end = context_end.min(after.len()).max(after_end);
    for line in &after[after_end..trailing_end] {
        lines.push(format!(" {line}"));
    }
    if lines.len() > UNDO_TREE_DIFF_PREVIEW_LINES {
        lines.truncate(UNDO_TREE_DIFF_PREVIEW_LINES);
        lines.push("…".to_owned());
    }
    lines.join("\n")
}

fn undo_tree_fringe(depth: usize, is_last: bool, is_current: bool, continues: &[bool]) -> String {
    let mut fringe = String::new();
    for &cont in continues {
        fringe.push_str(if cont { "│ " } else { "  " });
    }
    if depth > 0 {
        fringe.push_str(if is_last { "└─" } else { "├─" });
    }
    fringe.push(if is_current { '*' } else { '○' });
    fringe.push(' ');
    fringe
}

#[derive(Debug, Clone)]
struct UndoNode {
    parent: Option<usize>,
    children: Vec<usize>,
    snapshot: UndoSnapshot,
    sequence: u64,
    last_child: Option<usize>,
}

#[derive(Debug, Clone)]
struct UndoTree {
    nodes: Vec<UndoNode>,
    current: usize,
    next_sequence: u64,
    last_revision: u64,
}

#[derive(Debug, Clone)]
struct UndoTreeEntry {
    node_id: usize,
    fringe: String,
    label: String,
    detail: String,
    preview: Option<String>,
}

impl UndoTree {
    fn new(buffer: &TextBuffer) -> Self {
        let snapshot = UndoSnapshot::from_buffer(buffer);
        Self {
            nodes: vec![UndoNode {
                parent: None,
                children: Vec::new(),
                snapshot,
                sequence: 0,
                last_child: None,
            }],
            current: 0,
            next_sequence: 1,
            last_revision: buffer.revision(),
        }
    }

    fn update_revision(&mut self, revision: u64) {
        self.last_revision = revision;
    }

    fn preserve_root_cursor(&mut self, cursor: TextPoint, revision: u64) {
        if self.current != 0 || revision != self.last_revision {
            return;
        }
        if let Some(root) = self.nodes.first_mut() {
            root.snapshot.set_cursor(cursor);
        }
    }

    fn record_snapshot(&mut self, buffer: &TextBuffer) -> bool {
        let revision = buffer.revision();
        if revision == self.last_revision {
            return false;
        }
        let snapshot = UndoSnapshot::from_buffer(buffer);
        let parent = self.current;
        let node_id = self.nodes.len();
        let sequence = self.next_sequence;
        self.nodes.push(UndoNode {
            parent: Some(parent),
            children: Vec::new(),
            snapshot,
            sequence,
            last_child: None,
        });
        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.children.push(node_id);
            parent_node.last_child = Some(node_id);
        }
        self.current = node_id;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.last_revision = revision;
        true
    }

    fn undo(&mut self) -> Option<UndoSnapshot> {
        let parent = self.nodes.get(self.current)?.parent?;
        let current = self.current;
        if let Some(parent_node) = self.nodes.get_mut(parent) {
            parent_node.last_child = Some(current);
        }
        self.current = parent;
        self.nodes
            .get(self.current)
            .map(|node| node.snapshot.clone())
    }

    fn redo(&mut self, cursor: TextPoint, revision: u64) -> Option<UndoSnapshot> {
        self.preserve_root_cursor(cursor, revision);
        let next = {
            let node = self.nodes.get(self.current)?;
            node.last_child.or_else(|| node.children.last().copied())
        }?;
        self.current = next;
        self.nodes
            .get(self.current)
            .map(|node| node.snapshot.clone())
    }

    fn select(&mut self, node_id: usize, cursor: TextPoint, revision: u64) -> Option<UndoSnapshot> {
        if node_id >= self.nodes.len() {
            return None;
        }
        self.preserve_root_cursor(cursor, revision);
        if let Some(parent) = self.nodes[node_id].parent
            && let Some(parent_node) = self.nodes.get_mut(parent)
        {
            parent_node.last_child = Some(node_id);
        }
        self.current = node_id;
        self.nodes.get(node_id).map(|node| node.snapshot.clone())
    }

    fn picker_entries(&self) -> (Vec<UndoTreeEntry>, usize) {
        let mut entries = Vec::new();
        let mut selected_index = None;
        if !self.nodes.is_empty() {
            self.collect_entries(0, 0, true, &[], &mut entries, &mut selected_index);
        }
        (entries, selected_index.unwrap_or(0))
    }

    fn collect_entries(
        &self,
        node_id: usize,
        depth: usize,
        is_last: bool,
        continues: &[bool],
        entries: &mut Vec<UndoTreeEntry>,
        selected_index: &mut Option<usize>,
    ) {
        let Some(node) = self.nodes.get(node_id) else {
            return;
        };
        let is_current = node_id == self.current;
        if is_current {
            *selected_index = Some(entries.len());
        }
        let fringe = undo_tree_fringe(depth, is_last, is_current, continues);
        let cursor = node.snapshot.cursor();
        let label = if node.parent.is_none() {
            format!("root  line {}, col {}", cursor.line + 1, cursor.column + 1)
        } else {
            format!(
                "{}  line {}, col {}",
                node.sequence,
                cursor.line + 1,
                cursor.column + 1
            )
        };
        let detail = if is_current {
            format!("current | children: {}", node.children.len())
        } else if node.parent.is_none() {
            format!("root | children: {}", node.children.len())
        } else {
            format!("children: {}", node.children.len())
        };
        let parent_snapshot = node
            .parent
            .and_then(|parent_id| self.nodes.get(parent_id))
            .map(|parent| &parent.snapshot);
        entries.push(UndoTreeEntry {
            node_id,
            fringe,
            label,
            detail,
            preview: node.snapshot.diff_preview(parent_snapshot),
        });
        let child_count = node.children.len();
        for (index, child) in node.children.iter().enumerate() {
            let child_is_last = index + 1 == child_count;
            let mut child_continues = continues.to_vec();
            if depth > 0 {
                child_continues.push(!is_last);
            }
            self.collect_entries(
                *child,
                depth + 1,
                child_is_last,
                &child_continues,
                entries,
                selected_index,
            );
        }
    }
}
