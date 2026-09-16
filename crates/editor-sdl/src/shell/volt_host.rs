use editor_plugin_api::volt::AbiVoltHost;
use serde_json::Value;

struct VoltBufSnap {
    name: Option<String>,
    uri: Option<String>,
    cursor: TextPoint,
    text: String,
    line_count: u64,
    client_ids: Vec<String>,
    path: Option<PathBuf>,
    root: Option<PathBuf>,
    revision: u64,
}

struct VoltHostState {
    current_buf: u64,
    buffers: BTreeMap<u64, VoltBufSnap>,
    lsp: Option<Arc<LspClientManager>>,
    pending_locations: Option<(String, Vec<editor_lsp::LspLocation>)>,
    dispatch: extern "C" fn(u64, abi_stable::std_types::RString),
}

static VOLT_HOST: Mutex<Option<VoltHostState>> = Mutex::new(None);

fn volt_host_lock() -> std::sync::MutexGuard<'static, Option<VoltHostState>> {
    VOLT_HOST
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn abi_volt_host() -> AbiVoltHost {
    AbiVoltHost {
        buf_current: volt_host_buf_current,
        buf_name: volt_host_buf_name,
        buf_uri: volt_host_buf_uri,
        buf_cursor_line: volt_host_buf_cursor_line,
        buf_cursor_column: volt_host_buf_cursor_column,
        buf_line_count: volt_host_buf_line_count,
        buf_get_lines: volt_host_buf_get_lines,
        lsp_client_ids: volt_host_lsp_client_ids,
        lsp_request: volt_host_lsp_request,
        ui_open_locations: volt_host_ui_open_locations,
    }
}

fn install_volt_host_table() {
    editor_plugin_api::volt::install_host(abi_volt_host());
}

fn prepare_volt_host(runtime: &EditorRuntime) {
    install_volt_host_table();
    let Some(buffer_id) = active_shell_buffer_id(runtime).ok() else {
        return;
    };
    let Ok(buffer) = shell_buffer(runtime, buffer_id) else {
        return;
    };
    let path = buffer.lsp_path().map(Path::to_path_buf);
    let uri = path
        .as_deref()
        .map(LspClientManager::document_uri_for_path);
    let root = lsp_root_for_buffer(runtime, buffer).ok().flatten();
    let client_ids = runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .and_then(|lsp| path.as_deref().map(|path| lsp.session_labels_for_path(path)))
        .unwrap_or_default();
    let snap = VoltBufSnap {
        name: buffer.path().map(|path| path.display().to_string()),
        uri,
        cursor: buffer.cursor_point(),
        line_count: buffer.text.line_count() as u64,
        text: buffer.text.text(),
        client_ids,
        path,
        root,
        revision: buffer.text.revision(),
    };
    let mut slot = volt_host_lock();
    *slot = Some(VoltHostState {
        current_buf: buffer_id.get(),
        buffers: BTreeMap::from([(buffer_id.get(), snap)]),
        lsp: runtime.services().get::<Arc<LspClientManager>>().cloned(),
        pending_locations: None,
        dispatch: editor_plugin_api::volt::dispatch_c,
    });
}

fn apply_volt_host_actions(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let pending = volt_host_lock()
        .as_mut()
        .and_then(|state| state.pending_locations.take());
    let Some((title, locations)) = pending else {
        return Ok(false);
    };
    if locations.is_empty() {
        return Ok(false);
    }
    open_lsp_locations(runtime, &title, locations)?;
    Ok(true)
}

fn dispatch_volt_result(token: u64, payload: String) {
    let dispatch = {
        volt_host_lock()
            .as_ref()
            .map(|state| state.dispatch)
            .unwrap_or(editor_plugin_api::volt::dispatch_c)
    };
    dispatch(token, payload.into());
    ping_shell_wakeup();
}

extern "C" fn volt_host_buf_current() -> u64 {
    volt_host_lock()
        .as_ref()
        .map(|state| state.current_buf)
        .unwrap_or(0)
}

extern "C" fn volt_host_buf_name(buf: u64) -> abi_stable::std_types::ROption<abi_stable::std_types::RString> {
    volt_host_lock()
        .as_ref()
        .and_then(|state| state.buffers.get(&buf))
        .and_then(|snap| snap.name.clone())
        .map(Into::into)
        .into()
}

extern "C" fn volt_host_buf_uri(buf: u64) -> abi_stable::std_types::ROption<abi_stable::std_types::RString> {
    volt_host_lock()
        .as_ref()
        .and_then(|state| state.buffers.get(&buf))
        .and_then(|snap| snap.uri.clone())
        .map(Into::into)
        .into()
}

