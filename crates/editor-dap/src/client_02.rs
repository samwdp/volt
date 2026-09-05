impl DapSessionHandle {
    fn request<R>(&self, arguments: R::Arguments) -> Result<R::Response, DapClientError>
    where
        R: DapRequest,
        R::Response: for<'de> Deserialize<'de>,
    {
        if self.disconnected.load(Ordering::Acquire) {
            return Err(DapClientError::Protocol(format!(
                "debug adapter `{}` disconnected",
                self.info.adapter_id
            )));
        }

        let seq = self.next_request_id.fetch_add(1, Ordering::AcqRel);
        let (tx, rx) = std::sync::mpsc::channel();
        {
            let mut pending = self
                .pending
                .lock()
                .map_err(|_| DapClientError::LockPoisoned)?;
            pending.insert(seq, PendingResponse { tx });
        }

        let mut message = json!({
            "seq": seq,
            "type": "request",
            "command": R::COMMAND,
            "arguments": arguments,
        });
        if let Some(args) = message.get_mut("arguments") {
            strip_null_fields(args);
        }
        write_frame(
            &self.writer,
            &self.transport_log,
            &self.info.adapter_id,
            &message,
        )?;

        let response_body = rx.recv_timeout(READ_TIMEOUT).map_err(|_| {
            DapClientError::Protocol(format!(
                "timed out waiting for `{}` response from `{}`",
                R::COMMAND,
                self.info.adapter_id
            ))
        })??;

        parse_response_body::<R>(response_body, &self.info.adapter_id)
    }
}

fn send_configuration_done(handle: &DapSessionHandle) -> Result<(), DapClientError> {
    let seq = handle.next_request_id.fetch_add(1, Ordering::AcqRel);
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut pending = handle
            .pending
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?;
        pending.insert(seq, PendingResponse { tx });
    }
    let message = json!({
        "seq": seq,
        "type": "request",
        "command": "configurationDone",
        "arguments": {}
    });
    write_frame(
        &handle.writer,
        &handle.transport_log,
        &handle.info.adapter_id,
        &message,
    )?;
    let _ = rx.recv_timeout(READ_TIMEOUT).map_err(|_| {
        DapClientError::Protocol(format!(
            "timed out waiting for `configurationDone` response from `{}`",
            handle.info.adapter_id
        ))
    })??;
    Ok(())
}

fn parse_response_body<R>(body: Value, adapter_id: &str) -> Result<R::Response, DapClientError>
where
    R: DapRequest,
    R::Response: for<'de> Deserialize<'de>,
{
    // Adapters often omit `body` or send `null` for optional response structs.
    let body = match body {
        Value::Null => Value::Object(serde_json::Map::new()),
        other => other,
    };
    match serde_json::from_value::<R::Response>(body) {
        Ok(value) => Ok(value),
        Err(error) if mem::size_of::<R::Response>() == 0 => {
            // Many adapters send `{}` for unit responses.
            serde_json::from_value(Value::Null).map_err(|_| {
                DapClientError::Protocol(format!(
                    "failed to decode `{}` response from `{adapter_id}`: {error}",
                    R::COMMAND
                ))
            })
        }
        Err(error) => Err(DapClientError::Protocol(format!(
            "failed to decode `{}` response from `{adapter_id}`: {error}",
            R::COMMAND
        ))),
    }
}

fn strip_null_fields(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.retain(|_, child| !child.is_null());
            for child in map.values_mut() {
                strip_null_fields(child);
            }
        }
        Value::Array(items) => {
            for item in items {
                strip_null_fields(item);
            }
        }
        _ => {}
    }
}

