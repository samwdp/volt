#![allow(unused_imports)]
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    env,
    path::{Path, PathBuf},
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    thread,
};

use agent_client_protocol::{Agent, Client, ClientSideConnection};
use agent_client_protocol::{
    AuthCapabilities, AvailableCommand, ClientCapabilities, ContentBlock, CreateTerminalRequest,
    CreateTerminalResponse, Error, FileSystemCapabilities, ImageContent, Implementation,
    InitializeRequest, KillTerminalRequest, KillTerminalResponse, ListSessionsRequest,
    LoadSessionRequest, Meta, ModelId, ModelInfo, NewSessionRequest, PermissionOption,
    PermissionOptionId, PermissionOptionKind, Plan, ProtocolVersion, ReadTextFileRequest,
    ReadTextFileResponse, ReleaseTerminalRequest, ReleaseTerminalResponse,
    RequestPermissionOutcome, RequestPermissionRequest, RequestPermissionResponse, ResourceLink,
    SelectedPermissionOutcome, SessionConfigId, SessionConfigKind, SessionConfigOption,
    SessionConfigOptionCategory, SessionConfigSelectOption, SessionConfigSelectOptions,
    SessionConfigValueId, SessionInfo, SessionInfoUpdate, SessionMode, SessionModeId,
    SessionModeState, SessionModelState, SessionNotification, SessionUpdate,
    SetSessionConfigOptionRequest, SetSessionModeRequest, SetSessionModelRequest, StopReason,
    TerminalExitStatus, TerminalId, TerminalOutputRequest, TerminalOutputResponse, ToolCall,
    ToolCallUpdate, WaitForTerminalExitRequest, WaitForTerminalExitResponse, WriteTextFileRequest,
    WriteTextFileResponse,
};
use async_trait::async_trait;
use editor_jobs::{ProcessSupervisionMode, supervised_command_if_resolved};
use editor_picker::PickerResultOrder;
use editor_plugin_api::AcpClient as AcpClientConfig;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::Command,
    sync::{mpsc as tokio_mpsc, oneshot},
    task::LocalSet,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::super::*;

#[allow(unused_imports)]
use super::client::*;
#[allow(unused_imports)]
use super::input::*;
#[allow(unused_imports)]
use super::launch::*;
#[allow(unused_imports)]
use super::runtime::*;
#[allow(unused_imports)]
use super::session::*;

pub(crate) fn acp_file_mention_at_cursor(text: &str, cursor: usize) -> Option<FileMention> {
    let chars: Vec<char> = text.chars().collect();
    let cursor = cursor.min(chars.len());
    let mut start = cursor;
    while start > 0 {
        let previous = chars[start - 1];
        if previous.is_whitespace() {
            break;
        }
        start -= 1;
    }
    if start >= chars.len() || chars[start] != '@' {
        return None;
    }
    if start > 0 && !chars[start - 1].is_whitespace() {
        return None;
    }
    let mut end = start + 1;
    while end < chars.len() && !chars[end].is_whitespace() {
        end += 1;
    }
    if cursor < start || cursor > end {
        return None;
    }
    let query_end = cursor.max(start + 1).min(end);
    Some(FileMention {
        at_char: start,
        end_char: end,
        query: chars[start + 1..query_end].iter().collect(),
    })
}

pub(crate) fn acp_file_uri(path: &Path) -> String {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(path))
            .unwrap_or_else(|| path.to_path_buf())
    };
    url::Url::from_file_path(&absolute)
        .map(|url| url.to_string())
        .unwrap_or_else(|_| format!("file://{}", absolute.display()))
}

