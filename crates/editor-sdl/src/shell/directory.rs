use super::git::{leading_indent_bytes, push_span_bytes, split_icon_prefixed_content};
use super::*;

const TOKEN_OIL_HEADER: &str = "oil.header";
const TOKEN_OIL_DIRECTORY: &str = "oil.directory";
const TOKEN_OIL_DIRECTORY_CONFIG: &str = "oil.directory.config";
const TOKEN_OIL_DIRECTORY_GIT: &str = "oil.directory.git";
const TOKEN_OIL_DIRECTORY_GITHUB: &str = "oil.directory.github";
const TOKEN_OIL_DIRECTORY_NODE: &str = "oil.directory.node";
const TOKEN_OIL_FILE: &str = "oil.file";
const TOKEN_OIL_FILE_ARCHIVE: &str = "oil.file.archive";
const TOKEN_OIL_FILE_CODE: &str = "oil.file.code";
const TOKEN_OIL_FILE_CONFIG: &str = "oil.file.config";
const TOKEN_OIL_FILE_DOCUMENT: &str = "oil.file.document";
const TOKEN_OIL_FILE_GIT: &str = "oil.file.git";
const TOKEN_OIL_FILE_IMAGE: &str = "oil.file.image";
const TOKEN_OIL_FILE_LOCK: &str = "oil.file.lock";
const TOKEN_OIL_FILE_MEDIA: &str = "oil.file.media";

pub(super) type DirectorySortMode = editor_plugin_api::OilSortMode;

#[derive(Debug, Clone)]
pub(super) struct DirectoryViewState {
    pub(super) root: PathBuf,
    pub(super) entries: Vec<DirectoryEntry>,
    pub(super) show_hidden: bool,
    pub(super) sort_mode: DirectorySortMode,
    pub(super) trash_enabled: bool,
    pub(super) edit_snapshot: Vec<String>,
}

impl DirectoryViewState {
    pub(super) fn new(root: PathBuf, entries: Vec<DirectoryEntry>, defaults: OilDefaults) -> Self {
        Self {
            root,
            entries,
            show_hidden: defaults.show_hidden,
            sort_mode: defaults.sort_mode,
            trash_enabled: defaults.trash_enabled,
            edit_snapshot: Vec::new(),
        }
    }
}

pub(super) fn directory_entry_label(entry: &DirectoryEntry) -> String {
    match entry.kind() {
        DirectoryEntryKind::Directory => format!("{}/", entry.name()),
        DirectoryEntryKind::File => entry.name().to_owned(),
    }
}

pub(super) fn apply_directory_state(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    state: DirectoryViewState,
) -> Result<(), String> {
    let entries = directory_visible_entries(&state);
    let labels = entries
        .iter()
        .map(directory_entry_label)
        .collect::<Vec<_>>();
    let collapsed = shell_buffer(runtime, buffer_id)?
        .section_state()
        .map(|state| state.collapsed.clone())
        .unwrap_or_default();
    let user_library = shell_user_library(runtime);
    let lines = user_library
        .oil_directory_sections(
            &state.root,
            &entries,
            state.show_hidden,
            state.sort_mode,
            state.trash_enabled,
        )
        .render_lines(&collapsed);
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    {
        let section_state = buffer.ensure_section_state();
        section_state.collapsed = collapsed;
    }
    let mut state = state;
    state.edit_snapshot = labels;
    buffer.set_directory_state(state);
    buffer.set_section_lines(lines);
    Ok(())
}

pub(super) fn directory_visible_entries(state: &DirectoryViewState) -> Vec<DirectoryEntry> {
    let mut entries = state.entries.clone();
    if !state.show_hidden {
        entries.retain(|entry| !entry.name().starts_with('.'));
    }
    sort_directory_entries(&mut entries, state.sort_mode);
    entries
}

pub(super) fn sort_directory_entries(entries: &mut [DirectoryEntry], sort_mode: DirectorySortMode) {
    match sort_mode {
        DirectorySortMode::TypeThenName => {
            entries.sort_by(|left, right| {
                let left_is_file = matches!(left.kind(), DirectoryEntryKind::File);
                let right_is_file = matches!(right.kind(), DirectoryEntryKind::File);
                left_is_file.cmp(&right_is_file).then_with(|| {
                    left.name()
                        .to_ascii_lowercase()
                        .cmp(&right.name().to_ascii_lowercase())
                })
            });
        }
        DirectorySortMode::TypeThenNameDesc => {
            entries.sort_by(|left, right| {
                let left_is_file = matches!(left.kind(), DirectoryEntryKind::File);
                let right_is_file = matches!(right.kind(), DirectoryEntryKind::File);
                left_is_file.cmp(&right_is_file).then_with(|| {
                    right
                        .name()
                        .to_ascii_lowercase()
                        .cmp(&left.name().to_ascii_lowercase())
                })
            });
        }
    }
}

