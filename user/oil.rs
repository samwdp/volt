use editor_core::{Section, SectionAction, SectionItem, SectionTree};
use editor_fs::{DirectoryEntry, DirectoryEntryKind};
use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};
use std::path::Path;

pub const HOOK_OIL_OPEN: &str = "ui.oil.open";
pub const HOOK_OIL_OPEN_PARENT: &str = "ui.oil.open-parent";
pub const ACTION_OIL_ENTRY: &str = "oil.entry";
pub const SECTION_OIL_DIRECTORY: &str = "oil.directory";

/// User-configurable default options for new oil buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OilDefaults {
    pub show_hidden: bool,
    pub sort_mode: OilSortMode,
    pub trash_enabled: bool,
}

/// User-configurable sort modes for oil directory buffers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OilSortMode {
    TypeThenName,
    TypeThenNameDesc,
}

impl OilSortMode {
    /// Returns the human-readable label shown in the oil header.
    pub fn label(self) -> &'static str {
        match self {
            Self::TypeThenName => "type+name",
            Self::TypeThenNameDesc => "type+name desc",
        }
    }

    /// Returns the next sort mode in the cycle used by the oil UI.
    pub fn cycle(self) -> Self {
        match self {
            Self::TypeThenName => Self::TypeThenNameDesc,
            Self::TypeThenNameDesc => Self::TypeThenName,
        }
    }
}

/// User-configurable oil keybindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OilKeybindings {
    pub open_entry: &'static str,
    pub open_vertical_split: &'static str,
    pub open_horizontal_split: &'static str,
    pub open_new_pane: &'static str,
    pub preview_entry: &'static str,
    pub refresh: &'static str,
    pub close: &'static str,
    pub prefix: &'static str,
    pub open_parent: &'static str,
    pub open_workspace_root: &'static str,
    pub set_root: &'static str,
    pub show_help: &'static str,
    pub cycle_sort: &'static str,
    pub toggle_hidden: &'static str,
    pub toggle_trash: &'static str,
    pub open_external: &'static str,
    pub set_tab_local_root: &'static str,
}

/// Oil actions resolved from user-configured keybindings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OilKeyAction {
    OpenEntry,
    OpenVerticalSplit,
    OpenHorizontalSplit,
    OpenNewPane,
    PreviewEntry,
    Refresh,
    Close,
    StartPrefix,
    OpenParent,
    OpenWorkspaceRoot,
    SetRoot,
    ShowHelp,
    CycleSort,
    ToggleHidden,
    ToggleTrash,
    OpenExternal,
    SetTabLocalRoot,
}

/// Returns the default options applied to newly created oil buffers.
pub fn defaults() -> OilDefaults {
    OilDefaults {
        show_hidden: false,
        sort_mode: OilSortMode::TypeThenName,
        trash_enabled: false,
    }
}

/// Returns the user-configurable oil keybindings.
pub fn keybindings() -> OilKeybindings {
    OilKeybindings {
        open_entry: "Enter",
        open_vertical_split: "Ctrl+\\",
        open_horizontal_split: "Ctrl+|",
        open_new_pane: "Ctrl+t",
        preview_entry: "Ctrl+p",
        refresh: "Ctrl+l",
        close: "Ctrl+c",
        prefix: "g",
        open_parent: "-",
        open_workspace_root: "_",
        set_root: "`",
        show_help: "?",
        cycle_sort: "s",
        toggle_hidden: ".",
        toggle_trash: "\\",
        open_external: "x",
        set_tab_local_root: "~",
    }
}

/// Resolves a keydown chord to an oil action, if any.
pub fn keydown_action(chord: &str) -> Option<OilKeyAction> {
    let bindings = keybindings();
    if chord == bindings.open_entry {
        Some(OilKeyAction::OpenEntry)
    } else if chord == bindings.open_vertical_split {
        Some(OilKeyAction::OpenVerticalSplit)
    } else if chord == bindings.open_horizontal_split {
        Some(OilKeyAction::OpenHorizontalSplit)
    } else if chord == bindings.open_new_pane {
        Some(OilKeyAction::OpenNewPane)
    } else if chord == bindings.preview_entry {
        Some(OilKeyAction::PreviewEntry)
    } else if chord == bindings.refresh {
        Some(OilKeyAction::Refresh)
    } else if chord == bindings.close {
        Some(OilKeyAction::Close)
    } else {
        None
    }
}

