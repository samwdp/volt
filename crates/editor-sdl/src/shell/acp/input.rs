use std::{
    env,
    sync::{Arc, Mutex},
};

use agent_client_protocol::{
    AvailableCommand, ModelId, ModelInfo, PermissionOptionId, SessionConfigKind,
    SessionConfigOption, SessionConfigOptionCategory, SessionConfigSelectOption,
    SessionConfigSelectOptions, SessionMode, SessionModeId, SessionModeState, SessionModelState,
};
use editor_plugin_api::AcpClient as AcpClientConfig;

use super::super::*;

use super::client::*;
use super::cursor_ext::*;
use super::manager::*;
use super::runtime::*;
use super::session::*;

pub(crate) fn active_acp_client(runtime: &EditorRuntime) -> Result<AcpClientConfig, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let client_id = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager
            .client_id_for_buffer(buffer_id)
            .ok_or_else(|| "acp.new-session requires an active ACP buffer".to_owned())?
    };
    shell_user_library(runtime)
        .acp_client_by_id(&client_id)
        .ok_or_else(|| format!("unknown ACP client `{client_id}`"))
}

pub(crate) fn submit_acp_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    prompt: &str,
    text: &str,
) -> Result<(), String> {
    if try_run_universal_acp_slash(runtime, buffer_id, text)? {
        return Ok(());
    }
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let session_id = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager.session_for_buffer(buffer_id)
    };
    let workspace_root = active_workspace_root(runtime)?.or_else(|| git_root(runtime).ok());
    let images = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer.acp_pasted_images().to_vec()
    };
    let blocks = compose_acp_prompt_blocks(text, workspace_root.as_deref(), &images);
    let Some(session_id) = session_id else {
        let follow = {
            let buffer = shell_buffer_mut(runtime, buffer_id)?;
            let follow = buffer.acp_push_system_message("ACP session is not connected.");
            buffer.clear_input();
            follow
        };
        refresh_acp_output_markdown(runtime, buffer_id, follow)?;
        refresh_acp_input_hint(runtime, buffer_id)?;
        return Ok(());
    };
    let follow = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let follow = buffer.acp_push_user_prompt(format!("{prompt}{text}"));
        buffer.clear_input();
        follow
    };
    refresh_acp_output_markdown(runtime, buffer_id, follow)?;
    refresh_acp_input_hint(runtime, buffer_id)?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.prompt(session_id, blocks)
}

pub(crate) fn acp_complete_slash(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !matches!(
        &buffer.kind,
        BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND
    ) {
        return Ok(());
    }
    if let Some(mention) = buffer
        .input_field()
        .and_then(|input| acp_file_mention_at_cursor(input.text(), input.cursor_char()))
    {
        return open_file_mention_picker(
            runtime,
            buffer_id,
            CompletionTrigger::Auto(mention.query),
        );
    }
    let query = buffer.input_field().and_then(|input| {
        let text = input.text();
        text.strip_prefix('/')
            .map(|trimmed| trimmed.split_whitespace().next().unwrap_or("").to_owned())
    });
    let trigger = query
        .filter(|text| !text.is_empty())
        .map(CompletionTrigger::Auto)
        .unwrap_or(CompletionTrigger::Manual);
    open_slash_command_picker(runtime, buffer_id, trigger)
}

pub(crate) fn maybe_open_acp_input_completion(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let Some((text, cursor)) = ({
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !matches!(
            &buffer.kind,
            BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND
        ) {
            return Ok(());
        }
        buffer
            .input_field()
            .map(|input| (input.text().to_owned(), input.cursor_char()))
    }) else {
        return Ok(());
    };
    let picker_kind = shell_ui(runtime)?.picker_kind();
    let inline_picker_active = matches!(
        picker_kind,
        Some(kind) if kind.acp_inline_buffer_id() == Some(buffer_id)
    );
    if let Some(query) = acp_slash_completion_query(&text) {
        if shell_ui(runtime)?.picker_visible() {
            if inline_picker_active {
                shell_ui_mut(runtime)?.close_picker();
            } else {
                return Ok(());
            }
        }
        return open_slash_command_picker(
            runtime,
            buffer_id,
            CompletionTrigger::Auto(query.to_owned()),
        );
    }
    if let Some(mention) = acp_file_mention_at_cursor(&text, cursor) {
        if shell_ui(runtime)?.picker_visible() {
            if inline_picker_active {
                shell_ui_mut(runtime)?.close_picker();
            } else {
                return Ok(());
            }
        }
        return open_file_mention_picker(
            runtime,
            buffer_id,
            CompletionTrigger::Auto(mention.query),
        );
    }
    if inline_picker_active {
        shell_ui_mut(runtime)?.close_picker();
    }
    Ok(())
}

