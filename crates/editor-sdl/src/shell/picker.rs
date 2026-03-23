use super::*;
use editor_fs::discover_projects;

pub(super) fn ensure_picker_keybindings(runtime: &mut EditorRuntime) -> Result<(), String> {
    let bindings = [
        ("F3", "picker.open-commands"),
        ("F4", "picker.open-buffers"),
        ("F5", "picker.toggle-popup-window"),
        ("F6", "picker.open-keybindings"),
    ];

    for (chord, command) in bindings {
        if !runtime.commands().contains(command) {
            continue;
        }
        if runtime.keymaps().contains(&KeymapScope::Global, chord) {
            continue;
        }
        runtime
            .register_key_binding(
                chord,
                command,
                KeymapScope::Global,
                CommandSource::UserPackage("picker".to_owned()),
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

pub(super) fn picker_overlay(
    runtime: &EditorRuntime,
    provider: &str,
) -> Result<PickerOverlay, String> {
    match provider {
        "commands" => Ok(command_picker_overlay(runtime)),
        "buffers" => buffer_picker_overlay(runtime),
        "buffers.close" => buffer_close_picker_overlay(runtime),
        "keybindings" => Ok(keybinding_picker_overlay(runtime)),
        "treesitter.languages" => treesitter_install_picker_overlay(runtime),
        "workspace.projects" => workspace_project_picker_overlay(runtime),
        "workspace.switch" => workspace_switch_picker_overlay(runtime),
        "workspace.delete" => workspace_delete_picker_overlay(runtime),
        "workspace.files" => workspace_file_picker_overlay(runtime),
        "workspace.search" => workspace_search_picker_overlay(runtime),
        "undo-tree" => undo_tree_picker_overlay(runtime),
        "themes" => theme_picker_overlay(runtime),
        "nerd-fonts" => Ok(nerd_font_picker_overlay()),
        "acp-clients" => Ok(acp_clients_picker_overlay()),
        other => Err(format!("unknown picker provider `{other}`")),
    }
}

fn command_picker_overlay(runtime: &EditorRuntime) -> PickerOverlay {
    let entries = runtime
        .commands()
        .definitions()
        .into_iter()
        .map(|definition| PickerEntry {
            item: PickerItem::new(
                definition.name(),
                definition.name(),
                definition.description(),
                Some(definition.description()),
            ),
            action: PickerAction::ExecuteCommand(definition.name().to_owned()),
        })
        .collect();

    PickerOverlay::from_entries("Command Palette", entries)
}

fn buffer_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let ui = shell_ui(runtime)?;
    let entries = ui
        .active_workspace_buffer_ids()
        .into_iter()
        .flatten()
        .filter_map(|buffer_id| ui.buffer(*buffer_id))
        .map(|buffer| PickerEntry {
            item: PickerItem::new(
                buffer.id().to_string(),
                buffer.display_name(),
                buffer.kind_label(),
                Some(format!(
                    "{} | row {}, col {}",
                    buffer.kind_label(),
                    buffer.cursor_row() + 1,
                    buffer.cursor_col() + 1,
                )),
            ),
            action: PickerAction::FocusBuffer(buffer.id()),
        })
        .collect();

    Ok(PickerOverlay::from_entries("Buffers", entries))
}

fn buffer_close_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let ui = shell_ui(runtime)?;
    let entries = ui
        .active_workspace_buffer_ids()
        .into_iter()
        .flatten()
        .filter_map(|buffer_id| ui.buffer(*buffer_id))
        .map(|buffer| {
            let dirty = if buffer.is_dirty() {
                "modified"
            } else {
                "clean"
            };
            PickerEntry {
                item: PickerItem::new(
                    buffer.id().to_string(),
                    buffer.display_name(),
                    format!("{} | {dirty}", buffer.kind_label()),
                    Some(format!(
                        "{} | row {}, col {}",
                        buffer.kind_label(),
                        buffer.cursor_row() + 1,
                        buffer.cursor_col() + 1,
                    )),
                ),
                action: PickerAction::CloseBuffer(buffer.id()),
            }
        })
        .collect();

    Ok(PickerOverlay::from_entries("Close Buffers", entries))
}

fn treesitter_install_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let registry = runtime
        .services()
        .get::<SyntaxRegistry>()
        .ok_or_else(|| "syntax registry service missing".to_owned())?;
    let entries = registry
        .languages()
        .map(|language| {
            let detail = match language.grammar() {
                Some(grammar) => {
                    let installed = registry.is_installed(language.id()).unwrap_or(false);
                    let status = if installed { "installed" } else { "missing" };
                    format!("{status} | {}", grammar.repository_url())
                }
                None => "built-in grammar".to_owned(),
            };
            let preview = language.grammar().map(|grammar| {
                grammar
                    .install_directory(registry.install_root())
                    .display()
                    .to_string()
            });
            PickerEntry {
                item: PickerItem::new(language.id(), language.id(), detail, preview),
                action: PickerAction::InstallTreeSitterLanguage(language.id().to_owned()),
            }
        })
        .collect();

    Ok(PickerOverlay::from_entries("Tree-sitter Install", entries))
}