fn launch_arguments(configuration: &DebugConfiguration) -> Value {
    let mut body = json!({
        "name": configuration.name(),
        "noDebug": false,
        // SharpDbg defaults here, but be explicit: Volt has no DAP terminal pane yet
        // (`runInTerminal` is ignored). `externalTerminal` would pop a console window.
        "console": "internalConsole",
    });
    if let Some(program) = configuration.target_program() {
        // SharpDbg.Cli (dotnet tool) requires `program` (DLL/EXE). VS Code's
        // projectPath helper is extension-side only and is ignored here.
        body["program"] = Value::String(program.display().to_string());
    }
    if let Some(cwd) = configuration.cwd() {
        body["cwd"] = Value::String(cwd.display().to_string());
    }
    if !configuration.args().is_empty() {
        body["args"] = Value::Array(
            configuration
                .args()
                .iter()
                .cloned()
                .map(Value::String)
                .collect(),
        );
    }
    body
}

fn attach_arguments(configuration: &DebugConfiguration) -> Value {
    let mut body = json!({
        "name": configuration.name(),
    });
    if let Some(program) = configuration.target_program() {
        body["program"] = Value::String(program.display().to_string());
    }
    if let Some(cwd) = configuration.cwd() {
        body["cwd"] = Value::String(cwd.display().to_string());
    }
    if let Some(process_id) = configuration.process_id() {
        body["processId"] = Value::Number(process_id.into());
    }
    body
}

fn wait_for_initialized(
    initialized: &Arc<Mutex<bool>>,
    timeout: Duration,
) -> Result<(), DapClientError> {
    let deadline = Instant::now() + timeout;
    loop {
        if *initialized
            .lock()
            .map_err(|_| DapClientError::LockPoisoned)?
        {
            return Ok(());
        }
        if Instant::now() >= deadline {
            return Err(DapClientError::Protocol(
                "timed out waiting for DAP `initialized` event".to_owned(),
            ));
        }
        thread::sleep(Duration::from_millis(5));
    }
}

type TransportEnds = (Box<dyn Write + Send>, Box<dyn Read + Send>, Option<Child>);

fn connect_transport(
    adapter: &DebugAdapterSpec,
    transport: &DebugAdapterTransport,
) -> Result<TransportEnds, DapClientError> {
    match transport {
        DebugAdapterTransport::Stdio => {
            let mut child = spawn_adapter_command(adapter)?;
            let stdin = child.stdin.take().ok_or_else(|| {
                DapClientError::Protocol(format!(
                    "debug adapter `{}` is missing stdin pipe",
                    adapter.id()
                ))
            })?;
            let stdout = child.stdout.take().ok_or_else(|| {
                DapClientError::Protocol(format!(
                    "debug adapter `{}` is missing stdout pipe",
                    adapter.id()
                ))
            })?;
            Ok((Box::new(stdin), Box::new(stdout), Some(child)))
        }
        DebugAdapterTransport::Tcp { host, port } => {
            // Empty program means connect-only (adapter already listening), used by tests
            // and remote adapters.
            let child = if adapter.program().is_empty() {
                None
            } else {
                Some(spawn_adapter_command(adapter)?)
            };
            let stream = connect_tcp(host, *port, child.is_some())?;
            let reader = stream
                .try_clone()
                .map_err(|error| DapClientError::Io(error.to_string()))?;
            Ok((Box::new(stream), Box::new(reader), child))
        }
    }
}

fn connect_tcp(host: &str, port: u16, expect_retry: bool) -> Result<TcpStream, DapClientError> {
    let address = format!("{host}:{port}");
    let deadline = Instant::now() + TCP_CONNECT_TIMEOUT;
    let mut last_error = None;
    while Instant::now() < deadline {
        match TcpStream::connect(&address) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(READ_TIMEOUT))
                    .map_err(|error| DapClientError::Io(error.to_string()))?;
                stream
                    .set_write_timeout(Some(READ_TIMEOUT))
                    .map_err(|error| DapClientError::Io(error.to_string()))?;
                return Ok(stream);
            }
            Err(error) => {
                last_error = Some(error);
                if !expect_retry {
                    break;
                }
                thread::sleep(TCP_CONNECT_RETRY);
            }
        }
    }
    Err(DapClientError::Io(format!(
        "failed to connect to debug adapter at `{address}`: {}",
        last_error
            .map(|error| error.to_string())
            .unwrap_or_else(|| "timed out".to_owned())
    )))
}

