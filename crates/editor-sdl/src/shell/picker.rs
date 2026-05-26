use super::*;
use editor_fs::{ProjectCandidate, ProjectKind, ProjectSearchRoot, discover_projects};

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
        "workspace.dashboard" => git_worktree_dashboard_picker_overlay(runtime),
        "workspace.switch" => workspace_switch_picker_overlay(runtime),
        "workspace.delete" => workspace_delete_picker_overlay(runtime),
        "workspace.files" => workspace_file_picker_overlay(runtime),
        "workspace.search" => workspace_search_picker_overlay(runtime),
        "undo-tree" => undo_tree_picker_overlay(runtime),
        "themes" => theme_picker_overlay(runtime),
        "icon-fonts" => Ok(icon_font_picker_overlay(runtime)),
        "acp-clients" => Ok(acp_clients_picker_overlay(runtime)),
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
            quickfix: None,
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
            quickfix: None,
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
                quickfix: None,
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
                quickfix: None,
            }
        })
        .collect();

    Ok(PickerOverlay::from_entries("Tree-sitter Install", entries))
}

fn workspace_project_picker_overlay(runtime: &EditorRuntime) -> Result<PickerOverlay, String> {
    let roots = shell_user_library(runtime)
        .workspace_roots()
        .into_iter()
        .map(|root| ProjectSearchRoot::new(root.path, root.max_depth))
        .collect::<Vec<_>>();
    let entries = discover_projects(&roots)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|project| {
            let existing_workspace = find_workspace_by_root(runtime, project.root())?;
            let workspace_name = project.display_name();
            let detail = workspace_project_picker_detail(&project, existing_workspace.is_some());
            let action = existing_workspace.map_or(
                PickerAction::CreateWorkspace {
                    name: workspace_name.clone(),
                    root: project.root().to_path_buf(),
                },
                PickerAction::SwitchWorkspace,
            );
            Ok(PickerEntry {
                item: PickerItem::new(
                    project.root().display().to_string(),
                    workspace_name,
                    detail,
                    Some(workspace_project_picker_preview(&project)),
                ),
                action,
                quickfix: None,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;

    Ok(PickerOverlay::from_entries("Projects", entries))
}

fn workspace_project_picker_detail(project: &ProjectCandidate, is_open: bool) -> String {
    let mut parts = vec![project.kind().label().to_owned()];
    if project.kind() == ProjectKind::GitWorktree && project.repository_root() != project.root() {
        let context = project
            .worktree_parent_name()
            .unwrap_or_else(|| project.repository_display_name());
        parts.push(format!("project {context}"));
    }
    if is_open {
        parts.push("open workspace".to_owned());
    }
    parts.join(" | ")
}

fn workspace_project_picker_preview(project: &ProjectCandidate) -> String {
    if project.kind() == ProjectKind::GitWorktree && project.repository_root() != project.root() {
        return format!(
            "worktree {} | repo {}",
            project.root().display(),
            project.repository_root().display(),
        );
    }
    project.root().display().to_string()
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
            quickfix: None,
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
            quickfix: None,
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
            let search_text = workspace_relative_path(Some(root), &path);
            let label = relative_path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_else(|| search_text.clone());
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
                )
                .with_search_text(search_text)
                .with_fringe(editor_icons::seti_file_icon(&path)),
                action: PickerAction::OpenFile(path),
                quickfix: None,
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
            let command_names = binding.command_names();
            let description = command_names
                .iter()
                .map(|command_name| {
                    runtime
                        .commands()
                        .get(command_name)
                        .map(|definition| definition.description().to_owned())
                        .unwrap_or_else(|| {
                            format!("{command_name}: command description unavailable.")
                        })
                })
                .collect::<Vec<_>>()
                .join(" -> ");
            let command_label = command_names.join(" -> ");
            let scope = binding.scope().to_string();
            let mode = binding.vim_mode().to_string();
            PickerEntry {
                item: PickerItem::new(
                    format!("{scope}:{mode}:{}", binding.chord()),
                    format!("{} {}", binding.chord(), command_label),
                    format!("{} [{}] -> {}", binding.scope(), mode, command_label),
                    Some(description),
                ),
                action: if command_names.len() == 1 {
                    PickerAction::ExecuteCommand(binding.command_name().to_owned())
                } else {
                    PickerAction::ExecuteCommands(command_names.to_vec())
                },
                quickfix: None,
            }
        })
        .collect();

    let contextual = shell_user_library(runtime)
        .context_help_specs()
        .into_iter()
        .flat_map(|spec| contextual_keybinding_entries(&spec))
        .collect::<Vec<_>>();
    entries.extend(contextual);

    PickerOverlay::from_entries("Keybindings", entries)
}