fn workspace_project_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let entries = discover_projects(&user::workspace::project_search_roots())
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|project| {
            let existing_workspace = find_workspace_by_root(runtime, project.root())?;
            let detail = if existing_workspace.is_some() {
                format!("{} | open workspace", project.kind().label())
            } else {
                project.kind().label().to_owned()
            };
            let action = existing_workspace.map_or(
                PickerAction::CreateWorkspace {
                    name: project.name().to_owned(),
                    root: project.root().to_path_buf(),
                },
                PickerAction::SwitchWorkspace,
            );
            Ok(PickerEntry {
                item: PickerItem::new(
                    project.root().display().to_string(),
                    project.name(),
                    detail,
                    Some(project.root().display().to_string()),
                ),
                action,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(PickerOverlay::from_entries("Projects", entries))
}

pub(crate) fn workspace_switch_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    let entries = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?
        .workspaces()
        .map(|workspace| PickerEntry {
            item: PickerItem::new(
                workspace.id().to_string(),
                workspace.name(),
                workspace
                    .root()
                    .map(|root| root.display().to_string())
                    .unwrap_or_else(|| "default workspace".to_owned()),
                workspace.root().map(|root| root.display().to_string()),
            ),
            action: PickerAction::SwitchWorkspace(workspace.id()),
        })
        .collect();

    Ok(PickerOverlay::from_entries("Workspaces", entries))
}

pub(crate) fn workspace_delete_picker_overlay(
    runtime: &EditorRuntime,
) -> Result<PickerOverlay, String> {
    let default_workspace = shell_ui(runtime)?.default_workspace();
    let entries = runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?
        .workspaces()
        .filter(|workspace| workspace.id() != default_workspace)
        .map(|workspace| PickerEntry {
            item: PickerItem::new(
                workspace.id().to_string(),
                workspace.name(),
                workspace
                    .root()
                    .map(|root| root.display().to_string())
                    .unwrap_or_else(|| "workspace".to_owned()),
                Some("Deletes the selected workspace.".to_owned()),
            ),
            action: PickerAction::DeleteWorkspace(workspace.id()),
        })
        .collect();

    Ok(PickerOverlay::from_entries("Delete Workspace", entries))
}

fn workspace_file_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let workspace = runtime
        .model()
        .active_workspace()
        .map_err(|error| error.to_string())?;
    let Some(root) = workspace.root() else {
        return Ok(message_picker_overlay(
            "Workspace Files",
            "Workspace has no project root",
            "Open a project-backed workspace before listing files.",
            Some(
                "workspace.list-files works from a project workspace created by workspace.new."
                    .to_owned(),
            ),
        ));
    };

    let files = match list_repository_files(root) {
        Ok(files) => files,
        Err(error) => {
            return Ok(message_picker_overlay(
                "Workspace Files",
                "Unable to read workspace files",
                &error.to_string(),
                Some(root.display().to_string()),
            ));
        }
    };

    if files.is_empty() {
        return Ok(message_picker_overlay(
            "Workspace Files",
            "No visible files found",
            "Git did not report any tracked or unignored files for this workspace.",
            Some(root.display().to_string()),
        ));
    }

    let entries = files
        .into_iter()
        .map(|relative_path| {
            let path = root.join(&relative_path);
            let label = workspace_relative_path(Some(root), &path);
            let detail = relative_path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .map(|parent| parent.display().to_string())
                .unwrap_or_else(|| "workspace root".to_owned());
            PickerEntry {
                item: PickerItem::new(
                    path.display().to_string(),
                    label,
                    detail,
                    Some(path.display().to_string()),
                ),
                action: PickerAction::OpenFile(path),
            }
        })
        .collect();

    let mut overlay = PickerOverlay::from_entries("Workspace Files", entries);
    overlay.submit_action = Some(PickerAction::CreateWorkspaceFile {
        root: root.to_path_buf(),
    });
    Ok(overlay)
}

