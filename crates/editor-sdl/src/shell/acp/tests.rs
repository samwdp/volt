
use super::*;
use agent_client_protocol::{
    AvailableCommand, AvailableCommandInput, PermissionOption, PermissionOptionId,
    PermissionOptionKind, RequestPermissionRequest, SessionInfo, TextContent, ToolCallLocation,
    ToolCallStatus, ToolCallUpdate, ToolCallUpdateFields, UnstructuredCommandInput,
};
use tokio::sync::mpsc as tokio_mpsc;

fn test_acp_manager() -> (AcpManager, tokio_mpsc::UnboundedReceiver<AcpCommand>) {
    let (_event_tx, event_rx) = mpsc::channel();
    let (command_tx, command_rx) = tokio_mpsc::unbounded_channel();
    (
        AcpManager {
            runtime: AcpRuntime { sender: command_tx },
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
        },
        command_rx,
    )
}

fn test_buffer_id() -> Result<BufferId, String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*acp test*",
            BufferKind::Plugin(ACP_BUFFER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())
}

fn test_workspace_id() -> Result<WorkspaceId, String> {
    let state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())
}

fn install_acp_test_buffer(state: &mut ShellState) -> Result<(WorkspaceId, BufferId), String> {
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = state
        .runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            "*acp test*",
            BufferKind::Plugin(ACP_BUFFER_KIND.to_owned()),
            None,
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = state
        .runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP test buffer is missing".to_owned())?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, Vec::new(), &NullUserLibrary);
    shell_buffer.init_acp_view("GitHub Copilot");
    shell_ui_mut(&mut state.runtime)?.insert_buffer(shell_buffer);
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    Ok((workspace_id, buffer_id))
}

fn test_permission_options() -> Vec<PermissionOption> {
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
    ]
}

fn test_tool_call_update(title: &str) -> ToolCallUpdate {
    ToolCallUpdate::new(
        "tool-1",
        ToolCallUpdateFields::new()
            .status(ToolCallStatus::Pending)
            .title(title),
    )
}

fn test_pending_permission_request(
    request_id: u64,
    session_id: &str,
    title: &str,
) -> AcpPendingPermissionUi {
    AcpPendingPermissionUi {
        request_id,
        session_id: agent_client_protocol::SessionId::new(session_id),
        workspace_name: "project".to_owned(),
        tool_call: test_tool_call_update(title),
        options: test_permission_options(),
    }
}

fn text_chunk_event(session_id: &str, text: &str) -> AcpEvent {
    AcpEvent::SessionAgentChunk {
        session_id: agent_client_protocol::SessionId::new(session_id),
        content: ContentBlock::Text(TextContent::new(text)),
    }
}

#[test]
fn acp_session_buffer_name_wraps_title() {
    assert_eq!(acp_session_buffer_name("Fix login"), "*acp [Fix login]*");
}