extern "C" fn volt_host_buf_cursor_line(buf: u64) -> u64 {
    volt_host_lock()
        .as_ref()
        .and_then(|state| state.buffers.get(&buf))
        .map(|snap| snap.cursor.line as u64)
        .unwrap_or(0)
}

extern "C" fn volt_host_buf_cursor_column(buf: u64) -> u64 {
    volt_host_lock()
        .as_ref()
        .and_then(|state| state.buffers.get(&buf))
        .map(|snap| snap.cursor.column as u64)
        .unwrap_or(0)
}

extern "C" fn volt_host_buf_line_count(buf: u64) -> u64 {
    volt_host_lock()
        .as_ref()
        .and_then(|state| state.buffers.get(&buf))
        .map(|snap| snap.line_count)
        .unwrap_or(0)
}

extern "C" fn volt_host_buf_get_lines(
    buf: u64,
    start: u64,
    end: u64,
) -> abi_stable::std_types::RVec<abi_stable::std_types::RString> {
    let Some(text) = volt_host_lock()
        .as_ref()
        .and_then(|state| state.buffers.get(&buf))
        .map(|snap| snap.text.clone())
    else {
        return abi_stable::std_types::RVec::new();
    };
    text.lines()
        .skip(start as usize)
        .take(end.saturating_sub(start) as usize)
        .map(abi_stable::std_types::RString::from)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn volt_host_lsp_client_ids(buf: u64) -> abi_stable::std_types::RVec<abi_stable::std_types::RString> {
    volt_host_lock()
        .as_ref()
        .and_then(|state| state.buffers.get(&buf))
        .map(|snap| {
            snap.client_ids
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into()
        })
        .unwrap_or_default()
}

extern "C" fn volt_host_lsp_request(
    client: abi_stable::std_types::RString,
    method: abi_stable::std_types::RString,
    params: abi_stable::std_types::RString,
    token: u64,
) {
    let prepared = {
        let state = volt_host_lock();
        state.as_ref().and_then(|state| {
            let snap = state.buffers.get(&state.current_buf)?;
            Some((
                state.lsp.clone(),
                snap.path.clone(),
                snap.root.clone(),
                snap.text.clone(),
                snap.revision,
            ))
        })
    };
    let Some((lsp, path, root, text, revision)) = prepared else {
        dispatch_volt_result(token, "err:volt host is not prepared".to_owned());
        return;
    };
    let Some(lsp) = lsp else {
        dispatch_volt_result(token, "err:no language server manager".to_owned());
        return;
    };
    let Some(path) = path else {
        dispatch_volt_result(token, "err:buffer has no file path".to_owned());
        return;
    };
    let client = client.into_string();
    let method = method.into_string();
    let params = params.into_string();
    std::thread::spawn(move || {
        let payload = match serde_json::from_str::<Value>(&params) {
            Ok(params) => match lsp.json_rpc_request(editor_lsp::LspJsonRpcRequest {
                server_id: &client,
                path: &path,
                method: &method,
                params,
                root: root.as_deref(),
                text: Some(&text),
                revision: Some(revision),
            }) {
                Ok(value) => serde_json::to_string(&value)
                    .unwrap_or_else(|_| "err:failed to encode LSP result".to_owned()),
                Err(error) => format!("err:{error}"),
            },
            Err(error) => format!("err:{error}"),
        };
        dispatch_volt_result(token, payload);
    });
}

extern "C" fn volt_host_ui_open_locations(
    title: abi_stable::std_types::RString,
    payload: abi_stable::std_types::RString,
) {
    let Ok(value) = serde_json::from_str::<Value>(payload.as_str()) else {
        return;
    };
    let Some(entries) = value.as_array() else {
        return;
    };
    let mut locations = Vec::new();
    for entry in entries {
        let Some(uri) = entry.get("uri").and_then(Value::as_str) else {
            continue;
        };
        let line = entry.get("line").and_then(Value::as_u64).unwrap_or(0) as usize;
        let column = entry.get("column").and_then(Value::as_u64).unwrap_or(0) as usize;
        locations.push(editor_lsp::LspLocation::from_uri_position(
            "volt", uri, line, column,
        ));
    }
    if let Some(state) = volt_host_lock().as_mut() {
        state.pending_locations = Some((title.into_string(), locations));
    }
    ping_shell_wakeup();
}

fn run_volt_command_or_fallback(
    runtime: &mut EditorRuntime,
    name: &str,
    fallback: fn(&mut EditorRuntime) -> Result<(), String>,
) -> Result<(), String> {
    prepare_volt_host(runtime);
    if shell_user_library(runtime).run_volt_command(name) {
        apply_volt_host_actions(runtime)?;
        return Ok(());
    }
    fallback(runtime)
}
