use super::*;

/// Retained Workspace Files rows. One-letter queries must not clone the whole tree.
const WORKSPACE_FILES_PICKER_RESULT_LIMIT: usize = 256;
/// Retained command-palette rows. Finite but higher than Workspace Files.
const COMMAND_PICKER_RESULT_LIMIT: usize = 512;

fn overlay_result_limit(source: PickerSource) -> usize {
    match source {
        PickerSource::WorkspaceFiles => WORKSPACE_FILES_PICKER_RESULT_LIMIT,
        PickerSource::Commands => COMMAND_PICKER_RESULT_LIMIT,
        _ => usize::MAX,
    }
}

fn provider_extra_keybinds(spec: &PickerProviderSpec) -> Vec<PickerExtraKeybind> {
    spec.extra_keybinds()
        .iter()
        .map(|binding| PickerExtraKeybind::new(binding.chord(), binding.command_name()))
        .collect()
}

fn with_provider_extras(overlay: PickerOverlay, spec: &PickerProviderSpec) -> PickerOverlay {
    overlay.with_extra_keybinds(provider_extra_keybinds(spec))
}

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
    let spec = shell_user_library(runtime)
        .picker_providers()
        .into_iter()
        .find(|spec| spec.id() == provider)
        .ok_or_else(|| format!("unknown picker provider `{provider}`"))?;
    picker_overlay_from_spec(runtime, &spec)
}

fn picker_overlay_from_spec(
    runtime: &EditorRuntime,
    spec: &PickerProviderSpec,
) -> Result<PickerOverlay, String> {
    if spec.source() == PickerSource::WorkspaceSearch {
        return workspace_search_picker_overlay(runtime)
            .map(|overlay| with_provider_extras(overlay.with_title(spec.title()), spec));
    }
    if spec.source() == PickerSource::WorkspaceDashboard {
        return Ok(with_provider_extras(
            git_worktree_dashboard_picker_overlay(runtime)
                .unwrap_or_else(workspace_dashboard_unavailable_overlay)
                .with_title(spec.title()),
            spec,
        ));
    }

    let context = picker_provider_context(runtime, spec)?;
    user_picker_overlay(runtime, spec, context)
}

fn user_picker_overlay(
    runtime: &EditorRuntime,
    spec: &PickerProviderSpec,
    context: PickerProviderContext,
) -> Result<PickerOverlay, String> {
    let items = shell_user_library(runtime)
        .picker_provider_items(&context)
        .unwrap_or_else(|| spec.items().to_vec());
    let entries = items
        .iter()
        .map(|item| static_picker_entry(runtime, item))
        .collect::<Result<Vec<_>, String>>()?;
    let mut overlay = PickerOverlay::from_entries_with_limit(
        spec.title(),
        entries,
        overlay_result_limit(spec.source()),
    )
    .with_title(spec.title())
    .with_source(spec.source())
    .with_provider_id(spec.id());
    if matches!(
        spec.source(),
        PickerSource::Buffers
            | PickerSource::BufferClose
            | PickerSource::WorkspaceFiles
            | PickerSource::UndoTree
    ) {
        overlay = overlay.with_preview();
    }
    if spec.source() == PickerSource::WorkspaceFiles
        && let Some(root) = context.workspace_root.as_ref().into_option()
    {
        overlay.submit_action = Some(PickerAction::CreateWorkspaceFile {
            root: PathBuf::from(root.as_str()),
        });
    }
    if matches!(
        spec.source(),
        PickerSource::WorkspaceProjects | PickerSource::WorkspaceSwitch
    ) {
        overlay = overlay
            .with_result_order(PickerResultOrder::Source)
            .with_project_discovery_revision(
                editor_fs::current_project_discovery_snapshot().revision(),
            );
    }
    if spec.source() == PickerSource::UndoTree {
        overlay = overlay.with_result_order(PickerResultOrder::Source);
        if let Some(selected_index) = context.undo_tree.iter().position(|entry| entry.selected) {
            overlay.session.set_selected_index(selected_index);
        }
    }
    overlay.ensure_selected_workspace_file_preview();
    Ok(with_provider_extras(overlay, spec))
}

