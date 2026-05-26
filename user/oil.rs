use editor_core::{Section, SectionAction, SectionItem, SectionTree};
use editor_fs::{DirectoryEntry, DirectoryEntryKind};
use editor_plugin_api::{
    ContextHelpEntry, ContextHelpSpec, OilDefaults, OilFeatureSpec, OilKeyAction, OilKeybindings,
    OilSortMode, PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
    PluginVimMode, oil_hooks, oil_protocol,
};
use std::path::Path;

pub const ACTION_OIL_ENTRY: &str = oil_protocol::ACTION_OIL_ENTRY;
pub const SECTION_OIL_DIRECTORY: &str = oil_protocol::SECTION_OIL_DIRECTORY;

fn help_entry(chord: impl Into<String>, action: &str, description: &str) -> ContextHelpEntry {
    ContextHelpEntry::new(chord, action, description)
}

/// Public oil feature contract used by first-party and third-party code.
pub fn feature_spec() -> OilFeatureSpec {
    let keybindings = OilKeybindings::default();
    let prefixed = |suffix: &str| format!("{}{}", keybindings.prefix, suffix);
    OilFeatureSpec {
        defaults: OilDefaults::default(),
        keybindings,
        help: ContextHelpSpec::new(
            "Oil",
            "Oil",
            vec![
                help_entry(
                    keybindings.open_entry,
                    "open",
                    "Opens the file or enters the selected directory.",
                ),
                help_entry(
                    keybindings.open_vertical_split,
                    "open vertical split",
                    "Opens the selection in a vertical split.",
                ),
                help_entry(
                    keybindings.open_horizontal_split,
                    "open horizontal split",
                    "Opens the selection in a horizontal split.",
                ),
                help_entry("yy", "copy entry", "Copies the selected file or directory."),
                help_entry(
                    "Visual y",
                    "copy entries",
                    "Copies the selected files and directories.",
                ),
                help_entry(
                    "p",
                    "paste copies",
                    "Copies the yanked files and directories into the current oil root.",
                ),
                help_entry(
                    keybindings.open_new_pane,
                    "open new pane",
                    "Opens the selection in a new pane.",
                ),
                help_entry(
                    keybindings.preview_entry,
                    "preview",
                    "Previews the selected file.",
                ),
                help_entry(
                    keybindings.refresh,
                    "refresh",
                    "Refreshes the directory listing.",
                ),
                help_entry(keybindings.close, "close", "Closes the directory buffer."),
                help_entry(
                    keybindings.open_parent,
                    "parent directory",
                    "Navigates to the parent directory.",
                ),
                help_entry(
                    keybindings.open_workspace_root,
                    "workspace root",
                    "Navigates to the workspace root.",
                ),
                help_entry(
                    keybindings.set_root,
                    "set root",
                    "Sets the directory root to the selection.",
                ),
                help_entry(
                    prefixed(keybindings.set_tab_local_root),
                    "set root (tab)",
                    "Sets the directory root to the selection (tab-local).",
                ),
                help_entry(
                    prefixed(keybindings.cycle_sort),
                    "change sort",
                    "Cycles the directory sort order.",
                ),
                help_entry(
                    prefixed(keybindings.toggle_hidden),
                    "toggle hidden",
                    "Toggles hidden file visibility.",
                ),
                help_entry(
                    prefixed(keybindings.toggle_trash),
                    "toggle trash",
                    "Toggles trash usage for deletions.",
                ),
                help_entry(
                    prefixed(keybindings.open_external),
                    "open external",
                    "Opens the selection externally.",
                ),
                help_entry(
                    prefixed(keybindings.show_help),
                    "help",
                    "Shows the oil help popup.",
                ),
                help_entry(
                    prefixed(keybindings.create_git_worktree),
                    "create git worktree",
                    "Creates a git worktree from the current oil buffer.",
                ),
            ],
        ),
    }
}

/// Returns the default options applied to newly created oil buffers.
pub fn defaults() -> OilDefaults {
    feature_spec().defaults
}