pub(crate) fn acp_insert_slash_command(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    command: &str,
) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let Some(input) = buffer.input_field_mut() else {
        return Err("ACP buffer has no input field".to_owned());
    };
    let existing = input.text().to_owned();
    let trailing = existing
        .strip_prefix('/')
        .and_then(|text| text.split_once(' ').map(|(_, suffix)| suffix))
        .map(str::trim_start)
        .filter(|text| !text.is_empty());
    let next = match trailing {
        Some(rest) => format!("/{command} {rest}"),
        None => format!("/{command} "),
    };
    input.set_text(&next);
    refresh_acp_input_hint(runtime, buffer_id)?;
    Ok(())
}

pub(crate) fn acp_insert_file_mention(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    relative_path: &str,
) -> Result<(), String> {
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let Some(input) = buffer.input_field_mut() else {
        return Err("ACP buffer has no input field".to_owned());
    };
    let text = input.text().to_owned();
    let cursor = input.cursor_char();
    let mention = acp_file_mention_at_cursor(&text, cursor).unwrap_or(FileMention {
        at_char: cursor,
        end_char: cursor,
        query: String::new(),
    });
    let replacement = format!("@{relative_path} ");
    input.replace_char_range(mention.at_char, mention.end_char, &replacement);
    refresh_acp_input_hint(runtime, buffer_id)?;
    Ok(())
}

pub(crate) fn paste_image_into_active_input(
    runtime: &mut EditorRuntime,
    image: ClipboardImage,
) -> Result<bool, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        buffer_is_acp(&buffer.kind)
    };
    if !is_acp {
        return Ok(false);
    }
    close_acp_inline_picker_for(runtime, buffer_id, true)?;
    let token = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let Some(token) = buffer.acp_attach_pasted_image(image) else {
            return Ok(false);
        };
        if let Some(input) = buffer.input_field_mut() {
            if !input.text().is_empty()
                && !input
                    .text()
                    .chars()
                    .last()
                    .is_some_and(|character| character.is_whitespace())
            {
                input.insert_text(" ");
            }
            input.insert_text(&token);
            input.insert_text(" ");
            true
        } else {
            false
        }
    };
    if token {
        shell_ui_mut(runtime)?.close_picker();
        maybe_open_acp_input_completion(runtime, buffer_id)?;
        refresh_acp_input_hint(runtime, buffer_id)?;
    }
    Ok(token)
}

pub(crate) fn acp_image_mention_token(id: u64, name: &str) -> String {
    format!("![{name}](acp-image:{id})")
}

pub(crate) fn format_acp_mode_label(mode_id: &SessionModeId) -> String {
    let raw = mode_id.to_string();
    if let Some((_, suffix)) = raw.rsplit_once('#')
        && !suffix.is_empty()
    {
        return suffix.to_owned();
    }
    raw
}

pub(crate) fn format_acp_model_label(model_id: &ModelId) -> String {
    let raw = model_id.to_string();
    if let Some((_, suffix)) = raw.rsplit_once('/')
        && !suffix.is_empty()
    {
        return suffix.to_owned();
    }
    raw
}

/// Host-owned slash commands available for every ACP client.
pub(crate) fn universal_acp_slash_commands() -> Vec<AvailableCommand> {
    vec![
        AvailableCommand::new("clear", "Start a new ACP session"),
        AvailableCommand::new("capabilities", "Show ACP session capabilities"),
    ]
}