#[test]
fn session_load_replay_keeps_history_and_names_buffer() -> Result<(), String> {
    let (mut manager, mut command_rx) = test_acp_manager();
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let (workspace_id, buffer_id) = install_acp_test_buffer(&mut state)?;
    let temp_session_id = agent_client_protocol::SessionId::new("temp-session");
    let loaded_session_id = agent_client_protocol::SessionId::new("loaded-session");

    manager.pending_clients.insert(
        buffer_id,
        PendingAcpClient {
            client_id: "copilot".to_owned(),
            load_session: Some(PendingAcpLoadSession {
                session_id: loaded_session_id.clone(),
                title: Some("Fix login".to_owned()),
            }),
            workspace_root: PathBuf::from("."),
            workspace_id,
            workspace_name: "project".to_owned(),
        },
    );

    manager.handle_event(
        &mut state.runtime,
        AcpEvent::Connected {
            buffer_id,
            client_id: "copilot".to_owned(),
            session_id: temp_session_id.clone(),
            modes: None,
            models: None,
        },
    )?;

    assert!(
        manager.sessions.contains_key(&loaded_session_id),
        "loaded session id should be bound before replay"
    );
    assert!(!manager.sessions.contains_key(&temp_session_id));
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.name,
        "*acp [Fix login]*"
    );
    assert!(matches!(
        command_rx.try_recv().expect("load session command"),
        AcpCommand::LoadSession {
            target_session_id,
            ..
        } if target_session_id == loaded_session_id
    ));

    manager.handle_event(
        &mut state.runtime,
        AcpEvent::SessionUserPrompt {
            session_id: loaded_session_id.clone(),
            prompt: "hello from history".to_owned(),
        },
    )?;
    manager.handle_event(
        &mut state.runtime,
        text_chunk_event("loaded-session", "prior reply"),
    )?;
    manager.handle_event(
        &mut state.runtime,
        AcpEvent::SessionLoaded {
            buffer_id,
            old_session_id: temp_session_id,
            new_session_id: loaded_session_id.clone(),
            modes: None,
            models: None,
        },
    )?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.name, "*acp [Fix login]*");
    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert!(
        acp.output_items.iter().any(|item| matches!(
            item,
            AcpOutputItem::UserPrompt(text) if text == "hello from history"
        )),
        "user history should survive SessionLoaded"
    );
    assert!(
        acp.output_items.iter().any(|item| {
            matches!(item, AcpOutputItem::AgentBlocks(blocks) if blocks.iter().any(|block| {
                matches!(block, ContentBlock::Text(text) if text.text == "prior reply")
            }))
        }),
        "agent history should survive SessionLoaded"
    );
    Ok(())
}

#[test]
fn drain_acp_event_batch_limits_per_frame_work() {
    let (tx, rx) = mpsc::channel();
    let session_id = agent_client_protocol::SessionId::new("session");
    for _ in 0..(ACP_EVENT_DRAIN_LIMIT + 3) {
        tx.send(AcpEvent::SessionFinished {
            session_id: session_id.clone(),
        })
        .expect("send event");
    }

    let batch = drain_acp_event_batch(&rx, ACP_EVENT_DRAIN_LIMIT);
    assert_eq!(batch.len(), ACP_EVENT_DRAIN_LIMIT);
    assert!(rx.try_recv().is_ok());
}

#[test]
fn coalesce_acp_events_merges_adjacent_agent_text_chunks() {
    let events = vec![
        text_chunk_event("session-1", "hel"),
        text_chunk_event("session-1", "lo"),
        text_chunk_event("session-2", "x"),
        text_chunk_event("session-1", "!"),
    ];

    let coalesced = coalesce_acp_events(events);
    assert_eq!(coalesced.len(), 3);
    match &coalesced[0] {
        AcpEvent::SessionAgentChunk {
            content: ContentBlock::Text(text),
            ..
        } => assert_eq!(text.text, "hello"),
        _ => panic!("expected merged text chunk"),
    }
}

#[test]
fn split_acp_events_for_render_defers_later_plan_transitions() {
    let session_id = agent_client_protocol::SessionId::new("session");
    let plan = Plan::new(vec![PlanEntry::new(
        "Step",
        PlanEntryPriority::High,
        PlanEntryStatus::InProgress,
    )]);
    let events = vec![
        text_chunk_event("session", "hello"),
        AcpEvent::SessionPlan {
            session_id: session_id.clone(),
            plan: plan.clone(),
        },
        AcpEvent::SessionPlan {
            session_id: session_id.clone(),
            plan: plan.clone(),
        },
        AcpEvent::SessionFinished { session_id },
    ];

    let (ready, deferred) = split_acp_events_for_render(events);
    assert_eq!(ready.len(), 2);
    assert_eq!(deferred.len(), 2);
    assert!(matches!(ready[1], AcpEvent::SessionPlan { .. }));
    assert!(matches!(deferred[0], AcpEvent::SessionPlan { .. }));
    assert!(matches!(deferred[1], AcpEvent::SessionFinished { .. }));
}

