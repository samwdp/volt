fn build_lsp_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
    #[cfg(windows)] runtime_env: Option<&[(String, String)]>,
    #[cfg(not(windows))] _runtime_env: Option<&[(String, String)]>,
) -> Command {
    let (program, args) = supervised_command_if_resolved(
        program,
        args,
        env,
        #[cfg(windows)]
        runtime_env,
        #[cfg(not(windows))]
        None,
        ProcessSupervisionMode::Background,
    );
    let mut command = Command::new(&program);
    configure_lsp_command(&mut command);
    command
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    let mut env = env.to_vec();
    editor_tool_install::merge_effective_path(&mut env);
    #[cfg(windows)]
    if let Some(runtime_env) = runtime_env {
        apply_windows_runtime_environment(&mut command, &env, runtime_env);
    } else {
        apply_command_environment(&mut command, &env);
    }
    #[cfg(not(windows))]
    apply_command_environment(&mut command, &env);
    command
}

fn apply_command_environment(command: &mut Command, env: &[(String, String)]) {
    for (key, value) in env {
        command.env(key, value);
    }
}

#[cfg(windows)]
fn windows_launch_program_candidates(program: &str) -> Vec<String> {
    if Path::new(program).extension().is_some() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for extension in windows_command_extensions() {
        let candidate = format!("{program}{extension}");
        if candidate != program && !candidates.iter().any(|existing| existing == &candidate) {
            candidates.push(candidate);
        }
    }
    candidates
}

#[cfg(windows)]
fn windows_should_retry_spawn_error(error: &std::io::Error) -> bool {
    error.kind() == std::io::ErrorKind::NotFound || error.raw_os_error() == Some(193)
}

#[cfg(windows)]
fn windows_command_extensions() -> Vec<String> {
    std::env::var("PATHEXT")
        .ok()
        .map(|value| {
            value
                .split(';')
                .map(str::trim)
                .filter(|extension| !extension.is_empty())
                .map(|extension| extension.to_ascii_lowercase())
                .collect::<Vec<_>>()
        })
        .filter(|extensions| !extensions.is_empty())
        .unwrap_or_else(|| {
            [".com", ".exe", ".bat", ".cmd"]
                .into_iter()
                .map(str::to_owned)
                .collect()
        })
}