/// Merge host universal slash commands ahead of agent-provided ones.
/// Host names win when the agent advertises the same command.
pub(crate) fn merge_acp_slash_commands(
    agent_commands: impl IntoIterator<Item = AvailableCommand>,
) -> Vec<AvailableCommand> {
    let mut commands = universal_acp_slash_commands();
    let universal_names = commands
        .iter()
        .map(|command| command.name.clone())
        .collect::<std::collections::HashSet<_>>();
    for command in agent_commands {
        if !universal_names.contains(&command.name) {
            commands.push(command);
        }
    }
    commands
}

pub(crate) fn acp_leading_slash_command_name(text: &str) -> Option<&str> {
    let trimmed = text.strip_prefix('/')?.trim_start();
    let name = trimmed.split_whitespace().next()?;
    (!name.is_empty()).then_some(name)
}

pub(crate) fn try_run_universal_acp_slash(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    text: &str,
) -> Result<bool, String> {
    let trimmed = text.trim();
    if trimmed == "acp.capabilities" {
        {
            let buffer = shell_buffer_mut(runtime, buffer_id)?;
            buffer.clear_input();
        }
        show_acp_capabilities_on_buffer(runtime, buffer_id)?;
        return Ok(true);
    }
    let Some(name) = acp_leading_slash_command_name(text) else {
        return Ok(false);
    };
    match name {
        "clear" => {
            {
                let buffer = shell_buffer_mut(runtime, buffer_id)?;
                buffer.clear_input();
            }
            refresh_acp_input_hint(runtime, buffer_id)?;
            runtime
                .execute_command("acp.new-session")
                .map_err(|error| error.to_string())?;
            Ok(true)
        }
        "capabilities" => {
            {
                let buffer = shell_buffer_mut(runtime, buffer_id)?;
                buffer.clear_input();
            }
            show_acp_capabilities_on_buffer(runtime, buffer_id)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

pub(crate) fn command_input_hint(command: &AvailableCommand) -> Option<&str> {
    match command.input.as_ref() {
        Some(agent_client_protocol::AvailableCommandInput::Unstructured(input)) => {
            Some(input.hint.as_str())
        }
        _ => None,
    }
}

pub(crate) fn active_command_input_hint(
    commands: &[AvailableCommand],
    text: &str,
) -> Option<String> {
    let trimmed = text.strip_prefix('/')?.trim_start();
    let command_name = trimmed
        .split_whitespace()
        .next()
        .filter(|command| !command.is_empty())?;
    commands
        .iter()
        .find(|command| command.name == command_name)
        .and_then(command_input_hint)
        .map(str::to_owned)
}

pub(crate) fn build_acp_input_hint(
    mode_id: Option<&SessionModeId>,
    model_id: Option<&ModelId>,
    command_hint: Option<&str>,
) -> Option<String> {
    let mut segments = Vec::new();
    if let Some(mode_id) = mode_id {
        segments.push(format_acp_mode_label(mode_id));
    }
    if let Some(model_id) = model_id {
        segments.push(format_acp_model_label(model_id));
    }
    if let Some(command_hint) = command_hint.filter(|hint| !hint.trim().is_empty()) {
        segments.push(command_hint.to_owned());
    }
    if mode_id.is_some() {
        segments.push("shift+tab switch mode".to_owned());
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" · "))
    }
}

pub(crate) fn update_acp_input_hint(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mode_id: Option<&SessionModeId>,
    model_id: Option<&ModelId>,
    available_commands: &[AvailableCommand],
) {
    let input_text = shell_buffer(runtime, buffer_id)
        .ok()
        .and_then(|buffer| buffer.input_field().map(|input| input.text().to_owned()))
        .unwrap_or_default();
    let commands = merge_acp_slash_commands(available_commands.iter().cloned());
    let command_hint = active_command_input_hint(&commands, &input_text);
    let hint = build_acp_input_hint(mode_id, model_id, command_hint.as_deref());
    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id)
        && let Some(footer) = buffer.acp_footer_pane_mut()
    {
        footer.replace_lines(hint.into_iter().collect(), true);
    }
}