fn configure_adapter_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn spawn_adapter_command(adapter: &DebugAdapterSpec) -> Result<Child, DapClientError> {
    let mut command = Command::new(adapter.program());
    configure_adapter_command(&mut command);
    command
        .args(adapter.args())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut env = Vec::new();
    editor_tool_install::merge_effective_path(&mut env);
    for (key, value) in env {
        command.env(key, value);
    }
    match command.spawn() {
        Ok(child) => Ok(child),
        Err(error) => Err(DapClientError::AdapterMissing {
            adapter_id: adapter.id().to_owned(),
            program: adapter.program().to_owned(),
            detail: error.to_string(),
        }),
    }
}

struct DapReaderSession {
    workspace_id: u64,
    adapter_id: String,
    pending: Arc<Mutex<BTreeMap<u64, PendingResponse>>>,
    disconnected: Arc<AtomicBool>,
    transport_log: TransportLog,
    initialized: Arc<Mutex<bool>>,
    stop_state: Arc<Mutex<SessionStopState>>,
    events: Arc<Mutex<VecDeque<DapSessionEvent>>>,
}

fn spawn_reader_thread(reader: Box<dyn Read + Send>, session: DapReaderSession) -> JoinHandle<()> {
    thread::spawn(move || {
        let DapReaderSession {
            workspace_id,
            adapter_id,
            pending,
            disconnected,
            transport_log,
            initialized,
            stop_state,
            events,
        } = session;
        let mut reader = BufReader::new(reader);
        loop {
            if disconnected.load(Ordering::Acquire) {
                break;
            }
            match read_frame(&mut reader) {
                Ok(value) => {
                    record_transport_message(
                        &transport_log,
                        &adapter_id,
                        DapLogDirection::Receive,
                        &value,
                    );
                    let message_type = value
                        .get("type")
                        .and_then(Value::as_str)
                        .unwrap_or_default();
                    match message_type {
                        "response" => {
                            let request_seq = value
                                .get("request_seq")
                                .and_then(Value::as_u64)
                                .unwrap_or_default();
                            let success = value
                                .get("success")
                                .and_then(Value::as_bool)
                                .unwrap_or(false);
                            let body = value.get("body").cloned().unwrap_or(Value::Null);
                            let result = if success {
                                Ok(body)
                            } else {
                                let message = value
                                    .get("message")
                                    .and_then(Value::as_str)
                                    .unwrap_or("DAP request failed");
                                Err(DapClientError::Protocol(format!(
                                    "debug adapter `{adapter_id}` rejected request: {message}"
                                )))
                            };
                            if let Ok(mut pending) = pending.lock()
                                && let Some(pending_response) = pending.remove(&request_seq)
                            {
                                let _ = pending_response.tx.send(result);
                            }
                        }
                        "event" => {
                            let event = value
                                .get("event")
                                .and_then(Value::as_str)
                                .unwrap_or_default();
                            match event {
                                "initialized" => {
                                    if let Ok(mut flag) = initialized.lock() {
                                        *flag = true;
                                    }
                                }
                                "stopped" => {
                                    let thread_id = value
                                        .pointer("/body/threadId")
                                        .and_then(Value::as_u64)
                                        .or_else(|| {
                                            value
                                                .get("body")
                                                .and_then(|body| body.get("threadId"))
                                                .and_then(Value::as_u64)
                                        });
                                    let reason = value
                                        .pointer("/body/reason")
                                        .and_then(Value::as_str)
                                        .unwrap_or("pause")
                                        .to_owned();
                                    if let Ok(mut state) = stop_state.lock() {
                                        state.last_thread_id = thread_id.or(Some(1));
                                        state.selected_thread_id = None;
                                        state.selected_frame_id = None;
                                        state.last_reason = Some(reason);
                                        state.snapshot = None;
                                    }
                                    if let Ok(mut queue) = events.lock() {
                                        queue.push_back(DapSessionEvent::Stopped { workspace_id });
                                    }
                                }
                                "continued" => {
                                    if let Ok(mut state) = stop_state.lock() {
                                        state.snapshot = None;
                                    }
                                    if let Ok(mut queue) = events.lock() {
                                        queue
                                            .push_back(DapSessionEvent::Continued { workspace_id });
                                    }
                                }
                                "exited" | "terminated" => {
                                    mark_session_ended(&stop_state, &events, workspace_id);
                                }
                                _ => {}
                            }
                        }
                        "request" => {
                            // Reverse requests are ignored in this milestone.
                        }
                        _ => {}
                    }
                }
                Err(error) => {
                    record_transport_event(
                        &transport_log,
                        &adapter_id,
                        format!("transport read error: {error}"),
                    );
                    disconnected.store(true, Ordering::Release);
                    if let Ok(mut pending) = pending.lock() {
                        let pending_responses = mem::take(&mut *pending);
                        for (_, pending_response) in pending_responses {
                            let _ =
                                pending_response
                                    .tx
                                    .send(Err(DapClientError::Protocol(format!(
                                        "debug adapter `{adapter_id}` disconnected"
                                    ))));
                        }
                    }
                    break;
                }
            }
        }
    })
}