pub(super) fn picker_entries(
    runtime: &EditorRuntime,
    provider: &str,
) -> Result<Vec<PickerEntry>, String> {
    let spec = shell_user_library(runtime)
        .picker_providers()
        .into_iter()
        .find(|spec| spec.id() == provider)
        .ok_or_else(|| format!("unknown picker provider `{provider}`"))?;
    let context = picker_provider_context(runtime, &spec)?;
    let items = shell_user_library(runtime)
        .picker_provider_items(&context)
        .unwrap_or_else(|| spec.items().to_vec());
    items
        .iter()
        .map(|item| static_picker_entry(runtime, item))
        .collect()
}

fn workspace_dashboard_unavailable_overlay(error: String) -> PickerOverlay {
    PickerOverlay::from_entries(
        "Workspace Dashboard",
        vec![PickerEntry {
            item: PickerItem::new(
                "workspace-dashboard-unavailable",
                "Workspace dashboard unavailable",
                error.clone(),
                Some(error),
            ),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    )
}

fn picker_provider_context(
    runtime: &EditorRuntime,
    spec: &PickerProviderSpec,
) -> Result<PickerProviderContext, String> {
    let mut context = PickerProviderContext::new(spec.id(), spec.title(), spec.source());
    match spec.source() {
        PickerSource::User
        | PickerSource::Static
        | PickerSource::WorkspaceDashboard
        | PickerSource::WorkspaceSearch => {}
        PickerSource::Commands => {
            context.commands = runtime
                .commands()
                .definitions()
                .into_iter()
                .map(|definition| PickerCommandContext {
                    name: definition.name().into(),
                    description: definition.description().into(),
                })
                .collect::<Vec<_>>()
                .into();
        }
        PickerSource::Buffers | PickerSource::BufferClose => {
            let ui = shell_ui(runtime)?;
            context.buffers = ui
                .active_workspace_buffer_ids()
                .into_iter()
                .flatten()
                .filter_map(|buffer_id| ui.buffer(*buffer_id))
                .map(|buffer| PickerBufferContext {
                    id: buffer.id().get(),
                    display_name: buffer.display_name().into(),
                    path: buffer
                        .path()
                        .map(|path| path.display().to_string().into())
                        .into(),
                    kind_label: buffer.kind_label().into(),
                    preview: Some(buffer_picker_preview(buffer).into()).into(),
                    cursor_row: buffer.cursor_row(),
                    cursor_col: buffer.cursor_col(),
                    dirty: buffer.is_dirty(),
                })
                .collect::<Vec<_>>()
                .into();
        }
        PickerSource::Keybindings => {
            context.keybindings = runtime
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
                    let scope = binding.scope().to_string();
                    let mode = binding.vim_mode().to_string();
                    PickerKeybindingContext {
                        id: format!("{scope}:{mode}:{}", binding.chord()).into(),
                        chord: binding.chord().into(),
                        scope: scope.into(),
                        vim_mode: mode.into(),
                        command_names: command_names
                            .iter()
                            .map(|command_name| command_name.as_str().into())
                            .collect::<Vec<_>>()
                            .into(),
                        description: description.into(),
                    }
                })
                .collect::<Vec<_>>()
                .into();
        }
        PickerSource::TreesitterLanguages => {
            let registry = runtime
                .services()
                .get::<SyntaxRegistry>()
                .ok_or_else(|| "syntax registry service missing".to_owned())?;
            context.syntax_languages = registry
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
                            .installed_library_path(registry.install_root())
                            .display()
                            .to_string()
                    });
                    PickerSyntaxLanguageContext {
                        id: language.id().into(),
                        detail: detail.into(),
                        preview: preview.map(Into::into).into(),
                        is_installed: language
                            .grammar()
                            .is_some_and(|_| registry.is_installed(language.id()).unwrap_or(false)),
                    }
                })
                .collect::<Vec<_>>()
                .into();
        }
        PickerSource::WorkspaceProjects => {
            context.workspaces = workspace_contexts(runtime, None)?.into();
        }
        PickerSource::WorkspaceSwitch => {
            context.workspaces = workspace_contexts(runtime, None)?.into();
        }
        PickerSource::WorkspaceDelete => {
            context.workspaces =
                workspace_contexts(runtime, Some(shell_ui(runtime)?.default_workspace()))?.into();
        }
        PickerSource::WorkspaceFiles => {
            context.workspace_root = active_workspace_root(runtime)?
                .map(|root| root.display().to_string().into())
                .into();
        }
        PickerSource::Themes => {
            let registry = runtime
                .services()
                .get::<ThemeRegistry>()
                .ok_or_else(|| "theme registry service missing".to_owned())?;
            context.themes = registry
                .themes()
                .map(|theme| PickerThemeContext {
                    id: theme.id().into(),
                    name: theme.name().into(),
                })
                .collect::<Vec<_>>()
                .into();
        }
        PickerSource::IconFonts => {
            context.icons = shell_user_library(runtime)
                .icon_symbols()
                .iter()
                .map(|symbol| PickerIconContext {
                    id: symbol.id().into(),
                    label: format!("{} {}", symbol.glyph, symbol.name).into(),
                    detail: format!("{} | {}", symbol.category.label(), symbol.codepoint_label())
                        .into(),
                    glyph: symbol.glyph.into(),
                })
                .collect::<Vec<_>>()
                .into();
        }
        PickerSource::AcpClients => {
            context.acp_clients = shell_user_library(runtime)
                .acp_clients()
                .into_iter()
                .map(|client| PickerAcpClientContext {
                    id: client.id.into(),
                    label: client.label.into(),
                    detail: format!("{} {}", client.command, client.args.join(" ")).into(),
                })
                .collect::<Vec<_>>()
                .into();
        }
        PickerSource::UndoTree => {
            let buffer_id = active_shell_buffer_id(runtime)?;
            let buffer = shell_ui(runtime)?
                .buffer(buffer_id)
                .ok_or_else(|| "active buffer is missing".to_owned())?;
            let (entries, selected_index) = buffer.undo_tree_entries();
            context.undo_tree = entries
                .into_iter()
                .enumerate()
                .map(|(index, entry)| PickerUndoTreeContext {
                    buffer_id: buffer_id.get(),
                    node_id: entry.node_id,
                    fringe: entry.fringe.into(),
                    label: entry.label.into(),
                    detail: entry.detail.into(),
                    preview: entry.preview.map(Into::into).into(),
                    selected: index == selected_index,
                })
                .collect::<Vec<_>>()
                .into();
        }
    }
    Ok(context)
}

