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
}