#[cfg(windows)]
fn windows_fnm_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> Option<Vec<(String, String)>> {
    let mut command = Command::new("fnm");
    configure_lsp_command(&mut command);
    command
        .args(["env", "--shell", "cmd"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    apply_command_environment(&mut command, env);
    let output = command.output().ok()?;
    output.status.success().then_some(())?;
    parse_windows_cmd_environment(&String::from_utf8_lossy(&output.stdout))
}

#[cfg(windows)]
fn windows_fnm_launch_program_candidates(
    program: &str,
    fnm_env: &[(String, String)],
) -> Vec<String> {
    windows_runtime_launch_program_candidates(program, fnm_env)
}

#[cfg(windows)]
fn windows_nvm_environment(
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> Option<Vec<(String, String)>> {
    let nvm_home = windows_nvm_home(env)?;
    let nvm_exe = nvm_home.join("nvm.exe");
    nvm_exe.is_file().then_some(())?;

    let mut command = Command::new(&nvm_exe);
    configure_lsp_command(&mut command);
    command
        .arg("current")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    apply_command_environment(&mut command, env);
    let output = command.output().ok()?;
    output.status.success().then_some(())?;
    let version = parse_windows_nvm_current_version(&String::from_utf8_lossy(&output.stdout))?;
    let node_dir = windows_nvm_node_dir(&nvm_home, &version)?;

    let mut runtime_env = vec![("PATH".to_owned(), node_dir.to_string_lossy().into_owned())];
    runtime_env.push((
        "NVM_HOME".to_owned(),
        nvm_home.to_string_lossy().into_owned(),
    ));
    if let Some(nvm_symlink) = windows_effective_environment_value(env, "NVM_SYMLINK") {
        runtime_env.push(("NVM_SYMLINK".to_owned(), nvm_symlink));
    }
    Some(runtime_env)
}

#[cfg(windows)]
fn windows_nvm_launch_program_candidates(
    program: &str,
    nvm_env: &[(String, String)],
) -> Vec<String> {
    windows_runtime_launch_program_candidates(program, nvm_env)
}

#[cfg(windows)]
fn windows_runtime_launch_program_candidates(
    program: &str,
    runtime_env: &[(String, String)],
) -> Vec<String> {
    if Path::new(program).components().count() != 1 {
        return Vec::new();
    }

    let names = windows_launch_program_candidates(program)
        .into_iter()
        .chain(std::iter::once(program.to_owned()))
        .collect::<Vec<_>>();
    let Some(path_value) = explicit_windows_env_value(runtime_env, "PATH") else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for directory in path_value
        .split(';')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
    {
        for name in &names {
            let candidate = Path::new(directory).join(name);
            if candidate.is_file() {
                let candidate = candidate.to_string_lossy().into_owned();
                if !candidates.iter().any(|existing| existing == &candidate) {
                    candidates.push(candidate);
                }
            }
        }
    }
    candidates
}

#[cfg(windows)]
fn windows_nvm_home(env: &[(String, String)]) -> Option<PathBuf> {
    windows_effective_environment_value(env, "NVM_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            windows_effective_environment_value(env, "APPDATA")
                .map(|appdata| Path::new(&appdata).join("nvm"))
        })
}

#[cfg(windows)]
fn parse_windows_nvm_current_version(output: &str) -> Option<String> {
    let version = output
        .split_whitespace()
        .find(|token| {
            !token.is_empty()
                && !token.eq_ignore_ascii_case("none")
                && !token.eq_ignore_ascii_case("n/a")
                && token
                    .chars()
                    .next()
                    .is_some_and(|ch| ch == 'v' || ch.is_ascii_digit())
        })?
        .trim();
    Some(version.to_owned())
}

#[cfg(windows)]
fn windows_nvm_node_dir(nvm_home: &Path, version: &str) -> Option<PathBuf> {
    let mut candidates = vec![version.to_owned()];
    if let Some(stripped) = version.strip_prefix('v') {
        candidates.push(stripped.to_owned());
    } else {
        candidates.push(format!("v{version}"));
    }

    for candidate in candidates {
        let node_dir = nvm_home.join(candidate);
        if node_dir.join("node.exe").is_file() {
            return Some(node_dir);
        }
    }
    None
}

#[cfg(windows)]
fn parse_windows_cmd_environment(output: &str) -> Option<Vec<(String, String)>> {
    let vars = output
        .lines()
        .filter_map(|line| {
            let rest = line.trim().strip_prefix("SET ")?;
            let (key, value) = rest.split_once('=')?;
            (!key.is_empty()).then_some((key.to_owned(), value.to_owned()))
        })
        .collect::<Vec<_>>();
    (!vars.is_empty()).then_some(vars)
}

#[cfg(windows)]
fn apply_windows_runtime_environment(
    command: &mut Command,
    env: &[(String, String)],
    runtime_env: &[(String, String)],
) {
    let explicit_path = explicit_windows_env_value(env, "PATH");
    let mut applied_path = false;
    for (key, value) in runtime_env {
        if key.eq_ignore_ascii_case("PATH") {
            let merged_path = explicit_path
                .map(|path| format!("{value};{path}"))
                .unwrap_or_else(|| value.clone());
            command.env(key, merged_path);
            applied_path = true;
            continue;
        }
        command.env(key, value);
    }
    for (key, value) in env {
        if !key.eq_ignore_ascii_case("PATH") {
            command.env(key, value);
        }
    }
    if !applied_path && let Some(path) = explicit_path {
        command.env("PATH", path);
    }
}

#[cfg(windows)]
fn explicit_windows_env_value<'a>(env: &'a [(String, String)], key: &str) -> Option<&'a String> {
    env.iter()
        .find_map(|(entry_key, value)| entry_key.eq_ignore_ascii_case(key).then_some(value))
}

