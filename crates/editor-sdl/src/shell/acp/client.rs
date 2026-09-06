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
use super::input::*;
#[allow(unused_imports)]
use super::launch::*;
#[allow(unused_imports)]
use super::manager::*;
#[allow(unused_imports)]
use super::runtime::*;
#[allow(unused_imports)]
use super::session::*;

#[async_trait(?Send)]
impl Client for AcpClient {
    async fn request_permission(
        &self,
        args: RequestPermissionRequest,
    ) -> agent_client_protocol::Result<RequestPermissionResponse> {
        let (tx, rx) = oneshot::channel();
        {
            let mut state = self.state.borrow_mut();
            let request_id = state.next_permission_request_id;
            state.next_permission_request_id = state.next_permission_request_id.saturating_add(1);
            state.pending_permissions.push_back(PendingPermission {
                request_id,
                session_id: args.session_id.clone(),
                options: args.options.clone(),
                responder: tx,
            });
            state.emit(AcpEvent::PermissionRequested {
                request_id,
                session_id: args.session_id.clone(),
                tool_call: args.tool_call.clone(),
                options: args.options.clone(),
            });
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
        let env = args
            .env
            .iter()
            .map(|variable| (variable.name.clone(), variable.value.clone()))
            .collect::<Vec<_>>();
        let mut child = spawn_background_command(
            &args.command,
            &args.args,
            args.cwd.as_deref().map(Path::new),
            &env,
            BackgroundCommandPipes::TERMINAL,
        )
        .await
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

pub(crate) struct AcpClient {
    pub(crate) state: Rc<RefCell<AcpRuntimeState>>,
    pub(crate) next_terminal_id: RefCell<u64>,
}

impl AcpClient {
    pub(crate) fn new(state: Rc<RefCell<AcpRuntimeState>>) -> Self {
        Self {
            state,
            next_terminal_id: RefCell::new(1),
        }
    }

    pub(crate) fn next_terminal_id(&self) -> u64 {
        let mut next = self.next_terminal_id.borrow_mut();
        let id = *next;
        *next = next.saturating_add(1);
        id
    }
}

pub(crate) fn handle_session_update(
    state: Rc<RefCell<AcpRuntimeState>>,
    session_id: agent_client_protocol::SessionId,
    update: SessionUpdate,
) {
    match update {
        SessionUpdate::UserMessageChunk(chunk) => {
            if let ContentBlock::Text(text) = chunk.content {
                state.borrow().emit(AcpEvent::SessionUserPrompt {
                    session_id,
                    prompt: text.text,
                });
            }
        }
        SessionUpdate::AgentMessageChunk(chunk) => {
            state.borrow().emit(AcpEvent::SessionAgentChunk {
                session_id,
                content: chunk.content,
            });
        }
        SessionUpdate::AgentThoughtChunk(_) => {}
        SessionUpdate::ToolCall(call) => {
            state.borrow().emit(AcpEvent::SessionToolCall {
                session_id,
                tool_call: call,
            });
        }
        SessionUpdate::ToolCallUpdate(update) => {
            state
                .borrow()
                .emit(AcpEvent::SessionToolCallUpdate { session_id, update });
        }
        SessionUpdate::Plan(plan) => {
            state
                .borrow()
                .emit(AcpEvent::SessionPlan { session_id, plan });
        }
        SessionUpdate::AvailableCommandsUpdate(update) => {
            let commands = update.available_commands.clone();
            state.borrow().emit(AcpEvent::SessionCommands {
                session_id: session_id.clone(),
                commands,
            });
        }
        SessionUpdate::CurrentModeUpdate(update) => {
            let mode_id = update.current_mode_id.clone();
            state.borrow().emit(AcpEvent::SessionModeUpdate {
                session_id: session_id.clone(),
                mode_id,
            });
        }
        SessionUpdate::ConfigOptionUpdate(update) => {
            state.borrow().emit(AcpEvent::SessionConfigOptions {
                session_id: session_id.clone(),
                options: update.config_options,
            });
        }
        SessionUpdate::SessionInfoUpdate(update) => {
            state
                .borrow()
                .emit(AcpEvent::SessionInfoUpdated { session_id, update });
        }
        _ => {}
    }
}

#[cfg(test)]
pub(crate) fn permission_prompt_lines(request: &RequestPermissionRequest) -> Vec<String> {
    let mut lines = vec![format!(
        "{} Permission requested by agent.",
        editor_icons::symbols::cod::COD_WARNING
    )];
    if let Some(status) = request.tool_call.fields.status {
        lines.push(format!("  {}", format_acp_status_badge(&status)));
    }
    if let Some(title) = request.tool_call.fields.title.clone() {
        lines.push(format!(
            "{} **{}**",
            editor_icons::symbols::cod::COD_TOOLS,
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
                editor_icons::symbols::cod::COD_FILE,
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
        editor_icons::symbols::cod::COD_CHECKLIST
    ));
    lines
}

#[cfg(test)]
pub(crate) fn format_acp_status_badge(status: &impl std::fmt::Debug) -> String {
    let raw = format!("{status:?}");
    let icon = match raw.as_str() {
        "Pending" | "Running" | "InProgress" => editor_icons::symbols::cod::COD_LOADING,
        "Completed" | "Success" | "Succeeded" => editor_icons::symbols::cod::COD_CHECK,
        "Failed" | "Error" => editor_icons::symbols::cod::COD_ERROR,
        "Cancelled" | "Canceled" | "Denied" => editor_icons::symbols::cod::COD_CIRCLE_SLASH,
        _ => editor_icons::symbols::cod::COD_CIRCLE_SMALL_FILLED,
    };
    format!("{icon} {}", humanize_debug_label(&raw))
}

#[cfg(test)]
pub(crate) fn humanize_debug_label(value: &str) -> String {
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

pub(crate) fn format_permission_option_kind(kind: PermissionOptionKind) -> &'static str {
    match kind {
        PermissionOptionKind::AllowOnce => "allow once",
        PermissionOptionKind::AllowAlways => "allow always",
        PermissionOptionKind::RejectOnce => "reject once",
        PermissionOptionKind::RejectAlways => "reject always",
        _ => "custom",
    }
}

pub(crate) fn spawn_terminal_reader(
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

pub(crate) fn apply_output_limit(output: &str, limit: Option<u64>) -> (String, bool) {
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