/// Resolves a normal-mode oil chord to an oil action, if any.
pub fn chord_action(prefix_pending: bool, chord: &str) -> Option<OilKeyAction> {
    let bindings = keybindings();
    if prefix_pending {
        if chord == bindings.show_help {
            Some(OilKeyAction::ShowHelp)
        } else if chord == bindings.toggle_hidden {
            Some(OilKeyAction::ToggleHidden)
        } else if chord == bindings.toggle_trash {
            Some(OilKeyAction::ToggleTrash)
        } else if chord == bindings.cycle_sort {
            Some(OilKeyAction::CycleSort)
        } else if chord == bindings.open_external {
            Some(OilKeyAction::OpenExternal)
        } else if chord == bindings.set_tab_local_root {
            Some(OilKeyAction::SetTabLocalRoot)
        } else {
            None
        }
    } else if chord == bindings.prefix {
        Some(OilKeyAction::StartPrefix)
    } else if chord == bindings.open_parent {
        Some(OilKeyAction::OpenParent)
    } else if chord == bindings.open_workspace_root {
        Some(OilKeyAction::OpenWorkspaceRoot)
    } else if chord == bindings.set_root {
        Some(OilKeyAction::SetRoot)
    } else {
        None
    }
}

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
    sort_mode: OilSortMode,
    trash_enabled: bool,
) -> SectionTree {
    let header = format!(
        "Directory {} (hidden: {}, sort: {}, trash: {})",
        root.display(),
        if show_hidden { "on" } else { "off" },
        sort_mode.label(),
        if trash_enabled { "on" } else { "off" },
    );
    let items = entries
        .iter()
        .map(|entry| {
            let label = directory_entry_display_label(entry);
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

/// Returns the rendered oil label for a directory entry, including its icon.
pub fn directory_entry_display_label(entry: &DirectoryEntry) -> String {
    directory_entry_display_label_from_parts(entry.name(), entry.path(), entry.kind())
}

/// Removes a leading oil icon prefix from an editable line if one is present.
pub fn strip_entry_icon_prefix(label: &str) -> &str {
    let trimmed = label.trim_start();
    let Some((maybe_icon, rest)) = trimmed.split_once(' ') else {
        return trimmed;
    };
    if is_oil_icon(maybe_icon) {
        rest.trim_start()
    } else {
        trimmed
    }
}

fn directory_entry_display_label_from_parts(
    name: &str,
    path: &Path,
    kind: DirectoryEntryKind,
) -> String {
    let icon = oil_entry_icon(name, path, kind);
    match kind {
        DirectoryEntryKind::Directory => format!("{icon} {name}/"),
        DirectoryEntryKind::File => format!("{icon} {name}"),
    }
}

fn oil_entry_icon(name: &str, path: &Path, kind: DirectoryEntryKind) -> &'static str {
    match kind {
        DirectoryEntryKind::Directory => oil_directory_icon(name),
        DirectoryEntryKind::File => oil_file_icon(path),
    }
}

fn oil_directory_icon(name: &str) -> &'static str {
    match name.to_ascii_lowercase().as_str() {
        ".git" => crate::icon_font::symbols::seti::CUSTOM_FOLDER_GIT,
        ".github" => crate::icon_font::symbols::seti::CUSTOM_FOLDER_GITHUB,
        "node_modules" => crate::icon_font::symbols::seti::CUSTOM_FOLDER_NPM,
        ".cargo" | ".config" | ".vscode" => crate::icon_font::symbols::seti::CUSTOM_FOLDER_CONFIG,
        _ => crate::icon_font::symbols::seti::CUSTOM_FOLDER,
    }
}

fn oil_file_icon(path: &Path) -> &'static str {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let file_name_lower = file_name.to_ascii_lowercase();
    match file_name_lower.as_str() {
        "cargo.toml" => return crate::icon_font::symbols::seti::CUSTOM_TOML,
        "cargo.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" => {
            return crate::icon_font::symbols::seti::SETI_LOCK;
        }
        "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => {
            return crate::icon_font::symbols::seti::SETI_DOCKER;
        }
        "makefile" => return crate::icon_font::symbols::seti::SETI_MAKEFILE,
        "license" | "license.md" | "copying" => {
            return crate::icon_font::symbols::seti::SETI_LICENSE;
        }
        "readme" | "readme.md" | "readme.txt" => {
            return crate::icon_font::symbols::seti::SETI_MARKDOWN;
        }
        ".gitignore" | ".gitattributes" | ".gitmodules" => {
            return crate::icon_font::symbols::seti::SETI_GIT;
        }
        _ => {}
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase());

    match extension.as_deref() {
        Some("rs") => crate::icon_font::symbols::seti::SETI_RUST,
        Some("md") | Some("markdown") => crate::icon_font::symbols::seti::SETI_MARKDOWN,
        Some("toml") => crate::icon_font::symbols::seti::CUSTOM_TOML,
        Some("json") | Some("jsonc") => crate::icon_font::symbols::seti::SETI_JSON,
        Some("yaml") | Some("yml") | Some("ini") | Some("cfg") | Some("conf") | Some("env") => {
            crate::icon_font::symbols::seti::SETI_CONFIG
        }
        Some("html") | Some("htm") => crate::icon_font::symbols::seti::SETI_HTML,
        Some("css") | Some("scss") | Some("less") => crate::icon_font::symbols::seti::SETI_CSS,
        Some("js") | Some("mjs") | Some("cjs") | Some("jsx") => {
            crate::icon_font::symbols::seti::SETI_JAVASCRIPT
        }
        Some("ts") | Some("tsx") => crate::icon_font::symbols::seti::SETI_TYPESCRIPT,
        Some("sh") | Some("bash") | Some("zsh") | Some("fish") | Some("ps1") | Some("bat")
        | Some("cmd") => crate::icon_font::symbols::seti::SETI_SHELL,
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("svg")
        | Some("ico") | Some("bmp") | Some("tif") | Some("tiff") => {
            crate::icon_font::symbols::seti::SETI_IMAGE
        }
        Some("pdf") => crate::icon_font::symbols::seti::SETI_PDF,
        Some("xml") => crate::icon_font::symbols::seti::SETI_XML,
        Some("csv") => crate::icon_font::symbols::seti::SETI_CSV,
        Some("c") | Some("h") => crate::icon_font::symbols::seti::SETI_C,
        Some("cs") => crate::icon_font::symbols::seti::SETI_C_SHARP,
        Some("cc") | Some("cpp") | Some("cxx") | Some("hpp") | Some("hh") | Some("hxx") => {
            crate::icon_font::symbols::seti::SETI_CPP
        }
        Some("go") => crate::icon_font::symbols::seti::SETI_GO,
        Some("java") => crate::icon_font::symbols::seti::SETI_JAVA,
        Some("py") | Some("pyi") | Some("pyw") => crate::icon_font::symbols::seti::SETI_PYTHON,
        Some("zip") | Some("7z") | Some("gz") | Some("xz") | Some("rar") | Some("tar") => {
            crate::icon_font::symbols::cod::COD_FILE_ZIP
        }
        Some("mp3") | Some("wav") | Some("ogg") | Some("mp4") | Some("mov") | Some("mkv") => {
            crate::icon_font::symbols::cod::COD_FILE_MEDIA
        }
        Some("lock") => crate::icon_font::symbols::seti::SETI_LOCK,
        _ => crate::icon_font::symbols::seti::CUSTOM_DEFAULT,
    }
}