#[cfg(windows)]
fn windows_effective_environment_value(env: &[(String, String)], key: &str) -> Option<String> {
    explicit_windows_env_value(env, key)
        .map(String::to_owned)
        .or_else(|| std::env::var(key).ok())
}

fn read_message(reader: &mut impl BufRead) -> Result<Option<Value>, LspClientError> {
    let mut content_length = None;
    loop {
        let mut header = String::new();
        let read = reader.read_line(&mut header)?;
        if read == 0 {
            return Ok(None);
        }
        let trimmed = header.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        if let Some(raw_length) = trimmed.strip_prefix("Content-Length:") {
            content_length = raw_length.trim().parse::<usize>().ok();
        }
    }
    let content_length = content_length.ok_or_else(|| {
        LspClientError::Protocol("received JSON-RPC frame without Content-Length".to_owned())
    })?;
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    serde_json::from_slice(&body).map_err(|error| {
        LspClientError::Protocol(format!("failed to parse JSON-RPC payload: {error}"))
    })
}

fn write_response(
    server_id: &str,
    transport_log: &TransportLog,
    writer: &Arc<Mutex<ChildStdin>>,
    message: Value,
) -> Result<(), LspClientError> {
    let encoded = serde_json::to_vec(&message).map_err(|error| {
        LspClientError::Protocol(format!("failed to encode JSON-RPC response: {error}"))
    })?;
    let mut writer = writer
        .lock()
        .map_err(|_| LspClientError::Protocol("LSP writer mutex poisoned".to_owned()))?;
    write!(writer, "Content-Length: {}\r\n\r\n", encoded.len())?;
    writer.write_all(&encoded)?;
    writer.flush()?;
    record_transport_message(
        transport_log,
        server_id,
        LspLogDirection::Outgoing,
        &message,
    );
    Ok(())
}

fn launch_summary(
    pid: u32,
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    root: Option<&Path>,
) -> String {
    let mut lines = Vec::with_capacity(3);
    let mut command_line = format!("started process {pid}: {program}");
    if !args.is_empty() {
        command_line.push(' ');
        command_line.push_str(&args.join(" "));
    }
    lines.push(command_line);
    if let Some(cwd) = cwd {
        lines.push(format!("cwd: {}", cwd.display()));
    }
    if let Some(root) = root {
        lines.push(format!("root: {}", root.display()));
    }
    lines.join("\n")
}

fn format_transport_message(message: &Value) -> String {
    let sanitized = sanitize_transport_message(message);
    serde_json::to_string_pretty(&sanitized).unwrap_or_else(|_| sanitized.to_string())
}

fn sanitize_transport_message(message: &Value) -> Value {
    match message {
        Value::Array(items) => Value::Array(items.iter().map(sanitize_transport_message).collect()),
        Value::Object(object) => Value::Object(
            object
                .iter()
                .map(|(key, value)| {
                    let value = if transport_key_is_sensitive(key) {
                        Value::String("[redacted]".to_owned())
                    } else {
                        sanitize_transport_message(value)
                    };
                    (key.clone(), value)
                })
                .collect(),
        ),
        _ => message.clone(),
    }
}

fn transport_key_is_sensitive(key: &str) -> bool {
    let normalized = key
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(|character| character.to_lowercase())
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "apikey"
            | "authorization"
            | "connectionstring"
            | "datasourcename"
            | "passphrase"
            | "passwd"
            | "password"
            | "privatekey"
            | "secret"
    )
}

fn record_transport_message(
    transport_log: &TransportLog,
    server_id: &str,
    direction: LspLogDirection,
    message: &Value,
) {
    record_transport_entry(
        transport_log,
        LspLogEntry::new(direction, server_id, format_transport_message(message)),
    );
}

fn record_transport_event(
    transport_log: &TransportLog,
    server_id: &str,
    message: impl Into<String>,
) {
    record_transport_entry(
        transport_log,
        LspLogEntry::new(LspLogDirection::Event, server_id, message),
    );
}