fn workspace_search_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let Some(root) = active_workspace_root(runtime)? else {
        return Ok(message_picker_overlay(
            "Workspace Search",
            "Workspace has no project root",
            "Open a project-backed workspace before searching across files.",
            Some(
                "workspace.search works from a project workspace created by workspace.new."
                    .to_owned(),
            ),
        ));
    };

    Ok(PickerOverlay::workspace_search("Workspace Search", root))
}

fn keybinding_picker_overlay(runtime: &EditorRuntime) -> PickerOverlay {
    let mut entries: Vec<PickerEntry> = runtime
        .keymaps()
        .bindings()
        .into_iter()
        .map(|binding| {
            let description = runtime
                .commands()
                .get(binding.command_name())
                .map(|definition| definition.description().to_owned())
                .unwrap_or_else(|| "Command description unavailable.".to_owned());
            let scope = binding.scope().to_string();
            let mode = binding.vim_mode().to_string();
            PickerEntry {
                item: PickerItem::new(
                    format!("{scope}:{mode}:{}", binding.chord()),
                    format!("{} {}", binding.chord(), binding.command_name()),
                    format!(
                        "{} [{}] -> {}",
                        binding.scope(),
                        mode,
                        binding.command_name()
                    ),
                    Some(description),
                ),
                action: PickerAction::ExecuteCommand(binding.command_name().to_owned()),
            }
        })
        .collect();

    entries.extend(contextual_keybinding_entries(
        "GitStatus",
        GIT_STATUS_KEYBINDINGS,
    ));
    entries.extend(contextual_keybinding_entries(
        "GitView",
        GIT_VIEW_KEYBINDINGS,
    ));
    entries.extend(contextual_keybinding_entries("Oil", OIL_KEYBINDINGS));

    PickerOverlay::from_entries("Keybindings", entries)
}

fn nerd_font_picker_overlay() -> PickerOverlay {
    let entries = user::nerd_font::symbols()
        .iter()
        .map(|symbol| {
            let label = format!("{} {}", symbol.glyph, symbol.name);
            let detail = format!("{} | {}", symbol.category.label(), symbol.codepoint_label());
            PickerEntry {
                item: PickerItem::new(symbol.id(), label, detail, Some(symbol.glyph.to_owned())),
                action: PickerAction::CopyToClipboard(symbol.glyph.to_owned()),
            }
        })
        .collect();
    PickerOverlay::from_entries("Nerd Font Symbols", entries)
}

fn acp_clients_picker_overlay() -> PickerOverlay {
    let entries = user::acp::clients()
        .into_iter()
        .map(|client| {
            let detail = format!("{} {}", client.command, client.args.join(" "));
            PickerEntry {
                item: PickerItem::new(client.id.as_str(), client.label, detail, None::<String>),
                action: PickerAction::OpenAcpClient(client.id),
            }
        })
        .collect();
    PickerOverlay::from_entries("ACP Clients", entries)
}

#[derive(Debug, Clone, Copy)]
struct ContextKeybinding {
    chord: &'static str,
    action: &'static str,
    description: &'static str,
}

