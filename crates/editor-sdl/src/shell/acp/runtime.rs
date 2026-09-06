use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc,
    thread,
};

use agent_client_protocol::{Agent, ClientSideConnection};
use agent_client_protocol::{
    AuthCapabilities, AvailableCommand, ClientCapabilities, ContentBlock, FileSystemCapabilities,
    Implementation, InitializeRequest, ListSessionsRequest, LoadSessionRequest, Meta, ModelId,
    NewSessionRequest, PermissionOption, PermissionOptionId, PermissionOptionKind, Plan,
    ProtocolVersion, RequestPermissionOutcome, SelectedPermissionOutcome, SessionConfigId,
    SessionConfigOption, SessionConfigValueId, SessionInfo, SessionInfoUpdate, SessionModeId,
    SessionModeState, SessionModelState, SetSessionConfigOptionRequest, SetSessionModeRequest,
    SetSessionModelRequest, StopReason, TerminalExitStatus, TerminalId, ToolCall, ToolCallUpdate,
};
use editor_plugin_api::AcpClient as AcpClientConfig;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    sync::{mpsc as tokio_mpsc, oneshot},
    task::LocalSet,
};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

use super::super::*;

use super::client::*;
use super::launch::*;

pub(crate) struct PendingAcpClient {
    pub(crate) client_id: String,
    pub(crate) load_session: Option<PendingAcpLoadSession>,
    pub(crate) workspace_root: PathBuf,
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) workspace_name: String,
}

pub(crate) struct PendingAcpLoadSession {
    pub(crate) session_id: agent_client_protocol::SessionId,
    pub(crate) title: Option<String>,
}

pub(crate) struct AcpSessionInfo {
    pub(crate) client_id: String,
    pub(crate) buffer_id: BufferId,
    pub(crate) workspace_id: WorkspaceId,
    pub(crate) workspace_name: String,
    pub(crate) title: Option<String>,
    pub(crate) available_commands: Vec<AvailableCommand>,
    pub(crate) mode_state: Option<SessionModeState>,
    pub(crate) model_state: Option<SessionModelState>,
    pub(crate) config_options: Vec<SessionConfigOption>,
    pub(crate) mode_config_id: Option<SessionConfigId>,
    pub(crate) model_config_id: Option<SessionConfigId>,
}

#[derive(Debug, Clone)]
pub(crate) struct AcpPendingPermissionUi {
    pub(crate) request_id: u64,
    pub(crate) session_id: agent_client_protocol::SessionId,
    pub(crate) workspace_name: String,
    pub(crate) tool_call: ToolCallUpdate,
    pub(crate) options: Vec<PermissionOption>,
}

impl AcpPendingPermissionUi {
    pub(crate) fn title(&self) -> String {
        self.tool_call
            .fields
            .title
            .clone()
            .unwrap_or_else(|| "Tool".to_owned())
    }

    pub(crate) fn notification_key(&self) -> String {
        format!("acp.permission.{}", self.request_id)
    }

    pub(crate) fn notification_title(&self) -> String {
        format!(
            "{} {} is requesting permission",
            self.workspace_name,
            self.title()
        )
    }
}