pub(super) fn directory_entry_at_cursor(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
) -> Result<DirectoryEntry, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    let state = buffer
        .directory_state()
        .ok_or_else(|| "directory state is missing".to_owned())?;
    let line = buffer.cursor_point().line;
    let meta = buffer
        .section_line_meta(line)
        .and_then(|meta| meta.action.as_ref())
        .ok_or_else(|| "no directory entry selected".to_owned())?;
    if meta.id() != oil_protocol::ACTION_OIL_ENTRY {
        return Err("no directory entry selected".to_owned());
    }
    let detail = meta
        .detail()
        .ok_or_else(|| "directory entry detail missing".to_owned())?;
    let path = Path::new(detail);
    state
        .entries
        .iter()
        .find(|entry| entry.path() == path)
        .cloned()
        .ok_or_else(|| "directory entry not found".to_owned())
}

pub(super) fn oil_directory_line_spans(
    line: &SectionRenderLine,
    formatted_line: &str,
) -> Vec<LineSyntaxSpan> {
    let mut spans = Vec::new();
    let indent_bytes = leading_indent_bytes(formatted_line);
    let trimmed = &formatted_line[indent_bytes..];
    if trimmed.is_empty() {
        return spans;
    }
    match &line.kind {
        SectionRenderLineKind::Header { .. } => {
            push_span_bytes(
                &mut spans,
                formatted_line,
                indent_bytes,
                indent_bytes + trimmed.len(),
                TOKEN_OIL_HEADER,
            );
        }
        SectionRenderLineKind::Item => {
            let token = line
                .action
                .as_ref()
                .and_then(|action| action.detail())
                .map(|detail| oil_entry_theme_token(Path::new(detail), line.text.ends_with('/')))
                .unwrap_or(TOKEN_OIL_FILE);
            let (icon_bounds, content_start, content) =
                split_icon_prefixed_content(trimmed).unwrap_or((None, 0, trimmed));
            if let Some((icon_start, icon_end)) = icon_bounds {
                push_span_bytes(
                    &mut spans,
                    formatted_line,
                    indent_bytes + icon_start,
                    indent_bytes + icon_end,
                    token,
                );
            }
            push_span_bytes(
                &mut spans,
                formatted_line,
                indent_bytes + content_start,
                indent_bytes + content_start + content.len(),
                token,
            );
        }
        SectionRenderLineKind::Spacer => {}
    }
    spans
}

fn oil_entry_theme_token(path: &Path, is_directory: bool) -> &'static str {
    if is_directory {
        return oil_directory_theme_token(path);
    }
    oil_file_theme_token(path)
}

fn oil_directory_theme_token(path: &Path) -> &'static str {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match file_name.as_str() {
        ".git" => TOKEN_OIL_DIRECTORY_GIT,
        ".github" => TOKEN_OIL_DIRECTORY_GITHUB,
        "node_modules" => TOKEN_OIL_DIRECTORY_NODE,
        ".cargo" | ".config" | ".vscode" => TOKEN_OIL_DIRECTORY_CONFIG,
        _ => TOKEN_OIL_DIRECTORY,
    }
}