const GIT_STATUS_KEYBINDINGS: &[ContextKeybinding] = &[
    ContextKeybinding {
        chord: "g",
        action: "refresh status",
        description: "Refreshes the git status buffer.",
    },
    ContextKeybinding {
        chord: "n",
        action: "next section",
        description: "Moves to the next git status section.",
    },
    ContextKeybinding {
        chord: "p",
        action: "previous section",
        description: "Moves to the previous git status section.",
    },
    ContextKeybinding {
        chord: "s",
        action: "stage file / stage all",
        description: "Stages the file under the cursor, or all if none is selected.",
    },
    ContextKeybinding {
        chord: "S",
        action: "stage all",
        description: "Stages all unstaged changes.",
    },
    ContextKeybinding {
        chord: "u",
        action: "unstage file",
        description: "Unstages the file under the cursor.",
    },
    ContextKeybinding {
        chord: "U",
        action: "unstage all",
        description: "Unstages all staged changes.",
    },
    ContextKeybinding {
        chord: "c",
        action: "commit prefix",
        description: "Starts the commit prefix (press c again to open commit).",
    },
    ContextKeybinding {
        chord: "c c",
        action: "open commit buffer",
        description: "Opens the git commit buffer.",
    },
    ContextKeybinding {
        chord: "P",
        action: "push prefix",
        description: "Starts the push prefix (p pushremote, u upstream).",
    },
    ContextKeybinding {
        chord: "P p",
        action: "push to pushremote",
        description: "Pushes to the push remote.",
    },
    ContextKeybinding {
        chord: "P u",
        action: "push to upstream",
        description: "Pushes to the upstream remote.",
    },
    ContextKeybinding {
        chord: "f",
        action: "fetch prefix",
        description: "Starts the fetch prefix (p pushremote, u upstream, a all).",
    },
    ContextKeybinding {
        chord: "f p",
        action: "fetch pushremote",
        description: "Fetches from the push remote.",
    },
    ContextKeybinding {
        chord: "f u",
        action: "fetch upstream",
        description: "Fetches from the upstream remote.",
    },
    ContextKeybinding {
        chord: "f a",
        action: "fetch all",
        description: "Fetches from all remotes.",
    },
    ContextKeybinding {
        chord: "b",
        action: "branch prefix",
        description: "Starts the branch prefix (press b again for branches).",
    },
    ContextKeybinding {
        chord: "b b",
        action: "open branch picker",
        description: "Opens the git branch picker.",
    },
    ContextKeybinding {
        chord: "d",
        action: "diff prefix",
        description: "Starts the diff prefix (d/s/u/w/c/t).",
    },
    ContextKeybinding {
        chord: "d d",
        action: "diff dwim",
        description: "Opens the diff most relevant to the cursor line.",
    },
    ContextKeybinding {
        chord: "d s",
        action: "diff staged",
        description: "Opens the staged diff.",
    },
    ContextKeybinding {
        chord: "d u",
        action: "diff unstaged",
        description: "Opens the unstaged diff.",
    },
    ContextKeybinding {
        chord: "d w",
        action: "diff worktree",
        description: "Opens the worktree diff.",
    },
    ContextKeybinding {
        chord: "d c",
        action: "diff commit at point",
        description: "Opens the diff for the commit at the cursor.",
    },
    ContextKeybinding {
        chord: "d t",
        action: "diff stash at point",
        description: "Opens the diff for the stash at the cursor.",
    },
    ContextKeybinding {
        chord: "l",
        action: "log prefix",
        description: "Starts the log prefix (l/h/u/L/b/a).",
    },
    ContextKeybinding {
        chord: "l l",
        action: "log current",
        description: "Opens the log for the current file or selection.",
    },
    ContextKeybinding {
        chord: "l h",
        action: "log head",
        description: "Opens the HEAD log.",
    },
    ContextKeybinding {
        chord: "l u",
        action: "log related",
        description: "Opens the log related to the cursor selection.",
    },
    ContextKeybinding {
        chord: "l L",
        action: "log branches",
        description: "Opens logs for branches.",
    },
    ContextKeybinding {
        chord: "l b",
        action: "log all branches",
        description: "Opens logs across all branches.",
    },
    ContextKeybinding {
        chord: "l a",
        action: "log all",
        description: "Opens the full log.",
    },
    ContextKeybinding {
        chord: "z",
        action: "stash prefix",
        description: "Starts the stash prefix (z/i/w/x/a/p/k/v/l).",
    },
    ContextKeybinding {
        chord: "z z",
        action: "stash both",
        description: "Stashes both staged and unstaged changes.",
    },
    ContextKeybinding {
        chord: "z i",
        action: "stash index",
        description: "Stashes staged changes.",
    },
    ContextKeybinding {
        chord: "z w",
        action: "stash worktree",
        description: "Stashes unstaged changes.",
    },
    ContextKeybinding {
        chord: "z x",
        action: "stash keep index",
        description: "Stashes unstaged changes and keeps the index.",
    },
    ContextKeybinding {
        chord: "z a",
        action: "stash apply at point",
        description: "Applies the stash under the cursor.",
    },
    ContextKeybinding {
        chord: "z p",
        action: "stash pop at point",
        description: "Pops the stash under the cursor.",
    },
    ContextKeybinding {
        chord: "z k",
        action: "stash drop at point",
        description: "Drops the stash under the cursor.",
    },
    ContextKeybinding {
        chord: "z v",
        action: "stash show at point",
        description: "Shows the stash under the cursor.",
    },
    ContextKeybinding {
        chord: "z l",
        action: "open stash list",
        description: "Opens the stash list buffer.",
    },
    ContextKeybinding {
        chord: "m",
        action: "merge prefix",
        description: "Starts the merge prefix (m/e/n/s/p/a).",
    },
    ContextKeybinding {
        chord: "m m",
        action: "merge plain / continue",
        description: "Merges the selected branch or continues a merge.",
    },
    ContextKeybinding {
        chord: "m e",
        action: "merge edit message",
        description: "Merges and opens the commit message editor.",
    },
    ContextKeybinding {
        chord: "m n",
        action: "merge no commit",
        description: "Merges without committing.",
    },
    ContextKeybinding {
        chord: "m s",
        action: "merge squash",
        description: "Performs a squash merge.",
    },
    ContextKeybinding {
        chord: "m p",
        action: "merge preview",
        description: "Previews a merge.",
    },
    ContextKeybinding {
        chord: "m a",
        action: "merge abort",
        description: "Aborts the current merge.",
    },
    ContextKeybinding {
        chord: "r",
        action: "rebase prefix",
        description: "Starts the rebase prefix (p/u/e/i/r/s/a).",
    },
    ContextKeybinding {
        chord: "r p",
        action: "rebase onto pushremote",
        description: "Rebases onto the push remote.",
    },
    ContextKeybinding {
        chord: "r u",
        action: "rebase onto upstream",
        description: "Rebases onto the upstream.",
    },
    ContextKeybinding {
        chord: "r e",
        action: "rebase edit / onto branch",
        description: "Edits the rebase todo or rebases onto a branch.",
    },
    ContextKeybinding {
        chord: "r i",
        action: "rebase interactive",
        description: "Starts an interactive rebase.",
    },
    ContextKeybinding {
        chord: "r r",
        action: "rebase continue",
        description: "Continues the current rebase.",
    },
    ContextKeybinding {
        chord: "r s",
        action: "rebase skip",
        description: "Skips the current rebase step.",
    },
    ContextKeybinding {
        chord: "r a",
        action: "rebase abort",
        description: "Aborts the current rebase.",
    },
    ContextKeybinding {
        chord: "Y",
        action: "open cherry buffer",
        description: "Opens the cherry-pick buffer.",
    },
    ContextKeybinding {
        chord: "A",
        action: "cherry-pick prefix",
        description: "Starts the cherry-pick prefix (A/a/s).",
    },
    ContextKeybinding {
        chord: "A A",
        action: "cherry-pick / continue",
        description: "Cherry-picks the selected commit or continues.",
    },
    ContextKeybinding {
        chord: "A a",
        action: "cherry-pick apply / abort",
        description: "Applies a cherry-pick or aborts in progress.",
    },
    ContextKeybinding {
        chord: "A s",
        action: "cherry-pick skip",
        description: "Skips the current cherry-pick.",
    },
    ContextKeybinding {
        chord: "V",
        action: "revert prefix",
        description: "Starts the revert prefix (V/v/s/a).",
    },
    ContextKeybinding {
        chord: "V V",
        action: "revert / continue",
        description: "Reverts the selected commit or continues.",
    },
    ContextKeybinding {
        chord: "V v",
        action: "revert no-commit / abort",
        description: "Reverts without commit or aborts in progress.",
    },
    ContextKeybinding {
        chord: "V s",
        action: "revert skip",
        description: "Skips the current revert/cherry-pick.",
    },
    ContextKeybinding {
        chord: "V a",
        action: "revert abort",
        description: "Aborts the current revert/cherry-pick.",
    },
    ContextKeybinding {
        chord: "a",
        action: "cherry-pick apply at point",
        description: "Applies the commit under the cursor.",
    },
    ContextKeybinding {
        chord: "X",
        action: "reset prefix",
        description: "Starts the reset prefix (m/s/h/k).",
    },
    ContextKeybinding {
        chord: "X m",
        action: "reset mixed",
        description: "Resets to the selected commit (mixed).",
    },
    ContextKeybinding {
        chord: "X s",
        action: "reset soft",
        description: "Resets to the selected commit (soft).",
    },
    ContextKeybinding {
        chord: "X h",
        action: "reset hard",
        description: "Resets to the selected commit (hard).",
    },
    ContextKeybinding {
        chord: "X k",
        action: "reset keep",
        description: "Resets to the selected commit (keep).",
    },
    ContextKeybinding {
        chord: "x",
        action: "delete file(s)",
        description: "Deletes the file under the cursor or the visual selection.",
    },
];