pub(crate) enum AcpEvent {
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
    SessionUserPrompt {
        session_id: agent_client_protocol::SessionId,
        prompt: String,
    },
    SessionAgentChunk {
        session_id: agent_client_protocol::SessionId,
        content: ContentBlock,
    },
    SessionPlan {
        session_id: agent_client_protocol::SessionId,
        plan: Plan,
    },
    SessionToolCall {
        session_id: agent_client_protocol::SessionId,
        tool_call: ToolCall,
    },
    SessionToolCallUpdate {
        session_id: agent_client_protocol::SessionId,
        update: ToolCallUpdate,
    },
    SessionInfoUpdated {
        session_id: agent_client_protocol::SessionId,
        update: SessionInfoUpdate,
    },
    PermissionRequested {
        request_id: u64,
        session_id: agent_client_protocol::SessionId,
        tool_call: ToolCallUpdate,
        options: Vec<PermissionOption>,
    },
    PermissionResolved {
        request_id: u64,
        session_id: agent_client_protocol::SessionId,
        message: String,
    },
    SessionFinished {
        session_id: agent_client_protocol::SessionId,
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

pub(crate) fn refresh_acp_output_markdown(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    follow_output: bool,
) -> Result<(), String> {
    super::super::rebuild_acp_output_markdown(runtime, buffer_id, follow_output)
}

pub(crate) fn acp_session_buffer_name(session_title: &str) -> String {
    format!("*acp [{session_title}]*")
}

pub(crate) fn set_acp_buffer_name(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
    buffer_id: BufferId,
    name: String,
) {
    let _ = runtime
        .model_mut()
        .set_buffer_name(workspace_id, buffer_id, name.clone());
    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
        buffer.name = name;
    }
}

pub(crate) fn drain_acp_event_batch(
    events: &mpsc::Receiver<AcpEvent>,
    limit: usize,
) -> Vec<AcpEvent> {
    let mut batch = Vec::new();
    for _ in 0..limit {
        match events.try_recv() {
            Ok(event) => batch.push(event),
            Err(mpsc::TryRecvError::Empty | mpsc::TryRecvError::Disconnected) => break,
        }
    }
    batch
}

pub(crate) fn coalesce_acp_events(events: Vec<AcpEvent>) -> Vec<AcpEvent> {
    let mut coalesced = Vec::with_capacity(events.len());
    for event in events {
        match event {
            AcpEvent::SessionAgentChunk {
                session_id,
                content: ContentBlock::Text(text),
            } => {
                if let Some(AcpEvent::SessionAgentChunk {
                    session_id: last_session_id,
                    content: ContentBlock::Text(last_text),
                }) = coalesced.last_mut()
                    && last_session_id == &session_id
                {
                    last_text.text.push_str(&text.text);
                    continue;
                }
                coalesced.push(AcpEvent::SessionAgentChunk {
                    session_id,
                    content: ContentBlock::Text(text),
                });
            }
            event => coalesced.push(event),
        }
    }
    coalesced
}

pub(crate) fn throttled_acp_session_id(
    event: &AcpEvent,
) -> Option<&agent_client_protocol::SessionId> {
    match event {
        AcpEvent::SessionPlan { session_id, .. } | AcpEvent::SessionFinished { session_id } => {
            Some(session_id)
        }
        _ => None,
    }
}

pub(crate) fn split_acp_events_for_render(
    events: Vec<AcpEvent>,
) -> (Vec<AcpEvent>, VecDeque<AcpEvent>) {
    let mut throttled_sessions = HashSet::new();
    let mut ready = Vec::new();
    let mut deferred = VecDeque::new();
    for event in events {
        let Some(session_id) = throttled_acp_session_id(&event) else {
            ready.push(event);
            continue;
        };
        if !throttled_sessions.insert(session_id.clone()) {
            deferred.push_back(event);
            continue;
        }
        ready.push(event);
    }
    (ready, deferred)
}

pub(crate) enum AcpCommand {
    Connect {
        config: AcpClientConfig,
        workspace_root: PathBuf,
        buffer_id: BufferId,
    },
    Prompt {
        session_id: agent_client_protocol::SessionId,
        prompt: Vec<ContentBlock>,
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
        request_id: u64,
        decision: PermissionDecision,
    },
    ResolvePermissionOption {
        request_id: u64,
        option_id: PermissionOptionId,
    },
}

#[derive(Clone, Copy)]
pub(crate) enum PermissionDecision {
    Approve,
    Deny,
}

pub(crate) struct AcpRuntime {
    pub(crate) sender: tokio_mpsc::UnboundedSender<AcpCommand>,
}

impl AcpRuntime {
    pub(crate) fn new(event_tx: mpsc::Sender<AcpEvent>) -> Result<Self, String> {
        let (sender, receiver) = tokio_mpsc::unbounded_channel();
        thread::spawn(move || run_acp_runtime(receiver, event_tx));
        Ok(Self { sender })
    }

    pub(crate) fn send(&self, command: AcpCommand) -> Result<(), String> {
        self.sender
            .send(command)
            .map_err(|_| "ACP runtime is not running".to_owned())
    }
}

pub(crate) fn run_acp_runtime(
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

pub(crate) async fn acp_runtime_loop(
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
                request_id,
                decision,
            } => {
                resolve_permission_response(state.clone(), request_id, decision);
            }
            AcpCommand::ResolvePermissionOption {
                request_id,
                option_id,
            } => {
                resolve_permission_option(state.clone(), request_id, option_id);
            }
        }
    }
}

