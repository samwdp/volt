use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    env,
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use agent_client_protocol::{Agent, Client, ClientSideConnection};
use agent_client_protocol::{
    AvailableCommand, ClientCapabilities, ContentBlock, CreateTerminalRequest,
    CreateTerminalResponse, Error, FileSystemCapabilities, Implementation, InitializeRequest,
    KillTerminalRequest, KillTerminalResponse, ListSessionsRequest, LoadSessionRequest, ModelId,
    ModelInfo, NewSessionRequest, PermissionOption, PermissionOptionKind, ProtocolVersion,
    ReadTextFileRequest, ReadTextFileResponse, ReleaseTerminalRequest, ReleaseTerminalResponse,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse,
    SelectedPermissionOutcome, SessionConfigId, SessionConfigKind, SessionConfigOption,
    SessionConfigOptionCategory, SessionConfigSelectOption, SessionConfigSelectOptions,
    SessionConfigValueId, SessionInfo, SessionMode, SessionModeId, SessionModeState,
    SessionModelState, SessionNotification, SessionUpdate, SetSessionConfigOptionRequest,
    SetSessionModeRequest, SetSessionModelRequest, TerminalExitStatus, TerminalId,
    TerminalOutputRequest, TerminalOutputResponse, WaitForTerminalExitRequest,
    WaitForTerminalExitResponse, WriteTextFileRequest, WriteTextFileResponse,
};
use async_trait::async_trait;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    sync::{mpsc as tokio_mpsc, oneshot},
    task::LocalSet,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::*;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn configure_background_command(command: &mut Command) {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

pub(super) fn init_acp_manager(runtime: &mut EditorRuntime) -> Result<(), ShellError> {
    let manager = AcpManager::new().map_err(ShellError::Runtime)?;
    runtime.services_mut().insert(Arc::new(Mutex::new(manager)));
    Ok(())
}

pub(super) fn refresh_pending_acp(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let (events_changed, actions) = {
        let mut manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        let events_changed = manager.drain_events(runtime)?;
        (events_changed, manager.take_pending_ui_actions())
    };
    let mut changed = events_changed || !actions.is_empty();
    for action in actions {
        handle_acp_ui_action(runtime, action)?;
        changed = true;
    }
    Ok(changed)
}

pub(super) fn acp_connected(runtime: &EditorRuntime) -> Result<bool, String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    Ok(manager.has_sessions())
}

pub(super) fn open_acp_client(runtime: &mut EditorRuntime, client_id: &str) -> Result<(), String> {
    open_acp_client_buffer(runtime, client_id, true, None).map(|_| ())
}

pub(super) fn acp_new_session(runtime: &mut EditorRuntime) -> Result<(), String> {
    let client = active_acp_client(runtime)?;
    open_acp_client_with_config(runtime, client, false, None).map(|_| ())
}

fn open_acp_client_buffer(
    runtime: &mut EditorRuntime,
    client_id: &str,
    reuse_existing: bool,
    load_session_id: Option<agent_client_protocol::SessionId>,
) -> Result<BufferId, String> {
    let client = user::acp::client_by_id(client_id)
        .ok_or_else(|| format!("unknown ACP client `{client_id}`"))?;
    open_acp_client_with_config(runtime, client, reuse_existing, load_session_id)
}

fn open_acp_client_with_config(
    runtime: &mut EditorRuntime,
    client: user::acp::AcpClientConfig,
    reuse_existing: bool,
    load_session_id: Option<agent_client_protocol::SessionId>,
) -> Result<BufferId, String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    if reuse_existing
        && let Some(buffer_id) = {
            let manager = manager
                .lock()
                .map_err(|_| "acp manager lock was poisoned".to_owned())?;
            manager.buffer_for_client(&client.id)
        }
    {
        focus_acp_buffer(runtime, buffer_id)?;
        return Ok(buffer_id);
    }

    let buffer_id = create_acp_buffer(runtime, &client)?;
    let workspace_root = active_workspace_root(runtime)?
        .or_else(|| env::current_dir().ok())
        .ok_or_else(|| "ACP requires a workspace root or current directory".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.connect(client, workspace_root, buffer_id, load_session_id)?;
    Ok(buffer_id)
}

fn create_acp_buffer(
    runtime: &mut EditorRuntime,
    client: &user::acp::AcpClientConfig,
) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_name = format!("*acp {}*", client.label);
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            buffer_name.as_str(),
            BufferKind::Plugin(user::acp::ACP_BUFFER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let mut shell_buffer =
        ShellBuffer::from_runtime_buffer(buffer, initial_acp_buffer_lines(client));
    shell_buffer.clear_input();
    shell_buffer.set_language_id(Some("markdown".to_owned()));
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
    Ok(buffer_id)
}

fn initial_acp_buffer_lines(client: &user::acp::AcpClientConfig) -> Vec<String> {
    let mut lines = Vec::new();
    if client.id.eq_ignore_ascii_case("copilot") {
        lines.push("![copilot](copilot.png)".to_owned());
        lines.push(String::new());
    }
    lines.push(format!("# {}", client.label));
    lines.push(String::new());
    lines.push(format!(
        "{} Connected via ACP.",
        user::icon_font::symbols::fa::FA_CONNECTDEVELOP
    ));
    lines.push(String::new());
    lines
}

fn focus_acp_buffer(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
    Ok(())
}

fn active_acp_client(runtime: &EditorRuntime) -> Result<user::acp::AcpClientConfig, String> {
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
    user::acp::client_by_id(&client_id).ok_or_else(|| format!("unknown ACP client `{client_id}`"))
}

pub(super) fn submit_acp_prompt(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    prompt: &str,
    text: &str,
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
        buffer.clear_input();
        refresh_acp_input_hint(runtime, buffer_id)?;
        return Ok(());
    };
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!(
            "{} **You:** {prompt}{text}",
            user::icon_font::symbols::md::MD_ACCOUNT
        )]);
        buffer.clear_input();
    }
    refresh_acp_input_hint(runtime, buffer_id)?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.prompt(session_id, text.to_owned())
}