fn icon_font_picker_overlay(runtime: &EditorRuntime) -> PickerOverlay {
    let entries = shell_user_library(runtime)
        .icon_symbols()
        .iter()
        .map(|symbol| {
            let label = format!("{} {}", symbol.glyph, symbol.name);
            let detail = format!("{} | {}", symbol.category.label(), symbol.codepoint_label());
            PickerEntry {
                item: PickerItem::new(symbol.id(), label, detail, Some(symbol.glyph.to_owned())),
                action: PickerAction::CopyToClipboard(symbol.glyph.to_owned()),
                quickfix: None,
            }
        })
        .collect();
    PickerOverlay::from_entries("Bundled Icon Fonts", entries)
}

fn acp_clients_picker_overlay(runtime: &EditorRuntime) -> PickerOverlay {
    let entries = shell_user_library(runtime)
        .acp_clients()
        .into_iter()
        .map(|client| {
            let detail = format!("{} {}", client.command, client.args.join(" "));
            PickerEntry {
                item: PickerItem::new(client.id.as_str(), client.label, detail, None::<String>),
                action: PickerAction::OpenAcpClient(client.id),
                quickfix: None,
            }
        })
        .collect();
    PickerOverlay::from_entries("ACP Clients", entries)
}

fn contextual_keybinding_entries(spec: &editor_plugin_api::ContextHelpSpec) -> Vec<PickerEntry> {
    spec.entries
        .iter()
        .map(|binding| PickerEntry {
            item: PickerItem::new(
                format!("{}:Normal:{}", spec.scope, binding.chord),
                format!("{} {} {}", binding.chord, spec.scope, binding.action),
                format!("{} [Normal] -> {}", spec.scope, binding.action),
                Some(binding.description.clone()),
            ),
            action: PickerAction::NoOp,
            quickfix: None,
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
                quickfix: None,
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
        quickfix_entries: BTreeMap::new(),
        submit_action: None,
        mode: PickerMode::Static,
        kind: PickerKind::Generic,
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
            quickfix: None,
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
            quickfix: None,
        },
        PickerEntry {
            item: PickerItem::new(
                format!("discard:{buffer_id}"),
                "Discard and Close",
                "Close the buffer without saving.",
                None::<String>,
            ),
            action: PickerAction::CloseBufferDiscard(buffer_id),
            quickfix: None,
        },
        PickerEntry {
            item: PickerItem::new(
                format!("cancel:{buffer_id}"),
                "Cancel",
                "Keep the buffer open.",
                None::<String>,
            ),
            action: PickerAction::NoOp,
            quickfix: None,
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
    let window_effects = current_window_effect_settings(theme_registry);
    let cell_width = fonts
        .primary()
        .size_of_char('M')
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0
        .max(1) as i32;
    let corner_radius = shared_corner_radius(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let base_foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let popup_background = theme_color(theme_registry, "ui.picker.background", base_background);
    let foreground = theme_color(theme_registry, TOKEN_PICKER_FOREGROUND, base_foreground);
    let highlight_background = adjust_color(popup_background, if is_dark { 16 } else { -16 });
    let picker_highlight = theme_color(
        theme_registry,
        "ui.picker.highlight",
        theme_color(
            theme_registry,
            "ui.statusline.active",
            Color::RGB(110, 170, 255),
        ),
    );
    let muted = theme_color(
        theme_registry,
        TOKEN_PICKER_MUTED,
        blend_color(foreground, popup_background, 0.25),
    );
    let list_foreground = blend_color(foreground, popup_background, 0.12);
    let subtle = theme_color(
        theme_registry,
        TOKEN_PICKER_SUBTLE,
        blend_color(foreground, popup_background, 0.4),
    );
    // Border using two rounded rectangles (outer border color, inner background)
    fill_overlay_surface_rounded_rect(
        target,
        PixelRectToRect::rect(
            popup_rect.x,
            popup_rect.y,
            popup_rect.width,
            popup_rect.height,
        ),
        corner_radius,
        picker_highlight,
        window_effects,
    )?;
    let inner_rect = PixelRectToRect::rect(
        popup_rect.x + 2,
        popup_rect.y + 2,
        popup_rect.width.saturating_sub(4),
        popup_rect.height.saturating_sub(4),
    );
    let inner_radius = corner_radius.saturating_sub(2);
    fill_overlay_surface_rounded_rect(
        target,
        inner_rect,
        inner_radius,
        popup_background,
        window_effects,
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
    let fringe_width_chars = picker_fringe_width_chars(picker.session().matches());
    let fringe_width = if fringe_width_chars == 0 {
        0
    } else {
        fringe_width_chars
            .saturating_mul(cell_width.max(1) as usize)
            .saturating_add(12) as u32
    };

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
        let label_x = content_left + fringe_width as i32;
        let text_width = content_width.saturating_sub(fringe_width);
        let label_width = (text_width * 2 / 5).max(160);
        let detail_x = label_x + label_width as i32 + 16;
        let detail_width = text_width.saturating_sub(label_width + 16);
        if selected {
            fill_overlay_surface_rect(
                target,
                PixelRectToRect::rect(
                    popup_rect.x + 12,
                    row_y - 2,
                    popup_rect.width.saturating_sub(24),
                    row_height as u32,
                ),
                highlight_background,
                window_effects,
            )?;
        }

        if let Some(fringe) = matched.item().fringe() {
            draw_text(
                target,
                content_left,
                row_y,
                fringe,
                if selected {
                    foreground
                } else {
                    list_foreground
                },
            )?;
        }
        let label = truncate_text_to_width(matched.item().label(), label_width, cell_width);
        let detail = truncate_text_to_width(matched.item().detail(), detail_width, cell_width);
        draw_text(
            target,
            label_x,
            row_y,
            &label,
            if selected {
                foreground
            } else {
                list_foreground
            },
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
            &truncate_text_to_width(preview, popup_rect.width.saturating_sub(32), cell_width),
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

fn picker_fringe_width_chars(matches: &[editor_picker::PickerMatch]) -> usize {
    matches
        .iter()
        .filter_map(|matched| matched.item().fringe())
        .map(|fringe| fringe.chars().count())
        .max()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::*;

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        env::temp_dir().join(format!("volt-picker-{label}-{unique}"))
    }

    #[test]
    fn workspace_project_picker_shows_repo_context_for_worktrees()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("worktree-context");
        let repo = root.join("repo-store");
        let gitdir = repo.join(".git").join("worktrees").join("feature");
        let worktree = root.join("project").join("feature");
        fs::create_dir_all(&gitdir)?;
        fs::create_dir_all(&worktree)?;
        fs::write(
            worktree.join(".git"),
            "gitdir: ../../repo-store/.git/worktrees/feature\n",
        )?;
        fs::write(gitdir.join("commondir"), "../../\n")?;

        let projects = discover_projects(&[ProjectSearchRoot::new(&root, 3)])?;
        let worktree_project = projects
            .iter()
            .find(|project| project.root() == worktree)
            .expect("worktree should be discovered");
        assert_eq!(worktree_project.display_name(), "project [feature]");
        assert_eq!(
            workspace_project_picker_detail(worktree_project, false),
            "git worktree | project project",
        );
        assert_eq!(
            workspace_project_picker_preview(worktree_project),
            format!("worktree {} | repo {}", worktree.display(), repo.display()),
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