const GIT_VIEW_KEYBINDINGS: &[ContextKeybinding] = &[ContextKeybinding {
    chord: "g",
    action: "refresh view",
    description: "Refreshes git diff/log/stash buffers.",
}];

const OIL_KEYBINDINGS: &[ContextKeybinding] = &[
    ContextKeybinding {
        chord: "Enter",
        action: "open",
        description: "Opens the file or enters the selected directory.",
    },
    ContextKeybinding {
        chord: "Ctrl+s",
        action: "open vertical split",
        description: "Opens the selection in a vertical split.",
    },
    ContextKeybinding {
        chord: "Ctrl+h",
        action: "open horizontal split",
        description: "Opens the selection in a horizontal split.",
    },
    ContextKeybinding {
        chord: "Ctrl+t",
        action: "open new pane",
        description: "Opens the selection in a new pane.",
    },
    ContextKeybinding {
        chord: "Ctrl+p",
        action: "preview",
        description: "Previews the selected file.",
    },
    ContextKeybinding {
        chord: "Ctrl+l",
        action: "refresh",
        description: "Refreshes the directory listing.",
    },
    ContextKeybinding {
        chord: "Ctrl+c",
        action: "close",
        description: "Closes the directory buffer.",
    },
    ContextKeybinding {
        chord: "-",
        action: "parent directory",
        description: "Navigates to the parent directory.",
    },
    ContextKeybinding {
        chord: "_",
        action: "workspace root",
        description: "Navigates to the workspace root.",
    },
    ContextKeybinding {
        chord: "`",
        action: "set root",
        description: "Sets the directory root to the selection.",
    },
    ContextKeybinding {
        chord: "g~",
        action: "set root (tab)",
        description: "Sets the directory root to the selection (tab-local).",
    },
    ContextKeybinding {
        chord: "gs",
        action: "change sort",
        description: "Cycles the directory sort order.",
    },
    ContextKeybinding {
        chord: "g.",
        action: "toggle hidden",
        description: "Toggles hidden file visibility.",
    },
    ContextKeybinding {
        chord: "g\\",
        action: "toggle trash",
        description: "Toggles trash usage for deletions.",
    },
    ContextKeybinding {
        chord: "gx",
        action: "open external",
        description: "Opens the selection externally.",
    },
    ContextKeybinding {
        chord: "g?",
        action: "help",
        description: "Shows the oil help popup.",
    },
];