fn record_transport_entry(transport_log: &TransportLog, entry: LspLogEntry) {
    if let Ok(mut log) = transport_log.lock() {
        log.record(entry);
    }
}

fn record_notification(notifications: &NotificationLog, notification: LspNotification) {
    if let Ok(mut log) = notifications.lock() {
        log.record(notification);
    }
}

fn record_published_diagnostics(
    diagnostics: &DiagnosticsByPath,
    dirty_paths: &Arc<Mutex<BTreeSet<PathBuf>>>,
    diagnostics_generation: &AtomicU64,
    path: PathBuf,
    parsed: Vec<Diagnostic>,
) {
    if let Ok(mut guard) = diagnostics.lock() {
        guard.insert(path.clone(), parsed);
    }
    if let Ok(mut dirty) = dirty_paths.lock() {
        dirty.insert(path);
    }
    diagnostics_generation.fetch_add(1, Ordering::Release);
}

fn note_session_disconnect_diagnostics(
    diagnostics: &DiagnosticsByPath,
    dirty_paths: &Arc<Mutex<BTreeSet<PathBuf>>>,
) {
    if let Ok(guard) = diagnostics.lock()
        && let Ok(mut dirty) = dirty_paths.lock()
    {
        dirty.extend(guard.keys().cloned());
    }
}