pub(crate) fn compose_acp_prompt_blocks(
    text: &str,
    workspace_root: Option<&Path>,
    images: &[AcpPastedImage],
) -> Vec<ContentBlock> {
    let chars: Vec<char> = text.chars().collect();
    let mut blocks = Vec::new();
    let mut index = 0usize;
    let mut text_start = 0usize;

    pub(crate) fn flush_text(
        blocks: &mut Vec<ContentBlock>,
        chars: &[char],
        start: usize,
        end: usize,
    ) {
        if start >= end {
            return;
        }
        let text: String = chars[start..end].iter().collect();
        if !text.is_empty() {
            blocks.push(ContentBlock::Text(agent_client_protocol::TextContent::new(
                text,
            )));
        }
    }

    while index < chars.len() {
        if let Some((end, image)) = parse_acp_image_mention(&chars, index, images) {
            flush_text(&mut blocks, &chars, text_start, index);
            blocks.push(ContentBlock::Image(
                ImageContent::new(image.data.clone(), image.mime_type.clone())
                    .uri(format!("volt://agent/pasted-image?name={}", image.name)),
            ));
            index = end;
            text_start = end;
            continue;
        }
        if chars[index] == '@'
            && (index == 0 || chars[index - 1].is_whitespace())
            && let Some(mention) = acp_file_mention_at_cursor(text, index + 1)
        {
            let relative = chars[mention.at_char + 1..mention.end_char]
                .iter()
                .collect::<String>();
            if !relative.is_empty() {
                flush_text(&mut blocks, &chars, text_start, mention.at_char);
                blocks.push(file_mention_content_block(workspace_root, &relative));
                index = mention.end_char;
                text_start = mention.end_char;
                continue;
            }
        }
        index += 1;
    }
    flush_text(&mut blocks, &chars, text_start, chars.len());
    if blocks.is_empty() {
        blocks.push(ContentBlock::from(text.to_owned()));
    }
    blocks
}

pub(crate) fn parse_acp_image_mention<'a>(
    chars: &[char],
    index: usize,
    images: &'a [AcpPastedImage],
) -> Option<(usize, &'a AcpPastedImage)> {
    if index >= chars.len() || chars[index] != '!' {
        return None;
    }
    let rest: String = chars[index..].iter().collect();
    let rest = rest.strip_prefix("![")?;
    let (name, rest) = rest.split_once("](acp-image:")?;
    let (id_text, _rest) = rest.split_once(')')?;
    let id = id_text.parse::<u64>().ok()?;
    let image = images.iter().find(|image| image.id == id)?;
    let consumed =
        2 + name.chars().count() + "](acp-image:".chars().count() + id_text.chars().count() + 1;
    Some((index + consumed, image))
}

pub(crate) fn file_mention_content_block(
    workspace_root: Option<&Path>,
    relative: &str,
) -> ContentBlock {
    let path = workspace_root
        .map(|root| root.join(relative))
        .unwrap_or_else(|| PathBuf::from(relative));
    if is_image_path(&path)
        && let Some(image) = clipboard_image_from_path(&path)
    {
        return ContentBlock::Image(
            ImageContent::new(
                base64::engine::general_purpose::STANDARD.encode(image.bytes),
                image.mime_type,
            )
            .uri(acp_file_uri(&path)),
        );
    }
    ContentBlock::ResourceLink(ResourceLink::new(relative.to_owned(), acp_file_uri(&path)))
}

pub(crate) fn pending_slash_completion_trigger(
    buffer: &ShellBuffer,
    pending: PendingSlashTrigger,
) -> Option<CompletionTrigger> {
    let input = buffer.input_field()?;
    let text = input.text();
    match pending {
        PendingSlashTrigger::Auto => Some(CompletionTrigger::Auto(
            acp_slash_completion_query(text)?.to_owned(),
        )),
        PendingSlashTrigger::Manual => {
            if text.is_empty() || acp_slash_completion_query(text).is_some() {
                Some(CompletionTrigger::Manual)
            } else {
                None
            }
        }
    }
}