pub(super) fn acp_complete_slash(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !matches!(
        &buffer.kind,
        BufferKind::Plugin(plugin_kind) if plugin_kind == user::acp::ACP_BUFFER_KIND
    ) {
        return Ok(());
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

pub(super) fn maybe_open_slash_completion(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    if !matches!(
        &buffer.kind,
        BufferKind::Plugin(plugin_kind) if plugin_kind == user::acp::ACP_BUFFER_KIND
    ) {
        return Ok(());
    }
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let Some(input) = buffer.input_field() else {
        return Ok(());
    };
    let text = input.text();
    if !text.starts_with('/') {
        return Ok(());
    }
    let trimmed = text.trim_start_matches('/');
    if trimmed.chars().any(|character| character.is_whitespace()) {
        return Ok(());
    }
    open_slash_command_picker(
        runtime,
        buffer_id,
        CompletionTrigger::Auto(trimmed.to_owned()),
    )
}

pub(super) fn acp_insert_slash_command(
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

fn format_acp_mode_label(mode_id: &SessionModeId) -> String {
    let raw = mode_id.to_string();
    if let Some((_, suffix)) = raw.rsplit_once('#')
        && !suffix.is_empty()
    {
        return suffix.to_owned();
    }
    raw
}

fn format_acp_model_label(model_id: &ModelId) -> String {
    let raw = model_id.to_string();
    if let Some((_, suffix)) = raw.rsplit_once('/')
        && !suffix.is_empty()
    {
        return suffix.to_owned();
    }
    raw
}

fn command_input_hint(command: &AvailableCommand) -> Option<&str> {
    match command.input.as_ref() {
        Some(agent_client_protocol::AvailableCommandInput::Unstructured(input)) => {
            Some(input.hint.as_str())
        }
        _ => None,
    }
}

fn active_command_input_hint(commands: &[AvailableCommand], text: &str) -> Option<String> {
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

fn build_acp_input_hint(
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

fn update_acp_input_hint(
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
    let command_hint = active_command_input_hint(available_commands, &input_text);
    let hint = build_acp_input_hint(mode_id, model_id, command_hint.as_deref());
    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id)
        && let Some(input) = buffer.input_field_mut()
    {
        input.set_hint(hint);
    }
}

pub(super) fn refresh_acp_input_hint(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let is_acp = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        matches!(
            &buffer.kind,
            BufferKind::Plugin(plugin_kind) if plugin_kind == user::acp::ACP_BUFFER_KIND
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

fn config_option_matches(option: &SessionConfigOption, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    let id = option.id.to_string().to_ascii_lowercase();
    let name = option.name.to_ascii_lowercase();
    id.contains(&needle) || name.contains(&needle)
}

fn config_option_is_mode(option: &SessionConfigOption) -> bool {
    matches!(option.category, Some(SessionConfigOptionCategory::Mode))
        || (option.category.is_none() && config_option_matches(option, "mode"))
}

fn config_option_is_model(option: &SessionConfigOption) -> bool {
    matches!(option.category, Some(SessionConfigOptionCategory::Model))
        || (option.category.is_none() && config_option_matches(option, "model"))
}

fn flatten_config_select_options(
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

fn session_mode_state_from_config(option: &SessionConfigOption) -> Option<SessionModeState> {
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

fn session_model_state_from_config(option: &SessionConfigOption) -> Option<SessionModelState> {
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

pub(super) fn acp_pick_session(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn acp_load_session(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    session_id: &str,
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
    open_acp_client_buffer(runtime, &client_id, false, Some(target_session_id)).map(|_| ())
}

pub(super) fn acp_pick_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
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
    let entries = mode_state
        .available_modes
        .into_iter()
        .map(|mode| {
            let label = format_acp_mode_label(&mode.id);
            let detail = (mode.id == current_mode).then_some("current".to_owned());
            PickerEntry {
                item: PickerItem::new(
                    mode.id.to_string(),
                    label,
                    detail.unwrap_or_default(),
                    None::<String>,
                ),
                action: PickerAction::AcpSetMode {
                    buffer_id,
                    mode_id: mode.id.to_string(),
                },
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries("ACP Modes", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn acp_pick_model(runtime: &mut EditorRuntime) -> Result<(), String> {
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
    let entries = model_state
        .available_models
        .into_iter()
        .map(|model| {
            let mut detail = model
                .description
                .clone()
                .unwrap_or_else(|| model.model_id.to_string());
            if model.model_id == current_model {
                detail.push_str(" | current");
            }
            PickerEntry {
                item: PickerItem::new(
                    model.model_id.to_string(),
                    model.name,
                    detail,
                    None::<String>,
                ),
                action: PickerAction::AcpSetModel {
                    buffer_id,
                    model_id: model.model_id.to_string(),
                },
            }
        })
        .collect();
    let picker = PickerOverlay::from_entries("ACP Models", entries);
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

pub(super) fn acp_set_model(
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

pub(super) fn acp_set_mode(
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

pub(super) fn acp_cycle_mode(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn acp_disconnect(runtime: &mut EditorRuntime) -> Result<(), String> {
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

pub(super) fn acp_permission_approve(runtime: &mut EditorRuntime) -> Result<(), String> {
    resolve_permission(runtime, PermissionDecision::Approve)
}

pub(super) fn acp_permission_deny(runtime: &mut EditorRuntime) -> Result<(), String> {
    resolve_permission(runtime, PermissionDecision::Deny)
}

fn resolve_permission(
    runtime: &mut EditorRuntime,
    decision: PermissionDecision,
) -> Result<(), String> {
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
    manager.resolve_permission(session_id, decision);
    Ok(())
}

enum CompletionTrigger {
    Auto(String),
    Manual,
}

#[derive(Clone, Copy)]
enum PendingSlashTrigger {
    Auto,
    Manual,
}

enum AcpUiAction {
    OpenSlashCompletion {
        buffer_id: BufferId,
        trigger: PendingSlashTrigger,
    },
}

fn pending_slash_trigger(trigger: &CompletionTrigger) -> PendingSlashTrigger {
    match trigger {
        CompletionTrigger::Auto(_) => PendingSlashTrigger::Auto,
        CompletionTrigger::Manual => PendingSlashTrigger::Manual,
    }
}

fn pending_slash_completion_trigger(
    buffer: &ShellBuffer,
    pending: PendingSlashTrigger,
) -> Option<CompletionTrigger> {
    let input = buffer.input_field()?;
    let text = input.text();
    match pending {
        PendingSlashTrigger::Auto => {
            if !text.starts_with('/') {
                return None;
            }
            let trimmed = text.trim_start_matches('/');
            if trimmed.contains(' ') {
                return None;
            }
            Some(CompletionTrigger::Auto(trimmed.to_owned()))
        }
        PendingSlashTrigger::Manual => {
            if text.is_empty() || text.starts_with('/') {
                Some(CompletionTrigger::Manual)
            } else {
                None
            }
        }
    }
}

fn handle_acp_ui_action(runtime: &mut EditorRuntime, action: AcpUiAction) -> Result<(), String> {
    match action {
        AcpUiAction::OpenSlashCompletion { buffer_id, trigger } => {
            let buffer = shell_buffer(runtime, buffer_id)?;
            let Some(trigger) = pending_slash_completion_trigger(buffer, trigger) else {
                return Ok(());
            };
            open_slash_command_picker(runtime, buffer_id, trigger)?;
        }
    }
    Ok(())
}

fn open_slash_command_picker(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    trigger: CompletionTrigger,
) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let commands = {
        let mut manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        let commands = manager
            .available_commands_for_buffer(buffer_id)
            .unwrap_or_default();
        if commands.is_empty() {
            manager.queue_slash_completion(buffer_id, pending_slash_trigger(&trigger));
            return Ok(());
        }
        commands
    };
    let entries = commands
        .into_iter()
        .map(|command| {
            let mut detail = command.description.clone();
            if let Some(agent_client_protocol::AvailableCommandInput::Unstructured(input)) =
                command.input.as_ref()
            {
                detail.push_str(&format!(" | {}", input.hint));
            }
            PickerEntry {
                item: PickerItem::new(
                    command.name.as_str(),
                    format!("/{}", command.name),
                    detail,
                    None::<String>,
                ),
                action: PickerAction::AcpInsertSlashCommand {
                    buffer_id,
                    command: command.name,
                },
            }
        })
        .collect();
    let mut picker = PickerOverlay::from_entries("ACP Slash Commands", entries);
    match trigger {
        CompletionTrigger::Auto(query) => {
            if !query.is_empty() {
                picker.append_query(&query);
            }
        }
        CompletionTrigger::Manual => {}
    }
    shell_ui_mut(runtime)?.set_picker(picker);
    Ok(())
}

struct AcpManager {
    runtime: AcpRuntime,
    events: mpsc::Receiver<AcpEvent>,
    sessions: HashMap<agent_client_protocol::SessionId, AcpSessionInfo>,
    buffers: HashMap<BufferId, agent_client_protocol::SessionId>,
    pending_clients: HashMap<BufferId, PendingAcpClient>,
    pending_slash: HashMap<BufferId, PendingSlashTrigger>,
    pending_ui_actions: Vec<AcpUiAction>,
}

impl AcpManager {
    fn new() -> Result<Self, String> {
        let (event_tx, event_rx) = mpsc::channel();
        let runtime = AcpRuntime::new(event_tx)?;
        Ok(Self {
            runtime,
            events: event_rx,
            sessions: HashMap::new(),
            buffers: HashMap::new(),
            pending_clients: HashMap::new(),
            pending_slash: HashMap::new(),
            pending_ui_actions: Vec::new(),
        })
    }

    fn buffer_for_client(&self, client_id: &str) -> Option<BufferId> {
        self.sessions
            .values()
            .find(|session| session.client_id == client_id)
            .map(|session| session.buffer_id)
            .or_else(|| {
                self.pending_clients
                    .values()
                    .find(|pending| pending.client_id == client_id)
                    .map(|pending| pending.buffer_id)
            })
    }

    fn buffer_for_session(
        &self,
        session_id: &agent_client_protocol::SessionId,
    ) -> Option<BufferId> {
        self.sessions
            .get(session_id)
            .map(|session| session.buffer_id)
    }

    fn session_for_buffer(&self, buffer_id: BufferId) -> Option<agent_client_protocol::SessionId> {
        self.buffers.get(&buffer_id).cloned()
    }

    fn client_id_for_buffer(&self, buffer_id: BufferId) -> Option<String> {
        let session_id = self.session_for_buffer(buffer_id)?;
        self.sessions
            .get(&session_id)
            .map(|session| session.client_id.clone())
            .or_else(|| {
                self.pending_clients
                    .get(&buffer_id)
                    .map(|pending| pending.client_id.clone())
            })
    }

    fn available_commands_for_buffer(&self, buffer_id: BufferId) -> Option<Vec<AvailableCommand>> {
        let session_id = self.session_for_buffer(buffer_id)?;
        self.sessions
            .get(&session_id)
            .map(|session| session.available_commands.clone())
    }

    fn mode_state_for_buffer(&self, buffer_id: BufferId) -> Option<SessionModeState> {
        let session_id = self.session_for_buffer(buffer_id)?;
        self.sessions
            .get(&session_id)
            .and_then(|session| session.mode_state.clone())
    }

    fn model_state_for_buffer(&self, buffer_id: BufferId) -> Option<SessionModelState> {
        let session_id = self.session_for_buffer(buffer_id)?;
        self.sessions
            .get(&session_id)
            .and_then(|session| session.model_state.clone())
    }

    fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }

    fn queue_slash_completion(&mut self, buffer_id: BufferId, trigger: PendingSlashTrigger) {
        self.pending_slash.insert(buffer_id, trigger);
    }

    fn take_pending_ui_actions(&mut self) -> Vec<AcpUiAction> {
        std::mem::take(&mut self.pending_ui_actions)
    }

    fn connect(
        &mut self,
        client: user::acp::AcpClientConfig,
        workspace_root: PathBuf,
        buffer_id: BufferId,
        load_session_id: Option<agent_client_protocol::SessionId>,
    ) -> Result<(), String> {
        self.pending_clients.insert(
            buffer_id,
            PendingAcpClient {
                client_id: client.id.clone(),
                buffer_id,
                load_session_id,
                workspace_root: workspace_root.clone(),
            },
        );
        self.runtime.send(AcpCommand::Connect {
            config: client,
            workspace_root,
            buffer_id,
        })
    }

    fn prompt(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        prompt: String,
    ) -> Result<(), String> {
        self.runtime.send(AcpCommand::Prompt { session_id, prompt })
    }

    fn list_sessions(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        buffer_id: BufferId,
        cwd: PathBuf,
    ) -> Result<(), String> {
        self.runtime.send(AcpCommand::ListSessions {
            session_id,
            buffer_id,
            cwd,
        })
    }

    fn load_session(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        buffer_id: BufferId,
        target_session_id: agent_client_protocol::SessionId,
        cwd: PathBuf,
    ) -> Result<(), String> {
        self.runtime.send(AcpCommand::LoadSession {
            session_id,
            buffer_id,
            target_session_id,
            cwd,
        })
    }

    fn set_mode(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        mode_id: SessionModeId,
    ) -> Result<(), String> {
        if let Some(config_id) = self
            .sessions
            .get(&session_id)
            .and_then(|session| session.mode_config_id.clone())
        {
            return self.runtime.send(AcpCommand::SetConfigOption {
                session_id,
                config_id,
                value_id: SessionConfigValueId::new(mode_id.to_string()),
            });
        }
        self.runtime.send(AcpCommand::SetMode {
            session_id,
            mode_id,
        })
    }

    fn set_model(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        model_id: ModelId,
    ) -> Result<(), String> {
        if let Some(config_id) = self
            .sessions
            .get(&session_id)
            .and_then(|session| session.model_config_id.clone())
        {
            return self.runtime.send(AcpCommand::SetConfigOption {
                session_id,
                config_id,
                value_id: SessionConfigValueId::new(model_id.to_string()),
            });
        }
        self.runtime.send(AcpCommand::SetModel {
            session_id,
            model_id,
        })
    }

    fn disconnect(&mut self, session_id: agent_client_protocol::SessionId) {
        let _ = self.runtime.send(AcpCommand::Disconnect { session_id });
    }

    fn resolve_permission(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        decision: PermissionDecision,
    ) {
        let _ = self.runtime.send(AcpCommand::ResolvePermission {
            session_id,
            decision,
        });
    }

    fn drain_events(&mut self, runtime: &mut EditorRuntime) -> Result<bool, String> {
        let events: Vec<AcpEvent> = self.events.try_iter().collect();
        let changed = !events.is_empty();
        for event in events {
            self.handle_event(runtime, event)?;
        }
        Ok(changed)
    }

    fn handle_event(&mut self, runtime: &mut EditorRuntime, event: AcpEvent) -> Result<(), String> {
        match event {
            AcpEvent::Connected {
                buffer_id,
                client_id,
                session_id,
                modes,
                models,
            } => {
                let Some(pending) = self.pending_clients.remove(&buffer_id) else {
                    return Ok(());
                };
                self.buffers.insert(buffer_id, session_id.clone());
                self.sessions.insert(
                    session_id.clone(),
                    AcpSessionInfo {
                        client_id,
                        buffer_id,
                        available_commands: Vec::new(),
                        mode_state: modes,
                        model_state: models,
                        config_options: Vec::new(),
                        mode_config_id: None,
                        model_config_id: None,
                    },
                );
                if let Some(session) = self.sessions.get(&session_id) {
                    let mode_id = session
                        .mode_state
                        .as_ref()
                        .map(|state| &state.current_mode_id);
                    let model_id = session
                        .model_state
                        .as_ref()
                        .map(|state| &state.current_model_id);
                    update_acp_input_hint(
                        runtime,
                        buffer_id,
                        mode_id,
                        model_id,
                        &session.available_commands,
                    );
                }
                if let Some(target_session_id) = pending.load_session_id {
                    self.load_session(
                        session_id,
                        buffer_id,
                        target_session_id,
                        pending.workspace_root,
                    )?;
                }
            }
            AcpEvent::ClientFailed { buffer_id, message } => {
                self.pending_clients.remove(&buffer_id);
                if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                    buffer.append_output_lines(&[message]);
                }
            }
            AcpEvent::ClientLog { buffer_id, message } => {
                if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                    buffer.append_output_lines(&[message]);
                }
            }
            AcpEvent::SessionText { session_id, text } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                    && let Ok(buffer) = shell_buffer_mut(runtime, buffer_id)
                {
                    buffer.append_output_text(&text);
                }
            }
            AcpEvent::SessionLines { session_id, lines } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                    && let Ok(buffer) = shell_buffer_mut(runtime, buffer_id)
                {
                    buffer.append_output_lines(&lines);
                }
            }
            AcpEvent::SessionCommands {
                session_id,
                commands,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.available_commands = commands;
                    let mode_id = session
                        .mode_state
                        .as_ref()
                        .map(|state| &state.current_mode_id);
                    let model_id = session
                        .model_state
                        .as_ref()
                        .map(|state| &state.current_model_id);
                    update_acp_input_hint(
                        runtime,
                        session.buffer_id,
                        mode_id,
                        model_id,
                        &session.available_commands,
                    );
                    if !session.available_commands.is_empty()
                        && let Some(trigger) = self.pending_slash.remove(&session.buffer_id)
                    {
                        self.pending_ui_actions
                            .push(AcpUiAction::OpenSlashCompletion {
                                buffer_id: session.buffer_id,
                                trigger,
                            });
                    }
                }
            }
            AcpEvent::SessionConfigOptions {
                session_id,
                options,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    session.config_options = options;
                    let mode_option = session
                        .config_options
                        .iter()
                        .find(|option| config_option_is_mode(option));
                    if let Some(option) = mode_option {
                        session.mode_config_id = Some(option.id.clone());
                        if let Some(state) = session_mode_state_from_config(option) {
                            session.mode_state = Some(state);
                        }
                    } else {
                        session.mode_config_id = None;
                    }
                    let model_option = session
                        .config_options
                        .iter()
                        .find(|option| config_option_is_model(option));
                    if let Some(option) = model_option {
                        session.model_config_id = Some(option.id.clone());
                        if let Some(state) = session_model_state_from_config(option) {
                            session.model_state = Some(state);
                        }
                    } else {
                        session.model_config_id = None;
                    }
                    let mode_id = session
                        .mode_state
                        .as_ref()
                        .map(|state| &state.current_mode_id);
                    let model_id = session
                        .model_state
                        .as_ref()
                        .map(|state| &state.current_model_id);
                    update_acp_input_hint(
                        runtime,
                        session.buffer_id,
                        mode_id,
                        model_id,
                        &session.available_commands,
                    );
                }
            }
            AcpEvent::SessionModeUpdate {
                session_id,
                mode_id,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    match session.mode_state.as_mut() {
                        Some(state) => state.current_mode_id = mode_id,
                        None => {
                            session.mode_state = Some(SessionModeState::new(mode_id, Vec::new()));
                        }
                    }
                    let mode_id = session
                        .mode_state
                        .as_ref()
                        .map(|state| &state.current_mode_id);
                    let model_id = session
                        .model_state
                        .as_ref()
                        .map(|state| &state.current_model_id);
                    update_acp_input_hint(
                        runtime,
                        session.buffer_id,
                        mode_id,
                        model_id,
                        &session.available_commands,
                    );
                }
            }
            AcpEvent::SessionList {
                buffer_id,
                sessions,
            } => {
                if sessions.is_empty() {
                    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                        buffer.append_output_lines(&["ACP session list is empty.".to_owned()]);
                    }
                    return Ok(());
                }
                let current_session = self.buffers.get(&buffer_id).cloned();
                let entries = sessions
                    .into_iter()
                    .map(|session| {
                        let title = session
                            .title
                            .clone()
                            .unwrap_or_else(|| format!("Session {}", session.session_id));
                        let mut detail = session.cwd.display().to_string();
                        if let Some(updated_at) = session.updated_at {
                            detail.push_str(&format!(" | {updated_at}"));
                        }
                        if current_session
                            .as_ref()
                            .is_some_and(|current| *current == session.session_id)
                        {
                            detail.push_str(" | current");
                        }
                        PickerEntry {
                            item: PickerItem::new(
                                session.session_id.to_string(),
                                title,
                                detail,
                                None::<String>,
                            ),
                            action: PickerAction::AcpLoadSession {
                                buffer_id,
                                session_id: session.session_id.to_string(),
                            },
                        }
                    })
                    .collect();
                let picker = PickerOverlay::from_entries("ACP Sessions", entries);
                shell_ui_mut(runtime)?.set_picker(picker);
            }
            AcpEvent::SessionLoaded {
                buffer_id,
                old_session_id,
                new_session_id,
                modes,
                models,
            } => {
                if let Some(session) = self.sessions.remove(&old_session_id) {
                    self.buffers.insert(buffer_id, new_session_id.clone());
                    self.sessions.insert(
                        new_session_id.clone(),
                        AcpSessionInfo {
                            client_id: session.client_id,
                            buffer_id,
                            available_commands: Vec::new(),
                            mode_state: modes,
                            model_state: models,
                            config_options: Vec::new(),
                            mode_config_id: None,
                            model_config_id: None,
                        },
                    );
                    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                        buffer.replace_with_lines(Vec::new());
                        buffer.clear_input();
                    }
                    if let Some(session) = self.sessions.get(&new_session_id) {
                        let mode_id = session
                            .mode_state
                            .as_ref()
                            .map(|state| &state.current_mode_id);
                        let model_id = session
                            .model_state
                            .as_ref()
                            .map(|state| &state.current_model_id);
                        update_acp_input_hint(
                            runtime,
                            buffer_id,
                            mode_id,
                            model_id,
                            &session.available_commands,
                        );
                    }
                }
            }
            AcpEvent::SessionModeSet {
                session_id,
                mode_id,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    match session.mode_state.as_mut() {
                        Some(state) => state.current_mode_id = mode_id.clone(),
                        None => {
                            session.mode_state =
                                Some(SessionModeState::new(mode_id.clone(), Vec::new()));
                        }
                    }
                    let model_id = session
                        .model_state
                        .as_ref()
                        .map(|state| &state.current_model_id);
                    update_acp_input_hint(
                        runtime,
                        session.buffer_id,
                        Some(&mode_id),
                        model_id,
                        &session.available_commands,
                    );
                }
            }
            AcpEvent::SessionModelSet {
                session_id,
                model_id,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    match session.model_state.as_mut() {
                        Some(state) => state.current_model_id = model_id.clone(),
                        None => {
                            session.model_state =
                                Some(SessionModelState::new(model_id.clone(), Vec::new()));
                        }
                    }
                    let mode_id = session
                        .mode_state
                        .as_ref()
                        .map(|state| &state.current_mode_id);
                    update_acp_input_hint(
                        runtime,
                        session.buffer_id,
                        mode_id,
                        Some(&model_id),
                        &session.available_commands,
                    );
                }
            }
            AcpEvent::SessionConfigSet {
                session_id,
                config_id,
                value_id,
            } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    if session.mode_config_id.as_ref() == Some(&config_id) {
                        let mode_id = SessionModeId::new(value_id.to_string());
                        match session.mode_state.as_mut() {
                            Some(state) => state.current_mode_id = mode_id.clone(),
                            None => {
                                session.mode_state =
                                    Some(SessionModeState::new(mode_id.clone(), Vec::new()));
                            }
                        }
                    }
                    if session.model_config_id.as_ref() == Some(&config_id) {
                        let model_id = ModelId::new(value_id.to_string());
                        match session.model_state.as_mut() {
                            Some(state) => state.current_model_id = model_id.clone(),
                            None => {
                                session.model_state =
                                    Some(SessionModelState::new(model_id.clone(), Vec::new()));
                            }
                        }
                    }
                    let mode_id = session
                        .mode_state
                        .as_ref()
                        .map(|state| &state.current_mode_id);
                    let model_id = session
                        .model_state
                        .as_ref()
                        .map(|state| &state.current_model_id);
                    update_acp_input_hint(
                        runtime,
                        session.buffer_id,
                        mode_id,
                        model_id,
                        &session.available_commands,
                    );
                }
            }
            AcpEvent::Disconnected {
                session_id,
                message,
            } => {
                if let Some(session) = self.sessions.remove(&session_id) {
                    self.buffers.remove(&session.buffer_id);
                    self.pending_slash.remove(&session.buffer_id);
                    update_acp_input_hint(runtime, session.buffer_id, None, None, &[]);
                    let _ = message;
                }
            }
        }
        Ok(())
    }
}

struct PendingAcpClient {
    client_id: String,
    buffer_id: BufferId,
    load_session_id: Option<agent_client_protocol::SessionId>,
    workspace_root: PathBuf,
}

struct AcpSessionInfo {
    client_id: String,
    buffer_id: BufferId,
    available_commands: Vec<AvailableCommand>,
    mode_state: Option<SessionModeState>,
    model_state: Option<SessionModelState>,
    config_options: Vec<SessionConfigOption>,
    mode_config_id: Option<SessionConfigId>,
    model_config_id: Option<SessionConfigId>,
}

enum AcpEvent {
    Connected {
        buffer_id: BufferId,
        client_id: String,
        session_id: agent_client_protocol::SessionId,
        modes: Option<SessionModeState>,
        models: Option<SessionModelState>,
    },
    ClientFailed {
        buffer_id: BufferId,
        message: String,
    },
    ClientLog {
        buffer_id: BufferId,
        message: String,
    },
    SessionText {
        session_id: agent_client_protocol::SessionId,
        text: String,
    },
    SessionLines {
        session_id: agent_client_protocol::SessionId,
        lines: Vec<String>,
    },
    SessionCommands {
        session_id: agent_client_protocol::SessionId,
        commands: Vec<AvailableCommand>,
    },
    SessionConfigOptions {
        session_id: agent_client_protocol::SessionId,
        options: Vec<SessionConfigOption>,
    },
    SessionConfigSet {
        session_id: agent_client_protocol::SessionId,
        config_id: SessionConfigId,
        value_id: SessionConfigValueId,
    },
    SessionModeUpdate {
        session_id: agent_client_protocol::SessionId,
        mode_id: SessionModeId,
    },
    SessionList {
        buffer_id: BufferId,
        sessions: Vec<SessionInfo>,
    },
    SessionLoaded {
        buffer_id: BufferId,
        old_session_id: agent_client_protocol::SessionId,
        new_session_id: agent_client_protocol::SessionId,
        modes: Option<SessionModeState>,
        models: Option<SessionModelState>,
    },
    SessionModeSet {
        session_id: agent_client_protocol::SessionId,
        mode_id: SessionModeId,
    },
    SessionModelSet {
        session_id: agent_client_protocol::SessionId,
        model_id: ModelId,
    },
    Disconnected {
        session_id: agent_client_protocol::SessionId,
        message: String,
    },
}

enum AcpCommand {
    Connect {
        config: user::acp::AcpClientConfig,
        workspace_root: PathBuf,
        buffer_id: BufferId,
    },
    Prompt {
        session_id: agent_client_protocol::SessionId,
        prompt: String,
    },
    ListSessions {
        session_id: agent_client_protocol::SessionId,
        buffer_id: BufferId,
        cwd: PathBuf,
    },
    LoadSession {
        session_id: agent_client_protocol::SessionId,
        buffer_id: BufferId,
        target_session_id: agent_client_protocol::SessionId,
        cwd: PathBuf,
    },
    SetConfigOption {
        session_id: agent_client_protocol::SessionId,
        config_id: SessionConfigId,
        value_id: SessionConfigValueId,
    },
    SetMode {
        session_id: agent_client_protocol::SessionId,
        mode_id: SessionModeId,
    },
    SetModel {
        session_id: agent_client_protocol::SessionId,
        model_id: ModelId,
    },
    Disconnect {
        session_id: agent_client_protocol::SessionId,
    },
    ResolvePermission {
        session_id: agent_client_protocol::SessionId,
        decision: PermissionDecision,
    },
}

#[derive(Clone, Copy)]
enum PermissionDecision {
    Approve,
    Deny,
}

struct AcpRuntime {
    sender: tokio_mpsc::UnboundedSender<AcpCommand>,
}

impl AcpRuntime {
    fn new(event_tx: mpsc::Sender<AcpEvent>) -> Result<Self, String> {
        let (sender, receiver) = tokio_mpsc::unbounded_channel();
        thread::spawn(move || run_acp_runtime(receiver, event_tx));
        Ok(Self { sender })
    }

    fn send(&self, command: AcpCommand) -> Result<(), String> {
        self.sender
            .send(command)
            .map_err(|_| "ACP runtime is not running".to_owned())
    }
}

fn run_acp_runtime(
    receiver: tokio_mpsc::UnboundedReceiver<AcpCommand>,
    event_tx: mpsc::Sender<AcpEvent>,
) {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(_) => return,
    };
    let local = LocalSet::new();
    let state = Rc::new(RefCell::new(AcpRuntimeState::new(event_tx)));
    local.block_on(&runtime, async move {
        acp_runtime_loop(state, receiver).await;
    });
}

async fn acp_runtime_loop(
    state: Rc<RefCell<AcpRuntimeState>>,
    mut receiver: tokio_mpsc::UnboundedReceiver<AcpCommand>,
) {
    while let Some(command) = receiver.recv().await {
        match command {
            AcpCommand::Connect {
                config,
                workspace_root,
                buffer_id,
            } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    if let Err(error) =
                        connect_acp_client(state.clone(), config, workspace_root, buffer_id).await
                    {
                        send_client_failure(&state, buffer_id, error);
                    }
                });
            }
            AcpCommand::Prompt { session_id, prompt } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let session_id = session_id.clone();
                    if let Err(error) =
                        send_acp_prompt(state.clone(), session_id.clone(), prompt).await
                    {
                        send_session_lines(state, &session_id, vec![error]);
                    }
                });
            }
            AcpCommand::ListSessions {
                session_id,
                buffer_id,
                cwd,
            } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let session_id = session_id.clone();
                    let session_id_for_error = session_id.clone();
                    if let Err(error) =
                        list_acp_sessions(state.clone(), session_id, buffer_id, cwd).await
                    {
                        send_session_lines(state, &session_id_for_error, vec![error]);
                    }
                });
            }
            AcpCommand::LoadSession {
                session_id,
                buffer_id,
                target_session_id,
                cwd,
            } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let session_id = session_id.clone();
                    let session_id_for_error = session_id.clone();
                    if let Err(error) = load_acp_session(
                        state.clone(),
                        session_id,
                        buffer_id,
                        target_session_id,
                        cwd,
                    )
                    .await
                    {
                        send_session_lines(state, &session_id_for_error, vec![error]);
                    }
                });
            }
            AcpCommand::SetConfigOption {
                session_id,
                config_id,
                value_id,
            } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let session_id = session_id.clone();
                    if let Err(error) = set_acp_config_option(
                        state.clone(),
                        session_id.clone(),
                        config_id,
                        value_id,
                    )
                    .await
                    {
                        send_session_lines(state, &session_id, vec![error]);
                    }
                });
            }
            AcpCommand::SetMode {
                session_id,
                mode_id,
            } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let session_id = session_id.clone();
                    if let Err(error) =
                        set_acp_mode(state.clone(), session_id.clone(), mode_id).await
                    {
                        send_session_lines(state, &session_id, vec![error]);
                    }
                });
            }
            AcpCommand::SetModel {
                session_id,
                model_id,
            } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let session_id = session_id.clone();
                    if let Err(error) =
                        set_acp_model(state.clone(), session_id.clone(), model_id).await
                    {
                        send_session_lines(state, &session_id, vec![error]);
                    }
                });
            }
            AcpCommand::Disconnect { session_id } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    disconnect_acp_session(state, session_id).await;
                });
            }
            AcpCommand::ResolvePermission {
                session_id,
                decision,
            } => {
                resolve_permission_response(state.clone(), session_id, decision);
            }
        }
    }
}

