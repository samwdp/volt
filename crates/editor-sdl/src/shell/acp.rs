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
    ClientCapabilities, ContentBlock, CreateTerminalRequest, CreateTerminalResponse, Error,
    FileSystemCapabilities, Implementation, InitializeRequest, KillTerminalRequest,
    KillTerminalResponse, NewSessionRequest, PermissionOption, PermissionOptionKind,
    ProtocolVersion, ReadTextFileRequest, ReadTextFileResponse, ReleaseTerminalRequest,
    ReleaseTerminalResponse, RequestPermissionOutcome, RequestPermissionRequest,
    RequestPermissionResponse, SelectedPermissionOutcome, SessionNotification, SessionUpdate,
    TerminalExitStatus, TerminalId, TerminalOutputRequest, TerminalOutputResponse,
    WaitForTerminalExitRequest, WaitForTerminalExitResponse, WriteTextFileRequest,
    WriteTextFileResponse,
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

pub(super) fn init_acp_manager(runtime: &mut EditorRuntime) -> Result<(), ShellError> {
    let manager = AcpManager::new().map_err(ShellError::Runtime)?;
    runtime.services_mut().insert(Arc::new(Mutex::new(manager)));
    Ok(())
}

pub(super) fn refresh_pending_acp(runtime: &mut EditorRuntime) -> Result<(), String> {
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.drain_events(runtime)
}

pub(super) fn open_acp_client(runtime: &mut EditorRuntime, client_id: &str) -> Result<(), String> {
    let client = user::acp::client_by_id(client_id)
        .ok_or_else(|| format!("unknown ACP client `{client_id}`"))?;
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let manager = runtime
        .services()
        .get::<Arc<Mutex<AcpManager>>>()
        .ok_or_else(|| "acp manager service missing".to_owned())?
        .clone();
    if let Some(buffer_id) = {
        let manager = manager
            .lock()
            .map_err(|_| "acp manager lock was poisoned".to_owned())?;
        manager.buffer_for_client(&client.id)
    } {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, buffer_id)
            .map_err(|error| error.to_string())?;
        shell_ui_mut(runtime)?.focus_buffer(buffer_id);
        return Ok(());
    }
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
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(
        buffer,
        vec![format!("Connecting to {}...", client.label)],
    );
    shell_buffer.clear_input();
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);

    let workspace_root = active_workspace_root(runtime)?
        .or_else(|| env::current_dir().ok())
        .ok_or_else(|| "ACP requires a workspace root or current directory".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.connect(client, workspace_root, buffer_id)
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
        return Ok(());
    };
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.append_output_lines(&[format!("{prompt}{text}")]);
        buffer.clear_input();
    }
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.prompt(session_id, text.to_owned())
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