fn contextual_keybinding_entries(scope: &str, bindings: &[ContextKeybinding]) -> Vec<PickerEntry> {
    bindings
        .iter()
        .map(|binding| PickerEntry {
            item: PickerItem::new(
                format!("{scope}:Normal:{}", binding.chord),
                format!("{} {scope} {}", binding.chord, binding.action),
                format!("{scope} [Normal] -> {}", binding.action),
                Some(binding.description.to_owned()),
            ),
            action: PickerAction::NoOp,
        })
        .collect()
}

fn theme_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let registry = runtime
        .services()
        .get::<ThemeRegistry>()
        .ok_or_else(|| "theme registry service missing".to_owned())?;
    let entries = registry
        .themes()
        .map(|theme| {
            let theme_id = theme.id().to_owned();
            PickerEntry {
                item: PickerItem::new(&theme_id, theme.name(), "Theme", Some(theme_id.clone())),
                action: PickerAction::ActivateTheme(theme_id),
            }
        })
        .collect();
    Ok(PickerOverlay::from_entries("Themes", entries))
}

fn undo_tree_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_ui(runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "active buffer is missing".to_owned())?;
    let (entries, selected_index) = buffer.undo_tree_entries();
    if entries.is_empty() {
        return Ok(message_picker_overlay(
            "Undo Tree",
            "No undo history",
            "Make an edit to populate the undo tree.",
            None::<String>,
        ));
    }
    let mut actions = BTreeMap::new();
    let items = entries
        .into_iter()
        .map(|entry| {
            let item_id = format!("undo:{}", entry.node_id);
            actions.insert(
                item_id.clone(),
                PickerAction::UndoTreeNode {
                    buffer_id,
                    node_id: entry.node_id,
                },
            );
            PickerItem::new(item_id, entry.label, entry.detail, entry.preview)
        })
        .collect();
    let mut session = PickerSession::new("Undo Tree", items)
        .with_preserve_order()
        .with_result_limit(256);
    session.set_selected_index(selected_index);
    Ok(PickerOverlay {
        session,
        actions,
        submit_action: None,
        mode: PickerMode::Static,
    })
}