#[test]
fn drain_events_shows_incremental_plan_progress_across_frames() -> Result<(), String> {
    let (event_tx, event_rx) = mpsc::channel();
    let (command_tx, _command_rx) = tokio_mpsc::unbounded_channel();
    let mut manager = AcpManager {
        runtime: AcpRuntime { sender: command_tx },
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
    };
    let session_id = agent_client_protocol::SessionId::new("session-progress");
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let (workspace_id, buffer_id) = install_acp_test_buffer(&mut state)?;

    manager.sessions.insert(
        session_id.clone(),
        AcpSessionInfo {
            client_id: "copilot".to_owned(),
            buffer_id,
            workspace_id,
            workspace_name: "project".to_owned(),
            title: Some("Plan run".to_owned()),
            available_commands: Vec::new(),
            mode_state: None,
            model_state: None,
            config_options: Vec::new(),
            mode_config_id: None,
            model_config_id: None,
        },
    );

    event_tx
        .send(AcpEvent::SessionPlan {
            session_id: session_id.clone(),
            plan: Plan::new(vec![
                PlanEntry::new(
                    "Step one",
                    PlanEntryPriority::High,
                    PlanEntryStatus::InProgress,
                ),
                PlanEntry::new(
                    "Step two",
                    PlanEntryPriority::Medium,
                    PlanEntryStatus::Pending,
                ),
            ]),
        })
        .map_err(|error| error.to_string())?;
    event_tx
        .send(AcpEvent::SessionPlan {
            session_id: session_id.clone(),
            plan: Plan::new(vec![
                PlanEntry::new(
                    "Step one",
                    PlanEntryPriority::High,
                    PlanEntryStatus::Completed,
                ),
                PlanEntry::new(
                    "Step two",
                    PlanEntryPriority::Medium,
                    PlanEntryStatus::InProgress,
                ),
            ]),
        })
        .map_err(|error| error.to_string())?;
    event_tx
        .send(AcpEvent::SessionFinished {
            session_id: session_id.clone(),
        })
        .map_err(|error| error.to_string())?;

    assert!(manager.drain_events(&mut state.runtime)?);
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let acp = buffer
            .acp_state
            .as_ref()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        assert_eq!(acp.plan_entries[0].status, PlanEntryStatus::InProgress);
        assert_eq!(acp.plan_entries[1].status, PlanEntryStatus::Pending);
    }

    assert!(manager.drain_events(&mut state.runtime)?);
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let acp = buffer
            .acp_state
            .as_ref()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        assert_eq!(acp.plan_entries[0].status, PlanEntryStatus::Completed);
        assert_eq!(acp.plan_entries[1].status, PlanEntryStatus::InProgress);
    }

    assert!(manager.drain_events(&mut state.runtime)?);
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let acp = buffer
            .acp_state
            .as_ref()
            .ok_or_else(|| "ACP state missing".to_owned())?;
        assert!(
            acp.plan_entries
                .iter()
                .all(|entry| entry.status == PlanEntryStatus::Completed)
        );
    }

    Ok(())
}

#[cfg(windows)]
#[test]
fn parse_windows_cmd_environment_reads_fnm_env_output() {
    let parsed = parse_windows_cmd_environment(
        "SET PATH=C:\\fnm-node;C:\\tools\r\nSET FNM_DIR=C:\\Users\\sam\\AppData\\Roaming\\fnm\r\n",
    )
    .expect("fnm env output should parse");

    assert_eq!(
        parsed,
        vec![
            ("PATH".to_owned(), "C:\\fnm-node;C:\\tools".to_owned()),
            (
                "FNM_DIR".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned()
            )
        ]
    );
}

#[cfg(windows)]
fn temp_dir(prefix: &str) -> PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be available")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
}

#[cfg(windows)]
#[test]
fn parse_windows_nvm_helpers_read_current_root_and_settings() {
    let settings = parse_windows_nvm_settings(
        "root: C:\\Users\\sam\\AppData\\Roaming\\nvm\r\npath: C:\\Program Files\\nodejs\r\narch: 64\r\n",
    );

    assert_eq!(
        parse_windows_nvm_current_output("v20.11.1\r\n"),
        Some("v20.11.1".to_owned())
    );
    assert_eq!(
        parse_windows_nvm_current_output("No current version\r\n"),
        None
    );
    assert_eq!(
        parse_windows_nvm_root_output("Current Root: C:\\Users\\sam\\AppData\\Roaming\\nvm\r\n"),
        Some(PathBuf::from("C:\\Users\\sam\\AppData\\Roaming\\nvm"))
    );
    assert_eq!(
        settings.root,
        Some(PathBuf::from("C:\\Users\\sam\\AppData\\Roaming\\nvm"))
    );
    assert_eq!(
        settings.path,
        Some(PathBuf::from("C:\\Program Files\\nodejs"))
    );
}