struct AcpManager {
    runtime: AcpRuntime,
    events: mpsc::Receiver<AcpEvent>,
    sessions: HashMap<agent_client_protocol::SessionId, AcpSessionInfo>,
    buffers: HashMap<BufferId, agent_client_protocol::SessionId>,
    pending_clients: HashMap<String, PendingAcpClient>,
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
        })
    }

    fn buffer_for_client(&self, client_id: &str) -> Option<BufferId> {
        self.sessions
            .values()
            .find(|session| session.client_id == client_id)
            .map(|session| session.buffer_id)
            .or_else(|| {
                self.pending_clients
                    .get(client_id)
                    .map(|pending| pending.buffer_id)
            })
    }

    fn session_for_buffer(&self, buffer_id: BufferId) -> Option<agent_client_protocol::SessionId> {
        self.buffers.get(&buffer_id).cloned()
    }

    fn connect(
        &mut self,
        client: user::acp::AcpClientConfig,
        workspace_root: PathBuf,
        buffer_id: BufferId,
    ) -> Result<(), String> {
        if self
            .sessions
            .values()
            .any(|session| session.client_id == client.id)
        {
            return Ok(());
        }
        self.pending_clients
            .insert(client.id.clone(), PendingAcpClient { buffer_id });
        self.runtime.send(AcpCommand::Connect {
            config: client,
            workspace_root,
        })
    }

    fn prompt(
        &mut self,
        session_id: agent_client_protocol::SessionId,
        prompt: String,
    ) -> Result<(), String> {
        self.runtime.send(AcpCommand::Prompt { session_id, prompt })
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

    fn drain_events(&mut self, runtime: &mut EditorRuntime) -> Result<(), String> {
        let events: Vec<AcpEvent> = self.events.try_iter().collect();
        for event in events {
            self.handle_event(runtime, event)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, runtime: &mut EditorRuntime, event: AcpEvent) -> Result<(), String> {
        match event {
            AcpEvent::Connected {
                client_id,
                session_id,
            } => {
                let Some(pending) = self.pending_clients.remove(&client_id) else {
                    return Ok(());
                };
                self.buffers.insert(pending.buffer_id, session_id.clone());
                self.sessions.insert(
                    session_id.clone(),
                    AcpSessionInfo {
                        client_id,
                        buffer_id: pending.buffer_id,
                    },
                );
                if let Ok(buffer) = shell_buffer_mut(runtime, pending.buffer_id) {
                    buffer.append_output_lines(&["Connected.".to_owned()]);
                }
            }
            AcpEvent::ClientFailed { client_id, message } => {
                if let Some(pending) = self.pending_clients.remove(&client_id) {
                    if let Ok(buffer) = shell_buffer_mut(runtime, pending.buffer_id) {
                        buffer.append_output_lines(&[message]);
                    }
                }
            }
            AcpEvent::ClientLog { client_id, message } => {
                if let Some(buffer_id) = self
                    .pending_clients
                    .get(&client_id)
                    .map(|pending| pending.buffer_id)
                    .or_else(|| self.buffer_for_client(&client_id))
                {
                    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                        buffer.append_output_lines(&[message]);
                    }
                }
            }
            AcpEvent::SessionText { session_id, text } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                {
                    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                        buffer.append_output_text(&text);
                    }
                }
            }
            AcpEvent::SessionLines { session_id, lines } => {
                if let Some(buffer_id) = self
                    .sessions
                    .get(&session_id)
                    .map(|session| session.buffer_id)
                {
                    if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                        buffer.append_output_lines(&lines);
                    }
                }
            }
            AcpEvent::Disconnected {
                session_id,
                message,
            } => {
                if let Some(session) = self.sessions.remove(&session_id) {
                    self.buffers.remove(&session.buffer_id);
                    if let Ok(buffer) = shell_buffer_mut(runtime, session.buffer_id) {
                        buffer.append_output_lines(&[message]);
                    }
                }
            }
        }
        Ok(())
    }
}

struct PendingAcpClient {
    buffer_id: BufferId,
}

struct AcpSessionInfo {
    client_id: String,
    buffer_id: BufferId,
}