pub(crate) fn refresh_acp_input_hint(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        matches!(
            &buffer.kind,
            BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND
        )
    };
    if !is_acp {
        return Ok(());
    }
    let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() else {
        return Ok(());
    };
    let (mode_id, model_id, available_commands) = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        let session = manager
            .session_for_buffer(buffer_id)
            .and_then(|session_id| manager.sessions.get(&session_id));
        match session {
            Some(session) => (
                session
                    .mode_state
                    .as_ref()
                    .map(|state| state.current_mode_id.clone()),
                session
                    .model_state
                    .as_ref()
                    .map(|state| state.current_model_id.clone()),
                session.available_commands.clone(),
            ),
            None => (None, None, Vec::new()),
        }
    };
    update_acp_input_hint(
        runtime,
        buffer_id,
        mode_id.as_ref(),
        model_id.as_ref(),
        &available_commands,
    );
    Ok(())
}

pub(crate) fn config_option_matches(option: &SessionConfigOption, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    let id = option.id.to_string().to_ascii_lowercase();
    let name = option.name.to_ascii_lowercase();
    id.contains(&needle) || name.contains(&needle)
}

pub(crate) fn config_option_is_mode(option: &SessionConfigOption) -> bool {
    matches!(option.category, Some(SessionConfigOptionCategory::Mode))
        || (option.category.is_none() && config_option_matches(option, "mode"))
}

pub(crate) fn config_option_is_model(option: &SessionConfigOption) -> bool {
    matches!(option.category, Some(SessionConfigOptionCategory::Model))
        || (option.category.is_none() && config_option_matches(option, "model"))
}

pub(crate) fn flatten_config_select_options(
    options: &SessionConfigSelectOptions,
) -> Vec<SessionConfigSelectOption> {
    match options {
        SessionConfigSelectOptions::Ungrouped(options) => options.clone(),
        SessionConfigSelectOptions::Grouped(groups) => groups
            .iter()
            .flat_map(|group| group.options.clone())
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn session_mode_state_from_config(
    option: &SessionConfigOption,
) -> Option<SessionModeState> {
    let SessionConfigKind::Select(select) = &option.kind else {
        return None;
    };
    let available_modes = flatten_config_select_options(&select.options)
        .into_iter()
        .map(|option| {
            let mut mode =
                SessionMode::new(SessionModeId::new(option.value.to_string()), option.name);
            if let Some(description) = option.description {
                mode = mode.description(description);
            }
            mode
        })
        .collect();
    Some(SessionModeState::new(
        SessionModeId::new(select.current_value.to_string()),
        available_modes,
    ))
}

pub(crate) fn session_model_state_from_config(
    option: &SessionConfigOption,
) -> Option<SessionModelState> {
    let SessionConfigKind::Select(select) = &option.kind else {
        return None;
    };
    let available_models = flatten_config_select_options(&select.options)
        .into_iter()
        .map(|option| {
            let mut model = ModelInfo::new(ModelId::new(option.value.to_string()), option.name);
            if let Some(description) = option.description {
                model = model.description(description);
            }
            model
        })
        .collect();
    Some(SessionModelState::new(
        ModelId::new(select.current_value.to_string()),
        available_models,
    ))
}

pub(crate) fn acp_pick_session(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let session_id = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager.session_for_buffer(buffer_id)
    };
    let Some(session_id) = session_id else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session is not connected.".to_owned()]);
        return Ok(());
    };
    let workspace_root = active_workspace_root(runtime)?
        .or_else(|| env::current_dir().ok())
        .ok_or_else(|| "ACP requires a workspace root or current directory".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.list_sessions(session_id, buffer_id, workspace_root)
}

pub(crate) fn acp_load_session(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    session_id: &str,
    session_title: Option<&str>,
) -> Result<(), String> {
    let target_session_id = agent_client_protocol::SessionId::new(session_id);
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let session_data = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        (
            manager.buffer_for_session(&target_session_id),
            manager.client_id_for_buffer(buffer_id),
        )
    };
    if let Some(existing_buffer_id) = session_data.0 {
        focus_acp_buffer(runtime, existing_buffer_id)?;
        return Ok(());
    }
    let Some(client_id) = session_data.1 else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session is not connected.".to_owned()]);
        return Ok(());
    };
    open_acp_client_buffer(
        runtime,
        &client_id,
        false,
        Some(PendingAcpLoadSession {
            session_id: target_session_id,
            title: session_title.map(str::to_owned),
        }),
    )
    .map(|_| ())
}