fn message_picker_overlay(
    title: &str,
    label: &str,
    detail: &str,
    preview: Option<String>,
) -> PickerOverlay {
    PickerOverlay::from_entries(
        title,
        vec![PickerEntry {
            item: PickerItem::new(label, label, detail, preview),
            action: PickerAction::NoOp,
        }],
    )
}

pub(super) fn buffer_close_confirm_overlay(
    buffer_id: BufferId,
    buffer_name: &str,
) -> PickerOverlay {
    let entries = vec![
        PickerEntry {
            item: PickerItem::new(
                format!("save:{buffer_id}"),
                "Save and Close",
                "Write changes then close the buffer.",
                None::<String>,
            ),
            action: PickerAction::CloseBufferSave(buffer_id),
        },
        PickerEntry {
            item: PickerItem::new(
                format!("discard:{buffer_id}"),
                "Discard and Close",
                "Close the buffer without saving.",
                None::<String>,
            ),
            action: PickerAction::CloseBufferDiscard(buffer_id),
        },
        PickerEntry {
            item: PickerItem::new(
                format!("cancel:{buffer_id}"),
                "Cancel",
                "Keep the buffer open.",
                None::<String>,
            ),
            action: PickerAction::NoOp,
        },
    ];
    PickerOverlay::from_entries(format!("Close {buffer_name}?"), entries)
}