fn mark_session_ended(
    stop_state: &Mutex<SessionStopState>,
    events: &Mutex<VecDeque<DapSessionEvent>>,
    workspace_id: u64,
) {
    let first_end = match stop_state.lock() {
        Ok(mut state) => {
            let first = !state.ended;
            state.ended = true;
            state.snapshot = None;
            first
        }
        Err(_) => false,
    };
    if !first_end {
        return;
    }
    if let Ok(mut queue) = events.lock() {
        queue.push_back(DapSessionEvent::Terminated { workspace_id });
    }
}

fn write_frame(
    writer: &Arc<Mutex<Box<dyn Write + Send>>>,
    transport_log: &TransportLog,
    adapter_id: &str,
    message: &Value,
) -> Result<(), DapClientError> {
    let encoded = serde_json::to_vec(message).map_err(|error| {
        DapClientError::Protocol(format!("failed to encode DAP message: {error}"))
    })?;
    let header = format!("Content-Length: {}\r\n\r\n", encoded.len());
    let mut writer = writer.lock().map_err(|_| DapClientError::LockPoisoned)?;
    writer.write_all(header.as_bytes())?;
    writer.write_all(&encoded)?;
    writer.flush()?;
    record_transport_message(transport_log, adapter_id, DapLogDirection::Send, message);
    Ok(())
}

fn read_frame<R: BufRead>(reader: &mut R) -> Result<Value, DapClientError> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Err(DapClientError::Protocol(
                "debug adapter closed the transport".to_owned(),
            ));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            let value = value.trim().parse::<usize>().map_err(|error| {
                DapClientError::Protocol(format!("invalid Content-Length: {error}"))
            })?;
            content_length = Some(value);
        }
    }
    let length = content_length.ok_or_else(|| {
        DapClientError::Protocol("received DAP frame without Content-Length".to_owned())
    })?;
    let mut buffer = vec![0_u8; length];
    reader.read_exact(&mut buffer)?;
    serde_json::from_slice(&buffer)
        .map_err(|error| DapClientError::Protocol(format!("failed to decode DAP frame: {error}")))
}

fn record_transport_message(
    transport_log: &TransportLog,
    adapter_id: &str,
    direction: DapLogDirection,
    message: &Value,
) {
    let rendered = serde_json::to_string(message).unwrap_or_else(|_| "<unprintable>".to_owned());
    record_transport_event_inner(transport_log, adapter_id, direction, rendered);
}