pub(crate) fn acp_picker_entries(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    context: &AcpPickerContext,
) -> Vec<PickerEntry> {
    shell_user_library(runtime)
        .acp_picker_items(context)
        .into_iter()
        .map(|item| acp_picker_entry(buffer_id, item))
        .collect()
}

pub(crate) fn acp_picker_entry(buffer_id: BufferId, item: AcpPickerItemSpec) -> PickerEntry {
    let action = match item.action() {
        AcpActionSpec::SetMode { mode_id } => PickerAction::AcpSetMode {
            buffer_id,
            mode_id: mode_id.to_string(),
        },
        AcpActionSpec::SetModel { model_id } => PickerAction::AcpSetModel {
            buffer_id,
            model_id: model_id.to_string(),
        },
        AcpActionSpec::LoadSession { session_id } => PickerAction::AcpLoadSession {
            buffer_id,
            session_id: session_id.to_string(),
            session_title: item.label().to_string(),
        },
        AcpActionSpec::InsertSlashCommand { command } => PickerAction::AcpInsertSlashCommand {
            buffer_id,
            command: command.to_string(),
        },
    };
    PickerEntry {
        item: PickerItem::new(item.id(), item.label(), item.detail(), None::<String>),
        action,
        quickfix: None,
    }
}

pub(crate) fn acp_pick_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mode_state = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager.mode_state_for_buffer(buffer_id)
    };
    let Some(mode_state) = mode_state else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session modes are not available.".to_owned()]);
        return Ok(());
    };
    if mode_state.available_modes.is_empty() {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session modes are not available.".to_owned()]);
        return Ok(());
    }
    let current_mode = mode_state.current_mode_id.clone();
    let options = mode_state
        .available_modes
        .into_iter()
        .map(|mode| {
            AcpPickerOption::new(mode.id.to_string(), format_acp_mode_label(&mode.id))
                .with_current(mode.id == current_mode)
        })
        .collect();
    let context = AcpPickerContext::new(AcpPickerKind::Modes, "ACP Modes").with_options(options);
    let entries = acp_picker_entries(runtime, buffer_id, &context);
    let picker = PickerOverlay::from_entries("ACP Modes", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn acp_pick_model(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let model_state = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager.model_state_for_buffer(buffer_id)
    };
    let Some(model_state) = model_state else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP models are not available.".to_owned()]);
        return Ok(());
    };
    if model_state.available_models.is_empty() {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP models are not available.".to_owned()]);
        return Ok(());
    }
    let current_model = model_state.current_model_id.clone();
    let options = model_state
        .available_models
        .into_iter()
        .map(|model| {
            let detail = model
                .description
                .clone()
                .unwrap_or_else(|| model.model_id.to_string());
            AcpPickerOption::new(model.model_id.to_string(), model.name)
                .with_detail(detail)
                .with_current(model.model_id == current_model)
        })
        .collect();
    let context = AcpPickerContext::new(AcpPickerKind::Models, "ACP Models").with_options(options);
    let entries = acp_picker_entries(runtime, buffer_id, &context);
    let picker = PickerOverlay::from_entries("ACP Models", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn acp_set_model(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    model_id: &str,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let session_id = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager.session_for_buffer(buffer_id)
    };
    let Some(session_id) = session_id else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session is not connected.".to_owned()]);
        return Ok(());
    };
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.set_model(session_id, ModelId::new(model_id))
}

pub(crate) fn acp_set_mode(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mode_id: &str,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let (session_id, mode_state) = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        (
            manager.session_for_buffer(buffer_id),
            manager.mode_state_for_buffer(buffer_id),
        )
    };
    let Some(session_id) = session_id else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session is not connected.".to_owned()]);
        return Ok(());
    };
    let Some(mode_state) = mode_state else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session modes are not available.".to_owned()]);
        return Ok(());
    };
    if mode_state.available_modes.is_empty() {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session modes are not available.".to_owned()]);
        return Ok(());
    }
    let selected = mode_state
        .available_modes
        .into_iter()
        .find(|mode| mode.id.to_string() == mode_id)
        .map(|mode| mode.id);
    let Some(selected) = selected else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!("ACP mode `{mode_id}` is not available.")]);
        return Ok(());
    };
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.set_mode(session_id, selected)
}