fn workspace_contexts(
    runtime: &EditorRuntime,
    hidden_workspace: Option<WorkspaceId>,
) -> Result<Vec<PickerWorkspaceContext>, String> {
    let default_workspace = shell_ui(runtime).ok().map(|ui| ui.default_workspace());
    Ok(runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?
        .workspaces()
        .filter(|workspace| Some(workspace.id()) != hidden_workspace)
        .map(|workspace| PickerWorkspaceContext {
            id: workspace.id().get(),
            name: workspace.name().into(),
            root: workspace
                .root()
                .map(|root| root.display().to_string().into())
                .into(),
            is_default: Some(workspace.id()) == default_workspace,
        })
        .collect())
}

fn buffer_picker_preview(buffer: &ShellBuffer) -> String {
    const MAX_LINES: usize = 24;

    let mut lines = Vec::new();
    if let Some(path) = buffer.path() {
        lines.push(path.display().to_string());
    } else {
        lines.push(format!(
            "{} | row {}, col {}",
            buffer.kind_label(),
            buffer.cursor_row() + 1,
            buffer.cursor_col() + 1
        ));
    }
    lines.extend(
        (0..buffer.text.line_count().min(MAX_LINES))
            .filter_map(|line_index| buffer.text.line(line_index))
            .map(|line| line.trim_end().to_owned()),
    );
    lines.join("\n")
}

fn static_picker_entry(
    runtime: &EditorRuntime,
    item: &editor_plugin_api::PickerItemSpec,
) -> Result<PickerEntry, String> {
    let picker_item = if item.is_divider() {
        PickerItem::divider(item.id())
    } else {
        let mut picker_item = PickerItem::new(
            item.id(),
            item.label(),
            item.detail(),
            item.preview().map(str::to_owned),
        );
        if let Some(search_text) = item.search_text() {
            picker_item = picker_item.with_search_text(search_text);
        }
        if let Some(fringe) = item.fringe() {
            picker_item = picker_item.with_fringe(fringe);
        }
        picker_item
    };

    Ok(PickerEntry {
        item: picker_item,
        action: picker_action_from_spec(runtime, item.action())?,
        quickfix: None,
    })
}