#[cfg(windows)]
#[test]
fn windows_nvm_environment_prepends_current_node_path() {
    let root = temp_dir("volt-nvm-root");
    let version_dir = root.join("v20.11.1");
    std::fs::create_dir_all(&version_dir).expect("create nvm version directory");
    std::fs::File::create(version_dir.join("node.exe")).expect("create node shim");
    let env = vec![("PATH".to_owned(), "C:\\tools".to_owned())];

    let nvm_env =
        windows_nvm_environment_from_parts(Some(root.clone()), None, Some("v20.11.1"), &env, None)
            .expect("nvm environment should resolve");
    let path = explicit_environment_value(&nvm_env, "PATH").expect("PATH should be present");

    assert!(path.starts_with(&version_dir.to_string_lossy().into_owned()));
    assert!(path.ends_with(";C:\\tools"));
}

#[cfg(windows)]
#[test]
fn merge_node_manager_environment_keeps_manager_path_first() {
    let env = vec![
        ("PATH".to_owned(), "C:\\explicit".to_owned()),
        ("CUSTOM".to_owned(), "1".to_owned()),
    ];
    let launch_env = vec![("PATH".to_owned(), "C:\\launch".to_owned())];
    let manager_env = vec![
        ("PATH".to_owned(), "C:\\node".to_owned()),
        ("NVM_HOME".to_owned(), "C:\\nvm".to_owned()),
    ];

    let merged = merge_node_manager_environment(&env, Some(&launch_env), manager_env);

    assert_eq!(
        explicit_environment_value(&merged, "PATH"),
        Some(&"C:\\node;C:\\explicit".to_owned())
    );
    assert_eq!(
        explicit_environment_value(&merged, "NVM_HOME"),
        Some(&"C:\\nvm".to_owned())
    );
    assert_eq!(
        explicit_environment_value(&merged, "CUSTOM"),
        Some(&"1".to_owned())
    );
}