pub(crate) fn acp_cycle_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let (session_id, mode_state) = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        (
            manager.session_for_buffer(buffer_id),
            manager.mode_state_for_buffer(buffer_id),
        )
    };
    let Some(session_id) = session_id else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session is not connected.".to_owned()]);
        return Ok(());
    };
    let Some(mode_state) = mode_state else {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session modes are not available.".to_owned()]);
        return Ok(());
    };
    if mode_state.available_modes.is_empty() {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&["ACP session modes are not available.".to_owned()]);
        return Ok(());
    }
    let current_id = mode_state.current_mode_id;
    let next_index = mode_state
        .available_modes
        .iter()
        .position(|mode| mode.id == current_id)
        .map(|index| (index + 1) % mode_state.available_modes.len())
        .unwrap_or(0);
    let next_mode = mode_state
        .available_modes
        .get(next_index)
        .map(|mode| mode.id.clone())
        .ok_or_else(|| "ACP session mode list is empty".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.set_mode(session_id, next_mode)
}

pub(crate) fn acp_disconnect(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    let Some(session_id) = manager.session_for_buffer(buffer_id) else {
        return Ok(());
    };
    manager.disconnect(session_id);
    Ok(())
}

pub(crate) fn acp_permission_approve(runtime: &mut EditorRuntime) -> Result<(), String> {
    resolve_permission(runtime, PermissionDecision::Approve)
}

pub(crate) fn acp_permission_deny(runtime: &mut EditorRuntime) -> Result<(), String> {
    resolve_permission(runtime, PermissionDecision::Deny)
}

pub(crate) fn acp_resolve_permission_option(
    runtime: &mut EditorRuntime,
    request_id: u64,
    option_id: &str,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.resolve_permission_option(request_id, PermissionOptionId::new(option_id.to_owned()));
    Ok(())
}

pub(crate) fn acp_open_permission_request(
    runtime: &mut EditorRuntime,
    request_id: u64,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.open_permission_request(runtime, request_id)
}

pub(crate) fn acp_permission_picker_closed(
    runtime: &mut EditorRuntime,
    request_id: u64,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.permission_picker_closed(request_id);
    Ok(())
}

pub(crate) fn acp_permission_picker_submitted(
    _runtime: &mut EditorRuntime,
    _request_id: u64,
) -> Result<(), String> {
    Ok(())
}

pub(crate) fn acp_subagent_next(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let _ = buffer.acp_select_next_subagent();
    Ok(())
}

pub(crate) fn acp_subagent_previous(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let _ = buffer.acp_select_previous_subagent();
    Ok(())
}

pub(crate) fn acp_subagent_toggle_fold(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    let _ = buffer.acp_toggle_current_subagent_fold();
    Ok(())
}

pub(crate) fn acp_cancel(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let current_subagent_id = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.acp_cancel_target_subagent_id()
    };
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    let Some((connection_session_id, target_session_id)) =
        manager.cancel_target_for_buffer(buffer_id, current_subagent_id.as_deref())
    else {
        return Ok(());
    };
    manager.cancel(connection_session_id, target_session_id)
}

pub(crate) fn acp_show_capabilities(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = resolve_acp_capabilities_buffer(runtime)?;
    show_acp_capabilities_on_buffer(runtime, buffer_id)
}

fn resolve_acp_capabilities_buffer(runtime: &EditorRuntime) -> Result<BufferId, String> {
    if let Ok(buffer_id) = active_shell_buffer_id(runtime)
        && shell_buffer_is_acp(runtime, buffer_id)
    {
        return Ok(buffer_id);
    }
    let ids = workspace_acp_buffer_ids(runtime)?;
    if ids.is_empty() {
        return Err("no ACP buffer in the active workspace".to_owned());
    }
    if ids.len() == 1 {
        return Ok(ids[0]);
    }
    if let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        if let Some(buffer_id) = ids
            .iter()
            .copied()
            .find(|buffer_id| manager.session_for_buffer(*buffer_id).is_some())
        {
            return Ok(buffer_id);
        }
    }
    Ok(ids[0])
}