pub(crate) fn handle_acp_ui_action(
    runtime: &mut EditorRuntime,
    action: AcpUiAction,
) -> Result<(), String> {
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

pub(crate) fn open_slash_command_picker(
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
    let options = commands
        .into_iter()
        .map(|command| {
            let mut detail = command.description.clone();
            if let Some(agent_client_protocol::AvailableCommandInput::Unstructured(input)) =
                command.input.as_ref()
            {
                detail.push_str(&format!(" | {}", input.hint));
            }
            AcpPickerOption::new(command.name.clone(), format!("/{}", command.name))
                .with_detail(detail)
        })
        .collect();
    let context = AcpPickerContext::new(AcpPickerKind::SlashCommands, "ACP Slash Commands")
        .with_options(options);
    let entries = acp_picker_entries(runtime, buffer_id, &context);
    let mut picker = PickerOverlay::from_entries("ACP Slash Commands", entries);
    match trigger {
        CompletionTrigger::Auto(query) => {
            if !query.is_empty() {
                picker.append_query(&query);
            }
        }
        CompletionTrigger::Manual => {}
    }
    shell_ui_mut(runtime)?.set_picker(picker.with_kind(PickerKind::AcpSlash { buffer_id }));
    Ok(())
}

pub(crate) fn open_file_mention_picker(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    trigger: CompletionTrigger,
) -> Result<(), String> {
    if shell_ui(runtime)?.picker_visible() {
        return Ok(());
    }
    let root = git_root(runtime)
        .ok()
        .or_else(|| active_workspace_root(runtime).ok().flatten());
    let Some(root) = root else {
        let entries = vec![PickerEntry {
            item: PickerItem::new(
                "acp-files-no-root",
                "Workspace has no project root",
                "Open a project workspace before linking git files.",
                None::<String>,
            ),
            action: PickerAction::NoOp,
            quickfix: None,
        }];
        let picker = PickerOverlay::from_entries("ACP Files", entries);
        shell_ui_mut(runtime)?.set_picker(picker.with_kind(PickerKind::AcpFile { buffer_id }));
        return Ok(());
    };
    let entries = match list_repository_files(&root) {
        Ok(files) if files.is_empty() => vec![PickerEntry {
            item: PickerItem::new(
                "acp-files-empty",
                "No visible files found",
                "Git did not report any tracked or unignored files.",
                None::<String>,
            ),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
        Ok(files) => files
            .into_iter()
            .map(|relative_path| {
                let absolute = root.join(&relative_path);
                let label = relative_path.display().to_string();
                PickerEntry {
                    item: PickerItem::new(label.clone(), label.clone(), "", None::<String>)
                        .with_search_text(label.clone())
                        .with_fringe(editor_icons::seti_file_icon(&absolute)),
                    action: PickerAction::AcpInsertFileMention {
                        buffer_id,
                        relative_path: label,
                    },
                    quickfix: None,
                }
            })
            .collect(),
        Err(error) => vec![PickerEntry {
            item: PickerItem::new(
                "acp-files-error",
                "Unable to read git files",
                error.to_string(),
                None::<String>,
            ),
            action: PickerAction::NoOp,
            quickfix: None,
        }],
    };
    let mut picker = PickerOverlay::from_entries("ACP Files", entries);
    match trigger {
        CompletionTrigger::Auto(query) => {
            if !query.is_empty() {
                picker.append_query(&query);
            }
        }
        CompletionTrigger::Manual => {}
    }
    shell_ui_mut(runtime)?.set_picker(picker.with_kind(PickerKind::AcpFile { buffer_id }));
    Ok(())
}

pub(crate) struct AcpManager {
    pub(crate) runtime: AcpRuntime,
    pub(crate) events: mpsc::Receiver<AcpEvent>,
    pub(crate) deferred_events: VecDeque<AcpEvent>,
    pub(crate) sessions: HashMap<agent_client_protocol::SessionId, AcpSessionInfo>,
    pub(crate) buffers: HashMap<BufferId, agent_client_protocol::SessionId>,
    pub(crate) workspace_client_buffers: HashMap<(WorkspaceId, String), BufferId>,
    pub(crate) pending_clients: HashMap<BufferId, PendingAcpClient>,
    pub(crate) pending_slash: HashMap<BufferId, PendingSlashTrigger>,
    pub(crate) pending_ui_actions: Vec<AcpUiAction>,
    pub(crate) pending_permissions: VecDeque<AcpPendingPermissionUi>,
    pub(crate) active_permission_request: Option<u64>,
    pub(crate) permission_queue_paused: bool,
}

impl AcpManager {
    pub(crate) fn new() -> Result<Self, String> {
        let (event_tx, event_rx) = mpsc::channel();
        let runtime = AcpRuntime::new(event_tx)?;
        Ok(Self {
            runtime,
            events: event_rx,
            deferred_events: VecDeque::new(),
            sessions: HashMap::new(),
            buffers: HashMap::new(),
            workspace_client_buffers: HashMap::new(),
            pending_clients: HashMap::new(),
            pending_slash: HashMap::new(),
            pending_ui_actions: Vec::new(),
            pending_permissions: VecDeque::new(),
            active_permission_request: None,
            permission_queue_paused: false,
        })
    }

    pub(crate) fn buffer_for_client(
        &self,
        workspace_id: WorkspaceId,
        client_id: &str,
    ) -> Option<BufferId> {
        self.workspace_client_buffers
            .get(&(workspace_id, client_id.to_owned()))
            .copied()
    }

    pub(crate) fn clear_workspace_client_buffer(
        &mut self,
        workspace_id: WorkspaceId,
        client_id: &str,
        buffer_id: BufferId,
    ) {
        let key = (workspace_id, client_id.to_owned());
        if self.workspace_client_buffers.get(&key) == Some(&buffer_id) {
            self.workspace_client_buffers.remove(&key);
        }
    }

    pub(crate) fn buffer_for_session(
        &self,
        session_id: &agent_client_protocol::SessionId,
    ) -> Option<BufferId> {
        self.sessions
            .get(session_id)
            .map(|session| session.buffer_id)
    }

    pub(crate) fn session_for_buffer(
        &self,
        buffer_id: BufferId,
    ) -> Option<agent_client_protocol::SessionId> {
        self.buffers.get(&buffer_id).cloned()
    }

    pub(crate) fn client_id_for_buffer(&self, buffer_id: BufferId) -> Option<String> {
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

    pub(crate) fn available_commands_for_buffer(
        &self,
        buffer_id: BufferId,
    ) -> Option<Vec<AvailableCommand>> {
        let session_id = self.session_for_buffer(buffer_id)?;
        self.sessions
            .get(&session_id)
            .map(|session| session.available_commands.clone())
    }

    pub(crate) fn mode_state_for_buffer(&self, buffer_id: BufferId) -> Option<SessionModeState> {
        let session_id = self.session_for_buffer(buffer_id)?;
        self.sessions
            .get(&session_id)
            .and_then(|session| session.mode_state.clone())
    }

    pub(crate) fn model_state_for_buffer(&self, buffer_id: BufferId) -> Option<SessionModelState> {
        let session_id = self.session_for_buffer(buffer_id)?;
        self.sessions
            .get(&session_id)
            .and_then(|session| session.model_state.clone())
    }

    pub(crate) fn has_sessions(&self) -> bool {
        !self.sessions.is_empty()
    }

    pub(crate) fn queue_slash_completion(
        &mut self,
        buffer_id: BufferId,
        trigger: PendingSlashTrigger,
    ) {
        self.pending_slash.insert(buffer_id, trigger);
    }

    pub(crate) fn take_pending_ui_actions(&mut self) -> Vec<AcpUiAction> {
        std::mem::take(&mut self.pending_ui_actions)
    }

    pub(crate) fn close_buffer(&mut self, buffer_id: BufferId) {
        if let Some(pending) = self.pending_clients.remove(&buffer_id) {
            self.clear_workspace_client_buffer(pending.workspace_id, &pending.client_id, buffer_id);
        }
        self.pending_slash.remove(&buffer_id);
        self.pending_ui_actions.retain(|action| {
            !matches!(
                action,
                AcpUiAction::OpenSlashCompletion {
                    buffer_id: action_buffer_id,
                    ..
                } if *action_buffer_id == buffer_id
            )
        });
        if let Some(session_id) = self.buffers.remove(&buffer_id) {
            if let Some(session) = self.sessions.remove(&session_id) {
                self.clear_workspace_client_buffer(
                    session.workspace_id,
                    &session.client_id,
                    buffer_id,
                );
            }
            self.disconnect(session_id);
        }
    }

    pub(crate) fn rebind_session_id(
        &mut self,
        buffer_id: BufferId,
        old_session_id: &agent_client_protocol::SessionId,
        new_session_id: agent_client_protocol::SessionId,
    ) {
        if old_session_id == &new_session_id {
            self.buffers.insert(buffer_id, new_session_id);
            return;
        }
        if let Some(session) = self.sessions.remove(old_session_id) {
            self.sessions.insert(new_session_id.clone(), session);
        }
        self.buffers.insert(buffer_id, new_session_id);
    }

    pub(crate) fn connect(
        &mut self,
        client: AcpClientConfig,
        workspace_root: PathBuf,
        workspace_id: WorkspaceId,
        buffer_id: BufferId,
        load_session: Option<PendingAcpLoadSession>,
        workspace_name: String,
    ) -> Result<(), String> {
        let client_id = client.id.clone();
        self.pending_clients.insert(
            buffer_id,
            PendingAcpClient {
                client_id: client_id.clone(),
                load_session,
                workspace_root: workspace_root.clone(),
                workspace_id,
                workspace_name,
            },
        );
        self.workspace_client_buffers
            .insert((workspace_id, client_id.clone()), buffer_id);
        if let Err(error) = self.runtime.send(AcpCommand::Connect {
            config: client,
            workspace_root,
            buffer_id,
        }) {
            self.pending_clients.remove(&buffer_id);
            self.clear_workspace_client_buffer(workspace_id, &client_id, buffer_id);
            return Err(error);
        }
        Ok(())
    }

    pub(crate) fn prompt(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        prompt: Vec<ContentBlock>,
    ) -> Result<(), String> {
        self.runtime.send(AcpCommand::Prompt { session_id, prompt })
    }

    pub(crate) fn list_sessions(
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

    pub(crate) fn load_session(
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

    pub(crate) fn set_mode(
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

    pub(crate) fn set_model(
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

    pub(crate) fn disconnect(&mut self, session_id: agent_client_protocol::SessionId) {
        let _ = self.runtime.send(AcpCommand::Disconnect { session_id });
    }

    pub(crate) fn permission_request_for_session(
        &self,
        session_id: &agent_client_protocol::SessionId,
    ) -> Option<u64> {
        self.pending_permissions
            .iter()
            .find(|pending| pending.session_id == *session_id)
            .map(|pending| pending.request_id)
    }

    pub(crate) fn resolve_permission(&mut self, request_id: u64, decision: PermissionDecision) {
        let _ = self.runtime.send(AcpCommand::ResolvePermission {
            request_id,
            decision,
        });
    }

    pub(crate) fn resolve_permission_option(
        &mut self,
        request_id: u64,
        option_id: agent_client_protocol::PermissionOptionId,
    ) {
        let _ = self.runtime.send(AcpCommand::ResolvePermissionOption {
            request_id,
            option_id,
        });
    }

    pub(crate) fn queue_permission_request(&mut self, request: AcpPendingPermissionUi) {
        self.pending_permissions.push_back(request);
    }

    pub(crate) fn open_permission_request(
        &mut self,
        runtime: &mut EditorRuntime,
        request_id: u64,
    ) -> Result<(), String> {
        let Some(index) = self
            .pending_permissions
            .iter()
            .position(|pending| pending.request_id == request_id)
        else {
            return Ok(());
        };
        let Some(request) = self.pending_permissions.remove(index) else {
            return Ok(());
        };
        self.pending_permissions.push_front(request.clone());
        open_permission_picker(runtime, &request)?;
        self.active_permission_request = Some(request_id);
        self.permission_queue_paused = false;
        Ok(())
    }

    pub(crate) fn maybe_open_next_permission_request(
        &mut self,
        runtime: &mut EditorRuntime,
    ) -> Result<(), String> {
        if self.active_permission_request.is_some()
            || self.permission_queue_paused
            || shell_ui(runtime)?.picker_visible()
        {
            return Ok(());
        }
        let Some(request_id) = self
            .pending_permissions
            .front()
            .map(|pending| pending.request_id)
        else {
            return Ok(());
        };
        self.open_permission_request(runtime, request_id)
    }

    pub(crate) fn remove_permission_request(
        &mut self,
        request_id: u64,
    ) -> Option<AcpPendingPermissionUi> {
        let index = self
            .pending_permissions
            .iter()
            .position(|pending| pending.request_id == request_id)?;
        self.pending_permissions.remove(index)
    }

    pub(crate) fn remove_permission_requests_for_session(
        &mut self,
        session_id: &agent_client_protocol::SessionId,
    ) -> Vec<AcpPendingPermissionUi> {
        let mut removed = Vec::new();
        let mut index = 0;
        while index < self.pending_permissions.len() {
            if self.pending_permissions[index].session_id == *session_id {
                if let Some(request) = self.pending_permissions.remove(index) {
                    removed.push(request);
                }
            } else {
                index += 1;
            }
        }
        removed
    }

    pub(crate) fn permission_picker_closed(&mut self, request_id: u64) {
        if self.active_permission_request == Some(request_id) {
            self.active_permission_request = None;
            self.permission_queue_paused = true;
        }
    }

    pub(crate) fn permission_request_resolved(
        &mut self,
        runtime: &mut EditorRuntime,
        request_id: u64,
    ) -> Result<(), String> {
        let _ = self.remove_permission_request(request_id);
        if self.active_permission_request == Some(request_id) {
            self.active_permission_request = None;
        }
        if self.pending_permissions.is_empty() {
            self.permission_queue_paused = false;
        }
        self.maybe_open_next_permission_request(runtime)
    }

    pub(crate) fn dismiss_permission_picker_for_requests(
        &mut self,
        runtime: &mut EditorRuntime,
        request_ids: &[u64],
    ) -> Result<(), String> {
        if request_ids.is_empty() {
            return Ok(());
        }
        if let Some(PickerKind::AcpPermission { request_id }) = shell_ui(runtime)?.picker_kind()
            && request_ids.contains(&request_id)
        {
            shell_ui_mut(runtime)?.close_picker();
        }
        if request_ids
            .iter()
            .any(|request_id| self.active_permission_request == Some(*request_id))
        {
            self.active_permission_request = None;
        }
        if self.pending_permissions.is_empty() {
            self.permission_queue_paused = false;
        }
        self.maybe_open_next_permission_request(runtime)
    }

    pub(crate) fn drain_events(&mut self, runtime: &mut EditorRuntime) -> Result<bool, String> {
        let mut events = Vec::new();
        while events.len() < ACP_EVENT_DRAIN_LIMIT {
            let Some(event) = self.deferred_events.pop_front() else {
                break;
            };
            events.push(event);
        }
        if events.len() < ACP_EVENT_DRAIN_LIMIT {
            events.extend(drain_acp_event_batch(
                &self.events,
                ACP_EVENT_DRAIN_LIMIT - events.len(),
            ));
        }
        let events = coalesce_acp_events(events);
        let (events, deferred) = split_acp_events_for_render(events);
        self.deferred_events.extend(deferred);
        let changed = !events.is_empty();
        for event in events {
            self.handle_event(runtime, event)?;
        }
        Ok(changed)
    }

    pub(crate) fn handle_event(
        &mut self,
        runtime: &mut EditorRuntime,
        event: AcpEvent,
    ) -> Result<(), String> {
        match event {
            AcpEvent::Connected {
                buffer_id,
                client_id,
                session_id,
                modes,
                models,
            } => {
                let Some(pending) = self.pending_clients.remove(&buffer_id) else {
                    self.disconnect(session_id);
                    return Ok(());
                };
                self.buffers.insert(buffer_id, session_id.clone());
                self.sessions.insert(
                    session_id.clone(),
                    AcpSessionInfo {
                        client_id: client_id.clone(),
                        buffer_id,
                        workspace_id: pending.workspace_id,
                        workspace_name: pending.workspace_name.clone(),
                        title: pending
                            .load_session
                            .as_ref()
                            .and_then(|load| load.title.clone()),
                        available_commands: Vec::new(),
                        mode_state: modes,
                        model_state: models,
                        config_options: Vec::new(),
                        mode_config_id: None,
                        model_config_id: None,
                    },
                );

                let label = shell_user_library(runtime)
                    .acp_client_by_id(&client_id)
                    .map(|client| client.label)
                    .unwrap_or_else(|| "ACP".to_owned());

                if let Some(load_session) = pending.load_session {
                    let target_session_id = load_session.session_id;
                    // Bind the loaded id before session/load replay so history updates apply.
                    self.rebind_session_id(buffer_id, &session_id, target_session_id.clone());
                    let display_title = load_session
                        .title
                        .clone()
                        .unwrap_or_else(|| target_session_id.to_string());
                    set_acp_buffer_name(
                        runtime,
                        pending.workspace_id,
                        buffer_id,
                        acp_session_buffer_name(&display_title),
                    );
                    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                        buffer.acp_prepare_session_replay(label.as_str());
                        buffer.acp_set_session_title(load_session.title.clone());
                    }
                    if let Some(session) = self.sessions.get(&target_session_id) {
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
                    self.load_session(
                        session_id,
                        buffer_id,
                        target_session_id,
                        pending.workspace_root,
                    )?;
                } else {
                    let buffer_name = format!("*acp {} [{}]*", client_id, session_id);
                    set_acp_buffer_name(runtime, pending.workspace_id, buffer_id, buffer_name);
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
                }
            }
            AcpEvent::ClientFailed { buffer_id, message } => {
                if let Some(pending) = self.pending_clients.remove(&buffer_id) {
                    self.clear_workspace_client_buffer(
                        pending.workspace_id,
                        &pending.client_id,
                        buffer_id,
                    );
                }
                let follow = {
                    let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                        return Ok(());
                    };
                    buffer.acp_push_system_message(message)
                };
                refresh_acp_output_markdown(runtime, buffer_id, follow)?;
            }
            AcpEvent::ClientLog { buffer_id, message } => {
                let follow = {
                    let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                        return Ok(());
                    };
                    buffer.acp_push_system_message(message)
                };
                refresh_acp_output_markdown(runtime, buffer_id, follow)?;
            }
            AcpEvent::SessionUserPrompt { session_id, prompt } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                {
                    let follow = {
                        let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                            return Ok(());
                        };
                        buffer.acp_push_user_prompt(prompt)
                    };
                    refresh_acp_output_markdown(runtime, buffer_id, follow)?;
                }
            }
            AcpEvent::SessionAgentChunk {
                session_id,
                content,
            } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                {
                    let follow = {
                        let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                            return Ok(());
                        };
                        buffer.acp_append_agent_chunk(content)
                    };
                    refresh_acp_output_markdown(runtime, buffer_id, follow)?;
                }
            }
            AcpEvent::SessionPlan { session_id, plan } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                    && let Ok(buffer) = shell_buffer_mut(runtime, buffer_id)
                {
                    buffer.acp_set_plan(plan);
                }
            }
            AcpEvent::SessionToolCall {
                session_id,
                tool_call,
            } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                {
                    let follow = {
                        let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                            return Ok(());
                        };
                        buffer.acp_upsert_tool_call(tool_call)
                    };
                    refresh_acp_output_markdown(runtime, buffer_id, follow)?;
                }
            }
            AcpEvent::SessionToolCallUpdate { session_id, update } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                {
                    let follow = {
                        let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                            return Ok(());
                        };
                        buffer.acp_update_tool_call(update)
                    };
                    refresh_acp_output_markdown(runtime, buffer_id, follow)?;
                }
            }
            AcpEvent::SessionInfoUpdated { session_id, update } => {
                if let Some(session) = self.sessions.get_mut(&session_id) {
                    if let agent_client_protocol::MaybeUndefined::Value(title) = &update.title {
                        session.title = Some(title.clone());
                    } else if matches!(update.title, agent_client_protocol::MaybeUndefined::Null) {
                        session.title = None;
                    }
                    let workspace_id = session.workspace_id;
                    let buffer_id = session.buffer_id;
                    let renamed = session.title.clone().map(|title| {
                        (
                            buffer_id,
                            workspace_id,
                            acp_session_buffer_name(&title),
                            title,
                        )
                    });
                    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                        buffer.acp_set_session_info(&update);
                    }
                    if let Some((buffer_id, workspace_id, name, _)) = renamed {
                        set_acp_buffer_name(runtime, workspace_id, buffer_id, name);
                    }
                }
            }
            AcpEvent::PermissionRequested {
                request_id,
                session_id,
                tool_call,
                options,
            } => {
                let Some(session) = self.sessions.get(&session_id) else {
                    return Ok(());
                };
                let buffer_id = session.buffer_id;
                let workspace_name = session.workspace_name.clone();
                let follow = {
                    let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                        return Ok(());
                    };
                    buffer.acp_update_tool_call(tool_call.clone())
                };
                refresh_acp_output_markdown(runtime, buffer_id, follow)?;
                let request = AcpPendingPermissionUi {
                    request_id,
                    session_id: session_id.clone(),
                    workspace_name,
                    tool_call,
                    options,
                };
                apply_acp_notification(
                    runtime,
                    request.notification_key(),
                    NotificationSeverity::Warning,
                    request.notification_title(),
                    request
                        .options
                        .iter()
                        .map(|option| option.name.clone())
                        .collect(),
                    true,
                    Some(NotificationAction::OpenAcpPermissionPicker { request_id }),
                )?;
                self.queue_permission_request(request);
                self.maybe_open_next_permission_request(runtime)?;
            }
            AcpEvent::PermissionResolved {
                request_id,
                session_id,
                message,
            } => {
                if let Some(session) = self.sessions.get(&session_id) {
                    apply_acp_notification(
                        runtime,
                        format!("acp.permission.{request_id}"),
                        NotificationSeverity::Info,
                        format!("{} permission resolved", session.workspace_name),
                        vec![message],
                        false,
                        None,
                    )?;
                }
                self.permission_request_resolved(runtime, request_id)?;
            }
            AcpEvent::SessionFinished { session_id } => {
                if let Some(session) = self.sessions.get(&session_id) {
                    if let Ok(buffer) = shell_buffer_mut(runtime, session.buffer_id) {
                        buffer.acp_complete_plan();
                    }
                    let title = session
                        .title
                        .clone()
                        .unwrap_or_else(|| format!("Session {session_id}"));
                    apply_acp_notification(
                        runtime,
                        format!("acp.end-turn.{session_id}"),
                        NotificationSeverity::Success,
                        format!("{} {} has finished", session.workspace_name, title),
                        Vec::new(),
                        false,
                        None,
                    )?;
                }
            }
            AcpEvent::SessionLines { session_id, lines } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                {
                    let follow = {
                        let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) else {
                            return Ok(());
                        };
                        let mut follow = true;
                        for line in lines {
                            follow = buffer.acp_push_system_message(line);
                        }
                        follow
                    };
                    refresh_acp_output_markdown(runtime, buffer_id, follow)?;
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
                let options = sessions
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
                        let is_current = current_session
                            .as_ref()
                            .is_some_and(|current| *current == session.session_id);
                        AcpPickerOption::new(session.session_id.to_string(), title)
                            .with_detail(detail)
                            .with_current(is_current)
                    })
                    .collect();
                let context = AcpPickerContext::new(AcpPickerKind::Sessions, "ACP Sessions")
                    .with_options(options);
                let entries = acp_picker_entries(runtime, buffer_id, &context);
                let picker = PickerOverlay::from_entries("ACP Sessions", entries)
                    .with_result_order(PickerResultOrder::Source);
                shell_ui_mut(runtime)?.set_picker(picker);
            }
            AcpEvent::SessionLoaded {
                buffer_id,
                old_session_id,
                new_session_id,
                modes,
                models,
            } => {
                // May already be rebound to new_session_id before session/load replay.
                let Some(mut session) = self
                    .sessions
                    .remove(&new_session_id)
                    .or_else(|| self.sessions.remove(&old_session_id))
                else {
                    return Ok(());
                };
                if modes.is_some() {
                    session.mode_state = modes;
                }
                if models.is_some() {
                    session.model_state = models;
                }
                let workspace_id = session.workspace_id;
                let title = session.title.clone();
                self.buffers.insert(buffer_id, new_session_id.clone());
                self.sessions.insert(new_session_id.clone(), session);

                if let Some(title) = title.as_deref() {
                    set_acp_buffer_name(
                        runtime,
                        workspace_id,
                        buffer_id,
                        acp_session_buffer_name(title),
                    );
                } else {
                    set_acp_buffer_name(
                        runtime,
                        workspace_id,
                        buffer_id,
                        acp_session_buffer_name(&new_session_id.to_string()),
                    );
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
                    let removed_requests = self.remove_permission_requests_for_session(&session_id);
                    let removed_request_ids = removed_requests
                        .iter()
                        .map(|request| request.request_id)
                        .collect::<Vec<_>>();
                    self.buffers.remove(&session.buffer_id);
                    self.clear_workspace_client_buffer(
                        session.workspace_id,
                        &session.client_id,
                        session.buffer_id,
                    );
                    self.pending_slash.remove(&session.buffer_id);
                    update_acp_input_hint(runtime, session.buffer_id, None, None, &[]);
                    self.dismiss_permission_picker_for_requests(runtime, &removed_request_ids)?;
                    for request_id in removed_request_ids {
                        apply_acp_notification(
                            runtime,
                            format!("acp.permission.{request_id}"),
                            NotificationSeverity::Info,
                            format!("{} ACP session disconnected", session.workspace_name),
                            vec![message.clone()],
                            false,
                            None,
                        )?;
                    }
                    apply_acp_notification(
                        runtime,
                        format!("acp.disconnect.{session_id}"),
                        NotificationSeverity::Info,
                        format!("{} ACP session disconnected", session.workspace_name),
                        vec![message],
                        false,
                        None,
                    )?;
                }
            }
        }
        Ok(())
    }
}