fn picker_action_from_spec(
    runtime: &EditorRuntime,
    action: &PickerActionSpec,
) -> Result<PickerAction, String> {
    match action {
        PickerActionSpec::NoOp => Ok(PickerAction::NoOp),
        PickerActionSpec::ExecuteCommand { command } => {
            Ok(PickerAction::ExecuteCommand(command.to_string()))
        }
        PickerActionSpec::ExecuteCommands { commands } => Ok(PickerAction::ExecuteCommands(
            commands.iter().map(ToString::to_string).collect(),
        )),
        PickerActionSpec::EmitHook { hook, detail } => Ok(PickerAction::EmitHook {
            hook: hook.to_string(),
            detail: detail.as_ref().map(ToString::to_string).into(),
        }),
        PickerActionSpec::FocusBuffer { buffer_id } => Ok(PickerAction::FocusBuffer(
            resolve_buffer_id(runtime, *buffer_id)?,
        )),
        PickerActionSpec::CloseBuffer { buffer_id } => Ok(PickerAction::CloseBuffer(
            resolve_buffer_id(runtime, *buffer_id)?,
        )),
        PickerActionSpec::OpenAcpClient { client_id } => {
            Ok(PickerAction::OpenAcpClient(client_id.to_string()))
        }
        PickerActionSpec::OpenFile { path } => {
            Ok(PickerAction::OpenFile(PathBuf::from(path.as_str())))
        }
        PickerActionSpec::CreateWorkspaceFile { root } => Ok(PickerAction::CreateWorkspaceFile {
            root: PathBuf::from(root.as_str()),
        }),
        PickerActionSpec::InstallTreeSitterLanguage { language_id } => Ok(
            PickerAction::InstallTreeSitterLanguage(language_id.to_string()),
        ),
        PickerActionSpec::CreateWorkspace { name, root } => Ok(PickerAction::CreateWorkspace {
            name: name.to_string(),
            root: PathBuf::from(root.as_str()),
        }),
        PickerActionSpec::SwitchWorkspace { workspace_id } => Ok(PickerAction::SwitchWorkspace(
            resolve_workspace_id(runtime, *workspace_id)?,
        )),
        PickerActionSpec::DeleteWorkspace { workspace_id } => Ok(PickerAction::DeleteWorkspace(
            resolve_workspace_id(runtime, *workspace_id)?,
        )),
        PickerActionSpec::UndoTreeNode { buffer_id, node_id } => Ok(PickerAction::UndoTreeNode {
            buffer_id: resolve_buffer_id(runtime, *buffer_id)?,
            node_id: *node_id,
        }),
        PickerActionSpec::CopyToClipboard { text } => {
            Ok(PickerAction::CopyToClipboard(text.to_string()))
        }
        PickerActionSpec::ActivateTheme { theme_id } => {
            Ok(PickerAction::ActivateTheme(theme_id.to_string()))
        }
    }
}

fn resolve_buffer_id(runtime: &EditorRuntime, id: u64) -> Result<BufferId, String> {
    let ui = shell_ui(runtime)?;
    ui.active_workspace_buffer_ids()
        .into_iter()
        .flatten()
        .copied()
        .find(|buffer_id| buffer_id.get() == id)
        .ok_or_else(|| format!("unknown picker buffer id `{id}`"))
}