fn shell_buffer_is_acp(runtime: &EditorRuntime, buffer_id: BufferId) -> bool {
    shell_buffer(runtime, buffer_id).ok().is_some_and(|buffer| {
        matches!(
            &buffer.kind,
            BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND
        )
    })
}

fn workspace_acp_buffer_ids(runtime: &EditorRuntime) -> Result<Vec<BufferId>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    Ok(workspace
        .buffers()
        .filter_map(|buffer| match buffer.kind() {
            BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND => Some(buffer.id()),
            _ => None,
        })
        .collect())
}

fn capabilities_footer_line(report: &str) -> String {
    let bits = report
        .lines()
        .filter(|line| {
            line.starts_with("client:")
                || line.starts_with("protocol:")
                || line.starts_with("agent:")
                || line.starts_with("ACP session")
        })
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if bits.is_empty() {
        "ACP capabilities".to_owned()
    } else {
        bits.join(" · ")
    }
}

pub(crate) fn show_acp_capabilities_on_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    focus_acp_buffer(runtime, buffer_id)?;
    let report = {
        let manager = runtime
            .services()
            .get::<Arc<Mutex<AcpManager>>>()
            .ok_or_else(|| "acp manager service missing".to_owned())?
            .clone();
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager
            .capabilities_report_for_buffer(buffer_id)
            .unwrap_or_else(|| "ACP capabilities\nACP session is not connected.".to_owned())
    };
    let footer = capabilities_footer_line(&report);
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.acp_focus_output_pane();
        buffer.acp_push_system_message(report);
        if let Some(footer_pane) = buffer.acp_footer_pane_mut() {
            footer_pane.replace_lines(vec![footer], true);
        }
    }
    refresh_acp_output_markdown(runtime, buffer_id, true)?;
    Ok(())
}

pub(crate) fn acp_switch_pane(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (is_input, read_only) = {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let _ = buffer.acp_switch_pane();
        (
            matches!(buffer.acp_active_pane(), Some(AcpPane::Input)),
            buffer.is_read_only(),
        )
    };
    let ui = shell_ui_mut(runtime)?;
    ui.set_active_vim_target(if is_input {
        VimTarget::Input
    } else {
        VimTarget::Buffer
    });
    if read_only {
        ui.enter_normal_mode();
    }
    Ok(())
}

pub(crate) fn resolve_permission(
    runtime: &mut EditorRuntime,
    decision: PermissionDecision,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    let request_id =
        if let Some(PickerKind::AcpPermission { request_id }) = shell_ui(runtime)?.picker_kind() {
            Some(request_id)
        } else {
            let buffer_id = active_shell_buffer_id(runtime)?;
            manager
                .session_for_buffer(buffer_id)
                .and_then(|session_id| manager.permission_request_for_session(&session_id))
        };
    let Some(request_id) = request_id else {
        return Ok(());
    };
    manager.resolve_permission(request_id, decision);
    Ok(())
}