#[test]
fn active_command_input_hint_uses_unstructured_command_metadata() {
    let commands = vec![
        AvailableCommand::new("open", "Open a file").input(AvailableCommandInput::Unstructured(
            UnstructuredCommandInput::new("path to open"),
        )),
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
    assert!(rendered.contains("main.rs"));
    assert!(rendered.contains("12"));
    assert!(rendered.contains("Allow once (allow once)"));
    assert!(rendered.contains("Reject once (reject once)"));
}

#[test]
fn close_buffer_disconnects_sessions_and_clears_reuse_state() -> Result<(), String> {
    let (mut manager, mut command_rx) = test_acp_manager();
    let buffer_id = test_buffer_id()?;
    let workspace_id = test_workspace_id()?;
    let session_id = agent_client_protocol::SessionId::new("session-1");
    manager.sessions.insert(
        session_id.clone(),
        AcpSessionInfo {
            client_id: "copilot".to_owned(),
            buffer_id,
            workspace_id,
            workspace_name: "project".to_owned(),
            title: None,
            available_commands: Vec::new(),
            mode_state: None,
            model_state: None,
            config_options: Vec::new(),
            mode_config_id: None,
            model_config_id: None,
        },
    );
    manager.buffers.insert(buffer_id, session_id.clone());
    manager
        .workspace_client_buffers
        .insert((workspace_id, "copilot".to_owned()), buffer_id);
    manager
        .pending_slash
        .insert(buffer_id, PendingSlashTrigger::Manual);
    manager
        .pending_ui_actions
        .push(AcpUiAction::OpenSlashCompletion {
            buffer_id,
            trigger: PendingSlashTrigger::Manual,
        });

    manager.close_buffer(buffer_id);

    assert!(manager.buffer_for_client(workspace_id, "copilot").is_none());
    assert!(manager.session_for_buffer(buffer_id).is_none());
    assert!(!manager.pending_slash.contains_key(&buffer_id));
    assert!(manager.pending_ui_actions.is_empty());
    assert!(matches!(
        command_rx.try_recv().expect("disconnect command should be queued"),
        AcpCommand::Disconnect {
            session_id: disconnected
        } if disconnected == session_id
    ));
    Ok(())
}

#[test]
fn connected_event_for_closed_buffer_disconnects_orphaned_session() -> Result<(), String> {
    let (mut manager, mut command_rx) = test_acp_manager();
    let buffer_id = test_buffer_id()?;
    let session_id = agent_client_protocol::SessionId::new("session-closed");
    let mut state = ShellState::new().map_err(|error| error.to_string())?;

    manager.handle_event(
        &mut state.runtime,
        AcpEvent::Connected {
            buffer_id,
            client_id: "copilot".to_owned(),
            session_id: session_id.clone(),
            modes: None,
            models: None,
        },
    )?;

    assert!(manager.sessions.is_empty());
    assert!(manager.buffers.is_empty());
    assert!(matches!(
        command_rx.try_recv().expect("orphaned connect should disconnect"),
        AcpCommand::Disconnect {
            session_id: disconnected
        } if disconnected == session_id
    ));
    Ok(())
}

#[test]
fn permission_requests_queue_and_advance_after_resolution() -> Result<(), String> {
    let (mut manager, _command_rx) = test_acp_manager();
    let buffer_id = test_buffer_id()?;
    let workspace_id = test_workspace_id()?;
    let session_id = agent_client_protocol::SessionId::new("session-1");
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    manager.sessions.insert(
        session_id.clone(),
        AcpSessionInfo {
            client_id: "copilot".to_owned(),
            buffer_id,
            workspace_id,
            workspace_name: "project".to_owned(),
            title: None,
            available_commands: Vec::new(),
            mode_state: None,
            model_state: None,
            config_options: Vec::new(),
            mode_config_id: None,
            model_config_id: None,
        },
    );

    manager.handle_event(
        &mut state.runtime,
        AcpEvent::PermissionRequested {
            request_id: 1,
            session_id: session_id.clone(),
            tool_call: test_tool_call_update("Read file"),
            options: test_permission_options(),
        },
    )?;
    assert_eq!(
        shell_ui(&state.runtime)?.picker_kind(),
        Some(PickerKind::AcpPermission { request_id: 1 })
    );
    assert_eq!(manager.active_permission_request, Some(1));

    manager.handle_event(
        &mut state.runtime,
        AcpEvent::PermissionRequested {
            request_id: 2,
            session_id: session_id.clone(),
            tool_call: test_tool_call_update("Write file"),
            options: test_permission_options(),
        },
    )?;
    assert_eq!(
        manager
            .pending_permissions
            .iter()
            .map(|request| request.request_id)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(
        shell_ui(&state.runtime)?.picker_kind(),
        Some(PickerKind::AcpPermission { request_id: 1 })
    );

    shell_ui_mut(&mut state.runtime)?.close_picker();
    manager.handle_event(
        &mut state.runtime,
        AcpEvent::PermissionResolved {
            request_id: 1,
            session_id,
            message: "Permission approved.".to_owned(),
        },
    )?;

    assert_eq!(
        manager
            .pending_permissions
            .iter()
            .map(|request| request.request_id)
            .collect::<Vec<_>>(),
        vec![2]
    );
    assert_eq!(manager.active_permission_request, Some(2));
    assert_eq!(
        shell_ui(&state.runtime)?.picker_kind(),
        Some(PickerKind::AcpPermission { request_id: 2 })
    );
    Ok(())
}

#[test]
fn session_finished_marks_plan_entries_completed() -> Result<(), String> {
    let (mut manager, _command_rx) = test_acp_manager();
    let session_id = agent_client_protocol::SessionId::new("session-finish");
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let (workspace_id, buffer_id) = install_acp_test_buffer(&mut state)?;
    manager.sessions.insert(
        session_id.clone(),
        AcpSessionInfo {
            client_id: "copilot".to_owned(),
            buffer_id,
            workspace_id,
            workspace_name: "project".to_owned(),
            title: Some("Plan run".to_owned()),
            available_commands: Vec::new(),
            mode_state: None,
            model_state: None,
            config_options: Vec::new(),
            mode_config_id: None,
            model_config_id: None,
        },
    );

    let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
    buffer.acp_set_plan(Plan::new(vec![
        PlanEntry::new(
            "Map state",
            PlanEntryPriority::High,
            PlanEntryStatus::InProgress,
        ),
        PlanEntry::new(
            "Finalize output",
            PlanEntryPriority::Medium,
            PlanEntryStatus::Pending,
        ),
    ]));

    manager.handle_event(&mut state.runtime, AcpEvent::SessionFinished { session_id })?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let acp = buffer
        .acp_state
        .as_ref()
        .ok_or_else(|| "ACP state missing".to_owned())?;
    assert!(
        acp.plan_entries
            .iter()
            .all(|entry| entry.status == PlanEntryStatus::Completed)
    );

    Ok(())
}

#[test]
fn buffer_lookup_is_scoped_to_workspace() -> Result<(), String> {
    let (mut manager, _command_rx) = test_acp_manager();
    let buffer_id = test_buffer_id()?;
    let workspace_id = test_workspace_id()?;
    let mut other_state = ShellState::new().map_err(|error| error.to_string())?;
    let window_id = other_state
        .runtime
        .model()
        .active_window_id()
        .ok_or_else(|| "active window is missing".to_owned())?;
    let other_workspace_id = other_state
        .runtime
        .model_mut()
        .open_workspace(window_id, "other", Some(PathBuf::from("P:\\other")))
        .map_err(|error| error.to_string())?;
    manager
        .workspace_client_buffers
        .insert((workspace_id, "copilot".to_owned()), buffer_id);

    assert_eq!(
        manager.buffer_for_client(workspace_id, "copilot"),
        Some(buffer_id)
    );
    assert_eq!(
        manager.buffer_for_client(other_workspace_id, "copilot"),
        None
    );
    Ok(())
}

#[test]
fn open_permission_request_reorders_queue_for_requested_picker() -> Result<(), String> {
    let (mut manager, _command_rx) = test_acp_manager();
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    manager.queue_permission_request(test_pending_permission_request(1, "session-1", "Read file"));
    manager.queue_permission_request(test_pending_permission_request(
        2,
        "session-1",
        "Write file",
    ));

    manager.open_permission_request(&mut state.runtime, 1)?;
    assert_eq!(
        shell_ui(&state.runtime)?.picker_kind(),
        Some(PickerKind::AcpPermission { request_id: 1 })
    );

    shell_ui_mut(&mut state.runtime)?.close_picker();
    manager.permission_picker_closed(1);
    manager.open_permission_request(&mut state.runtime, 2)?;

    assert_eq!(
        manager
            .pending_permissions
            .iter()
            .map(|request| request.request_id)
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert_eq!(manager.active_permission_request, Some(2));
    assert!(!manager.permission_queue_paused);
    assert_eq!(
        shell_ui(&state.runtime)?.picker_kind(),
        Some(PickerKind::AcpPermission { request_id: 2 })
    );
    Ok(())
}

#[test]
fn session_list_picker_preserves_source_order() -> Result<(), String> {
    let (mut manager, _command_rx) = test_acp_manager();
    let buffer_id = test_buffer_id()?;
    let mut state = ShellState::new().map_err(|error| error.to_string())?;

    manager.handle_event(
        &mut state.runtime,
        AcpEvent::SessionList {
            buffer_id,
            sessions: vec![
                SessionInfo::new(
                    "session-2",
                    std::env::current_dir().map_err(|error| error.to_string())?,
                )
                .title("Zulu")
                .updated_at("2026-03-31T23:59:59Z"),
                SessionInfo::new(
                    "session-1",
                    std::env::current_dir().map_err(|error| error.to_string())?,
                )
                .title("Alpha")
                .updated_at("2026-03-01T00:00:00Z"),
            ],
        },
    )?;

    let picker = shell_ui(&state.runtime)?
        .picker()
        .ok_or_else(|| "missing session picker".to_owned())?;
    assert_eq!(
        picker
            .session()
            .matches()
            .iter()
            .map(|matched| matched.item().label())
            .collect::<Vec<_>>(),
        vec!["Zulu", "Alpha"]
    );
    Ok(())
}

#[test]
fn pending_slash_completion_trigger_rejects_multiline_input() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let (_workspace_id, buffer_id) = install_acp_test_buffer(&mut state)?;
    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
        let input = buffer
            .input_field_mut()
            .ok_or_else(|| "ACP input field missing".to_owned())?;
        input.set_text("/fix\nmore context");
    }

    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "ACP shell buffer missing".to_owned())?;
    assert!(pending_slash_completion_trigger(buffer, PendingSlashTrigger::Auto).is_none());
    assert!(pending_slash_completion_trigger(buffer, PendingSlashTrigger::Manual).is_none());
    Ok(())
}