fn record_transport_event(
    transport_log: &TransportLog,
    adapter_id: &str,
    message: impl Into<String>,
) {
    record_transport_event_inner(
        transport_log,
        adapter_id,
        DapLogDirection::Event,
        message.into(),
    );
}

fn record_transport_event_inner(
    transport_log: &TransportLog,
    adapter_id: &str,
    direction: DapLogDirection,
    message: String,
) {
    if let Ok(mut log) = transport_log.lock() {
        log.push(DapLogEntry {
            adapter_id: adapter_id.to_owned(),
            direction,
            message,
        });
    }
}

fn active_thread_id(handle: &DapSessionHandle) -> Result<u64, DapClientError> {
    let state = handle
        .stop_state
        .lock()
        .map_err(|_| DapClientError::LockPoisoned)?;
    state
        .selected_thread_id
        .or(state.last_thread_id)
        .ok_or_else(|| {
            DapClientError::Protocol(
                "Debug Session has no stopped thread; wait for a stop before stepping".to_owned(),
            )
        })
}

fn clear_stopped_snapshot(handle: &DapSessionHandle) -> Result<(), DapClientError> {
    let mut state = handle
        .stop_state
        .lock()
        .map_err(|_| DapClientError::LockPoisoned)?;
    state.snapshot = None;
    Ok(())
}

fn evaluate_expression(
    handle: &DapSessionHandle,
    expression: &str,
    frame_id: Option<u64>,
    context: EvaluateArgumentsContext,
) -> Result<DapWatchExpression, DapClientError> {
    match handle.request::<Evaluate>(EvaluateArguments {
        expression: expression.to_owned(),
        frame_id,
        context: Some(context),
        format: None,
    }) {
        Ok(response) => Ok(DapWatchExpression {
            expression: expression.to_owned(),
            value: response.result,
            type_name: response.type_,
            ok: true,
            variables_reference: response.variables_reference,
            children: Vec::new(),
            expanded: false,
        }),
        Err(error) => Ok(DapWatchExpression {
            expression: expression.to_owned(),
            value: error.to_string(),
            type_name: None,
            ok: false,
            variables_reference: 0,
            children: Vec::new(),
            expanded: false,
        }),
    }
}

fn capture_stopped_snapshot(
    handle: &DapSessionHandle,
    thread_id: u64,
    preferred_frame_id: Option<u64>,
    reason: String,
    watches: &[String],
    expanded: &BTreeSet<DapVariablePath>,
) -> Result<DapStoppedSnapshot, DapClientError> {
    let stack = handle.request::<StackTrace>(StackTraceArguments {
        thread_id,
        start_frame: Some(0),
        levels: Some(32),
        format: None,
    })?;
    let frame = preferred_frame_id
        .and_then(|frame_id| stack.stack_frames.iter().find(|frame| frame.id == frame_id))
        .or_else(|| stack.stack_frames.first());
    let frame_id = frame.map(|frame| frame.id);
    let position = frame.and_then(|frame| {
        let path = frame
            .source
            .as_ref()
            .and_then(|source| source.path.as_ref())
            .map(PathBuf::from)?;
        let line = u32::try_from(frame.line).ok().filter(|line| *line > 0)?;
        let column = u32::try_from(frame.column).unwrap_or(1).max(1);
        Some(DapExecutionPosition { path, line, column })
    });

    let mut locals = match frame {
        Some(frame) => match load_locals(handle, frame.id) {
            Ok(locals) => locals,
            Err(error) => {
                record_transport_event(
                    &handle.transport_log,
                    handle.info.adapter_id(),
                    format!("failed to load Locals: {error}"),
                );
                Vec::new()
            }
        },
        None => Vec::new(),
    };
    apply_expanded_paths(handle, &mut locals, None, &[], expanded);

    let mut watch_rows = Vec::with_capacity(watches.len());
    for expression in watches {
        watch_rows.push(evaluate_expression(
            handle,
            expression,
            frame_id,
            EvaluateArgumentsContext::Watch,
        )?);
    }
    apply_expanded_watch_roots(handle, &mut watch_rows, expanded);

    Ok(DapStoppedSnapshot {
        thread_id,
        frame_id,
        reason,
        position,
        locals,
        watches: watch_rows,
    })
}