pub(crate) fn open_cursor_question_picker(
    runtime: &mut EditorRuntime,
    request_id: u64,
    title: Option<&str>,
    question: &CursorQuestion,
) -> Result<(), String> {
    let detail = if question.allow_multiple {
        format!("{} · send one choice", question.prompt)
    } else {
        question.prompt.clone()
    };
    let mut entries = question
        .options
        .iter()
        .map(|option| PickerEntry {
            item: PickerItem::new(
                option.id.clone(),
                option.label.clone(),
                detail.clone(),
                None::<String>,
            ),
            action: PickerAction::AcpCursorAnswer {
                request_id,
                question_id: question.id.clone(),
                option_id: option.id.clone(),
            },
            quickfix: None,
        })
        .collect::<Vec<_>>();
    entries.push(PickerEntry {
        item: PickerItem::new("skip", "Skip", "Continue without an answer", None::<String>),
        action: PickerAction::AcpCursorSkip { request_id },
        quickfix: None,
    });
    let heading = title
        .filter(|title| !title.is_empty())
        .unwrap_or("Cursor question");
    let picker = PickerOverlay::from_entries(format!("{heading} · {}", question.prompt), entries)
        .with_kind(PickerKind::AcpCursor { request_id });
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn open_cursor_plan_picker(
    runtime: &mut EditorRuntime,
    request_id: u64,
    name: Option<&str>,
) -> Result<(), String> {
    let title = name
        .filter(|name| !name.is_empty())
        .unwrap_or("Cursor plan");
    let entries = vec![
        PickerEntry {
            item: PickerItem::new("accept", "Accept plan", title, None::<String>),
            action: PickerAction::AcpCursorPlan {
                request_id,
                accepted: true,
            },
            quickfix: None,
        },
        PickerEntry {
            item: PickerItem::new("reject", "Reject plan", title, None::<String>),
            action: PickerAction::AcpCursorPlan {
                request_id,
                accepted: false,
            },
            quickfix: None,
        },
    ];
    let picker = PickerOverlay::from_entries(format!("Cursor plan · {title}"), entries)
        .with_kind(PickerKind::AcpCursor { request_id });
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn acp_answer_cursor_question(
    runtime: &mut EditorRuntime,
    request_id: u64,
    question_id: String,
    option_id: String,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.answer_cursor_question(runtime, request_id, question_id, option_id)
}

pub(crate) fn acp_skip_cursor(runtime: &mut EditorRuntime, request_id: u64) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.resolve_cursor(request_id, named_outcome("skipped"));
    Ok(())
}

pub(crate) fn acp_resolve_cursor_plan(
    runtime: &mut EditorRuntime,
    request_id: u64,
    accepted: bool,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.resolve_cursor(request_id, plan_outcome(accepted));
    Ok(())
}

pub(crate) fn acp_cancel_cursor(
    runtime: &mut EditorRuntime,
    request_id: u64,
) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.resolve_cursor(request_id, named_outcome("cancelled"));
    Ok(())
}

pub(crate) fn open_permission_picker(
    runtime: &mut EditorRuntime,
    request: &AcpPendingPermissionUi,
) -> Result<(), String> {
    let entries = request
        .options
        .iter()
        .map(|option| PickerEntry {
            item: PickerItem::new(
                option.option_id.to_string(),
                option.name.clone(),
                format!(
                    "{} · {}",
                    request.workspace_name,
                    format_permission_option_kind(option.kind)
                ),
                None::<String>,
            ),
            action: PickerAction::AcpResolvePermission {
                request_id: request.request_id,
                option_id: option.option_id.to_string(),
            },
            quickfix: None,
        })
        .collect::<Vec<_>>();
    let picker =
        PickerOverlay::from_entries(format!("ACP Permission · {}", request.title()), entries)
            .with_kind(PickerKind::AcpPermission {
                request_id: request.request_id,
            });
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(crate) fn apply_acp_notification(
    runtime: &mut EditorRuntime,
    key: String,
    severity: NotificationSeverity,
    title: String,
    body_lines: Vec<String>,
    active: bool,
    action: Option<NotificationAction>,
) -> Result<(), String> {
    let workspace_id = shell_ui(runtime)?.active_workspace();
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key,
            severity,
            title,
            body_lines,
            progress: None,
            active,
            action,
            workspace_id: Some(workspace_id),
        },
        Instant::now(),
    );
    Ok(())
}

pub(crate) enum CompletionTrigger {
    Auto(String),
    Manual,
}

#[derive(Clone, Copy)]
pub(crate) enum PendingSlashTrigger {
    Auto,
    Manual,
}

pub(crate) enum AcpUiAction {
    OpenSlashCompletion {
        buffer_id: BufferId,
        trigger: PendingSlashTrigger,
    },
}

pub(crate) fn pending_slash_trigger(trigger: &CompletionTrigger) -> PendingSlashTrigger {
    match trigger {
        CompletionTrigger::Auto(_) => PendingSlashTrigger::Auto,
        CompletionTrigger::Manual => PendingSlashTrigger::Manual,
    }
}

pub(crate) fn acp_slash_completion_query(text: &str) -> Option<&str> {
    let trimmed = text.strip_prefix('/')?;
    (!trimmed.chars().any(|character| character.is_whitespace())).then_some(trimmed)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FileMention {
    pub(crate) at_char: usize,
    pub(crate) end_char: usize,
    pub(crate) query: String,
}