async fn connect_acp_client(
    state: Rc<RefCell<AcpRuntimeState>>,
    config: user::acp::AcpClientConfig,
    workspace_root: PathBuf,
    buffer_id: BufferId,
) -> Result<(), String> {
    let mut command = Command::new(&config.command);
    configure_background_command(&mut command);
    command.args(&config.args);
    if let Some(cwd) = config.cwd.as_ref() {
        command.current_dir(cwd);
    } else {
        command.current_dir(&workspace_root);
    }
    for (key, value) in &config.env {
        command.env(key, value);
    }
    command.stdin(std::process::Stdio::piped());
    command.stdout(std::process::Stdio::piped());
    command.stderr(std::process::Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("failed to start ACP client: {error}"))?;
    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| "ACP client stdin unavailable".to_owned())?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "ACP client stdout unavailable".to_owned())?;
    if let Some(stderr) = child.stderr.take() {
        let state = state.clone();
        tokio::task::spawn_local(async move {
            drain_stderr(state, buffer_id, stderr).await;
        });
    }

    let client = Rc::new(AcpClient::new(state.clone()));
    let (connection, io_task) =
        ClientSideConnection::new(client, stdin.compat_write(), stdout.compat(), |task| {
            tokio::task::spawn_local(task);
        });
    let state_clone = state.clone();
    tokio::task::spawn_local(async move {
        if let Err(error) = io_task.await {
            send_client_log(&state_clone, buffer_id, format!("ACP I/O error: {error}"));
        }
    });

    let capabilities = ClientCapabilities::new()
        .fs(FileSystemCapabilities::new()
            .read_text_file(true)
            .write_text_file(true))
        .terminal(true);
    let init_request = InitializeRequest::new(ProtocolVersion::LATEST)
        .client_capabilities(capabilities)
        .client_info(
            Implementation::new("volt", env!("CARGO_PKG_VERSION")).title("Volt SDL shell"),
        );
    connection
        .initialize(init_request)
        .await
        .map_err(|error| format!("ACP initialize failed: {error}"))?;
    let session = connection
        .new_session(NewSessionRequest::new(workspace_root))
        .await
        .map_err(|error| format!("ACP new session failed: {error}"))?;
    let session_id = session.session_id.clone();
    let modes = session.modes.clone();
    let models = session.models.clone();

    state.borrow_mut().sessions.insert(
        session_id.clone(),
        AcpSession {
            connection: Rc::new(connection),
            child,
        },
    );
    let _ = state.borrow().event_tx.send(AcpEvent::Connected {
        buffer_id,
        client_id: config.id,
        session_id,
        modes,
        models,
    });
    Ok(())
}