enum AcpEvent {
    Connected {
        client_id: String,
        session_id: agent_client_protocol::SessionId,
    },
    ClientFailed {
        client_id: String,
        message: String,
    },
    ClientLog {
        client_id: String,
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
    Disconnected {
        session_id: agent_client_protocol::SessionId,
        message: String,
    },
}

enum AcpCommand {
    Connect {
        config: user::acp::AcpClientConfig,
        workspace_root: PathBuf,
    },
    Prompt {
        session_id: agent_client_protocol::SessionId,
        prompt: String,
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
            } => {
                let state = state.clone();
                tokio::task::spawn_local(async move {
                    let client_id = config.id.clone();
                    if let Err(error) =
                        connect_acp_client(state.clone(), config, workspace_root).await
                    {
                        send_client_failure(&state, &client_id, error);
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
) -> Result<(), String> {
    let mut command = Command::new(&config.command);
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
        let client_id = config.id.clone();
        tokio::task::spawn_local(async move {
            drain_stderr(state, client_id, stderr).await;
        });
    }

    let client = Rc::new(AcpClient::new(state.clone()));
    let (connection, io_task) =
        ClientSideConnection::new(client, stdin.compat_write(), stdout.compat(), |task| {
            tokio::task::spawn_local(task);
        });
    let client_id = config.id.clone();
    let state_clone = state.clone();
    tokio::task::spawn_local(async move {
        if let Err(error) = io_task.await {
            send_client_log(&state_clone, &client_id, format!("ACP I/O error: {error}"));
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

    state.borrow_mut().sessions.insert(
        session_id.clone(),
        AcpSession {
            connection: Rc::new(connection),
            child,
        },
    );
    let _ = state.borrow().event_tx.send(AcpEvent::Connected {
        client_id: config.id,
        session_id,
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
            .rposition(|pending| pending.session_id == session_id);
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
    client_id: String,
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
                    send_client_log(&state, &client_id, message);
                }
            }
            Err(error) => {
                send_client_log(&state, &client_id, format!("ACP stderr error: {error}"));
                break;
            }
        }
    }
}

fn send_client_log(state: &Rc<RefCell<AcpRuntimeState>>, client_id: &str, message: String) {
    let _ = state.borrow().event_tx.send(AcpEvent::ClientLog {
        client_id: client_id.to_owned(),
        message,
    });
}

fn send_client_failure(state: &Rc<RefCell<AcpRuntimeState>>, client_id: &str, message: String) {
    let _ = state.borrow().event_tx.send(AcpEvent::ClientFailed {
        client_id: client_id.to_owned(),
        message,
    });
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
        if terminal.exit_status.borrow().is_none() {
            if let Ok(Some(status)) = terminal.child.try_wait() {
                let exit =
                    TerminalExitStatus::new().exit_code(status.code().map(|code| code as u32));
                *terminal.exit_status.borrow_mut() = Some(exit);
            }
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
        let mut state = self.state.borrow_mut();
        let terminal = state
            .terminals
            .get_mut(&args.terminal_id)
            .ok_or_else(|| Error::resource_not_found(None))?;
        let _ = terminal.child.kill().await;
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
            let mut lines = vec![format!("[tool] {} ({:?})", call.title, call.status)];
            lines.extend(render_tool_call_content(&call.content));
            let _ = state
                .borrow()
                .event_tx
                .send(AcpEvent::SessionLines { session_id, lines });
        }
        SessionUpdate::ToolCallUpdate(update) => {
            let mut lines = vec![format!("[tool] update {}", update.tool_call_id)];
            if let Some(status) = update.fields.status {
                lines.push(format!("status: {status:?}"));
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
                lines.push(format!("[plan] {} ({:?})", entry.content, entry.status));
            }
            if !lines.is_empty() {
                let _ = state
                    .borrow()
                    .event_tx
                    .send(AcpEvent::SessionLines { session_id, lines });
            }
        }
        SessionUpdate::AvailableCommandsUpdate(update) => {
            let lines = vec![format!(
                "[acp] {} commands available",
                update.available_commands.len()
            )];
            let _ = state
                .borrow()
                .event_tx
                .send(AcpEvent::SessionLines { session_id, lines });
        }
        SessionUpdate::CurrentModeUpdate(update) => {
            let lines = vec![format!("[acp] mode: {}", update.current_mode_id)];
            let _ = state
                .borrow()
                .event_tx
                .send(AcpEvent::SessionLines { session_id, lines });
        }
        SessionUpdate::ConfigOptionUpdate(update) => {
            let lines = vec![format!(
                "[acp] {} config options updated",
                update.config_options.len()
            )];
            let _ = state
                .borrow()
                .event_tx
                .send(AcpEvent::SessionLines { session_id, lines });
        }
        SessionUpdate::SessionInfoUpdate(update) => {
            let mut lines = Vec::new();
            if let Some(title) = update.title.value() {
                lines.push(format!("[acp] title: {title}"));
            }
            if let Some(updated_at) = update.updated_at.value() {
                lines.push(format!("[acp] updated: {updated_at}"));
            }
            if !lines.is_empty() {
                let _ = state
                    .borrow()
                    .event_tx
                    .send(AcpEvent::SessionLines { session_id, lines });
            }
        }
        _ => {}
    }
}

fn render_tool_call_content(content: &[agent_client_protocol::ToolCallContent]) -> Vec<String> {
    let mut lines = Vec::new();
    for item in content {
        match item {
            agent_client_protocol::ToolCallContent::Content(content) => {
                if let ContentBlock::Text(text) = &content.content {
                    lines.push(text.text.clone());
                }
            }
            agent_client_protocol::ToolCallContent::Diff(diff) => {
                lines.push(format!(
                    "[diff] {} ({} -> {})",
                    diff.path.display(),
                    diff.old_text.as_ref().map_or("new", |_| "old"),
                    "new"
                ));
            }
            agent_client_protocol::ToolCallContent::Terminal(terminal) => {
                lines.push(format!("[terminal] {}", terminal.terminal_id));
            }
            _ => {}
        }
    }
    lines
}

fn permission_prompt_lines(request: &RequestPermissionRequest) -> Vec<String> {
    let mut lines = vec!["Permission requested by agent.".to_owned()];
    if let Some(status) = request.tool_call.fields.status {
        lines.push(format!("Status: {status:?}"));
    }
    if let Some(title) = request.tool_call.fields.title.clone() {
        lines.push(format!("Tool: {title}"));
    }
    lines.push("Use acp.permission-approve or acp.permission-deny.".to_owned());
    lines
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
