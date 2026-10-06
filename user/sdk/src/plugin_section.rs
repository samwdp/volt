//! Magit-style section-tree plugin buffers.
//!
//! Distinct from [`crate::PluginBufferSections`] (split panes). A section-tree
//! buffer renders a [`crate::SectionTree`] with collapse state and per-line
//! [`crate::SectionAction`] metadata.

use abi_stable::{
    StableAbi,
    std_types::{ROption, RString},
};
use serde::{Deserialize, Serialize};

use crate::abi::AbiSectionTree;
use crate::{SectionTree, sections::SectionCollapseState};

/// How a [`crate::PluginBuffer`] presents content in the host.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi, Default)]
pub enum PluginBufferContentMode {
    /// Plain text buffer seeded from `initial_lines`.
    #[default]
    Plain = 0,
    /// Split-pane layout from [`crate::PluginBufferSections`].
    SplitPanes = 1,
    /// Magit/oil-style [`SectionTree`] with interactable items.
    SectionTree = 2,
}

/// JSON-friendly mirror of [`PluginSectionContext`] for the evaluator protocol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSectionContextJson {
    pub kind: String,
    pub buffer_name: String,
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub cursor_line: usize,
    #[serde(default)]
    pub action_id: Option<String>,
    #[serde(default)]
    pub action_detail: Option<String>,
    #[serde(default)]
    pub chord: Option<String>,
    #[serde(default)]
    pub opaque_state: Option<String>,
}

/// Context passed into [`crate::UserLibrary::run_plugin_section_handler`].
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginSectionContext {
    kind: RString,
    buffer_name: RString,
    workspace_root: ROption<RString>,
    cursor_line: usize,
    action_id: ROption<RString>,
    action_detail: ROption<RString>,
    chord: ROption<RString>,
    opaque_state: ROption<RString>,
}

impl PluginSectionContext {
    /// Creates a refresh context for `kind`.
    pub fn new(kind: impl Into<RString>, buffer_name: impl Into<RString>) -> Self {
        Self {
            kind: kind.into(),
            buffer_name: buffer_name.into(),
            workspace_root: ROption::RNone,
            cursor_line: 0,
            action_id: ROption::RNone,
            action_detail: ROption::RNone,
            chord: ROption::RNone,
            opaque_state: ROption::RNone,
        }
    }

    pub fn with_workspace_root(mut self, root: impl Into<RString>) -> Self {
        self.workspace_root = ROption::RSome(root.into());
        self
    }

    pub fn with_cursor_line(mut self, cursor_line: usize) -> Self {
        self.cursor_line = cursor_line;
        self
    }

    pub fn with_action(
        mut self,
        action_id: impl Into<RString>,
        action_detail: Option<impl Into<RString>>,
    ) -> Self {
        self.action_id = ROption::RSome(action_id.into());
        self.action_detail = match action_detail {
            Some(detail) => ROption::RSome(detail.into()),
            None => ROption::RNone,
        };
        self
    }

    pub fn with_chord(mut self, chord: impl Into<RString>) -> Self {
        self.chord = ROption::RSome(chord.into());
        self
    }

    pub fn with_opaque_state(mut self, opaque: impl Into<RString>) -> Self {
        self.opaque_state = ROption::RSome(opaque.into());
        self
    }

    pub fn kind(&self) -> &str {
        self.kind.as_str()
    }

    pub fn buffer_name(&self) -> &str {
        self.buffer_name.as_str()
    }

    pub fn workspace_root(&self) -> Option<&str> {
        match &self.workspace_root {
            ROption::RSome(root) => Some(root.as_str()),
            ROption::RNone => None,
        }
    }

    pub fn cursor_line(&self) -> usize {
        self.cursor_line
    }

    pub fn action_id(&self) -> Option<&str> {
        match &self.action_id {
            ROption::RSome(id) => Some(id.as_str()),
            ROption::RNone => None,
        }
    }

    pub fn action_detail(&self) -> Option<&str> {
        match &self.action_detail {
            ROption::RSome(detail) => Some(detail.as_str()),
            ROption::RNone => None,
        }
    }

    pub fn chord(&self) -> Option<&str> {
        match &self.chord {
            ROption::RSome(chord) => Some(chord.as_str()),
            ROption::RNone => None,
        }
    }

    pub fn opaque_state(&self) -> Option<&str> {
        match &self.opaque_state {
            ROption::RSome(opaque) => Some(opaque.as_str()),
            ROption::RNone => None,
        }
    }