async fn send_acp_prompt(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    prompt: String,
) -> Result<(), String> {
    let connection = {
        state
            .borrow()
            .sessions
            .get(&session_id)
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    {
        let mut state = state.borrow_mut();
        state.pending_agent_newline.insert(session_id.clone(), true);
    }
    let request = agent_client_protocol::PromptRequest::new(
        session_id.clone(),
        vec![ContentBlock::from(prompt)],
    );
    connection
        .prompt(request)
        .await
        .map_err(|error| format!("ACP prompt failed: {error}"))?;
    Ok(())
}

async fn list_acp_sessions(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    buffer_id: BufferId,
    cwd: PathBuf,
) -> Result<(), String> {
    let connection = {
        state
            .borrow()
            .sessions
            .get(&session_id)
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    let request = ListSessionsRequest::new().cwd(cwd);
    let response = connection
        .list_sessions(request)
        .await
        .map_err(|error| format!("ACP list sessions failed: {error}"))?;
    let _ = state.borrow().event_tx.send(AcpEvent::SessionList {
        buffer_id,
        sessions: response.sessions,
    });
    Ok(())
}

async fn load_acp_session(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    buffer_id: BufferId,
    target_session_id: agent_client_protocol::SessionId,
    cwd: PathBuf,
) -> Result<(), String> {
    let connection = {
        state
            .borrow()
            .sessions
            .get(&session_id)
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    let request = LoadSessionRequest::new(target_session_id.clone(), cwd);
    let response = connection
        .load_session(request)
        .await
        .map_err(|error| format!("ACP load session failed: {error}"))?;
    {
        let mut state = state.borrow_mut();
        if let Some(session) = state.sessions.remove(&session_id) {
            state.sessions.insert(target_session_id.clone(), session);
        }
        state.pending_agent_newline.remove(&session_id);
        state
            .pending_agent_newline
            .insert(target_session_id.clone(), true);
    }
    resolve_all_pending_permissions(&state, &session_id);
    let _ = state.borrow().event_tx.send(AcpEvent::SessionLoaded {
        buffer_id,
        old_session_id: session_id,
        new_session_id: target_session_id,
        modes: response.modes,
        models: response.models,
    });
    Ok(())
}

async fn set_acp_config_option(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    config_id: SessionConfigId,
    value_id: SessionConfigValueId,
) -> Result<(), String> {
    let connection = {
        state
            .borrow()
            .sessions
            .get(&session_id)
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    let request =
        SetSessionConfigOptionRequest::new(session_id.clone(), config_id.clone(), value_id.clone());
    connection
        .set_session_config_option(request)
        .await
        .map_err(|error| format!("ACP set config option failed: {error}"))?;
    let _ = state.borrow().event_tx.send(AcpEvent::SessionConfigSet {
        session_id,
        config_id,
        value_id,
    });
    Ok(())
}

async fn set_acp_mode(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    mode_id: SessionModeId,
) -> Result<(), String> {
    let connection = {
        state
            .borrow()
            .sessions
            .get(&session_id)
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    let request = SetSessionModeRequest::new(session_id.clone(), mode_id.clone());
    connection
        .set_session_mode(request)
        .await
        .map_err(|error| format!("ACP set mode failed: {error}"))?;
    let _ = state.borrow().event_tx.send(AcpEvent::SessionModeSet {
        session_id,
        mode_id,
    });
    Ok(())
}

async fn set_acp_model(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    model_id: ModelId,
) -> Result<(), String> {
    let connection = {
        state
            .borrow()
            .sessions
            .get(&session_id)
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    let request = SetSessionModelRequest::new(session_id.clone(), model_id.clone());
    connection
        .set_session_model(request)
        .await
        .map_err(|error| format!("ACP set model failed: {error}"))?;
    let _ = state.borrow().event_tx.send(AcpEvent::SessionModelSet {
        session_id,
        model_id,
    });
    Ok(())
}

async fn disconnect_acp_session(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
) {
    let session = {
        let mut state = state.borrow_mut();
        state.pending_agent_newline.remove(&session_id);
        state.sessions.remove(&session_id)
    };
    if let Some(mut session) = session {
        let _ = session.child.kill().await;
    }
    resolve_all_pending_permissions(&state, &session_id);
    let _ = state.borrow().event_tx.send(AcpEvent::Disconnected {
        session_id,
        message: "Disconnected.".to_owned(),
    });
}

fn resolve_all_pending_permissions(
    state: &Rc<RefCell<AcpRuntimeState>>,
    session_id: &agent_client_protocol::SessionId,
) {
    let mut pending = Vec::new();
    {
        let mut state = state.borrow_mut();
        let mut index = 0;
        while index < state.pending_permissions.len() {
            if state.pending_permissions[index].session_id == *session_id {
                if let Some(entry) = state.pending_permissions.remove(index) {
                    pending.push(entry);
                }
            } else {
                index += 1;
            }
        }
    }
    for pending in pending {
        let _ = pending.responder.send(RequestPermissionOutcome::Cancelled);
    }
}

fn resolve_permission_response(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    decision: PermissionDecision,
) {
    let pending = {
        let mut state = state.borrow_mut();
        let index = state
            .pending_permissions
            .iter()
            .position(|pending| pending.session_id == session_id);
        index.and_then(|index| state.pending_permissions.remove(index))
    };
    let Some(pending) = pending else {
        return;
    };
    let outcome = match decision {
        PermissionDecision::Approve => choose_permission_outcome(
            &pending.options,
            PermissionOptionKind::AllowOnce,
            PermissionOptionKind::AllowAlways,
        ),
        PermissionDecision::Deny => choose_permission_outcome(
            &pending.options,
            PermissionOptionKind::RejectOnce,
            PermissionOptionKind::RejectAlways,
        ),
    };
    let _ = pending.responder.send(outcome.clone());
    let label = match decision {
        PermissionDecision::Approve => "Permission approved.",
        PermissionDecision::Deny => "Permission denied.",
    };
    send_session_lines(state, &session_id, vec![label.to_owned()]);
}

fn choose_permission_outcome(
    options: &[PermissionOption],
    preferred: PermissionOptionKind,
    fallback: PermissionOptionKind,
) -> RequestPermissionOutcome {
    let option = options
        .iter()
        .find(|option| option.kind == preferred)
        .or_else(|| options.iter().find(|option| option.kind == fallback))
        .or_else(|| options.first());
    option
        .map(|option| {
            RequestPermissionOutcome::Selected(SelectedPermissionOutcome::new(
                option.option_id.clone(),
            ))
        })
        .unwrap_or(RequestPermissionOutcome::Cancelled)
}

async fn drain_stderr(
    state: Rc<RefCell<AcpRuntimeState>>,
    buffer_id: BufferId,
    stderr: tokio::process::ChildStderr,
) {
    let mut reader = BufReader::new(stderr);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => {
                let message = line.trim_end().to_owned();
                if !message.is_empty() {
                    send_client_log(&state, buffer_id, message);
                }
            }
            Err(error) => {
                send_client_log(&state, buffer_id, format!("ACP stderr error: {error}"));
                break;
            }
        }
    }
}

fn send_client_log(state: &Rc<RefCell<AcpRuntimeState>>, buffer_id: BufferId, message: String) {
    let _ = state
        .borrow()
        .event_tx
        .send(AcpEvent::ClientLog { buffer_id, message });
}

fn send_client_failure(state: &Rc<RefCell<AcpRuntimeState>>, buffer_id: BufferId, message: String) {
    let _ = state
        .borrow()
        .event_tx
        .send(AcpEvent::ClientFailed { buffer_id, message });
}

fn send_session_lines(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: &agent_client_protocol::SessionId,
    lines: Vec<String>,
) {
    let _ = state.borrow().event_tx.send(AcpEvent::SessionLines {
        session_id: session_id.clone(),
        lines,
    });
}

struct AcpRuntimeState {
    sessions: HashMap<agent_client_protocol::SessionId, AcpSession>,
    terminals: HashMap<TerminalId, AcpTerminal>,
    pending_permissions: VecDeque<PendingPermission>,
    pending_agent_newline: HashMap<agent_client_protocol::SessionId, bool>,
    event_tx: mpsc::Sender<AcpEvent>,
}

impl AcpRuntimeState {
    fn new(event_tx: mpsc::Sender<AcpEvent>) -> Self {
        Self {
            sessions: HashMap::new(),
            terminals: HashMap::new(),
            pending_permissions: VecDeque::new(),
            pending_agent_newline: HashMap::new(),
            event_tx,
        }
    }
}

struct AcpSession {
    connection: Rc<ClientSideConnection>,
    child: tokio::process::Child,
}

struct PendingPermission {
    session_id: agent_client_protocol::SessionId,
    options: Vec<PermissionOption>,
    responder: oneshot::Sender<RequestPermissionOutcome>,
}

struct AcpTerminal {
    output: Rc<RefCell<String>>,
    exit_status: Rc<RefCell<Option<TerminalExitStatus>>>,
    output_limit: Option<u64>,
    child: tokio::process::Child,
}

#[async_trait(?Send)]
impl Client for AcpClient {
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> agent_client_protocol::Result<RequestPermissionResponse> {
        let (tx, rx) = oneshot::channel();
        {
            let mut state = self.state.borrow_mut();
            let session_id = args.session_id.clone();
            let lines = permission_prompt_lines(&args);
            state.pending_permissions.push_back(PendingPermission {
                session_id: args.session_id.clone(),
                options: args.options.clone(),
                responder: tx,
            });
            let _ = state
                .event_tx
                .send(AcpEvent::SessionLines { session_id, lines });
        }
        let outcome = rx.await.unwrap_or(RequestPermissionOutcome::Cancelled);
        Ok(RequestPermissionResponse::new(outcome))
    }

    async fn session_notification(
        &self,
        args: SessionNotification,
    ) -> agent_client_protocol::Result<()> {
        let session_id = args.session_id.clone();
        handle_session_update(self.state.clone(), session_id, args.update);
        Ok(())
    }

    async fn write_text_file(
        &self,
        args: WriteTextFileRequest,
    ) -> agent_client_protocol::Result<WriteTextFileResponse> {
        let path = args.path.clone();
        let result = tokio::task::spawn_blocking(move || std::fs::write(&path, args.content))
            .await
            .map_err(|error| Error::internal_error().data(error.to_string()))?;
        match result {
            Ok(()) => Ok(WriteTextFileResponse::new()),
            Err(error) => Err(Error::internal_error().data(error.to_string())),
        }
    }

    async fn read_text_file(
        &self,
        args: ReadTextFileRequest,
    ) -> agent_client_protocol::Result<ReadTextFileResponse> {
        let path = args.path.clone();
        let path_for_error = path.clone();
        let start_line = args.line.unwrap_or(1).saturating_sub(1) as usize;
        let limit = args.limit.map(|limit| limit as usize);
        let result: Result<String, std::io::Error> = tokio::task::spawn_blocking(move || {
            let content = std::fs::read_to_string(&path)?;
            if start_line == 0 && limit.is_none() {
                return Ok(content);
            }
            let mut lines = content.lines().skip(start_line);
            let mut collected = Vec::new();
            if let Some(limit) = limit {
                collected.extend(lines.by_ref().take(limit));
            } else {
                collected.extend(lines);
            }
            Ok(collected.join("\n"))
        })
        .await
        .map_err(|error| Error::internal_error().data(error.to_string()))?;
        match result {
            Ok(content) => Ok(ReadTextFileResponse::new(content)),
            Err(error) => {
                let message = error.to_string();
                if error.kind() == std::io::ErrorKind::NotFound {
                    Err(Error::resource_not_found(Some(
                        path_for_error.display().to_string(),
                    )))
                } else {
                    Err(Error::internal_error().data(message))
                }
            }
        }
    }

    async fn create_terminal(
        &self,
        args: CreateTerminalRequest,
    ) -> agent_client_protocol::Result<CreateTerminalResponse> {
        let mut command = Command::new(args.command);
        configure_background_command(&mut command);
        command.args(args.args);
        if let Some(cwd) = args.cwd.as_ref() {
            command.current_dir(cwd);
        }
        for variable in args.env {
            command.env(variable.name, variable.value);
        }
        command.stdout(std::process::Stdio::piped());
        command.stderr(std::process::Stdio::piped());
        let mut child = command
            .spawn()
            .map_err(|error| Error::internal_error().data(error.to_string()))?;
        let output = Rc::new(RefCell::new(String::new()));
        let exit_status = Rc::new(RefCell::new(None));
        if let Some(stdout) = child.stdout.take() {
            spawn_terminal_reader(output.clone(), stdout);
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_terminal_reader(output.clone(), stderr);
        }
        let terminal_id = TerminalId::new(format!(
            "acp-{}-{}",
            std::process::id(),
            self.next_terminal_id()
        ));
        self.state.borrow_mut().terminals.insert(
            terminal_id.clone(),
            AcpTerminal {
                output,
                exit_status,
                output_limit: args.output_byte_limit,
                child,
            },
        );
        Ok(CreateTerminalResponse::new(terminal_id))
    }

    async fn terminal_output(
        &self,
        args: TerminalOutputRequest,
    ) -> agent_client_protocol::Result<TerminalOutputResponse> {
        let mut state = self.state.borrow_mut();
        let terminal = state
            .terminals
            .get_mut(&args.terminal_id)
            .ok_or_else(|| Error::resource_not_found(None))?;
        if terminal.exit_status.borrow().is_none()
            && let Ok(Some(status)) = terminal.child.try_wait()
        {
            let exit = TerminalExitStatus::new().exit_code(status.code().map(|code| code as u32));
            *terminal.exit_status.borrow_mut() = Some(exit);
        }
        let output = terminal.output.borrow().clone();
        let (trimmed, truncated) = apply_output_limit(&output, terminal.output_limit);
        let mut response = TerminalOutputResponse::new(trimmed, truncated);
        if let Some(exit_status) = terminal.exit_status.borrow().clone() {
            response = response.exit_status(exit_status);
        }
        Ok(response)
    }

    async fn wait_for_terminal_exit(
        &self,
        args: WaitForTerminalExitRequest,
    ) -> agent_client_protocol::Result<WaitForTerminalExitResponse> {
        let terminal = self.state.borrow_mut().terminals.remove(&args.terminal_id);
        let Some(mut terminal) = terminal else {
            return Err(Error::resource_not_found(None));
        };
        let status = terminal
            .child
            .wait()
            .await
            .map_err(|error| Error::internal_error().data(error.to_string()))?;
        let exit = TerminalExitStatus::new().exit_code(status.code().map(|code| code as u32));
        *terminal.exit_status.borrow_mut() = Some(exit.clone());
        let terminal_id = args.terminal_id.clone();
        self.state
            .borrow_mut()
            .terminals
            .insert(terminal_id, terminal);
        Ok(WaitForTerminalExitResponse::new(exit))
    }

    async fn release_terminal(
        &self,
        args: ReleaseTerminalRequest,
    ) -> agent_client_protocol::Result<ReleaseTerminalResponse> {
        let terminal = self.state.borrow_mut().terminals.remove(&args.terminal_id);
        if let Some(mut terminal) = terminal {
            let _ = terminal.child.kill().await;
        }
        Ok(ReleaseTerminalResponse::new())
    }

    async fn kill_terminal(
        &self,
        args: KillTerminalRequest,
    ) -> agent_client_protocol::Result<KillTerminalResponse> {
        let terminal = self.state.borrow_mut().terminals.remove(&args.terminal_id);
        let Some(mut terminal) = terminal else {
            return Err(Error::resource_not_found(None));
        };
        let _ = terminal.child.kill().await;
        self.state
            .borrow_mut()
            .terminals
            .insert(args.terminal_id, terminal);
        Ok(KillTerminalResponse::new())
    }
}

struct AcpClient {
    state: Rc<RefCell<AcpRuntimeState>>,
    next_terminal_id: RefCell<u64>,
}

impl AcpClient {
    fn new(state: Rc<RefCell<AcpRuntimeState>>) -> Self {
        Self {
            state,
            next_terminal_id: RefCell::new(1),
        }
    }

    fn next_terminal_id(&self) -> u64 {
        let mut next = self.next_terminal_id.borrow_mut();
        let id = *next;
        *next = next.saturating_add(1);
        id
    }
}

fn handle_session_update(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    update: SessionUpdate,
) {
    match update {
        SessionUpdate::AgentMessageChunk(chunk) => {
            if let ContentBlock::Text(text) = chunk.content {
                let prefix_newline = {
                    let mut state = state.borrow_mut();
                    state
                        .pending_agent_newline
                        .remove(&session_id)
                        .unwrap_or(false)
                };
                let mut output = text.text;
                if prefix_newline && !output.starts_with('\n') {
                    output = format!("\n{output}");
                }
                let _ = state.borrow().event_tx.send(AcpEvent::SessionText {
                    session_id,
                    text: output,
                });
            }
        }
        SessionUpdate::AgentThoughtChunk(_) => {}
        SessionUpdate::ToolCall(call) => {
            let mut lines = vec![format!(
                "{} **{}** · {}",
                user::icon_font::symbols::cod::COD_TOOLS,
                call.title,
                format_acp_status_badge(&call.status)
            )];
            lines.extend(render_tool_call_content(&call.content));
            let _ = state
                .borrow()
                .event_tx
                .send(AcpEvent::SessionLines { session_id, lines });
        }
        SessionUpdate::ToolCallUpdate(update) => {
            let mut lines = vec![format!(
                "{} linked update `{}`",
                user::icon_font::symbols::cod::COD_LINK,
                update.tool_call_id
            )];
            if let Some(status) = update.fields.status {
                lines.push(format!("  {}", format_acp_status_badge(&status)));
            }
            if let Some(content) = update.fields.content {
                lines.extend(render_tool_call_content(&content));
            }
            let _ = state
                .borrow()
                .event_tx
                .send(AcpEvent::SessionLines { session_id, lines });
        }
        SessionUpdate::Plan(plan) => {
            let mut lines = Vec::new();
            for entry in plan.entries {
                lines.push(format!(
                    "{} {} · {}",
                    user::icon_font::symbols::cod::COD_NOTEBOOK,
                    entry.content,
                    format_acp_status_badge(&entry.status)
                ));
            }
            if !lines.is_empty() {
                let _ = state
                    .borrow()
                    .event_tx
                    .send(AcpEvent::SessionLines { session_id, lines });
            }
        }
        SessionUpdate::AvailableCommandsUpdate(update) => {
            let commands = update.available_commands.clone();
            let _ = state.borrow().event_tx.send(AcpEvent::SessionCommands {
                session_id: session_id.clone(),
                commands,
            });
        }
        SessionUpdate::CurrentModeUpdate(update) => {
            let mode_id = update.current_mode_id.clone();
            let _ = state.borrow().event_tx.send(AcpEvent::SessionModeUpdate {
                session_id: session_id.clone(),
                mode_id,
            });
        }
        SessionUpdate::ConfigOptionUpdate(update) => {
            let _ = state
                .borrow()
                .event_tx
                .send(AcpEvent::SessionConfigOptions {
                    session_id: session_id.clone(),
                    options: update.config_options,
                });
        }
        SessionUpdate::SessionInfoUpdate(update) => {
            let _ = update;
        }
        _ => {}
    }
}

fn format_acp_status_badge(status: &impl std::fmt::Debug) -> String {
    let raw = format!("{status:?}");
    let icon = match raw.as_str() {
        "Pending" | "Running" | "InProgress" => user::icon_font::symbols::cod::COD_LOADING,
        "Completed" | "Success" | "Succeeded" => user::icon_font::symbols::cod::COD_CHECK,
        "Failed" | "Error" => user::icon_font::symbols::cod::COD_ERROR,
        "Cancelled" | "Canceled" | "Denied" => user::icon_font::symbols::cod::COD_CIRCLE_SLASH,
        _ => user::icon_font::symbols::cod::COD_CIRCLE_SMALL_FILLED,
    };
    format!("{icon} {}", humanize_debug_label(&raw))
}

fn humanize_debug_label(value: &str) -> String {
    let mut output = String::new();
    let mut previous_was_word = false;
    for character in value.chars() {
        if matches!(character, '_' | '-') {
            if !output.ends_with(' ') {
                output.push(' ');
            }
            previous_was_word = false;
            continue;
        }
        let starts_new_word = character.is_ascii_uppercase() && previous_was_word;
        if starts_new_word && !output.ends_with(' ') {
            output.push(' ');
        }
        output.push(character);
        previous_was_word = character.is_ascii_lowercase() || character.is_ascii_digit();
    }
    output
}

fn render_tool_call_content(content: &[agent_client_protocol::ToolCallContent]) -> Vec<String> {
    let mut lines = Vec::new();
    for item in content {
        match item {
            agent_client_protocol::ToolCallContent::Content(content) => {
                if let ContentBlock::Text(text) = &content.content {
                    for line in text.text.lines() {
                        if line.trim().is_empty() {
                            lines.push(String::new());
                        } else {
                            lines.push(format!("  {line}"));
                        }
                    }
                }
            }
            agent_client_protocol::ToolCallContent::Diff(diff) => {
                lines.push(format!(
                    "  {} `{}` · {} -> {}",
                    user::icon_font::symbols::cod::COD_DIFF_MODIFIED,
                    diff.path.display(),
                    diff.old_text.as_ref().map_or("new", |_| "old"),
                    "new"
                ));
            }
            agent_client_protocol::ToolCallContent::Terminal(terminal) => {
                lines.push(format!(
                    "  {} terminal `{}`",
                    user::icon_font::symbols::cod::COD_TERMINAL,
                    terminal.terminal_id
                ));
            }
            _ => {}
        }
    }
    lines
}

fn permission_prompt_lines(request: &RequestPermissionRequest) -> Vec<String> {
    let mut lines = vec![format!(
        "{} Permission requested by agent.",
        user::icon_font::symbols::cod::COD_WARNING
    )];
    if let Some(status) = request.tool_call.fields.status {
        lines.push(format!("  {}", format_acp_status_badge(&status)));
    }
    if let Some(title) = request.tool_call.fields.title.clone() {
        lines.push(format!(
            "{} **{}**",
            user::icon_font::symbols::cod::COD_TOOLS,
            title
        ));
    }
    if let Some(locations) = request.tool_call.fields.locations.as_ref() {
        for location in locations.iter().take(3) {
            let suffix = location
                .line
                .map(|line| format!(":{line}"))
                .unwrap_or_default();
            lines.push(format!(
                "  {} `{}`{suffix}",
                user::icon_font::symbols::cod::COD_FILE,
                location.path.display()
            ));
        }
        if locations.len() > 3 {
            lines.push(format!("  ... {} more location(s)", locations.len() - 3));
        }
    }
    if !request.options.is_empty() {
        lines.push(String::new());
        for option in &request.options {
            lines.push(format!(
                "  - {} ({})",
                option.name,
                format_permission_option_kind(option.kind)
            ));
        }
    }
    lines.push(format!(
        "{} Use `acp.permission-approve` or `acp.permission-deny`.",
        user::icon_font::symbols::cod::COD_CHECKLIST
    ));
    lines
}

fn format_permission_option_kind(kind: PermissionOptionKind) -> &'static str {
    match kind {
        PermissionOptionKind::AllowOnce => "allow once",
        PermissionOptionKind::AllowAlways => "allow always",
        PermissionOptionKind::RejectOnce => "reject once",
        PermissionOptionKind::RejectAlways => "reject always",
        _ => "custom",
    }
}

fn spawn_terminal_reader(
    output: Rc<RefCell<String>>,
    stream: impl tokio::io::AsyncRead + Unpin + 'static,
) {
    tokio::task::spawn_local(async move {
        let mut reader = BufReader::new(stream);
        let mut buffer = [0u8; 4096];
        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(count) => {
                    let chunk = String::from_utf8_lossy(&buffer[..count]);
                    output.borrow_mut().push_str(&chunk);
                }
                Err(_) => break,
            }
        }
    });
}

fn apply_output_limit(output: &str, limit: Option<u64>) -> (String, bool) {
    let Some(limit) = limit else {
        return (output.to_owned(), false);
    };
    let limit = limit as usize;
    if output.len() <= limit {
        return (output.to_owned(), false);
    }
    let mut start = output.len().saturating_sub(limit);
    while start < output.len() && !output.is_char_boundary(start) {
        start += 1;
    }
    (output[start..].to_owned(), true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_client_protocol::{
        AvailableCommandInput, PermissionOptionId, ToolCallLocation, ToolCallStatus,
        ToolCallUpdate, ToolCallUpdateFields, UnstructuredCommandInput,
    };

    #[test]
    fn active_command_input_hint_uses_unstructured_command_metadata() {
        let commands = vec![
            AvailableCommand::new("open", "Open a file").input(
                AvailableCommandInput::Unstructured(UnstructuredCommandInput::new("path to open")),
            ),
            AvailableCommand::new("status", "Show status"),
        ];

        assert_eq!(
            active_command_input_hint(&commands, "/open "),
            Some("path to open".to_owned())
        );
        assert_eq!(
            active_command_input_hint(&commands, "/open src\\main.rs"),
            Some("path to open".to_owned())
        );
        assert_eq!(active_command_input_hint(&commands, "/status"), None);
        assert_eq!(active_command_input_hint(&commands, "hello"), None);
    }

    #[test]
    fn permission_prompt_lines_show_locations_and_choices() {
        let request = RequestPermissionRequest::new(
            "session-1",
            ToolCallUpdate::new(
                "tool-1",
                ToolCallUpdateFields::new()
                    .status(ToolCallStatus::Pending)
                    .title("Read project file")
                    .locations(vec![ToolCallLocation::new("src\\main.rs").line(12u32)]),
            ),
            vec![
                PermissionOption::new(
                    PermissionOptionId::new("allow-once"),
                    "Allow once",
                    PermissionOptionKind::AllowOnce,
                ),
                PermissionOption::new(
                    PermissionOptionId::new("reject-once"),
                    "Reject once",
                    PermissionOptionKind::RejectOnce,
                ),
            ],
        );

        let rendered = permission_prompt_lines(&request).join("\n");
        assert!(rendered.contains("Read project file"));
        assert!(rendered.contains("src\\main.rs:12"));
        assert!(rendered.contains("Allow once (allow once)"));
        assert!(rendered.contains("Reject once (reject once)"));
    }
}
