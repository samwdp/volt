use std::collections::BTreeSet;

/// Action metadata attached to a section line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionAction {
    id: String,
    detail: Option<String>,
}

impl SectionAction {
    /// Creates a new action identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            detail: None,
        }
    }

    /// Adds a detail payload to the action.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Returns the action identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the optional action detail payload.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

/// One line of content within a section.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionItem {
    text: String,
    action: Option<SectionAction>,
}

impl SectionItem {
    /// Creates a section item with display text.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            action: None,
        }
    }

    /// Adds an action to the item.
    pub fn with_action(mut self, action: SectionAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Returns the display text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Returns the optional action.
    pub fn action(&self) -> Option<&SectionAction> {
        self.action.as_ref()
    }
}

/// A section in a sectioned buffer tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Section {
    id: String,
    title: String,
    items: Vec<SectionItem>,
    children: Vec<Section>,
}

impl Section {
    /// Creates a new section with an identifier and title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            items: Vec::new(),
            children: Vec::new(),
        }
    }

    /// Appends items to the section.
    pub fn with_items(mut self, items: Vec<SectionItem>) -> Self {
        self.items = items;
        self
    }

    /// Appends child sections.
    pub fn with_children(mut self, children: Vec<Section>) -> Self {
        self.children = children;
        self
    }

    /// Pushes a single item into the section.
    pub fn push_item(&mut self, item: SectionItem) {
        self.items.push(item);
    }

    /// Pushes a child section into this section.
    pub fn push_child(&mut self, child: Section) {
        self.children.push(child);
    }

    /// Returns the section identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the display title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the section items.
    pub fn items(&self) -> &[SectionItem] {
        &self.items
    }

    /// Returns the child sections.
    pub fn children(&self) -> &[Section] {
        &self.children
    }
}

/// Root container for sectioned buffer content.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SectionTree {
    sections: Vec<Section>,
}

impl SectionTree {
    /// Creates a new tree from top-level sections.
    pub fn new(sections: Vec<Section>) -> Self {
        Self { sections }
    }

    /// Returns the top-level sections.
    pub fn sections(&self) -> &[Section] {
        &self.sections
    }

    /// Renders the tree into display lines, honoring collapsed sections.
    pub fn render_lines(&self, state: &SectionCollapseState) -> Vec<SectionRenderLine> {
        let mut lines = Vec::new();
        for (index, section) in self.sections.iter().enumerate() {
            if index > 0 {
                lines.push(SectionRenderLine {
                    text: String::new(),
                    depth: 0,
                    section_id: String::new(),
                    action: None,
                    kind: SectionRenderLineKind::Spacer,
                });
            }
            render_section(section, 0, state, &mut lines);
        }
        lines
    }
}

fn render_section(
    section: &Section,
    depth: usize,
    state: &SectionCollapseState,
    lines: &mut Vec<SectionRenderLine>,
) {
    let collapsed = state.is_collapsed(section.id());
    lines.push(SectionRenderLine {
        text: section.title().to_owned(),
        depth,
        section_id: section.id().to_owned(),
        action: None,
        kind: SectionRenderLineKind::Header {
            id: section.id().to_owned(),
            collapsed,
        },
    });

    if collapsed {
        return;
    }

    let item_depth = depth.saturating_add(1);
    for item in section.items() {
        lines.push(SectionRenderLine {
            text: item.text().to_owned(),
            depth: item_depth,
            section_id: section.id().to_owned(),
            action: item.action().cloned(),
            kind: SectionRenderLineKind::Item,
        });
    }

    let child_depth = depth.saturating_add(1);
    for child in section.children() {
        render_section(child, child_depth, state, lines);
    }
}

/// Tracks collapsed section state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SectionCollapseState {
    collapsed: BTreeSet<String>,
}

impl SectionCollapseState {
    /// Returns true if the section is collapsed.
    pub fn is_collapsed(&self, id: &str) -> bool {
        self.collapsed.contains(id)
    }

    /// Toggles a section's collapsed state, returning the new state.
    pub fn toggle(&mut self, id: &str) -> bool {
        if self.collapsed.remove(id) {
            false
        } else {
            self.collapsed.insert(id.to_owned());
            true
        }
    }
}

/// One line of rendered output from a sectioned buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionRenderLine {
    pub text: String,
    pub depth: usize,
    pub section_id: String,
    pub action: Option<SectionAction>,
    pub kind: SectionRenderLineKind,
}

/// Kind of rendered line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionRenderLineKind {
    Header { id: String, collapsed: bool },
    Item,
    Spacer,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_lines_respects_collapsed_state() {
        let mut child = Section::new("child", "Child");
        child.push_item(SectionItem::new("child item"));
        let mut root = Section::new("root", "Root");
        root.push_item(SectionItem::new("root item"));
        root.push_child(child);
        let tree = SectionTree::new(vec![root]);

        let lines = tree.render_lines(&SectionCollapseState::default());
        assert_eq!(lines.len(), 4);

        let mut state = SectionCollapseState::default();
        state.toggle("root");
        let collapsed = tree.render_lines(&state);
        assert_eq!(collapsed.len(), 1);
        assert!(matches!(
            collapsed[0].kind,
            SectionRenderLineKind::Header { .. }
        ));
    }
}