#[test]
fn acp_slash_completion_query_requires_leading_single_token_command() {
    assert_eq!(acp_slash_completion_query("/"), Some(""));
    assert_eq!(acp_slash_completion_query("/fix"), Some("fix"));
    assert!(acp_slash_completion_query("/fix more").is_none());
    assert!(acp_slash_completion_query("/fix\nmore").is_none());
    assert!(acp_slash_completion_query("i have this code //").is_none());
    assert!(acp_slash_completion_query(" //").is_none());
}

#[test]
fn acp_file_mention_at_cursor_requires_token_start() {
    let mention = acp_file_mention_at_cursor("@", 1).expect("bare @");
    assert_eq!(mention.at_char, 0);
    assert_eq!(mention.end_char, 1);
    assert_eq!(mention.query, "");

    let mention = acp_file_mention_at_cursor("look at @src/main.rs", 20).expect("path");
    assert_eq!(mention.query, "src/main.rs");
    assert_eq!(mention.at_char, 8);

    assert!(acp_file_mention_at_cursor("user@host.com", 13).is_none());
    assert!(acp_file_mention_at_cursor("look at src", 11).is_none());
}

#[test]
fn compose_acp_prompt_blocks_splits_file_mentions_into_resource_links() {
    let root = PathBuf::from("/workspace");
    let blocks = compose_acp_prompt_blocks("please review @src/main.rs thanks", Some(&root), &[]);
    assert_eq!(blocks.len(), 3);
    match &blocks[0] {
        ContentBlock::Text(text) => assert_eq!(text.text, "please review "),
        other => panic!("expected leading text, got {other:?}"),
    }
    match &blocks[1] {
        ContentBlock::ResourceLink(link) => {
            assert_eq!(link.name, "src/main.rs");
            assert!(link.uri.ends_with("/src/main.rs") || link.uri.contains("src/main.rs"));
            assert!(link.uri.starts_with("file:"));
        }
        other => panic!("expected resource link, got {other:?}"),
    }
    match &blocks[2] {
        ContentBlock::Text(text) => assert_eq!(text.text, " thanks"),
        other => panic!("expected trailing text, got {other:?}"),
    }
}

#[test]
fn compose_acp_prompt_blocks_embeds_pasted_images() {
    let images = vec![AcpPastedImage {
        id: 1,
        name: "Image".to_owned(),
        mime_type: "image/png".to_owned(),
        data: "abc123".to_owned(),
    }];
    let blocks = compose_acp_prompt_blocks("see ![Image](acp-image:1)", None, &images);
    assert_eq!(blocks.len(), 2);
    match &blocks[0] {
        ContentBlock::Text(text) => assert_eq!(text.text, "see "),
        other => panic!("expected leading text, got {other:?}"),
    }
    match &blocks[1] {
        ContentBlock::Image(image) => {
            assert_eq!(image.data, "abc123");
            assert_eq!(image.mime_type, "image/png");
            assert_eq!(
                image.uri.as_deref(),
                Some("volt://agent/pasted-image?name=Image")
            );
        }
        other => panic!("expected image block, got {other:?}"),
    }
}