fn load_locals(
    handle: &DapSessionHandle,
    frame_id: u64,
) -> Result<Vec<DapLocalVariable>, DapClientError> {
    let scopes = handle.request::<Scopes>(ScopesArguments { frame_id })?;
    let locals_scope = scopes
        .scopes
        .iter()
        .find(|scope| {
            scope.name.eq_ignore_ascii_case("locals")
                || matches!(
                    scope.presentation_hint,
                    Some(dap_types::ScopePresentationHint::Locals)
                )
        })
        .or_else(|| scopes.scopes.first());
    let Some(scope) = locals_scope else {
        return Ok(Vec::new());
    };
    if scope.variables_reference == 0 {
        return Ok(Vec::new());
    }
    load_variable_children(handle, scope.variables_reference)
}

fn variable_node_from_dap(variable: dap_types::Variable) -> DapVariableNode {
    DapVariableNode {
        name: variable.name,
        value: variable.value,
        type_name: variable.type_,
        variables_reference: variable.variables_reference,
        children: Vec::new(),
        expanded: false,
    }
}

fn load_variable_children(
    handle: &DapSessionHandle,
    variables_reference: u64,
) -> Result<Vec<DapVariableNode>, DapClientError> {
    if variables_reference == 0 {
        return Ok(Vec::new());
    }
    let response = handle.request::<Variables>(VariablesArguments {
        variables_reference,
        filter: None,
        start: None,
        count: None,
        format: None,
    })?;
    Ok(response
        .variables
        .into_iter()
        .map(variable_node_from_dap)
        .collect())
}

fn flatten_variable_nodes(
    nodes: &[DapVariableNode],
    watch: Option<&str>,
    prefix: &[String],
    depth: usize,
    rows: &mut Vec<DapVariableRow>,
) {
    for node in nodes {
        let mut segments = prefix.to_vec();
        segments.push(node.name.clone());
        let path = match watch {
            Some(expression) => DapVariablePath::watch(expression, segments.clone()),
            None => DapVariablePath::locals(segments.clone()),
        };
        rows.push(DapVariableRow {
            path,
            depth,
            expandable: node.expandable(),
            expanded: node.expanded,
            name: node.name.clone(),
            value: node.value.clone(),
            type_name: node.type_name.clone(),
            ok: true,
        });
        if node.expanded {
            flatten_variable_nodes(&node.children, watch, &segments, depth + 1, rows);
        }
    }
}

fn apply_expanded_paths(
    handle: &DapSessionHandle,
    nodes: &mut [DapVariableNode],
    watch: Option<&str>,
    prefix: &[String],
    expanded: &BTreeSet<DapVariablePath>,
) {
    for node in nodes {
        let mut segments = prefix.to_vec();
        segments.push(node.name.clone());
        let path = match watch {
            Some(expression) => DapVariablePath::watch(expression, segments.clone()),
            None => DapVariablePath::locals(segments.clone()),
        };
        if expanded.contains(&path) && node.variables_reference > 0 {
            match load_variable_children(handle, node.variables_reference) {
                Ok(children) => {
                    node.children = children;
                    node.expanded = true;
                    apply_expanded_paths(handle, &mut node.children, watch, &segments, expanded);
                }
                Err(error) => {
                    record_transport_event(
                        &handle.transport_log,
                        handle.info.adapter_id(),
                        format!("failed to expand `{}`: {error}", node.name),
                    );
                }
            }
        }
    }
}

