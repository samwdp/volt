    use std::{
        env, fs,
        io::{Read, Write, pipe},
        net::TcpListener,
        path::{Path, PathBuf},
        process::{Command, Stdio},
        sync::{
            Arc, Mutex,
            atomic::{AtomicBool, AtomicU64, Ordering},
        },
        thread,
        time::{Duration, Instant},
    };

    use dap_types::DisconnectArguments;
    use serde_json::{Value, json};

    use super::{
        DapClientError, DapClientManager, DapLogDirection, DapSessionEvent, DapVariablePath,
        read_frame,
    };
    use crate::{
        BreakpointState, BreakpointToggle, DebugAdapterRegistry, DebugAdapterSpec,
        DebugAdapterTransport, DebugConfiguration, DebugRequestKind,
    };

    fn write_frame_to(writer: &mut impl Write, message: &Value) {
        let encoded = serde_json::to_vec(message).expect("encode");
        write!(writer, "Content-Length: {}\r\n\r\n", encoded.len()).expect("header");
        writer.write_all(&encoded).expect("body");
        writer.flush().expect("flush");
    }

    fn json_value_contains_null(value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::Array(items) => items.iter().any(json_value_contains_null),
            Value::Object(map) => map.values().any(json_value_contains_null),
            _ => false,
        }
    }

    fn fake_variables_for_reference(reference: u64, running: bool) -> Vec<Value> {
        match reference {
            2 => vec![
                json!({
                    "name": "Name",
                    "value": "\"Ada\"",
                    "type": "string",
                    "variablesReference": 0
                }),
                json!({
                    "name": "Address",
                    "value": "Address { ... }",
                    "type": "Address",
                    "variablesReference": 3
                }),
            ],
            3 => vec![json!({
                "name": "City",
                "value": "\"London\"",
                "type": "string",
                "variablesReference": 0
            })],
            _ => vec![
                json!({
                    "name": "x",
                    "value": "42",
                    "type": "i32",
                    "variablesReference": 0
                }),
                json!({
                    "name": "running",
                    "value": if running { "true" } else { "false" },
                    "type": "bool",
                    "variablesReference": 0
                }),
                json!({
                    "name": "person",
                    "value": "Person { ... }",
                    "type": "Person",
                    "variablesReference": 2
                }),
            ],
        }
    }

    fn fake_adapter_loop(
        reader: impl Read,
        mut writer: impl Write,
        last_disconnect: Arc<Mutex<Option<DisconnectArguments>>>,
    ) {
        let mut reader = std::io::BufReader::new(reader);
        let mut seq = 1_u64;
        let mut stopped_line = 10_u64;
        let mut program_path = "main.rs".to_owned();
        let mut running = false;
        while let Ok(message) = read_frame(&mut reader) {
            let command = message
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let request_seq = message.get("seq").and_then(Value::as_u64).unwrap_or(0);
            if matches!(command.as_str(), "continue" | "next" | "stepIn" | "stepOut")
                && message
                    .get("arguments")
                    .is_some_and(json_value_contains_null)
            {
                let response = json!({
                    "seq": seq,
                    "type": "response",
                    "request_seq": request_seq,
                    "success": false,
                    "command": command,
                    "message": "null optional fields are not allowed"
                });
                seq += 1;
                write_frame_to(&mut writer, &response);
                continue;
            }
            match command.as_str() {
                "initialize" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "initialize",
                        "body": {
                            "supportsConfigurationDoneRequest": true,
                            "supportTerminateDebuggee": true,
                            "supportsRestartRequest": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "initialized",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "configurationDone" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "configurationDone",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "launch" | "attach" => {
                    if let Some(program) = message
                        .pointer("/arguments/program")
                        .and_then(Value::as_str)
                    {
                        program_path = program.to_owned();
                    }
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": command,
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    stopped_line = 10;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "entry",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "setBreakpoints" => {
                    let breakpoints = message
                        .pointer("/arguments/breakpoints")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default();
                    let body_breakpoints: Vec<Value> = breakpoints
                        .iter()
                        .map(|bp| {
                            let line = bp.get("line").and_then(Value::as_u64).unwrap_or(1);
                            // Verify odd lines; leave even lines unverified for tests.
                            json!({
                                "verified": line % 2 == 1,
                                "line": line
                            })
                        })
                        .collect();
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "setBreakpoints",
                        "body": { "breakpoints": body_breakpoints }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "continue" => {
                    // Omit `body` like adapters that send success with no ContinueResponse fields.
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "continue"
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = true;
                    if program_path.contains("exit-on-continue") {
                        let exited = json!({
                            "seq": seq,
                            "type": "event",
                            "event": "exited",
                            "body": { "exitCode": 0 }
                        });
                        seq += 1;
                        write_frame_to(&mut writer, &exited);
                        let terminated = json!({
                            "seq": seq,
                            "type": "event",
                            "event": "terminated",
                            "body": {}
                        });
                        write_frame_to(&mut writer, &terminated);
                        break;
                    }
                }
                "next" | "stepIn" | "stepOut" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": command,
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    stopped_line += 1;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "step",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "pause" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "pause",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "pause",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "restart" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "restart",
                        "body": {}
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                    running = false;
                    stopped_line = 10;
                    let event = json!({
                        "seq": seq,
                        "type": "event",
                        "event": "stopped",
                        "body": {
                            "reason": "entry",
                            "threadId": 1,
                            "allThreadsStopped": true
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &event);
                }
                "stackTrace" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "stackTrace",
                        "body": {
                            "stackFrames": [{
                                "id": 1,
                                "name": "main",
                                "source": {
                                    "name": "main.rs",
                                    "path": program_path
                                },
                                "line": stopped_line,
                                "column": 1
                            }, {
                                "id": 2,
                                "name": "caller",
                                "source": {
                                    "name": "main.rs",
                                    "path": program_path
                                },
                                "line": stopped_line.saturating_add(10),
                                "column": 1
                            }],
                            "totalFrames": 2
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "threads" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "threads",
                        "body": {
                            "threads": [
                                { "id": 1, "name": "main" },
                                { "id": 2, "name": "worker" }
                            ]
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "evaluate" => {
                    let expression = message
                        .pointer("/arguments/expression")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    let frame_id = message
                        .pointer("/arguments/frameId")
                        .and_then(Value::as_u64)
                        .unwrap_or(1);
                    if expression == "fail" {
                        let response = json!({
                            "seq": seq,
                            "type": "response",
                            "request_seq": request_seq,
                            "success": false,
                            "command": "evaluate",
                            "message": "cannot evaluate"
                        });
                        seq += 1;
                        write_frame_to(&mut writer, &response);
                        continue;
                    }
                    let (result, type_name, variables_reference) = if expression == "person" {
                        ("Person { ... }".to_owned(), Some("Person"), 2_u64)
                    } else {
                        (
                            format!("{expression}@{frame_id}={stopped_line}"),
                            None,
                            0_u64,
                        )
                    };
                    let mut body = json!({
                        "result": result,
                        "variablesReference": variables_reference
                    });
                    if let Some(type_name) = type_name
                        && let Some(object) = body.as_object_mut()
                    {
                        object.insert("type".to_owned(), Value::String(type_name.to_owned()));
                    }
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "evaluate",
                        "body": body
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "scopes" => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "scopes",
                        "body": {
                            "scopes": [{
                                "name": "Locals",
                                "variablesReference": 1,
                                "expensive": false
                            }]
                        }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "variables" => {
                    let reference = message
                        .pointer("/arguments/variablesReference")
                        .and_then(Value::as_u64)
                        .unwrap_or(1);
                    let variables = fake_variables_for_reference(reference, running);
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "variables",
                        "body": { "variables": variables }
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
                "disconnect" => {
                    let args = message.get("arguments").cloned().unwrap_or(Value::Null);
                    let parsed: DisconnectArguments =
                        serde_json::from_value(args).unwrap_or(DisconnectArguments {
                            restart: None,
                            terminate_debuggee: None,
                            suspend_debuggee: None,
                        });
                    if let Ok(mut slot) = last_disconnect.lock() {
                        *slot = Some(parsed);
                    }
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": true,
                        "command": "disconnect",
                        "body": {}
                    });
                    write_frame_to(&mut writer, &response);
                    break;
                }
                _ => {
                    let response = json!({
                        "seq": seq,
                        "type": "response",
                        "request_seq": request_seq,
                        "success": false,
                        "command": command,
                        "message": "unsupported"
                    });
                    seq += 1;
                    write_frame_to(&mut writer, &response);
                }
            }
        }
    }

    fn tcp_fake_spec(port: u16) -> DebugAdapterSpec {
        DebugAdapterSpec::new("fake-dap", "rust", ["rs"], "", [] as [&str; 0])
            .with_transport(DebugAdapterTransport::Tcp {
                host: "127.0.0.1".to_owned(),
                port,
            })
            .with_preference(10)
    }

    /// Prefer TCP fake for full client protocol tests; framing covered separately for stdio.
    fn start_tcp_fake() -> (
        u16,
        Arc<Mutex<Option<DisconnectArguments>>>,
        thread::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let port = listener.local_addr().expect("addr").port();
        let last_disconnect = Arc::new(Mutex::new(None));
        let last_disconnect_thread = Arc::clone(&last_disconnect);
        let handle = thread::spawn(move || {
            let (stream, _) = listener.accept().expect("accept");
            let reader = stream.try_clone().expect("clone");
            fake_adapter_loop(reader, stream, last_disconnect_thread);
        });
        (port, last_disconnect, handle)
    }

    #[test]
    fn client_initialize_launch_disconnect_against_fake_tcp_adapter() {
        let (port, last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");

        let manager = DapClientManager::new(registry);
        let info = manager
            .start(
                7,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("launch demo", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");
        assert_eq!(info.workspace_id(), 7);
        assert_eq!(info.adapter_id(), "fake-dap");
        assert!(info.support_terminate_debuggee());
        assert!(manager.session_info(7).expect("info").is_some());

        let stopped = manager.stop_session(7).expect("stop");
        assert_eq!(stopped.adapter_id(), "fake-dap");
        assert!(manager.session_info(7).expect("info").is_none());

        // Give reader/fake a moment to record disconnect.
        thread::sleep(Duration::from_millis(50));
        let disconnect = last_disconnect
            .lock()
            .expect("lock")
            .clone()
            .expect("disconnect args");
        assert_eq!(disconnect.terminate_debuggee, Some(true));

        let _ = fake.join();
        let log = manager.log_snapshot();
        assert!(log.entries().iter().any(|entry| {
            entry.direction() == DapLogDirection::Send && entry.message().contains("initialize")
        }));
        assert!(
            log.entries().iter().any(|entry| {
                entry.direction() == DapLogDirection::Send
                    && entry.message().contains("configurationDone")
            }),
            "SharpDbg-style adapters require configurationDone after setBreakpoints"
        );
    }

    #[test]
    fn debug_stop_after_attach_leaves_process_running() {
        let (port, last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");

        let manager = DapClientManager::new(registry);
        manager
            .start(
                3,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("attach demo", DebugRequestKind::Attach),
            )
            .expect("start");
        manager.stop_session(3).expect("stop");
        thread::sleep(Duration::from_millis(50));
        let disconnect = last_disconnect
            .lock()
            .expect("lock")
            .clone()
            .expect("disconnect args");
        assert_eq!(disconnect.terminate_debuggee, Some(false));
        let _ = fake.join();
    }

    #[test]
    fn one_session_per_workspace_enforced() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                1,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("one", DebugRequestKind::Launch),
            )
            .expect("start");
        let err = manager
            .start(
                1,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("two", DebugRequestKind::Launch),
            )
            .expect_err("second start");
        assert!(matches!(err, DapClientError::SessionExists(1)));
        manager.stop_session(1).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn missing_adapter_binary_is_clear() {
        let mut registry = DebugAdapterRegistry::new();
        registry
            .register(DebugAdapterSpec::new(
                "missing",
                "rust",
                ["rs"],
                "volt-definitely-missing-dap-adapter-xyz",
                [] as [&str; 0],
            ))
            .expect("register");
        let manager = DapClientManager::new(registry);
        let error = manager
            .start(
                9,
                Some("missing"),
                None,
                DebugConfiguration::new("missing", DebugRequestKind::Launch),
            )
            .expect_err("missing binary");
        let message = error.to_string();
        assert!(message.contains("volt-definitely-missing-dap-adapter-xyz"));
        assert!(message.contains("install the adapter"));
        assert!(matches!(error, DapClientError::AdapterMissing { .. }));
    }

    #[test]
    fn toggle_breakpoint_without_session_stays_pending() {
        let manager = DapClientManager::new(DebugAdapterRegistry::new());
        assert_eq!(
            manager.toggle_breakpoint(1, "main.rs", 3).expect("toggle"),
            BreakpointToggle::Added
        );
        let listed = manager.list_breakpoints(1).expect("list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].state(), BreakpointState::Pending);
        assert_eq!(
            manager
                .toggle_breakpoint(1, "main.rs", 3)
                .expect("toggle off"),
            BreakpointToggle::Removed
        );
        assert!(manager.list_breakpoints(1).expect("list").is_empty());
    }

    #[test]
    fn start_session_syncs_stored_breakpoints_to_adapter() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .toggle_breakpoint(4, "src/main.rs", 5)
            .expect("toggle odd");
        manager
            .toggle_breakpoint(4, "src/main.rs", 8)
            .expect("toggle even");

        manager
            .start(
                4,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("sync", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");

        let bps = manager
            .breakpoints_for_path(4, std::path::Path::new("src/main.rs"))
            .expect("bps");
        assert_eq!(bps.len(), 2);
        assert_eq!(bps[0].line(), 5);
        assert_eq!(bps[0].state(), BreakpointState::Verified);
        assert_eq!(bps[1].line(), 8);
        assert_eq!(bps[1].state(), BreakpointState::Unverified);

        let log = manager.log_snapshot();
        assert!(log.entries().iter().any(|entry| {
            entry.direction() == DapLogDirection::Send && entry.message().contains("setBreakpoints")
        }));

        manager.stop_session(4).expect("stop");
        let pending = manager.list_breakpoints(4).expect("list");
        assert!(
            pending
                .iter()
                .all(|bp| bp.state() == BreakpointState::Pending)
        );
        let _ = fake.join();
    }

    #[test]
    fn live_toggle_calls_set_breakpoints() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                5,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("live", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");

        manager
            .toggle_breakpoint(5, "lib.rs", 11)
            .expect("live toggle");
        let bps = manager
            .breakpoints_for_path(5, std::path::Path::new("lib.rs"))
            .expect("bps");
        assert_eq!(bps.len(), 1);
        assert_eq!(bps[0].state(), BreakpointState::Verified);

        let set_count = manager
            .log_snapshot()
            .entries()
            .iter()
            .filter(|entry| {
                entry.direction() == DapLogDirection::Send
                    && entry.message().contains("setBreakpoints")
            })
            .count();
        assert!(set_count >= 1);

        manager.stop_session(5).expect("stop");
        let _ = fake.join();
    }

    fn wait_for_stopped(manager: &DapClientManager, workspace_id: u64) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            let events = manager.drain_events().expect("drain");
            if events.iter().any(|event| {
                matches!(
                    event,
                    DapSessionEvent::Stopped {
                        workspace_id: id
                    } if *id == workspace_id
                )
            }) {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out waiting for stopped event");
    }

    fn wait_for_terminated(manager: &DapClientManager, workspace_id: u64) {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            let events = manager.drain_events().expect("drain");
            if events.iter().any(|event| {
                matches!(
                    event,
                    DapSessionEvent::Terminated {
                        workspace_id: id
                    } if *id == workspace_id
                )
            }) {
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out waiting for terminated event");
    }

    fn assert_control_requests_omit_nulls(manager: &DapClientManager) {
        let sends: Vec<String> = manager
            .log_snapshot()
            .entries()
            .iter()
            .filter(|entry| entry.direction() == DapLogDirection::Send)
            .map(|entry| entry.message().to_owned())
            .filter(|message| {
                message.contains("\"command\":\"continue\"")
                    || message.contains("\"command\":\"next\"")
                    || message.contains("\"command\":\"stepIn\"")
                    || message.contains("\"command\":\"stepOut\"")
            })
            .collect();
        assert!(
            !sends.is_empty(),
            "expected continue/step requests in the DAP send log"
        );
        for message in &sends {
            assert!(
                !message.contains(":null"),
                "control request must omit null optionals: {message}"
            );
        }
    }

    #[test]
    fn continue_step_pause_and_locals_against_fake_adapter() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                11,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("step", DebugRequestKind::Launch)
                    .with_target_program("src/main.rs"),
            )
            .expect("start");

        wait_for_stopped(&manager, 11);
        let snapshot = manager.refresh_stopped_snapshot(11).expect("snapshot");
        assert_eq!(snapshot.reason(), "entry");
        let position = snapshot.position().expect("position");
        assert_eq!(position.path().as_os_str(), "src/main.rs");
        assert_eq!(position.line(), 10);
        assert!(
            snapshot
                .locals()
                .iter()
                .any(|local| local.name() == "x" && local.value() == "42"),
            "locals should include x=42"
        );

        manager.continue_session(11).expect("continue");
        manager.pause_session(11).expect("pause");
        wait_for_stopped(&manager, 11);
        let paused = manager.refresh_stopped_snapshot(11).expect("paused");
        assert_eq!(paused.reason(), "pause");

        manager.step_over(11).expect("step");
        wait_for_stopped(&manager, 11);
        let stepped = manager.refresh_stopped_snapshot(11).expect("stepped");
        assert_eq!(stepped.reason(), "step");
        assert_eq!(stepped.position().expect("pos").line(), 11);

        manager.step_into(11).expect("into");
        wait_for_stopped(&manager, 11);
        manager.step_out(11).expect("out");
        wait_for_stopped(&manager, 11);
        assert_eq!(
            manager
                .refresh_stopped_snapshot(11)
                .expect("out snap")
                .position()
                .expect("pos")
                .line(),
            13
        );
        assert_control_requests_omit_nulls(&manager);

        manager.stop_session(11).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn expand_collapse_and_reapply_nested_locals_and_watches() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                15,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("expand", DebugRequestKind::Launch)
                    .with_target_program("src/main.rs"),
            )
            .expect("start");

        wait_for_stopped(&manager, 15);
        let snapshot = manager.refresh_stopped_snapshot(15).expect("snapshot");
        let person = snapshot
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(person.expandable());
        assert!(!person.expanded());
        assert!(person.children().is_empty());

        let expanded = manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person"]))
            .expect("expand person");
        let person = expanded
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(person.expanded());
        assert_eq!(person.children().len(), 2);
        assert_eq!(person.children()[0].name(), "Name");
        assert_eq!(person.children()[0].value(), "\"Ada\"");
        assert_eq!(person.children()[1].name(), "Address");
        assert!(person.children()[1].expandable());

        let nested = manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person", "Address"]))
            .expect("expand address");
        let address = nested
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person")
            .children()
            .iter()
            .find(|child| child.name() == "Address")
            .expect("address");
        assert_eq!(address.children().len(), 1);
        assert_eq!(address.children()[0].name(), "City");

        let collapsed = manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person"]))
            .expect("collapse person");
        let person = collapsed
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(!person.expanded());
        assert!(person.children().is_empty());

        manager
            .toggle_variable_expand(15, &DapVariablePath::locals(["person"]))
            .expect("re-expand");
        manager.step_over(15).expect("step");
        wait_for_stopped(&manager, 15);
        let stepped = manager.refresh_stopped_snapshot(15).expect("stepped");
        let person = stepped
            .locals()
            .iter()
            .find(|local| local.name() == "person")
            .expect("person");
        assert!(
            person.expanded() && person.children().iter().any(|child| child.name() == "Name"),
            "expand path must survive a step: {:?}",
            person.children()
        );

        manager.add_expression(15, "person").expect("watch");
        let with_watch = manager.refresh_stopped_snapshot(15).expect("watch snap");
        assert!(
            with_watch
                .watches()
                .iter()
                .any(|watch| watch.expression() == "person" && watch.expandable()),
            "person watch should be expandable: {:?}",
            with_watch.watches()
        );
        let watch_expanded = manager
            .toggle_variable_expand(15, &DapVariablePath::watch("person", Vec::<String>::new()))
            .expect("expand watch");
        let watch = watch_expanded
            .watches()
            .iter()
            .find(|watch| watch.expression() == "person")
            .expect("watch");
        assert!(watch.expanded());
        assert!(watch.children().iter().any(|child| child.name() == "Name"));

        manager.stop_session(15).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn continue_to_process_exit_queues_terminated() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                14,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("exit", DebugRequestKind::Launch)
                    .with_target_program("exit-on-continue.rs"),
            )
            .expect("start");

        wait_for_stopped(&manager, 14);
        manager.continue_session(14).expect("continue");
        wait_for_terminated(&manager, 14);
        manager.stop_session(14).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn restart_reuses_configuration_against_fake_adapter() {
        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                12,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("restart-me", DebugRequestKind::Launch)
                    .with_target_program("target/debug/demo"),
            )
            .expect("start");
        wait_for_stopped(&manager, 12);
        manager.step_over(12).expect("step");
        wait_for_stopped(&manager, 12);
        assert_eq!(
            manager
                .refresh_stopped_snapshot(12)
                .expect("before")
                .position()
                .expect("pos")
                .line(),
            11
        );

        let info = manager.restart_session(12).expect("restart");
        assert_eq!(info.configuration_name(), "restart-me");
        assert!(manager.session_info(12).expect("still live").is_some());
        wait_for_stopped(&manager, 12);
        let after = manager.refresh_stopped_snapshot(12).expect("after");
        assert_eq!(after.position().expect("pos").line(), 10);

        manager.stop_session(12).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn watches_eval_switch_context_and_breakpoint_extras_against_fake_adapter() {
        use super::DapEvaluateContext;

        let (port, _last_disconnect, fake) = start_tcp_fake();
        let mut registry = DebugAdapterRegistry::new();
        registry.register(tcp_fake_spec(port)).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .start(
                13,
                Some("fake-dap"),
                None,
                DebugConfiguration::new("polish", DebugRequestKind::Launch)
                    .with_target_program("src/main.rs"),
            )
            .expect("start");
        wait_for_stopped(&manager, 13);

        manager
            .set_breakpoint_extras(
                13,
                "src/main.rs",
                11,
                Some(Some("x > 0".to_owned())),
                Some(Some("3".to_owned())),
                Some(Some("log {x}".to_owned())),
            )
            .expect("bp extras");
        let bp = manager
            .list_breakpoints(13)
            .expect("list")
            .into_iter()
            .find(|bp| bp.line() == 11)
            .expect("bp");
        assert_eq!(bp.condition(), Some("x > 0"));
        assert_eq!(bp.hit_condition(), Some("3"));
        assert_eq!(bp.log_message(), Some("log {x}"));

        manager.add_expression(13, "x").expect("add watch");
        let snapshot = manager.refresh_stopped_snapshot(13).expect("snap");
        assert!(
            snapshot
                .watches()
                .iter()
                .any(|watch| watch.expression() == "x" && watch.ok()),
            "watch should evaluate: {:?}",
            snapshot.watches()
        );

        let eval = manager
            .evaluate(13, "y", DapEvaluateContext::Repl)
            .expect("eval");
        assert!(eval.ok());
        assert!(eval.value().contains("y@"));

        let threads = manager.list_threads(13).expect("threads");
        assert_eq!(threads.len(), 2);
        manager.switch_thread(13, 2).expect("switch thread");
        assert_eq!(
            manager
                .stopped_snapshot(13)
                .expect("snap")
                .expect("present")
                .thread_id(),
            2
        );

        let frames = manager.list_stack_frames(13).expect("frames");
        assert!(frames.len() >= 2);
        let switched = manager.switch_stack_frame(13, 2).expect("switch frame");
        assert_eq!(switched.frame_id(), Some(2));
        assert_eq!(switched.position().expect("pos").line(), 20);

        assert!(manager.remove_expression(13, "x").expect("remove"));
        assert!(manager.list_expressions(13).expect("list").is_empty());

        manager.stop_session(13).expect("stop");
        let _ = fake.join();
    }

    #[test]
    fn stdio_framing_round_trips_initialize() {
        let (client_reader, adapter_writer) = pipe().expect("pipe");
        let (adapter_reader, mut client_writer) = pipe().expect("pipe");
        let last_disconnect = Arc::new(Mutex::new(None));
        let last_disconnect_thread = Arc::clone(&last_disconnect);
        let done = Arc::new(AtomicBool::new(false));
        let done_thread = Arc::clone(&done);
        let fake = thread::spawn(move || {
            fake_adapter_loop(adapter_reader, adapter_writer, last_disconnect_thread);
            done_thread.store(true, Ordering::Release);
        });

        // Drive a minimal handshake using raw frames to prove stdio framing works.
        write_frame_to(
            &mut client_writer,
            &json!({
                "seq": 1,
                "type": "request",
                "command": "initialize",
                "arguments": { "adapterID": "fake" }
            }),
        );
        let mut reader = std::io::BufReader::new(client_reader);
        let response = read_frame(&mut reader).expect("initialize response");
        assert_eq!(response["command"], "initialize");
        assert_eq!(response["success"], true);
        let event = read_frame(&mut reader).expect("initialized event");
        assert_eq!(event["event"], "initialized");

        write_frame_to(
            &mut client_writer,
            &json!({
                "seq": 2,
                "type": "request",
                "command": "launch",
                "arguments": { "program": "demo" }
            }),
        );
        let launch = read_frame(&mut reader).expect("launch response");
        assert_eq!(launch["command"], "launch");
        let stopped = read_frame(&mut reader).expect("stopped event");
        assert_eq!(stopped["event"], "stopped");

        write_frame_to(
            &mut client_writer,
            &json!({
                "seq": 3,
                "type": "request",
                "command": "disconnect",
                "arguments": { "terminateDebuggee": true }
            }),
        );
        let disconnect = read_frame(&mut reader).expect("disconnect response");
        assert_eq!(disconnect["command"], "disconnect");
        let _ = fake.join();
        assert!(done.load(Ordering::Acquire));
        assert_eq!(
            last_disconnect
                .lock()
                .expect("lock")
                .as_ref()
                .and_then(|args| args.terminate_debuggee),
            Some(true)
        );
    }

    #[test]
    fn launch_arguments_always_send_program_path() {
        let config = DebugConfiguration::new("Debug (dotnet)", DebugRequestKind::Launch)
            .with_target_program("bin/Debug/net10.0/App.dll")
            .with_cwd(".");
        let body = super::launch_arguments(&config);
        assert_eq!(body["program"], "bin/Debug/net10.0/App.dll");
        assert_eq!(body["console"], "internalConsole");
        assert!(body.get("projectPath").is_none());
    }

    const STRUCT_CTOR_PROGRAM: &str = r#"Console.WriteLine("Hello, World!");
var a = 1;
var b = new foo();
Console.WriteLine(a);

public struct foo{
public string bar => "bar";
}
"#;

    fn sharpdbg_spec() -> Option<DebugAdapterSpec> {
        Command::new("sharpdbg")
            .arg("--help")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .filter(|status| status.success())
            .map(|_| {
                DebugAdapterSpec::new(
                    "sharpdbg",
                    "csharp",
                    ["cs"],
                    "sharpdbg",
                    ["--interpreter=vscode"],
                )
            })
    }

    fn wait_for_stopped_or_terminated(
        manager: &DapClientManager,
        workspace_id: u64,
        timeout: Duration,
    ) -> DapSessionEvent {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            let events = manager.drain_events().expect("drain");
            if let Some(event) = events.into_iter().find(|event| {
                matches!(
                    event,
                    DapSessionEvent::Stopped {
                        workspace_id: id,
                    } if *id == workspace_id
                ) || matches!(
                    event,
                    DapSessionEvent::Terminated {
                        workspace_id: id,
                    } if *id == workspace_id
                )
            }) {
                return event;
            }
            thread::sleep(Duration::from_millis(20));
        }
        panic!(
            "timed out waiting for stopped/terminated. log:\n{}",
            dap_log_text(manager)
        );
    }

    fn dap_log_text(manager: &DapClientManager) -> String {
        manager
            .log_snapshot()
            .entries()
            .iter()
            .map(|entry| format!("{:?} {}", entry.direction(), entry.message()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn find_named_dll(root: &Path, file_name: &str) -> Option<PathBuf> {
        let mut found = None;
        fn walk(dir: &Path, file_name: &str, found: &mut Option<PathBuf>) {
            let Ok(entries) = fs::read_dir(dir) else {
                return;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, file_name, found);
                } else if path.file_name().is_some_and(|name| name == file_name) {
                    *found = Some(path);
                }
            }
        }
        walk(&root.join("bin").join("Debug"), file_name, &mut found);
        found
    }

    fn build_csharp_fixture(source: &str) -> (PathBuf, PathBuf, PathBuf) {
        static FIXTURE_SEQ: AtomicU64 = AtomicU64::new(0);
        let seq = FIXTURE_SEQ.fetch_add(1, Ordering::Relaxed);
        let root = env::temp_dir().join(format!("volt-sharpdbg-step-{}-{seq}", std::process::id()));
        fs::create_dir_all(&root).expect("temp project");
        fs::write(
            root.join("StepStruct.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk">
  <PropertyGroup>
    <OutputType>Exe</OutputType>
    <TargetFramework>net10.0</TargetFramework>
    <ImplicitUsings>enable</ImplicitUsings>
    <Nullable>enable</Nullable>
    <DebugType>portable</DebugType>
    <Optimize>false</Optimize>
  </PropertyGroup>
</Project>
"#,
        )
        .expect("csproj");
        let program = root.join("Program.cs");
        fs::write(&program, source).expect("program");
        static DOTNET_BUILD: Mutex<()> = Mutex::new(());
        let _build_lock = DOTNET_BUILD
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let build = Command::new("dotnet")
            .args(["build", "-c", "Debug", "--nologo"])
            .current_dir(&root)
            .output()
            .expect("dotnet build");
        assert!(
            build.status.success(),
            "dotnet build failed\nstdout:{}\nstderr:{}",
            String::from_utf8_lossy(&build.stdout),
            String::from_utf8_lossy(&build.stderr)
        );
        let dll = find_named_dll(&root, "StepStruct.dll").expect("built dll");
        (root, program, dll)
    }

    fn start_struct_ctor_session(
        workspace_id: u64,
        source: &str,
    ) -> Option<(DapClientManager, PathBuf)> {
        let spec = sharpdbg_spec()?;
        let (root, program, dll) = build_csharp_fixture(source);
        let mut registry = DebugAdapterRegistry::new();
        registry.register(spec).expect("register");
        let manager = DapClientManager::new(registry);
        manager
            .toggle_breakpoint(workspace_id, program.as_path(), 1)
            .expect("bp");
        manager
            .start(
                workspace_id,
                Some("sharpdbg"),
                None,
                DebugConfiguration::new("Debug (dotnet)", DebugRequestKind::Launch)
                    .with_target_program(&dll)
                    .with_cwd(&root),
            )
            .expect("start");
        let first = wait_for_stopped_or_terminated(&manager, workspace_id, Duration::from_secs(20));
        assert!(
            matches!(first, DapSessionEvent::Stopped { .. }),
            "expected first stop, got {first:?}\n{}",
            dap_log_text(&manager)
        );
        Some((manager, root))
    }

    fn snapshot_line(manager: &DapClientManager, workspace_id: u64) -> Option<u32> {
        manager
            .refresh_stopped_snapshot(workspace_id)
            .ok()
            .and_then(|snapshot| snapshot.position().map(super::DapExecutionPosition::line))
    }

    fn step_over_until_line(
        manager: &DapClientManager,
        workspace_id: u64,
        target: u32,
        budget: usize,
    ) -> Vec<Option<u32>> {
        let mut lines = Vec::new();
        for step in 0..budget {
            let line = snapshot_line(manager, workspace_id);
            lines.push(line);
            if line == Some(target) {
                return lines;
            }
            manager.step_over(workspace_id).expect("step");
            let event =
                wait_for_stopped_or_terminated(manager, workspace_id, Duration::from_secs(20));
            assert!(
                matches!(event, DapSessionEvent::Stopped { .. }),
                "Session ended after {step} steps, lines={lines:?}, event={event:?}\n{}",
                dap_log_text(manager)
            );
        }
        lines
    }

    #[test]
    fn sharpdbg_step_over_struct_construction_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(91, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_ctor = step_over_until_line(&manager, 91, 3, 6);
        assert_eq!(
            to_ctor.last().copied().flatten(),
            Some(3),
            "never reached `var b`; lines={to_ctor:?}\n{}",
            dap_log_text(&manager)
        );
        manager.step_over(91).expect("step over ctor");
        let event = wait_for_stopped_or_terminated(&manager, 91, Duration::from_secs(20));
        assert!(
            matches!(event, DapSessionEvent::Stopped { .. }),
            "Session ended stepping over `new foo()`, lines={to_ctor:?}, event={event:?}\n{}",
            dap_log_text(&manager)
        );
        assert_eq!(
            snapshot_line(&manager, 91),
            Some(4),
            "expected Console.WriteLine(a) after ctor\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(91).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sharpdbg_step_into_struct_construction_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(92, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_ctor = step_over_until_line(&manager, 92, 3, 8);
        assert_eq!(
            to_ctor.last().copied().flatten(),
            Some(3),
            "never reached `var b`; lines={to_ctor:?}\n{}",
            dap_log_text(&manager)
        );
        manager.step_into(92).expect("step into ctor");
        let event = wait_for_stopped_or_terminated(&manager, 92, Duration::from_secs(20));
        assert!(
            matches!(event, DapSessionEvent::Stopped { .. }),
            "Session ended on step-into `new foo()`, lines={to_ctor:?}, event={event:?}\n{}",
            dap_log_text(&manager)
        );
        assert!(
            snapshot_line(&manager, 92).is_some(),
            "lost execution position after step-into `new foo()`\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(92).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sharpdbg_double_step_over_struct_construction_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(93, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_ctor = step_over_until_line(&manager, 93, 3, 6);
        assert_eq!(to_ctor.last().copied().flatten(), Some(3));
        manager.step_over(93).expect("first next");
        manager.step_over(93).expect("second next while running");
        let event = wait_for_stopped_or_terminated(&manager, 93, Duration::from_secs(20));
        assert!(
            matches!(event, DapSessionEvent::Stopped { .. }),
            "Session ended after stacked `next` on `new foo()`, lines={to_ctor:?}, event={event:?}\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(93).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn sharpdbg_step_into_from_entry_keeps_session_through_struct_ctor() {
        let Some((manager, root)) = start_struct_ctor_session(94, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let mut lines = Vec::new();
        for step in 0..12 {
            let line = snapshot_line(&manager, 94);
            lines.push(line);
            if line == Some(4) {
                manager.stop_session(94).expect("stop");
                let _ = fs::remove_dir_all(&root);
                return;
            }
            manager.step_into(94).expect("step into");
            let event = wait_for_stopped_or_terminated(&manager, 94, Duration::from_secs(20));
            assert!(
                matches!(event, DapSessionEvent::Stopped { .. }),
                "Session ended on F11-from-entry after {step} steps, lines={lines:?}, event={event:?}\n{}",
                dap_log_text(&manager)
            );
        }
        manager.stop_session(94).expect("stop");
        let _ = fs::remove_dir_all(&root);
        panic!(
            "never reached Console.WriteLine(a) with F11-only; lines={lines:?}\n{}",
            dap_log_text(&manager)
        );
    }

    #[test]
    fn sharpdbg_expand_struct_local_keeps_session() {
        let Some((manager, root)) = start_struct_ctor_session(95, STRUCT_CTOR_PROGRAM) else {
            return;
        };
        let to_after = step_over_until_line(&manager, 95, 4, 8);
        assert_eq!(
            to_after.last().copied().flatten(),
            Some(4),
            "never reached line after ctor; lines={to_after:?}\n{}",
            dap_log_text(&manager)
        );
        let snapshot = manager.refresh_stopped_snapshot(95).expect("snapshot");
        assert!(
            snapshot
                .locals()
                .iter()
                .any(|local| local.name() == "b" && local.expandable()),
            "expected expandable local `b`: {:?}\n{}",
            snapshot
                .locals()
                .iter()
                .map(|local| (local.name(), local.value(), local.expandable()))
                .collect::<Vec<_>>(),
            dap_log_text(&manager)
        );
        let expanded = manager
            .toggle_variable_expand(95, &super::DapVariablePath::locals(["b"]))
            .expect("expand b");
        let b = expanded
            .locals()
            .iter()
            .find(|local| local.name() == "b")
            .expect("b");
        assert!(
            b.children().iter().any(|child| child.name() == "bar"),
            "expected property `bar` under `b`: {:?}\n{}",
            b.children()
                .iter()
                .map(|child| (child.name(), child.value()))
                .collect::<Vec<_>>(),
            dap_log_text(&manager)
        );
        let drained = manager.drain_events().expect("drain");
        assert!(
            !drained
                .iter()
                .any(|event| matches!(event, DapSessionEvent::Terminated { .. })),
            "Session ended expanding `b.bar`: {drained:?}\n{}",
            dap_log_text(&manager)
        );
        assert!(
            manager.stopped_snapshot(95).ok().flatten().is_some(),
            "lost stopped snapshot expanding `b`\n{}",
            dap_log_text(&manager)
        );
        manager.stop_session(95).expect("stop");
        let _ = fs::remove_dir_all(&root);
    }