/// Returns the user-configurable oil keybindings.
pub fn keybindings() -> OilKeybindings {
    feature_spec().keybindings
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
        } else if chord == bindings.create_git_worktree {
            Some(OilKeyAction::CreateGitWorktree)
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

fn oil_action_command(
    name: &'static str,
    description: &'static str,
    action: &'static str,
) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(oil_hooks::ACTION, Some(action))],
    )
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
            vec![PluginAction::emit_hook(oil_hooks::OPEN, None::<&str>)],
        ),
        PluginCommand::new(
            "oil.open-parent",
            "Opens a parent-directory view.",
            vec![PluginAction::emit_hook(
                oil_hooks::OPEN_PARENT,
                None::<&str>,
            )],
        ),
        oil_action_command(
            "oil.open-entry",
            "Opens the selected oil entry.",
            "open-entry",
        ),
        oil_action_command(
            "oil.open-vertical-split",
            "Opens the selected oil entry in a vertical split.",
            "open-vertical-split",
        ),
        oil_action_command(
            "oil.open-horizontal-split",
            "Opens the selected oil entry in a horizontal split.",
            "open-horizontal-split",
        ),
        oil_action_command(
            "oil.open-new-pane",
            "Opens the selected oil entry in a new pane.",
            "open-new-pane",
        ),
        oil_action_command(
            "oil.preview-entry",
            "Previews the selected oil entry.",
            "preview-entry",
        ),
        oil_action_command("oil.refresh", "Refreshes the active oil buffer.", "refresh"),
        oil_action_command("oil.close", "Closes the active oil buffer.", "close"),
        oil_action_command(
            "oil.open-workspace-root",
            "Opens the workspace root in the active oil buffer.",
            "open-workspace-root",
        ),
        oil_action_command(
            "oil.set-root",
            "Sets the active oil root to the selected directory.",
            "set-root",
        ),
        oil_action_command("oil.show-help", "Shows oil help.", "show-help"),
        oil_action_command(
            "oil.cycle-sort",
            "Cycles the active oil sort mode.",
            "cycle-sort",
        ),
        oil_action_command(
            "oil.toggle-hidden",
            "Toggles hidden files in the active oil buffer.",
            "toggle-hidden",
        ),
        oil_action_command(
            "oil.toggle-trash",
            "Toggles trash mode in the active oil buffer.",
            "toggle-trash",
        ),
        oil_action_command(
            "oil.open-external",
            "Opens the selected oil entry externally.",
            "open-external",
        ),
        oil_action_command(
            "oil.set-tab-local-root",
            "Sets the tab-local oil root to the selected directory.",
            "set-tab-local-root",
        ),
        PluginCommand::new(
            "oil.git-worktree",
            "Creates a git worktree from an oil directory buffer.",
            vec![PluginAction::emit_hook(
                oil_hooks::GIT_WORKTREE,
                None::<&str>,
            )],
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new("g w n", "oil.git-worktree", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
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
    let mut trimmed = label.trim_start();
    while let Some((maybe_icon, rest)) = trimmed.split_once(' ') {
        if !is_oil_icon(maybe_icon) {
            break;
        }
        trimmed = rest.trim_start();
    }
    trimmed
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
    editor_icons::seti_directory_icon(name)
}

fn oil_file_icon(path: &Path) -> &'static str {
    editor_icons::seti_file_icon(path)
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
            | crate::icon_font::symbols::seti::SETI_C_SHARP
            | crate::icon_font::symbols::seti::SETI_CONFIG
            | crate::icon_font::symbols::seti::SETI_CPP
            | crate::icon_font::symbols::seti::SETI_CSS
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
        "Use yy to copy the selected entry and p to paste it into the current directory."
            .to_owned(),
        "Use visual-line selection plus y to copy multiple entries.".to_owned(),
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
        format!(
            "{}: create git worktree",
            prefixed(bindings.create_git_worktree)
        ),
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

        let repeated = format!(
            "{} {} {} {} Cargo.toml",
            crate::icon_font::symbols::seti::CUSTOM_TOML,
            crate::icon_font::symbols::seti::CUSTOM_TOML,
            crate::icon_font::symbols::seti::CUSTOM_TOML,
            crate::icon_font::symbols::seti::CUSTOM_TOML
        );
        assert_eq!(strip_entry_icon_prefix(&repeated), "Cargo.toml");

        let repeated_csharp = format!(
            "{} {} {} {} test1.cs",
            crate::icon_font::symbols::seti::SETI_C_SHARP,
            crate::icon_font::symbols::seti::SETI_C_SHARP,
            crate::icon_font::symbols::seti::SETI_C_SHARP,
            crate::icon_font::symbols::seti::SETI_C_SHARP
        );
        assert_eq!(strip_entry_icon_prefix(&repeated_csharp), "test1.cs");

        let repeated_css = format!(
            "{} {} {} {} site.css",
            crate::icon_font::symbols::seti::SETI_CSS,
            crate::icon_font::symbols::seti::SETI_CSS,
            crate::icon_font::symbols::seti::SETI_CSS,
            crate::icon_font::symbols::seti::SETI_CSS
        );
        assert_eq!(strip_entry_icon_prefix(&repeated_css), "site.css");

        assert_eq!(strip_entry_icon_prefix("plain.txt"), "plain.txt");
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
        assert_eq!(
            chord_action(true, bindings.create_git_worktree),
            Some(OilKeyAction::CreateGitWorktree)
        );
    }

    #[test]
    fn package_binds_git_worktree_command() {
        let package = package();
        let command = package
            .commands()
            .iter()
            .find(|command| command.name() == "oil.git-worktree")
            .expect("oil.git-worktree command");
        assert_eq!(
            command.actions()[0].hook().map(|hook| hook.hook_name()),
            Some(oil_hooks::GIT_WORKTREE)
        );

        let binding = package
            .key_bindings()
            .iter()
            .find(|binding| binding.command_name() == "oil.git-worktree")
            .expect("oil.git-worktree binding");

        assert!(!binding.chord().trim().is_empty());
        assert_eq!(binding.scope(), PluginKeymapScope::Workspace);
        assert_eq!(binding.vim_mode(), PluginVimMode::Normal);
    }

    #[test]
    fn package_exports_oil_actions_as_commands() {
        let package = package();
        let names: Vec<_> = package
            .commands()
            .iter()
            .map(|command| command.name())
            .collect();

        for name in [
            "oil.open-directory",
            "oil.open-parent",
            "oil.open-entry",
            "oil.open-vertical-split",
            "oil.open-horizontal-split",
            "oil.open-new-pane",
            "oil.preview-entry",
            "oil.refresh",
            "oil.close",
            "oil.open-workspace-root",
            "oil.set-root",
            "oil.show-help",
            "oil.cycle-sort",
            "oil.toggle-hidden",
            "oil.toggle-trash",
            "oil.open-external",
            "oil.set-tab-local-root",
            "oil.git-worktree",
        ] {
            assert!(names.contains(&name), "missing command {name}");
        }
    }

    #[test]
    fn help_lines_reflect_current_keybindings() {
        let bindings = keybindings();
        let prefixed = |suffix: &str| format!("{}{}", bindings.prefix, suffix);
        let lines = help_lines();
        assert!(lines.contains(&format!("{}: open file/directory", bindings.open_entry)));
        assert!(
            lines.contains(
                &"Use yy to copy the selected entry and p to paste it into the current directory."
                    .to_owned()
            )
        );
        assert!(lines.contains(&format!(
            "{}: toggle hidden files",
            prefixed(bindings.toggle_hidden)
        )));
        assert!(lines.contains(&format!(
            "{}: toggle trash",
            prefixed(bindings.toggle_trash)
        )));
        assert!(lines.contains(&format!(
            "{}: create git worktree",
            prefixed(bindings.create_git_worktree)
        )));
    }
}