fn oil_file_theme_token(path: &Path) -> &'static str {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match file_name.as_str() {
        "cargo.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" => {
            return TOKEN_OIL_FILE_LOCK;
        }
        ".gitignore" | ".gitattributes" | ".gitmodules" => {
            return TOKEN_OIL_FILE_GIT;
        }
        "cargo.toml" | "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" | "makefile" => {
            return TOKEN_OIL_FILE_CONFIG;
        }
        "license" | "license.md" | "copying" | "readme" | "readme.md" | "readme.txt" => {
            return TOKEN_OIL_FILE_DOCUMENT;
        }
        _ => {}
    }

    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("rs") | Some("c") | Some("h") | Some("cs") | Some("cc") | Some("cpp")
        | Some("cxx") | Some("hpp") | Some("hh") | Some("hxx") | Some("go") | Some("java")
        | Some("py") | Some("pyi") | Some("pyw") | Some("js") | Some("mjs") | Some("cjs")
        | Some("jsx") | Some("ts") | Some("tsx") | Some("sh") | Some("bash") | Some("zsh")
        | Some("fish") | Some("ps1") | Some("bat") | Some("cmd") => TOKEN_OIL_FILE_CODE,
        Some("json") | Some("jsonc") | Some("yaml") | Some("yml") | Some("toml") | Some("ini")
        | Some("cfg") | Some("conf") | Some("env") => TOKEN_OIL_FILE_CONFIG,
        Some("md") | Some("markdown") | Some("txt") | Some("pdf") | Some("html") | Some("htm")
        | Some("css") | Some("scss") | Some("less") | Some("xml") | Some("csv") => {
            TOKEN_OIL_FILE_DOCUMENT
        }
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("svg")
        | Some("ico") | Some("bmp") | Some("tif") | Some("tiff") => TOKEN_OIL_FILE_IMAGE,
        Some("zip") | Some("7z") | Some("gz") | Some("xz") | Some("rar") | Some("tar") => {
            TOKEN_OIL_FILE_ARCHIVE
        }
        Some("mp3") | Some("wav") | Some("ogg") | Some("mp4") | Some("mov") | Some("mkv") => {
            TOKEN_OIL_FILE_MEDIA
        }
        Some("lock") => TOKEN_OIL_FILE_LOCK,
        _ => TOKEN_OIL_FILE,
    }
}

#[derive(Debug, Clone)]
pub(super) struct DirectoryLine {
    pub(super) label: String,
    pub(super) rel_path: PathBuf,
    pub(super) is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum DirectoryEditAction {
    CreateFile(PathBuf),
    CreateDir(PathBuf),
    Delete { path: PathBuf, is_dir: bool },
    Rename { from: PathBuf, to: PathBuf },
}

pub(super) fn directory_edit_lines(
    buffer: &ShellBuffer,
    user_library: &dyn UserLibrary,
) -> Vec<String> {
    let mut lines = Vec::new();
    for line_index in 0..buffer.line_count() {
        let raw = buffer.text.line(line_index).unwrap_or_default();
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        if line_index == 0 && trimmed.starts_with("Directory ") {
            continue;
        }
        lines.push(user_library.oil_strip_entry_icon_prefix(trimmed).to_owned());
    }
    lines
}

pub(super) fn parse_directory_line(
    line: &str,
    user_library: &dyn UserLibrary,
) -> Result<DirectoryLine, String> {
    let trimmed = user_library.oil_strip_entry_icon_prefix(line.trim());
    let is_dir = trimmed.ends_with('/');
    let raw = trimmed.trim_end_matches('/');
    if raw.is_empty() {
        return Err("directory entry is empty".to_owned());
    }
    let rel_path = PathBuf::from(raw);
    if rel_path.is_absolute() {
        return Err(format!("absolute paths are not supported: {raw}"));
    }
    if rel_path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(format!("parent directory segments are not allowed: {raw}"));
    }
    Ok(DirectoryLine {
        label: trimmed.to_owned(),
        rel_path,
        is_dir,
    })
}

#[cfg(test)]
mod directory_line_tests {
    use super::*;
    use editor_plugin_host::NullUserLibrary;
    use std::path::{Path, PathBuf};

    #[test]
    fn parse_directory_line_strips_file_icons() {
        let line = format!("{} Cargo.toml", editor_icons::symbols::seti::CUSTOM_TOML);
        let user_library = NullUserLibrary;
        let parsed = parse_directory_line(&line, &user_library)
            .expect("icon-prefixed file line should parse");
        assert_eq!(parsed.label, "Cargo.toml");
        assert_eq!(parsed.rel_path, PathBuf::from("Cargo.toml"));
        assert!(!parsed.is_dir);
    }

    #[test]
    fn parse_directory_line_strips_directory_icons() {
        let line = format!("{} src/", editor_icons::symbols::seti::CUSTOM_FOLDER);
        let user_library = NullUserLibrary;
        let parsed = parse_directory_line(&line, &user_library)
            .expect("icon-prefixed directory line should parse");
        assert_eq!(parsed.label, "src/");
        assert_eq!(parsed.rel_path, PathBuf::from("src"));
        assert!(parsed.is_dir);
    }

    #[test]
    fn parse_directory_line_strips_repeated_icons() {
        let icon = editor_icons::symbols::seti::SETI_C_SHARP;
        let line = format!("{icon} {icon} {icon} {icon} Program.cs");
        let user_library = NullUserLibrary;
        let parsed = parse_directory_line(&line, &user_library)
            .expect("repeated icon-prefixed file line should parse");
        assert_eq!(parsed.label, "Program.cs");
        assert_eq!(parsed.rel_path, PathBuf::from("Program.cs"));
        assert!(!parsed.is_dir);
    }

