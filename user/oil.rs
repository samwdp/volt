use editor_core::{Section, SectionAction, SectionItem, SectionTree};
use editor_fs::{DirectoryEntry, DirectoryEntryKind};
use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};
use std::path::Path;

pub const HOOK_OIL_OPEN: &str = "ui.oil.open";
pub const HOOK_OIL_OPEN_PARENT: &str = "ui.oil.open-parent";
pub const ACTION_OIL_ENTRY: &str = "oil.entry";
pub const SECTION_OIL_DIRECTORY: &str = "oil.directory";

/// Returns the metadata for the directory editing package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "oil",
        true,
        "Directory manipulation buffers inspired by oil.nvim.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "oil.open-directory",
            "Opens an editable directory buffer.",
            vec![PluginAction::emit_hook(HOOK_OIL_OPEN, None::<&str>)],
        ),
        PluginCommand::new(
            "oil.open-parent",
            "Opens a parent-directory view.",
            vec![PluginAction::emit_hook(HOOK_OIL_OPEN_PARENT, None::<&str>)],
        ),
    ])
}

/// Builds the oil directory sections for rendering.
pub fn directory_sections(
    root: &Path,
    entries: &[DirectoryEntry],
    show_hidden: bool,
    sort_label: &str,
    trash_enabled: bool,
) -> SectionTree {
    let header = format!(
        "Directory {} (hidden: {}, sort: {}, trash: {})",
        root.display(),
        if show_hidden { "on" } else { "off" },
        sort_label,
        if trash_enabled { "on" } else { "off" },
    );
    let items = entries
        .iter()
        .map(|entry| {
            let label = match entry.kind() {
                DirectoryEntryKind::Directory => format!("{}/", entry.name()),
                DirectoryEntryKind::File => entry.name().to_owned(),
            };
            SectionItem::new(label).with_action(
                SectionAction::new(ACTION_OIL_ENTRY)
                    .with_detail(entry.path().display().to_string()),
            )
        })
        .collect();
    SectionTree::new(vec![
        Section::new(SECTION_OIL_DIRECTORY, header).with_items(items),
    ])
}

/// Returns help text for oil directory buffers.
pub fn help_lines() -> Vec<String> {
    vec![
        "Oil directory buffer".to_owned(),
        "".to_owned(),
        "Edit entries in INSERT mode, then press Escape to apply queued actions.".to_owned(),
        "Delete a line to remove a file or directory.".to_owned(),
        "Add a line to create a file; add a trailing / to create a directory.".to_owned(),
        "Enter: open file/directory".to_owned(),
        "Ctrl+s: open in vertical split".to_owned(),
        "Ctrl+h: open in horizontal split".to_owned(),
        "Ctrl+t: open in new pane".to_owned(),
        "Ctrl+p: preview file".to_owned(),
        "Ctrl+l: refresh listing".to_owned(),
        "Ctrl+c: close directory buffer".to_owned(),
        "-: parent directory".to_owned(),
        "_: workspace root".to_owned(),
        "`: set root to selection".to_owned(),
        "g~: set root to selection (tab-local)".to_owned(),
        "gs: cycle sort order".to_owned(),
        "g.: toggle hidden files".to_owned(),
        "g\\: toggle trash".to_owned(),
        "gx: open externally".to_owned(),
        "g?: show help".to_owned(),
    ]
}