fn is_oil_icon(glyph: &str) -> bool {
    matches!(
        glyph,
        crate::icon_font::symbols::seti::CUSTOM_FOLDER
            | crate::icon_font::symbols::seti::CUSTOM_FOLDER_CONFIG
            | crate::icon_font::symbols::seti::CUSTOM_FOLDER_GIT
            | crate::icon_font::symbols::seti::CUSTOM_FOLDER_GITHUB
            | crate::icon_font::symbols::seti::CUSTOM_FOLDER_NPM
            | crate::icon_font::symbols::seti::CUSTOM_DEFAULT
            | crate::icon_font::symbols::seti::CUSTOM_TOML
            | crate::icon_font::symbols::seti::SETI_C
            | crate::icon_font::symbols::seti::SETI_CONFIG
            | crate::icon_font::symbols::seti::SETI_CPP
            | crate::icon_font::symbols::seti::SETI_CSV
            | crate::icon_font::symbols::seti::SETI_DOCKER
            | crate::icon_font::symbols::seti::SETI_GIT
            | crate::icon_font::symbols::seti::SETI_GO
            | crate::icon_font::symbols::seti::SETI_HTML
            | crate::icon_font::symbols::seti::SETI_IMAGE
            | crate::icon_font::symbols::seti::SETI_JAVA
            | crate::icon_font::symbols::seti::SETI_JAVASCRIPT
            | crate::icon_font::symbols::seti::SETI_JSON
            | crate::icon_font::symbols::seti::SETI_LICENSE
            | crate::icon_font::symbols::seti::SETI_LOCK
            | crate::icon_font::symbols::seti::SETI_MAKEFILE
            | crate::icon_font::symbols::seti::SETI_MARKDOWN
            | crate::icon_font::symbols::seti::SETI_PDF
            | crate::icon_font::symbols::seti::SETI_PYTHON
            | crate::icon_font::symbols::seti::SETI_RUST
            | crate::icon_font::symbols::seti::SETI_SHELL
            | crate::icon_font::symbols::seti::SETI_TYPESCRIPT
            | crate::icon_font::symbols::seti::SETI_XML
            | crate::icon_font::symbols::cod::COD_FILE_MEDIA
            | crate::icon_font::symbols::cod::COD_FILE_ZIP
    )
}