    #[test]
    fn repeated_icons_do_not_turn_existing_entries_into_renames() {
        let root = PathBuf::from("workspace");
        let icon = editor_icons::symbols::seti::SETI_C_SHARP;
        let before = vec!["DateRange.cs".to_owned(), "Program.cs".to_owned()];
        let after = vec![
            format!("{icon} {icon} {icon} {icon} DateRange.cs"),
            format!("{icon} {icon} {icon} {icon} Program.cs"),
            "test.razor".to_owned(),
        ];
        let user_library = NullUserLibrary;
        let actions = directory_edit_actions(&root, &before, &after, &user_library)
            .expect("repeated icons should be stripped before diffing");
        assert_eq!(
            actions,
            vec![DirectoryEditAction::CreateFile(root.join("test.razor"))]
        );
    }

    #[test]
    fn oil_theme_tokens_follow_entry_kind() {
        assert_eq!(
            oil_directory_theme_token(Path::new(".git")),
            TOKEN_OIL_DIRECTORY_GIT
        );
        assert_eq!(
            oil_file_theme_token(Path::new("Cargo.toml")),
            TOKEN_OIL_FILE_CONFIG
        );
        assert_eq!(
            oil_file_theme_token(Path::new("Program.cs")),
            TOKEN_OIL_FILE_CODE
        );
        assert_eq!(
            oil_file_theme_token(Path::new("logo.png")),
            TOKEN_OIL_FILE_IMAGE
        );
        assert_eq!(
            oil_file_theme_token(Path::new("archive.zip")),
            TOKEN_OIL_FILE_ARCHIVE
        );
    }
}

pub(super) fn parse_directory_lines(
    lines: &[String],
    user_library: &dyn UserLibrary,
) -> Result<Vec<DirectoryLine>, String> {
    let mut seen = BTreeSet::new();
    let mut parsed = Vec::with_capacity(lines.len());
    for line in lines {
        let entry = parse_directory_line(line, user_library)?;
        if !seen.insert(entry.rel_path.clone()) {
            return Err(format!("duplicate entry `{}`", entry.label));
        }
        parsed.push(entry);
    }
    Ok(parsed)
}

pub(super) fn diff_directory_lines(
    before: &[String],
    after: &[String],
) -> (Vec<usize>, Vec<usize>) {
    let before_set = before.iter().collect::<BTreeSet<_>>();
    let after_set = after.iter().collect::<BTreeSet<_>>();
    let removed = before
        .iter()
        .enumerate()
        .filter(|(_, line)| !after_set.contains(line))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    let added = after
        .iter()
        .enumerate()
        .filter(|(_, line)| !before_set.contains(line))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    (removed, added)
}

pub(super) fn directory_edit_actions(
    root: &Path,
    before: &[String],
    after: &[String],
    user_library: &dyn UserLibrary,
) -> Result<Vec<DirectoryEditAction>, String> {
    if before == after {
        return Ok(Vec::new());
    }
    let before_parsed = parse_directory_lines(before, user_library)?;
    let after_parsed = parse_directory_lines(after, user_library)?;
    let (removed_indices, added_indices) = diff_directory_lines(before, after);
    let removed = removed_indices
        .iter()
        .map(|index| &before_parsed[*index])
        .collect::<Vec<_>>();
    let added = added_indices
        .iter()
        .map(|index| &after_parsed[*index])
        .collect::<Vec<_>>();
    let mut actions = Vec::new();
    let rename_count = if !removed.is_empty() && removed.len() == added.len() {
        removed.len()
    } else {
        0
    };
    for index in 0..rename_count {
        let src = removed[index];
        let dst = added[index];
        if !src.is_dir && dst.is_dir {
            return Err(format!(
                "cannot move file `{}` to directory path `{}`",
                src.label, dst.label
            ));
        }
        actions.push(DirectoryEditAction::Rename {
            from: root.join(&src.rel_path),
            to: root.join(&dst.rel_path),
        });
    }
    for src in removed.iter().skip(rename_count) {
        actions.push(DirectoryEditAction::Delete {
            path: root.join(&src.rel_path),
            is_dir: src.is_dir,
        });
    }
    for dst in added.iter().skip(rename_count) {
        let path = root.join(&dst.rel_path);
        if dst.is_dir {
            actions.push(DirectoryEditAction::CreateDir(path));
        } else {
            actions.push(DirectoryEditAction::CreateFile(path));
        }
    }
    Ok(actions)
}