fn spawn_inert_child() -> std::io::Result<(Child, ChildStdin)> {
    #[cfg(windows)]
    let mut child = {
        use std::os::windows::process::CommandExt as _;

        Command::new("cmd")
            .args(["/C", "more"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()?
    };
    #[cfg(not(windows))]
    let mut child = Command::new("sh")
        .args(["-c", "cat >/dev/null"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let stdin = child.stdin.take().ok_or_else(|| {
        std::io::Error::other("inert language server child is missing stdin pipe")
    })?;
    Ok((child, stdin))
}

fn session_notification_key(server_id: &str, root: Option<&Path>) -> String {
    format!("session:{server_id}:{}", notification_root_key(root))
}

fn notification_root_key(root: Option<&Path>) -> String {
    match root {
        Some(root) => root.display().to_string(),
        None => "global".to_owned(),
    }
}

fn client_capabilities() -> Result<ClientCapabilities, LspClientError> {
    serde_json::from_value::<ClientCapabilities>(json!({
        "workspace": {
            "workspaceEdit": {
                "documentChanges": true
            },
            "configuration": true,
            "workspaceFolders": true
        },
        "window": {
            "workDoneProgress": true,
            "showDocument": {
                "support": true
            }
        },
        "textDocument": {
            "hover": {
                "contentFormat": ["markdown"]
            },
            "signatureHelp": {
                "signatureInformation": {
                    "documentationFormat": ["markdown"],
                    "parameterInformation": {
                        "labelOffsetSupport": true
                    },
                    "activeParameterSupport": true
                }
            },
            "completion": {
                "completionItem": {
                    "documentationFormat": ["markdown"],
                    "resolveSupport": {
                        "properties": ["documentation", "detail"]
                    },
                    "snippetSupport": true
                }
            },
            "inlineCompletion": {
                "dynamicRegistration": false
            },
            "codeAction": {
                "dynamicRegistration": false,
                "isPreferredSupport": true,
                "disabledSupport": true,
                "codeActionLiteralSupport": {
                    "codeActionKind": {
                        "valueSet": [
                            "quickfix",
                            "refactor",
                            "refactor.extract",
                            "refactor.inline",
                            "refactor.rewrite",
                            "source",
                            "source.fixAll",
                            "source.organizeImports"
                        ]
                    }
                }
            },
            "formatting": {
                "dynamicRegistration": false
            },
            "rangeFormatting": {
                "dynamicRegistration": false
            },
            "publishDiagnostics": {
                "relatedInformation": false
            },
            "synchronization": {
                "didSave": true
            }
        }
    }))
    .map_err(|error| {
        LspClientError::Protocol(format!("failed to build LSP client capabilities: {error}"))
    })
}

fn work_done_progress_params(
    next_progress_token: &AtomicU64,
    method: &str,
) -> WorkDoneProgressParams {
    let token = next_progress_token.fetch_add(1, Ordering::AcqRel);
    WorkDoneProgressParams {
        work_done_token: Some(NumberOrString::String(format!("progress:{method}:{token}"))),
    }
}

fn request_timeout_for_method(method: &str) -> Duration {
    if method == Initialize::METHOD {
        INITIALIZE_REQUEST_TIMEOUT
    } else if method == CodeActionRequest::METHOD {
        CODE_ACTION_REQUEST_TIMEOUT
    } else if method == INLINE_COMPLETION_METHOD {
        INLINE_COMPLETION_REQUEST_TIMEOUT
    } else {
        REQUEST_TIMEOUT
    }
}

fn normalize_configuration_section(section: Option<&str>) -> Option<&str> {
    section.map(str::trim).filter(|section| !section.is_empty())
}

fn workspace_configuration_section_for_session(session: &LanguageServerSession) -> Option<&str> {
    normalize_configuration_section(session.workspace_configuration_section())
        .or_else(|| is_csharp_server(session.server_id()).then_some(CSHARP_WORKSPACE_SECTION))
}

fn normalized_workspace_configuration_settings(
    section: Option<&str>,
    settings: Option<Value>,
) -> Option<Value> {
    let settings = settings?;
    let Some(section) = normalize_configuration_section(section) else {
        return Some(settings);
    };
    match settings {
        Value::Object(mut object) => object.remove(section).or(Some(Value::Object(object))),
        other => Some(other),
    }
}

fn effective_workspace_configuration_settings(
    base_settings: Option<&Value>,
    runtime_override: Option<&Value>,
) -> Option<Value> {
    match (base_settings, runtime_override) {
        (Some(base_settings), Some(runtime_override)) => {
            Some(merge_json_values(base_settings, runtime_override))
        }
        (Some(base_settings), None) => Some(base_settings.clone()),
        (None, Some(runtime_override)) => Some(runtime_override.clone()),
        (None, None) => None,
    }
}

fn settings_contains_key(settings: Option<&Value>, key: &str) -> bool {
    match settings {
        Some(Value::Object(settings)) => settings.contains_key(key),
        Some(_) => false,
        None => false,
    }
}

fn text_document_sync_kind(capability: Option<TextDocumentSyncCapability>) -> TextDocumentSyncKind {
    match capability {
        Some(TextDocumentSyncCapability::Kind(kind)) => kind,
        Some(TextDocumentSyncCapability::Options(options)) => {
            options.change.unwrap_or(TextDocumentSyncKind::FULL)
        }
        None => TextDocumentSyncKind::FULL,
    }
}

fn text_document_content_change(
    sync_kind: TextDocumentSyncKind,
    previous_text: &str,
    text: &str,
) -> TextDocumentContentChangeEvent {
    if sync_kind == TextDocumentSyncKind::INCREMENTAL {
        TextDocumentContentChangeEvent {
            range: Some(full_document_range(previous_text)),
            range_length: None,
            text: text.to_owned(),
        }
    } else {
        TextDocumentContentChangeEvent {
            range: None,
            range_length: None,
            text: text.to_owned(),
        }
    }
}

fn usable_edit_chain(
    edits: Option<&[editor_buffer::TextEdit]>,
    last_revision: Option<u64>,
    current_revision: u64,
) -> Option<&[editor_buffer::TextEdit]> {
    let edits = edits?;
    let last_revision = last_revision?;
    if last_revision == current_revision {
        return Some(&[]);
    }
    let start = edits
        .iter()
        .position(|edit| edit.before_revision == last_revision)?;
    let suffix = &edits[start..];
    if suffix.last()?.after_revision != current_revision
        || suffix
            .windows(2)
            .any(|pair| pair[0].after_revision != pair[1].before_revision)
    {
        return None;
    }
    Some(suffix)
}

fn incremental_content_changes(
    previous_text: &str,
    new_text: &str,
    edits: &[editor_buffer::TextEdit],
) -> Option<Vec<TextDocumentContentChangeEvent>> {
    if edits.is_empty() {
        return Some(Vec::new());
    }
    let mut working = previous_text.to_owned();
    let mut changes = Vec::with_capacity(edits.len());
    for (index, edit) in edits.iter().enumerate() {
        if edit.start_byte > edit.old_end_byte
            || edit.old_end_byte > working.len()
            || edit.start_byte > working.len()
        {
            return None;
        }
        let inserted = inserted_text_after_edit(new_text, edit, &edits[index + 1..])?;
        changes.push(TextDocumentContentChangeEvent {
            range: Some(Range::new(
                lsp_position_on_text(&working, edit.start_position),
                lsp_position_on_text(&working, edit.old_end_position),
            )),
            range_length: None,
            text: inserted.clone(),
        });
        working.replace_range(edit.start_byte..edit.old_end_byte, &inserted);
    }
    (working == new_text).then_some(changes)
}

fn inserted_text_after_edit(
    new_text: &str,
    edit: &editor_buffer::TextEdit,
    remaining: &[editor_buffer::TextEdit],
) -> Option<String> {
    let mut start = edit.start_byte;
    let mut end = edit.new_end_byte;
    if start > end {
        return None;
    }
    for later in remaining {
        (start, end) = map_exclusive_range_through_edit(start, end, later)?;
    }
    new_text.get(start..end).map(str::to_owned)
}

fn map_exclusive_range_through_edit(
    start: usize,
    end: usize,
    edit: &editor_buffer::TextEdit,
) -> Option<(usize, usize)> {
    let delta = edit.new_end_byte as isize - edit.old_end_byte as isize;
    if end <= edit.start_byte {
        Some((start, end))
    } else if start >= edit.old_end_byte {
        Some((
            usize::try_from(start as isize + delta).ok()?,
            usize::try_from(end as isize + delta).ok()?,
        ))
    } else {
        None
    }
}

fn lsp_position_on_text(text: &str, point: TextPoint) -> Position {
    let line = line_slice(text, point.line);
    Position::new(point.line as u32, utf16_column(line, point.column))
}

fn line_slice(text: &str, line_index: usize) -> &str {
    let mut remaining = text;
    for _ in 0..line_index {
        let Some(index) = remaining.find('\n') else {
            return "";
        };
        remaining = remaining.get(index.saturating_add(1)..).unwrap_or("");
    }
    remaining
        .split_once('\n')
        .map(|(line, _)| line)
        .unwrap_or(remaining)
}

fn utf16_column(line: &str, char_column: usize) -> u32 {
    let mut character = 0u32;
    for (index, ch) in line.chars().enumerate() {
        if index >= char_column {
            break;
        }
        if ch == '\n' {
            break;
        }
        if ch != '\r' {
            character = character.saturating_add(ch.len_utf16() as u32);
        }
    }
    character
}

fn full_document_range(text: &str) -> Range {
    Range::new(Position::new(0, 0), text_end_position(text))
}

fn text_end_position(text: &str) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;
    for ch in text.chars() {
        if ch == '\n' {
            line = line.saturating_add(1);
            character = 0;
        } else if ch != '\r' {
            character = character.saturating_add(ch.len_utf16() as u32);
        }
    }
    Position::new(line, character)
}

fn merge_json_values(base: &Value, override_value: &Value) -> Value {
    match (base, override_value) {
        (Value::Object(base), Value::Object(override_value)) => {
            let mut merged = base.clone();
            for (key, override_value) in override_value {
                let merged_value = merged
                    .get(key)
                    .map(|base_value| merge_json_values(base_value, override_value))
                    .unwrap_or_else(|| override_value.clone());
                merged.insert(key.clone(), merged_value);
            }
            Value::Object(merged)
        }
        _ => override_value.clone(),
    }
}

fn workspace_configuration_null_response(params: Option<&Value>) -> Value {
    let item_count = params
        .and_then(|params| params.get("items"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    Value::Array((0..item_count).map(|_| Value::Null).collect())
}

fn configuration_item_section(item: &Value) -> Option<&str> {
    normalize_configuration_section(item.get("section").and_then(Value::as_str))
}

fn with_csharp_solution_path_override(
    current: Option<Value>,
    solution_path: &Path,
) -> Option<Value> {
    let override_value = json!({
        "solutionPathOverride": solution_path.display().to_string(),
    });
    Some(
        match normalized_workspace_configuration_settings(Some(CSHARP_WORKSPACE_SECTION), current) {
            Some(current) => merge_json_values(&current, &override_value),
            None => override_value,
        },
    )
}

fn without_csharp_solution_path_override(current: Option<Value>) -> Option<Value> {
    let current =
        normalized_workspace_configuration_settings(Some(CSHARP_WORKSPACE_SECTION), current)?;
    let Value::Object(mut current) = current else {
        return Some(current);
    };
    current.remove("solutionPathOverride");
    (!current.is_empty()).then_some(Value::Object(current))
}

fn wrap_workspace_configuration_settings(section: &str, settings: Value) -> Value {
    let mut object = serde_json::Map::new();
    object.insert(section.to_owned(), settings);
    Value::Object(object)
}

fn is_copilot_server(server_id: &str) -> bool {
    server_id == COPILOT_SERVER_ID
}

fn is_csharp_server(server_id: &str) -> bool {
    matches!(server_id, CSHARP_SERVER_ID | ROSLYN_LANGUAGE_SERVER_ID)
}

fn is_csharp_metadata_uri(uri: &str) -> bool {
    uri.starts_with("csharp:/")
}

fn initialization_options_for_server(
    server_id: &str,
    override_value: Option<&Value>,
) -> Option<Value> {
    let base = if is_copilot_server(server_id) {
        Some(json!({
            "editorInfo": {
                "name": "Volt",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "editorPluginInfo": {
                "name": "Volt Copilot",
                "version": env!("CARGO_PKG_VERSION"),
            }
        }))
    } else if is_csharp_server(server_id) {
        Some(json!({
            "experimental": {
                "csharp": {
                    "metadataUris": true,
                }
            }
        }))
    } else {
        None
    };
    match (base, override_value) {
        (Some(base), Some(override_value)) => Some(merge_json_values(&base, override_value)),
        (Some(base), None) => Some(base),
        (None, Some(override_value)) => Some(override_value.clone()),
        (None, None) => None,
    }
}

fn csharp_metadata_request_params(uri: &str) -> Value {
    json!({
        "textDocument": {
            "uri": uri,
        }
    })
}

fn parse_csharp_metadata_response(
    uri: &str,
    value: &Value,
) -> Result<Option<Value>, LspClientError> {
    if value.is_null() {
        return Ok(None);
    }
    if let Some(source) = value.as_str() {
        return Ok(Some(json!({
            "uri": uri,
            "source": source,
        })));
    }
    let Some(metadata) = value.as_object() else {
        return Err(LspClientError::Protocol(
            "failed to decode csharp metadata response: expected an object".to_owned(),
        ));
    };
    let mut metadata = metadata.clone();
    metadata
        .entry("uri".to_owned())
        .or_insert_with(|| Value::String(uri.to_owned()));
    if !metadata.contains_key("source")
        && let Some(source) = metadata.get("text").cloned()
    {
        metadata.insert("source".to_owned(), source);
    }
    Ok(Some(Value::Object(metadata)))
}

fn session_lifecycle_notification(
    server_id: &str,
    root: Option<&Path>,
    level: LspNotificationLevel,
    body_lines: Vec<String>,
    active: bool,
) -> LspNotification {
    let mut lines = body_lines
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>();
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    LspNotification {
        key: session_notification_key(server_id, root),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active,
        action: None,
    }
}
