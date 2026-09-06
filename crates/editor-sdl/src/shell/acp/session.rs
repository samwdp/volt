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
use super::manager::*;
#[allow(unused_imports)]
use super::runtime::*;

pub(crate) fn init_acp_manager(runtime: &mut EditorRuntime) -> Result<(), ShellError> {
    let manager = AcpManager::new().map_err(ShellError::Runtime)?;
    runtime.services_mut().insert(Arc::new(Mutex::new(manager)));
    Ok(())
}

pub(crate) fn refresh_pending_acp(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() else {
        return Ok(false);
    };
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
    if changed {
        remap_acp_output_visual_anchors(runtime)?;
    }
    Ok(changed)
}

pub(crate) fn acp_connected(runtime: &EditorRuntime) -> Result<bool, String> {
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

pub(crate) fn open_acp_client(runtime: &mut EditorRuntime, client_id: &str) -> Result<(), String> {
    open_acp_client_buffer(runtime, client_id, true, None).map(|_| ())
}

pub(crate) fn acp_new_session(runtime: &mut EditorRuntime) -> Result<(), String> {
    let client = active_acp_client(runtime)?;
    open_acp_client_with_config(runtime, client, false, None).map(|_| ())
}

pub(crate) fn close_acp_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() else {
        return Ok(());
    };
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.close_buffer(buffer_id);
    Ok(())
}

pub(crate) fn close_acp_workspace_buffers(
    runtime: &mut EditorRuntime,
    workspace_id: WorkspaceId,
) -> Result<(), String> {
    let Some(manager) = runtime.services().get::<Arc<Mutex<AcpManager>>>().cloned() else {
        return Ok(());
    };
    let buffer_ids = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffers()
        .filter(|buffer| {
            matches!(
                buffer.kind(),
                BufferKind::Plugin(plugin_kind) if plugin_kind == ACP_BUFFER_KIND
            )
        })
        .map(|buffer| buffer.id())
        .collect::<Vec<_>>();
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    for buffer_id in buffer_ids {
        manager.close_buffer(buffer_id);
    }
    Ok(())
}

pub(crate) fn open_acp_client_buffer(
    runtime: &mut EditorRuntime,
    client_id: &str,
    reuse_existing: bool,
    load_session: Option<PendingAcpLoadSession>,
) -> Result<BufferId, String> {
    let client = shell_user_library(runtime)
        .acp_client_by_id(client_id)
        .ok_or_else(|| format!("unknown ACP client `{client_id}`"))?;
    open_acp_client_with_config(runtime, client, reuse_existing, load_session)
}

pub(crate) fn open_acp_client_with_config(
    runtime: &mut EditorRuntime,
    client: AcpClientConfig,
    reuse_existing: bool,
    load_session: Option<PendingAcpLoadSession>,
) -> Result<BufferId, String> {
    let active_workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
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
            manager.buffer_for_client(active_workspace_id, &client.id)
        }
    {
        if shell_ui(runtime)
            .ok()
            .and_then(|ui| ui.buffer(buffer_id))
            .is_none()
        {
            let mut manager = manager
                .lock()
                .map_err(|_| "acp manager lock was poisoned".to_owned())?;
            manager.close_buffer(buffer_id);
        } else {
            focus_acp_buffer(runtime, buffer_id)?;
            return Ok(buffer_id);
        }
    }

    let (buffer_id, workspace_id, workspace_name) = create_acp_buffer(runtime, &client)?;
    let workspace_root = active_workspace_root(runtime)?
        .or_else(|| env::current_dir().ok())
        .ok_or_else(|| "ACP requires a workspace root or current directory".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "acp manager lock was poisoned".to_owned())?;
    manager.connect(
        client,
        workspace_root,
        workspace_id,
        buffer_id,
        load_session,
        workspace_name,
    )?;
    Ok(buffer_id)
}

pub(crate) fn create_acp_buffer(
    runtime: &mut EditorRuntime,
    client: &AcpClientConfig,
) -> Result<(BufferId, WorkspaceId, String), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace_name = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .name()
        .to_owned();
    let buffer_name = format!("*acp {}*", client.label);
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            buffer_name.as_str(),
            BufferKind::Plugin(ACP_BUFFER_KIND.to_owned()),
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
    let user_library = shell_user_library(runtime);
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, Vec::new(), &*user_library);
    shell_buffer.init_acp_view(&client.label);
    shell_buffer.clear_input();
    shell_buffer.set_forced_language_id("markdown");
    shell_ui_mut(runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
    Ok((buffer_id, workspace_id, workspace_name))
}

pub(crate) fn focus_acp_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
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