fn resolve_workspace_id(runtime: &EditorRuntime, id: u64) -> Result<WorkspaceId, String> {
    runtime
        .model()
        .active_window()
        .map_err(|error| error.to_string())?
        .workspaces()
        .map(|workspace| workspace.id())
        .find(|workspace_id| workspace_id.get() == id)
        .ok_or_else(|| format!("unknown picker workspace id `{id}`"))
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
    draw: PickerOverlayDraw<'_>,
) -> Result<(), ShellError> {
    let PickerOverlayDraw {
        picker,
        size: WindowSize { width, height },
        line_height,
        theme_registry,
        picker_layout,
        truncate_strategy,
    } = draw;
    let popup_rect = picker_card_rect(width, height, picker_layout);
    let window_effects = current_window_effect_settings(theme_registry);
    let cell_width = fonts
        .primary()
        .size_of_char('M')
        .map_err(|error| ShellError::Sdl(error.to_string()))?
        .0
        .max(1) as i32;
    let corner_radius = overlay_radius(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let base_foreground = theme_color(
        theme_registry,
        "ui.foreground",
        Color::RGBA(215, 221, 232, 255),
    );
    let is_dark = is_dark_color(base_background);
    let popup_background = theme_color(theme_registry, TOKEN_PICKER_BACKGROUND, base_background);
    let foreground = theme_color(theme_registry, TOKEN_PICKER_FOREGROUND, base_foreground);
    let highlight_background = theme_color(
        theme_registry,
        TOKEN_PICKER_SELECTION,
        adjust_color(popup_background, if is_dark { 16 } else { -16 }),
    );
    let accent = theme_color(
        theme_registry,
        "ui.picker.highlight",
        theme_color(
            theme_registry,
            TOKEN_STATUSLINE_ACTIVE,
            Color::RGB(110, 170, 255),
        ),
    );
    let border = theme_color(
        theme_registry,
        TOKEN_PICKER_BORDER,
        adjust_color(popup_background, if is_dark { 24 } else { -24 }),
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
    let card_rect = PixelRectToRect::rect(
        popup_rect.x,
        popup_rect.y,
        popup_rect.width,
        popup_rect.height,
    );
    paint_overlay_card(
        target,
        card_rect,
        OverlayCardStyle {
            radius: corner_radius,
            border,
            background: popup_background,
            window_effects,
            accent: Some(accent),
            shadow: false,
        },
    )?;

    draw_text(
        target,
        popup_rect.x + 16,
        popup_rect.y + 16,
        picker.session().title(),
        foreground,
    )?;

    let query = picker.session().query();
    let query_y = popup_rect.y + line_height + 20;
    let query_rect = PixelRectToRect::rect(
        popup_rect.x + 12,
        query_y - 2,
        popup_rect.width.saturating_sub(24),
        (line_height + 8).max(20) as u32,
    );
    let input_background = theme_color(
        theme_registry,
        "ui.input.background",
        adjust_color(popup_background, if is_dark { 10 } else { -10 }),
    );
    fill_overlay_surface_rounded_rect(
        target,
        query_rect,
        corner_radius.min(8),
        input_background,
        window_effects,
    )?;
    let query_text = if query.is_empty() { "filter" } else { query };
    draw_text(
        target,
        popup_rect.x + 20,
        query_y,
        query_text,
        if query.is_empty() { muted } else { foreground },
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

    let preview_text = picker
        .show_preview()
        .then(|| {
            picker
                .session()
                .selected()
                .and_then(|selected| selected.item().preview())
        })
        .flatten();
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
    let content_left = popup_rect.x + 18;
    let content_width = popup_rect.width.saturating_sub(36);
    let preview_layout = picker_preview_layout(
        preview_text,
        content_left,
        content_width,
        list_top,
        list_height,
        line_height,
    );
    let list_content_width = preview_layout
        .as_ref()
        .map(|layout| layout.list_width)
        .unwrap_or(content_width);

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
        if matched.item().is_divider() {
            fill_overlay_surface_rect(
                target,
                PixelRectToRect::rect(
                    popup_rect.x + 12,
                    row_y + row_height / 2,
                    list_content_width.saturating_add(12),
                    1,
                ),
                subtle,
                window_effects,
            )?;
            continue;
        }
        let selected = selected_id.as_deref() == Some(matched.item().id());
        let label_x = content_left + fringe_width as i32;
        let text_width = list_content_width.saturating_sub(fringe_width);
        if selected {
            fill_rounded_rect_with_left_accent(
                target,
                PixelRectToRect::rect(
                    popup_rect.x + 12,
                    row_y - 2,
                    list_content_width.saturating_add(12),
                    row_height as u32,
                ),
                corner_radius.min(8),
                highlight_background,
                accent,
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
        let label = truncate_picker_label(
            matched.item().label(),
            text_width,
            cell_width,
            truncate_strategy,
        );
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
    }

    if let Some(layout) = preview_layout {
        fill_overlay_surface_rect(
            target,
            PixelRectToRect::rect(
                layout.separator_x,
                layout.y.saturating_sub(4),
                1,
                layout.height.saturating_add(8),
            ),
            subtle,
            window_effects,
        )?;
        for (index, line) in layout.lines.iter().enumerate() {
            let char_map = LineCharMap::new(line);
            let max_cols = (layout.width / cell_width.max(1) as u32) as usize;
            let end_col = char_map
                .char_col_for_display_col(max_cols)
                .min(char_map.len());
            let syntax_spans = index
                .checked_sub(1)
                .and_then(|line_index| picker.preview_syntax_lines().get(&line_index))
                .map(Vec::as_slice);
            draw_buffer_text(
                target,
                BufferTextRun {
                    x: layout.x,
                    y: layout.y + index as i32 * line_height,
                    line,
                    segment: LineWrapSegment {
                        start_col: 0,
                        end_col,
                    },
                    char_map: &char_map,
                    line_syntax_spans: syntax_spans,
                    default_color: subtle,
                    cell_width,
                },
                theme_registry,
            )?;
        }
    }

    Ok(())
}

struct PickerPreviewLayout {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    separator_x: i32,
    list_width: u32,
    lines: Vec<String>,
}

fn picker_preview_layout(
    preview: Option<&str>,
    content_left: i32,
    content_width: u32,
    list_top: i32,
    list_height: i32,
    line_height: i32,
) -> Option<PickerPreviewLayout> {
    let preview = preview?;
    if content_width < 560 || list_height <= line_height {
        return None;
    }
    let preview_width = (content_width * 2 / 5)
        .max(240)
        .min(content_width.saturating_sub(280));
    if preview_width == 0 {
        return None;
    }
    let gap = 18;
    let list_width = content_width
        .saturating_sub(preview_width)
        .saturating_sub(gap);
    let preview_x = content_left + list_width as i32 + gap as i32;
    let preview_rows = (list_height / line_height.max(1)).max(1) as usize;
    let lines = picker_preview_lines(preview, preview_rows);
    if lines.is_empty() {
        return None;
    }
    Some(PickerPreviewLayout {
        x: preview_x,
        y: list_top,
        width: preview_width,
        height: list_height.max(0) as u32,
        separator_x: preview_x - (gap as i32 / 2),
        list_width,
        lines,
    })
}

fn picker_preview_lines(preview: &str, max_lines: usize) -> Vec<String> {
    preview
        .lines()
        .take(max_lines)
        .map(|line| line.trim_end().to_owned())
        .collect()
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
    use super::*;
    use editor_plugin_api::{PickerActionSpec, PickerItemSpec, PickerProviderSpec};

    #[test]
    fn static_user_picker_provider_builds_executable_entries() -> Result<(), String> {
        let provider = PickerProviderSpec::static_items(
            "tools",
            "Tools",
            vec![
                PickerItemSpec::new(
                    "open-commands",
                    "Command palette",
                    "Open command picker",
                    PickerActionSpec::execute_command("picker.open-commands"),
                )
                .with_search_text("commands palette")
                .with_preview("Runs picker.open-commands"),
            ],
        );

        let mut runtime = EditorRuntime::new();
        runtime
            .services_mut()
            .insert(UserLibraryService(Arc::new(NullUserLibrary)));
        let context = picker_provider_context(&runtime, &provider)?;
        let picker = user_picker_overlay(&runtime, &provider, context)?;
        let selected = picker
            .session()
            .selected()
            .ok_or_else(|| "static picker has no selected item".to_owned())?;
        assert_eq!(picker.session().title(), "Tools");
        assert_eq!(selected.item().label(), "Command palette");
        assert_eq!(selected.item().preview(), Some("Runs picker.open-commands"));
        assert!(matches!(
            picker.selected_action(),
            Some(PickerAction::ExecuteCommand(command)) if command == "picker.open-commands"
        ));
        Ok(())
    }

    #[test]
    fn static_picker_keeps_all_matches_for_large_lists() -> Result<(), String> {
        let items = (0..80)
            .map(|index| {
                PickerItemSpec::new(
                    format!("file-{index:03}"),
                    format!("file-{index:03}.rs"),
                    "src",
                    PickerActionSpec::no_op(),
                )
            })
            .collect::<Vec<_>>();
        let provider = PickerProviderSpec::static_items("files", "Files", items);

        let mut runtime = EditorRuntime::new();
        runtime
            .services_mut()
            .insert(UserLibraryService(Arc::new(NullUserLibrary)));
        let context = picker_provider_context(&runtime, &provider)?;
        let picker = user_picker_overlay(&runtime, &provider, context)?;

        assert_eq!(picker.session().item_count(), 80);
        assert_eq!(picker.session().match_count(), 80);
        Ok(())
    }

    struct BoundedSourceLibrary {
        source: PickerSource,
        items: Vec<PickerItemSpec>,
    }

    impl UserLibrary for BoundedSourceLibrary {
        fn picker_provider_items(
            &self,
            context: &PickerProviderContext,
        ) -> Option<Vec<PickerItemSpec>> {
            (context.source == self.source).then(|| self.items.clone())
        }
    }

    fn preview_items(count: usize, prefix: &str) -> Vec<PickerItemSpec> {
        (0..count)
            .map(|index| {
                PickerItemSpec::new(
                    format!("{prefix}-{index:03}"),
                    format!("{prefix}-{index:03}.rs"),
                    "src",
                    PickerActionSpec::no_op(),
                )
                .with_preview(format!("preview-{prefix}-{index:03}"))
            })
            .collect()
    }

    fn overlay_for_source(
        source: PickerSource,
        id: &str,
        title: &str,
        items: Vec<PickerItemSpec>,
    ) -> Result<PickerOverlay, String> {
        let provider = PickerProviderSpec::new(id, title, source);
        let mut runtime = EditorRuntime::new();
        runtime
            .services_mut()
            .insert(UserLibraryService(Arc::new(BoundedSourceLibrary {
                source,
                items,
            })));
        let context = PickerProviderContext::new(id, title, source);
        user_picker_overlay(&runtime, &provider, context)
    }

    #[test]
    fn workspace_files_overlay_caps_matches_without_dropping_source_items() -> Result<(), String> {
        let picker = overlay_for_source(
            PickerSource::WorkspaceFiles,
            "workspace.files",
            "Workspace Files",
            preview_items(300, "file"),
        )?;

        assert_eq!(picker.session().item_count(), 300);
        assert_eq!(
            picker.session().match_count(),
            WORKSPACE_FILES_PICKER_RESULT_LIMIT
        );
        assert!(
            picker
                .session()
                .matches()
                .iter()
                .any(|matched| matched.item().preview() == Some("preview-file-000"))
        );
        Ok(())
    }

    #[test]
    fn command_picker_overlay_uses_finite_result_limit() -> Result<(), String> {
        let picker = overlay_for_source(
            PickerSource::Commands,
            "commands",
            "Command Palette",
            preview_items(600, "cmd"),
        )?;

        assert_eq!(picker.session().item_count(), 600);
        assert_eq!(picker.session().match_count(), COMMAND_PICKER_RESULT_LIMIT);
        Ok(())
    }

    #[test]
    fn provider_extra_keybinds_copy_onto_open_picker_instance() -> Result<(), String> {
        let provider = PickerProviderSpec::static_items(
            "tools",
            "Tools",
            vec![PickerItemSpec::new(
                "open-commands",
                "Command palette",
                "Open command picker",
                PickerActionSpec::execute_command("picker.open-commands"),
            )],
        )
        .with_extra_keybind("Ctrl+d", "workspace.worktree-remove");

        let mut runtime = EditorRuntime::new();
        runtime
            .services_mut()
            .insert(UserLibraryService(Arc::new(NullUserLibrary)));
        let context = picker_provider_context(&runtime, &provider)?;
        let picker = user_picker_overlay(&runtime, &provider, context)?;

        assert_eq!(picker.extra_keybinds().len(), 1);
        assert_eq!(picker.extra_keybinds()[0].chord(), "Ctrl+d");
        assert_eq!(
            picker.extra_keybinds()[0].command_name(),
            "workspace.worktree-remove"
        );
        Ok(())
    }

    #[test]
    fn picker_preview_is_opt_in() {
        let picker = PickerOverlay::from_entries("Files", Vec::new());
        assert!(!picker.show_preview());
        assert!(picker.with_preview().show_preview());
    }

    #[test]
    fn picker_preview_layout_splits_preview_to_the_right() {
        let layout = picker_preview_layout(Some("path\nline 1\nline 2"), 20, 900, 120, 300, 20)
            .expect("preview layout should exist on wide pickers");

        assert_eq!(layout.y, 120);
        assert!(layout.x > 20);
        assert!(layout.list_width < 900);
        assert_eq!(layout.lines, vec!["path", "line 1", "line 2"]);
    }
}