pub(super) fn render_picker_overlay(
    target: &mut DrawTarget<'_>,
    fonts: &FontSet<'_>,
    picker: &PickerOverlay,
    width: u32,
    height: u32,
    line_height: i32,
    theme_registry: Option<&ThemeRegistry>,
) -> Result<(), ShellError> {
    let popup_rect = centered_rect(width, height, width * 2 / 3, height * 3 / 5);
    let picker_roundness = theme_registry
        .and_then(|registry| registry.resolve_number(OPTION_PICKER_ROUNDNESS))
        .map(|value| value.clamp(0.0, 64.0).round() as u32)
        .unwrap_or(16);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(29, 32, 40));
    let foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let popup_background = adjust_color(base_background, if is_dark { 8 } else { -8 });
    let highlight_background = adjust_color(popup_background, if is_dark { 16 } else { -16 });
    let muted = blend_color(foreground, base_background, 0.5);
    let subtle = blend_color(foreground, base_background, 0.7);
    fill_rounded_rect(
        target,
        PixelRectToRect::rect(
            popup_rect.x,
            popup_rect.y,
            popup_rect.width,
            popup_rect.height,
        ),
        picker_roundness,
        popup_background,
    )?;
    fill_rect(
        target,
        PixelRectToRect::rect(
            popup_rect.x + 14,
            popup_rect.y,
            popup_rect.width.saturating_sub(28),
            2,
        ),
        Color::RGB(110, 170, 255),
    )?;

    draw_text(
        target,
        popup_rect.x + 16,
        popup_rect.y + 16,
        picker.session().title(),
        foreground,
    )?;

    let query = format!("Query > {}", picker.session().query());
    draw_text(
        target,
        popup_rect.x + 16,
        popup_rect.y + line_height + 24,
        &query,
        muted,
    )?;

    let summary = format!(
        "{} / {} results",
        picker.session().match_count(),
        picker.session().item_count(),
    );
    draw_text(
        target,
        popup_rect.x + 16,
        popup_rect.y + (line_height * 2) + 28,
        &summary,
        subtle,
    )?;

    let row_height = (line_height + 8).max(24);
    let list_top = popup_rect.y + (line_height * 3) + 42;
    let list_height = popup_rect.height as i32 - ((line_height * 4) + 62).max(0);
    let visible_rows = (list_height.max(row_height) / row_height).max(1) as usize;
    let selected_id = picker
        .session()
        .selected()
        .map(|selected| selected.item().id().to_owned());
    let selected_index = selected_id
        .as_deref()
        .and_then(|selected_id| {
            picker
                .session()
                .matches()
                .iter()
                .position(|matched| matched.item().id() == selected_id)
        })
        .unwrap_or(0);
    let scroll_top =
        picker_scroll_top(picker.session().match_count(), selected_index, visible_rows);

    if picker.session().matches().is_empty() {
        draw_text(target, popup_rect.x + 16, list_top, "No matches.", subtle)?;
        return Ok(());
    }

    for (index, matched) in picker
        .session()
        .matches()
        .iter()
        .skip(scroll_top)
        .take(visible_rows)
        .enumerate()
    {
        let row_y = list_top + index as i32 * row_height;
        let selected = selected_id.as_deref() == Some(matched.item().id());
        let content_left = popup_rect.x + 18;
        let content_width = popup_rect.width.saturating_sub(36);
        let label_width = (content_width * 2 / 5).max(160);
        let detail_x = content_left + label_width as i32 + 16;
        let detail_width = content_width.saturating_sub(label_width + 16);
        if selected {
            fill_rect(
                target,
                PixelRectToRect::rect(
                    popup_rect.x + 12,
                    row_y - 2,
                    popup_rect.width.saturating_sub(24),
                    row_height as u32,
                ),
                highlight_background,
            )?;
        }

        let label = truncate_text_to_width(fonts, matched.item().label(), label_width)?;
        let detail = truncate_text_to_width(fonts, matched.item().detail(), detail_width)?;
        draw_text(
            target,
            content_left,
            row_y,
            &label,
            if selected { foreground } else { muted },
        )?;
        draw_text(target, detail_x, row_y, &detail, muted)?;
    }

    if let Some(preview) = picker
        .session()
        .selected()
        .and_then(|selected| selected.item().preview())
    {
        draw_text(
            target,
            popup_rect.x + 16,
            popup_rect.y + popup_rect.height as i32 - line_height - 18,
            &truncate_text_to_width(fonts, preview, popup_rect.width.saturating_sub(32))?,
            subtle,
        )?;
    }

    Ok(())
}

fn picker_scroll_top(match_count: usize, selected_index: usize, visible_rows: usize) -> usize {
    let visible_rows = visible_rows.max(1);
    if match_count <= visible_rows {
        return 0;
    }

    selected_index
        .saturating_sub(visible_rows.saturating_sub(1))
        .min(match_count - visible_rows)
}