pub(super) fn apply_directory_edit_actions(actions: &[DirectoryEditAction]) -> Result<(), String> {
    for action in actions {
        match action {
            DirectoryEditAction::Rename { from, to } => {
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        format!("failed to create `{}`: {error}", parent.display())
                    })?;
                }
                fs::rename(from, to)
                    .map_err(|error| format!("failed to move `{}`: {error}", from.display()))?;
            }
            DirectoryEditAction::Delete { path, is_dir } => {
                if *is_dir {
                    fs::remove_dir_all(path).map_err(|error| {
                        format!("failed to remove `{}`: {error}", path.display())
                    })?;
                } else {
                    fs::remove_file(path).map_err(|error| {
                        format!("failed to remove `{}`: {error}", path.display())
                    })?;
                }
            }
            DirectoryEditAction::CreateDir(path) => {
                fs::create_dir_all(path)
                    .map_err(|error| format!("failed to create `{}`: {error}", path.display()))?;
            }
            DirectoryEditAction::CreateFile(path) => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        format!("failed to create `{}`: {error}", parent.display())
                    })?;
                }
                fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .map_err(|error| format!("failed to create `{}`: {error}", path.display()))?;
            }
        }
    }
    Ok(())
}

pub(super) fn apply_directory_edit_queue(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let (root, before) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let Some(state) = buffer.directory_state() else {
            return Ok(());
        };
        let snapshot = if state.edit_snapshot.is_empty() {
            directory_visible_entries(state)
                .iter()
                .map(directory_entry_label)
                .collect()
        } else {
            state.edit_snapshot.clone()
        };
        (state.root.clone(), snapshot)
    };
    let after = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let user_library = shell_user_library(runtime);
        directory_edit_lines(buffer, &*user_library)
    };
    let user_library = shell_user_library(runtime);
    let actions = directory_edit_actions(&root, &before, &after, &*user_library)?;
    if actions.is_empty() {
        return Ok(());
    }
    apply_directory_edit_actions(&actions)?;
    refresh_directory_buffer(runtime, buffer_id)?;
    Ok(())
}

pub(super) fn update_directory_state(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    update: impl FnOnce(&mut DirectoryViewState),
) -> Result<(), String> {
    let state = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer
            .directory_state()
            .cloned()
            .ok_or_else(|| "directory state is missing".to_owned())?
    };
    let mut state = state;
    update(&mut state);
    apply_directory_state(runtime, buffer_id, state)
}

pub(super) fn directory_root_for_entry(entry: &DirectoryEntry) -> Result<PathBuf, String> {
    match entry.kind() {
        DirectoryEntryKind::Directory => Ok(entry.path().to_path_buf()),
        DirectoryEntryKind::File => entry
            .path()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "entry has no parent directory".to_owned()),
    }
}

pub(super) fn directory_cd_from_cursor(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let entry = directory_entry_at_cursor(runtime, buffer_id)?;
    let root = directory_root_for_entry(&entry)?;
    set_directory_root(runtime, buffer_id, root)
}

#[derive(Debug, Clone, Copy)]
pub(super) enum DirectoryOpenMode {
    Current,
    SplitHorizontal,
    SplitVertical,
    NewPane,
    Preview,
}

pub(super) fn open_directory_entry(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    entry: DirectoryEntry,
    mode: DirectoryOpenMode,
) -> Result<(), String> {
    match entry.kind() {
        DirectoryEntryKind::Directory => {
            set_directory_root(runtime, buffer_id, entry.path().to_path_buf())
        }
        DirectoryEntryKind::File => match mode {
            DirectoryOpenMode::Current => {
                open_workspace_file(runtime, entry.path())?;
                Ok(())
            }
            DirectoryOpenMode::SplitHorizontal => {
                open_file_in_split(runtime, entry.path(), PaneSplitDirection::Horizontal, true)
            }
            DirectoryOpenMode::SplitVertical => {
                open_file_in_split(runtime, entry.path(), PaneSplitDirection::Vertical, true)
            }
            DirectoryOpenMode::NewPane => {
                open_file_in_split(runtime, entry.path(), PaneSplitDirection::Vertical, true)
            }
            DirectoryOpenMode::Preview => open_oil_preview_popup(runtime, entry.path()),
        },
    }
}