fn apply_expanded_watch_roots(
    handle: &DapSessionHandle,
    watches: &mut [DapWatchExpression],
    expanded: &BTreeSet<DapVariablePath>,
) {
    for watch in watches {
        let path = DapVariablePath::watch(&watch.expression, Vec::<String>::new());
        if expanded.contains(&path) && watch.expandable() {
            match load_variable_children(handle, watch.variables_reference) {
                Ok(children) => {
                    watch.children = children;
                    watch.expanded = true;
                    apply_expanded_paths(
                        handle,
                        &mut watch.children,
                        Some(watch.expression.as_str()),
                        &[],
                        expanded,
                    );
                }
                Err(error) => {
                    record_transport_event(
                        &handle.transport_log,
                        handle.info.adapter_id(),
                        format!(
                            "failed to expand Watch Expression `{}`: {error}",
                            watch.expression
                        ),
                    );
                }
            }
        }
    }
}

fn find_variable_node_mut<'a>(
    nodes: &'a mut [DapVariableNode],
    segments: &[String],
) -> Option<&'a mut DapVariableNode> {
    let (head, tail) = segments.split_first()?;
    let index = nodes.iter().position(|node| node.name == *head)?;
    let node = nodes.get_mut(index)?;
    if tail.is_empty() {
        Some(node)
    } else {
        find_variable_node_mut(&mut node.children, tail)
    }
}

fn expand_variable_node(
    handle: &DapSessionHandle,
    node: &mut DapVariableNode,
) -> Result<bool, DapClientError> {
    if node.variables_reference == 0 {
        return Ok(false);
    }
    node.children = load_variable_children(handle, node.variables_reference)?;
    node.expanded = true;
    Ok(true)
}

fn expand_watch_root(
    handle: &DapSessionHandle,
    watch: &mut DapWatchExpression,
) -> Result<bool, DapClientError> {
    if !watch.expandable() {
        return Ok(false);
    }
    watch.children = load_variable_children(handle, watch.variables_reference)?;
    watch.expanded = true;
    Ok(true)
}

fn expand_variable_path(
    handle: &DapSessionHandle,
    snapshot: &mut DapStoppedSnapshot,
    path: &DapVariablePath,
) -> Result<bool, DapClientError> {
    match path.watch_expression() {
        Some(expression) => {
            let watch = snapshot
                .watches
                .iter_mut()
                .find(|watch| watch.expression == expression)
                .ok_or_else(|| {
                    DapClientError::Protocol(format!("Watch Expression `{expression}` not found"))
                })?;
            if path.segments.is_empty() {
                expand_watch_root(handle, watch)
            } else {
                let node = find_variable_node_mut(&mut watch.children, &path.segments).ok_or_else(
                    || DapClientError::Protocol("variable path not found".to_owned()),
                )?;
                expand_variable_node(handle, node)
            }
        }
        None => {
            let node = find_variable_node_mut(&mut snapshot.locals, &path.segments)
                .ok_or_else(|| DapClientError::Protocol("variable path not found".to_owned()))?;
            expand_variable_node(handle, node)
        }
    }
}

fn collapse_variable_path(snapshot: &mut DapStoppedSnapshot, path: &DapVariablePath) {
    match path.watch_expression() {
        Some(expression) => {
            let Some(watch) = snapshot
                .watches
                .iter_mut()
                .find(|watch| watch.expression == expression)
            else {
                return;
            };
            if path.segments.is_empty() {
                watch.children.clear();
                watch.expanded = false;
                return;
            }
            if let Some(node) = find_variable_node_mut(&mut watch.children, &path.segments) {
                node.children.clear();
                node.expanded = false;
            }
        }
        None => {
            if let Some(node) = find_variable_node_mut(&mut snapshot.locals, &path.segments) {
                node.children.clear();
                node.expanded = false;
            }
        }
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