/// Returns help text for oil directory buffers.
pub fn help_lines() -> Vec<String> {
    let bindings = keybindings();
    let prefixed = |suffix: &str| format!("{}{}", bindings.prefix, suffix);
    vec![
        "Oil directory buffer".to_owned(),
        "".to_owned(),
        "Edit entries in INSERT mode, then press Escape to apply queued actions.".to_owned(),
        "Delete a line to remove a file or directory.".to_owned(),
        "Add a line to create a file; add a trailing / to create a directory.".to_owned(),
        format!("{}: open file/directory", bindings.open_entry),
        format!("{}: open in vertical split", bindings.open_vertical_split),
        format!(
            "{}: open in horizontal split",
            bindings.open_horizontal_split
        ),
        format!("{}: open in new pane", bindings.open_new_pane),
        format!("{}: preview file", bindings.preview_entry),
        format!("{}: refresh listing", bindings.refresh),
        format!("{}: close directory buffer", bindings.close),
        format!("{}: parent directory", bindings.open_parent),
        format!("{}: workspace root", bindings.open_workspace_root),
        format!("{}: set root to selection", bindings.set_root),
        format!(
            "{}: set root to selection (tab-local)",
            prefixed(bindings.set_tab_local_root)
        ),
        format!("{}: cycle sort order", prefixed(bindings.cycle_sort)),
        format!("{}: toggle hidden files", prefixed(bindings.toggle_hidden)),
        format!("{}: toggle trash", prefixed(bindings.toggle_trash)),
        format!("{}: open externally", prefixed(bindings.open_external)),
        format!("{}: show help", prefixed(bindings.show_help)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn rust_files_get_rust_icon_labels() {
        let label = directory_entry_display_label_from_parts(
            "main.rs",
            Path::new("main.rs"),
            DirectoryEntryKind::File,
        );
        assert_eq!(
            label,
            format!("{} main.rs", crate::icon_font::symbols::seti::SETI_RUST)
        );
    }

    #[test]
    fn special_directories_get_folder_icons() {
        let label = directory_entry_display_label_from_parts(
            ".git",
            Path::new(".git"),
            DirectoryEntryKind::Directory,
        );
        assert_eq!(
            label,
            format!(
                "{} .git/",
                crate::icon_font::symbols::seti::CUSTOM_FOLDER_GIT
            )
        );
    }

    #[test]
    fn icon_prefixes_are_stripped_for_editing() {
        let label = format!(
            "{} Cargo.toml",
            crate::icon_font::symbols::seti::CUSTOM_TOML
        );
        assert_eq!(strip_entry_icon_prefix(&label), "Cargo.toml");
        assert_eq!(strip_entry_icon_prefix("plain.txt"), "plain.txt");
    }

    #[test]
    fn default_oil_options_are_exported() {
        assert_eq!(
            defaults(),
            OilDefaults {
                show_hidden: false,
                sort_mode: OilSortMode::TypeThenName,
                trash_enabled: false,
            }
        );
    }

    #[test]
    fn default_oil_keybindings_map_to_actions() {
        let bindings = keybindings();

        assert_eq!(
            keydown_action(bindings.open_entry),
            Some(OilKeyAction::OpenEntry)
        );
        assert_eq!(
            chord_action(false, bindings.prefix),
            Some(OilKeyAction::StartPrefix)
        );
        assert_eq!(
            chord_action(true, bindings.toggle_hidden),
            Some(OilKeyAction::ToggleHidden)
        );
        assert_eq!(
            chord_action(true, bindings.toggle_trash),
            Some(OilKeyAction::ToggleTrash)
        );
    }

    #[test]
    fn help_lines_reflect_default_keybindings() {
        let lines = help_lines();
        assert!(lines.contains(&"Enter: open file/directory".to_owned()));
        assert!(lines.contains(&"g.: toggle hidden files".to_owned()));
        assert!(lines.contains(&"g\\: toggle trash".to_owned()));
    }
}