    /// Serializes this context for the section-handler evaluator protocol.
    pub fn to_json(&self) -> PluginSectionContextJson {
        PluginSectionContextJson {
            kind: self.kind().to_owned(),
            buffer_name: self.buffer_name().to_owned(),
            workspace_root: self.workspace_root().map(str::to_owned),
            cursor_line: self.cursor_line(),
            action_id: self.action_id().map(str::to_owned),
            action_detail: self.action_detail().map(str::to_owned),
            chord: self.chord().map(str::to_owned),
            opaque_state: self.opaque_state().map(str::to_owned),
        }
    }

    /// Builds a context from the evaluator protocol JSON.
    pub fn from_json(value: PluginSectionContextJson) -> Self {
        let mut context =
            Self::new(value.kind, value.buffer_name).with_cursor_line(value.cursor_line);
        if let Some(root) = value.workspace_root {
            context = context.with_workspace_root(root);
        }
        if let Some(action_id) = value.action_id {
            context = context.with_action(action_id, value.action_detail);
        }
        if let Some(chord) = value.chord {
            context = context.with_chord(chord);
        }
        if let Some(opaque) = value.opaque_state {
            context = context.with_opaque_state(opaque);
        }
        context
    }
}

/// JSON-friendly mirror of [`PluginSectionResult`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSectionResultJson {
    pub tree: SectionTreeJson,
    #[serde(default)]
    pub opaque_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionTreeJson {
    pub sections: Vec<SectionJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionJson {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub items: Vec<SectionItemJson>,
    #[serde(default)]
    pub children: Vec<SectionJson>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionItemJson {
    pub text: String,
    #[serde(default)]
    pub action_id: Option<String>,
    #[serde(default)]
    pub action_detail: Option<String>,
}

fn section_to_json(section: &crate::Section) -> SectionJson {
    SectionJson {
        id: section.id().to_owned(),
        title: section.title().to_owned(),
        items: section
            .items()
            .iter()
            .map(|item| SectionItemJson {
                text: item.text().to_owned(),
                action_id: item.action().map(|action| action.id().to_owned()),
                action_detail: item
                    .action()
                    .and_then(|action| action.detail().map(str::to_owned)),
            })
            .collect(),
        children: section.children().iter().map(section_to_json).collect(),
    }
}

fn section_from_json(section: SectionJson) -> crate::Section {
    use crate::{Section, SectionAction, SectionItem};
    let items = section
        .items
        .into_iter()
        .map(|item| {
            let mut section_item = SectionItem::new(item.text);
            if let Some(action_id) = item.action_id {
                let mut action = SectionAction::new(action_id);
                if let Some(detail) = item.action_detail {
                    action = action.with_detail(detail);
                }
                section_item = section_item.with_action(action);
            }
            section_item
        })
        .collect();
    let children = section
        .children
        .into_iter()
        .map(section_from_json)
        .collect();
    Section::new(section.id, section.title)
        .with_items(items)
        .with_children(children)
}

/// Result returned by a section-tree handler.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct PluginSectionResult {
    tree: AbiSectionTree,
    opaque_state: ROption<RString>,
}

impl PluginSectionResult {
    /// Creates a result from a rendered tree.
    pub fn new(tree: SectionTree) -> Self {
        Self {
            tree: tree.into(),
            opaque_state: ROption::RNone,
        }
    }

    pub fn with_opaque_state(mut self, opaque: impl Into<RString>) -> Self {
        self.opaque_state = ROption::RSome(opaque.into());
        self
    }

    pub fn tree(&self) -> SectionTree {
        self.tree.clone().into()
    }

    pub fn opaque_state(&self) -> Option<&str> {
        match &self.opaque_state {
            ROption::RSome(opaque) => Some(opaque.as_str()),
            ROption::RNone => None,
        }
    }

    pub fn to_json(&self) -> PluginSectionResultJson {
        let tree = self.tree();
        PluginSectionResultJson {
            tree: SectionTreeJson {
                sections: tree.sections().iter().map(section_to_json).collect(),
            },
            opaque_state: self.opaque_state().map(str::to_owned),
        }
    }

    pub fn from_json(value: PluginSectionResultJson) -> Self {
        let tree = SectionTree::new(
            value
                .tree
                .sections
                .into_iter()
                .map(section_from_json)
                .collect(),
        );
        let mut result = Self::new(tree);
        if let Some(opaque) = value.opaque_state {
            result = result.with_opaque_state(opaque);
        }
        result
    }
}

/// Host-side collapse helper shared with git/oil apply paths.
pub fn render_section_tree(
    tree: &SectionTree,
    collapse: &SectionCollapseState,
) -> Vec<crate::SectionRenderLine> {
    tree.render_lines(collapse)
}