pub(crate) async fn connect_acp_client(
    state: Rc<RefCell<AcpRuntimeState>>,
    config: AcpClientConfig,
    workspace_root: PathBuf,
    buffer_id: BufferId,
) -> Result<(), String> {
    let cwd = config
        .cwd
        .as_deref()
        .map(Path::new)
        .unwrap_or(workspace_root.as_path());
    let mut child = spawn_background_command(
        &config.command,
        &config.args,
        Some(cwd),
        &config.env,
        BackgroundCommandPipes::ACP_CLIENT,
    )
    .await
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

    let capability_meta = Meta::from_iter([
        ("terminal_output".into(), true.into()),
        ("terminal-auth".into(), true.into()),
        ("parameterizedModelPicker".into(), true.into()),
    ]);
    let capabilities = ClientCapabilities::new()
        .fs(FileSystemCapabilities::new()
            .read_text_file(true)
            .write_text_file(true))
        .terminal(true)
        .auth(AuthCapabilities::new().terminal(true))
        .meta(capability_meta);
    let init_request = InitializeRequest::new(ProtocolVersion::LATEST)
        .client_capabilities(capabilities)
        .client_info(
            Implementation::new("volt", core::env!("CARGO_PKG_VERSION")).title("Volt SDL shell"),
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
    state.borrow().emit(AcpEvent::Connected {
        buffer_id,
        client_id: config.id,
        session_id,
        modes,
        models,
    });
    Ok(())
}

pub(crate) async fn send_acp_prompt(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    prompt: Vec<ContentBlock>,
) -> Result<(), String> {
    let connection = {
        state
            .borrow()
            .sessions
            .get(&session_id)
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    let request = agent_client_protocol::PromptRequest::new(session_id.clone(), prompt);
    let response = connection
        .prompt(request)
        .await
        .map_err(|error| format!("ACP prompt failed: {error}"))?;
    if matches!(response.stop_reason, StopReason::EndTurn) {
        state
            .borrow()
            .emit(AcpEvent::SessionFinished { session_id });
    }
    Ok(())
}

pub(crate) async fn list_acp_sessions(
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
    state.borrow().emit(AcpEvent::SessionList {
        buffer_id,
        sessions: response.sessions,
    });
    Ok(())
}

pub(crate) async fn load_acp_session(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    buffer_id: BufferId,
    target_session_id: agent_client_protocol::SessionId,
    cwd: PathBuf,
) -> Result<(), String> {
    let connection = {
        let state = state.borrow();
        state
            .sessions
            .get(&session_id)
            .or_else(|| state.sessions.get(&target_session_id))
            .map(|session| session.connection.clone())
    }
    .ok_or_else(|| "ACP session is not connected".to_owned())?;
    // Rebind before await so mid-load prompts resolve against the loaded id.
    {
        let mut state = state.borrow_mut();
        if session_id != target_session_id
            && let Some(session) = state.sessions.remove(&session_id)
        {
            state.sessions.insert(target_session_id.clone(), session);
        }
    }
    let request = LoadSessionRequest::new(target_session_id.clone(), cwd);
    let response = connection
        .load_session(request)
        .await
        .map_err(|error| format!("ACP load session failed: {error}"))?;
    resolve_all_pending_permissions(&state, &session_id);
    resolve_all_pending_permissions(&state, &target_session_id);
    state.borrow().emit(AcpEvent::SessionLoaded {
        buffer_id,
        old_session_id: session_id,
        new_session_id: target_session_id,
        modes: response.modes,
        models: response.models,
    });
    Ok(())
}

pub(crate) async fn set_acp_config_option(
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
    state.borrow().emit(AcpEvent::SessionConfigSet {
        session_id,
        config_id,
        value_id,
    });
    Ok(())
}

pub(crate) async fn set_acp_mode(
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
    state.borrow().emit(AcpEvent::SessionModeSet {
        session_id,
        mode_id,
    });
    Ok(())
}

pub(crate) async fn set_acp_model(
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
    state.borrow().emit(AcpEvent::SessionModelSet {
        session_id,
        model_id,
    });
    Ok(())
}

pub(crate) async fn disconnect_acp_session(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
) {
    let session = {
        let mut state = state.borrow_mut();
        state.sessions.remove(&session_id)
    };
    if let Some(mut session) = session {
        let _ = session.child.kill().await;
    }
    resolve_all_pending_permissions(&state, &session_id);
    state.borrow().emit(AcpEvent::Disconnected {
        session_id,
        message: "Disconnected.".to_owned(),
    });
}

pub(crate) fn resolve_all_pending_permissions(
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

pub(crate) fn resolve_permission_response(
    state: Rc<RefCell<AcpRuntimeState>>,
    request_id: u64,
    decision: PermissionDecision,
) {
    let pending = {
        let mut state = state.borrow_mut();
        let index = state
            .pending_permissions
            .iter()
            .position(|pending| pending.request_id == request_id);
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
    state.borrow().emit(AcpEvent::PermissionResolved {
        request_id,
        session_id: pending.session_id.clone(),
        message: label.to_owned(),
    });
}

pub(crate) fn resolve_permission_option(
    state: Rc<RefCell<AcpRuntimeState>>,
    request_id: u64,
    option_id: PermissionOptionId,
) {
    let pending = {
        let mut state = state.borrow_mut();
        let index = state
            .pending_permissions
            .iter()
            .position(|pending| pending.request_id == request_id);
        index.and_then(|index| state.pending_permissions.remove(index))
    };
    let Some(pending) = pending else {
        return;
    };
    let message = pending
        .options
        .iter()
        .find(|option| option.option_id == option_id)
        .map(|option| format!("Permission `{}` selected.", option.name))
        .unwrap_or_else(|| "Permission selected.".to_owned());
    let _ = pending.responder.send(RequestPermissionOutcome::Selected(
        SelectedPermissionOutcome::new(option_id.clone()),
    ));
    state.borrow().emit(AcpEvent::PermissionResolved {
        request_id,
        session_id: pending.session_id.clone(),
        message,
    });
}

pub(crate) fn choose_permission_outcome(
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

pub(crate) async fn drain_stderr(
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

pub(crate) fn send_client_log(
    state: &Rc<RefCell<AcpRuntimeState>>,
    buffer_id: BufferId,
    message: String,
) {
    state
        .borrow()
        .emit(AcpEvent::ClientLog { buffer_id, message });
}

pub(crate) fn send_client_failure(
    state: &Rc<RefCell<AcpRuntimeState>>,
    buffer_id: BufferId,
    message: String,
) {
    state
        .borrow()
        .emit(AcpEvent::ClientFailed { buffer_id, message });
}

pub(crate) fn send_session_lines(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: &agent_client_protocol::SessionId,
    lines: Vec<String>,
) {
    state.borrow().emit(AcpEvent::SessionLines {
        session_id: session_id.clone(),
        lines,
    });
}

pub(crate) struct AcpRuntimeState {
    pub(crate) sessions: HashMap<agent_client_protocol::SessionId, AcpSession>,
    pub(crate) terminals: HashMap<TerminalId, AcpTerminal>,
    pub(crate) pending_permissions: VecDeque<PendingPermission>,
    pub(crate) next_permission_request_id: u64,
    pub(crate) event_tx: mpsc::Sender<AcpEvent>,
}

impl AcpRuntimeState {
    pub(crate) fn new(event_tx: mpsc::Sender<AcpEvent>) -> Self {
        Self {
            sessions: HashMap::new(),
            terminals: HashMap::new(),
            pending_permissions: VecDeque::new(),
            next_permission_request_id: 1,
            event_tx,
        }
    }

    pub(crate) fn emit(&self, event: AcpEvent) {
        if self.event_tx.send(event).is_ok() {
            super::super::ping_shell_wakeup();
        }
    }
}

pub(crate) struct AcpSession {
    pub(crate) connection: Rc<ClientSideConnection>,
    pub(crate) child: tokio::process::Child,
}

pub(crate) struct PendingPermission {
    pub(crate) request_id: u64,
    pub(crate) session_id: agent_client_protocol::SessionId,
    pub(crate) options: Vec<PermissionOption>,
    pub(crate) responder: oneshot::Sender<RequestPermissionOutcome>,
}

pub(crate) struct AcpTerminal {
    pub(crate) output: Rc<RefCell<String>>,
    pub(crate) exit_status: Rc<RefCell<Option<TerminalExitStatus>>>,
    pub(crate) output_limit: Option<u64>,
    pub(crate) child: tokio::process::Child,
}
