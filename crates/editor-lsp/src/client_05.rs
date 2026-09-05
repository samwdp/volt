fn progress_notification_key(server_id: &str, root: Option<&Path>, token: &str) -> String {
    format!(
        "progress:{server_id}:{}:{token}",
        notification_root_key(root)
    )
}

fn parse_progress_token_key(value: Option<&Value>) -> Option<String> {
    let token = value?;
    if let Some(token) = token.as_str() {
        return Some(token.to_owned());
    }
    token.as_u64().map(|token| token.to_string())
}

fn parse_optional_progress_text(value: Option<&Value>) -> Option<Option<String>> {
    value.map(|value| {
        value
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_owned)
    })
}

fn parse_progress_percentage(value: Option<&Value>) -> Option<Option<u32>> {
    value.map(|value| {
        value
            .as_u64()
            .and_then(|percentage| u32::try_from(percentage.min(100)).ok())
    })
}

fn progress_body_lines(track: &ProgressTrack) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(title) = track.title.as_deref() {
        lines.push(title.to_owned());
    }
    if let Some(message) = track.message.as_deref()
        && lines.last().is_none_or(|title| title != message)
    {
        lines.push(message.to_owned());
    }
    if lines.is_empty() {
        lines.push("Working".to_owned());
    }
    lines
}

fn completion_level_for_message(message: Option<&str>) -> LspNotificationLevel {
    let Some(message) = message else {
        return LspNotificationLevel::Success;
    };
    let normalized = message.to_ascii_lowercase();
    if normalized.contains("fail") || normalized.contains("error") {
        LspNotificationLevel::Error
    } else if normalized.contains("warn") {
        LspNotificationLevel::Warning
    } else {
        LspNotificationLevel::Success
    }
}

fn parse_progress_notification(
    server_id: &str,
    root: Option<&Path>,
    params: &Value,
    progress_tracks: &mut BTreeMap<String, ProgressTrack>,
) -> Option<LspNotification> {
    let token = parse_progress_token_key(params.get("token"))?;
    let value = params.get("value")?;
    let kind = value.get("kind")?.as_str()?;
    match kind {
        "begin" => {
            let title = parse_optional_progress_text(value.get("title")).flatten();
            let message = parse_optional_progress_text(value.get("message")).flatten();
            let percentage = parse_progress_percentage(value.get("percentage")).flatten();
            let track = ProgressTrack {
                title,
                message,
                percentage,
            };
            let progress = track.percentage.map(Some).unwrap_or(None);
            let body_lines = progress_body_lines(&track);
            progress_tracks.insert(token.clone(), track);
            Some(LspNotification {
                key: progress_notification_key(server_id, root, &token),
                server_id: server_id.to_owned(),
                root: root.map(Path::to_path_buf),
                level: LspNotificationLevel::Info,
                title: format!("LSP · {server_id}"),
                body_lines,
                progress: Some(LspNotificationProgress::new(progress)),
                active: true,
                action: None,
            })
        }
        "report" => {
            let track = progress_tracks.entry(token.clone()).or_default();
            if let Some(title) = parse_optional_progress_text(value.get("title")) {
                track.title = title;
            }
            if let Some(message) = parse_optional_progress_text(value.get("message")) {
                track.message = message;
            }
            if let Some(percentage) = parse_progress_percentage(value.get("percentage")) {
                track.percentage = percentage;
            }
            Some(LspNotification {
                key: progress_notification_key(server_id, root, &token),
                server_id: server_id.to_owned(),
                root: root.map(Path::to_path_buf),
                level: LspNotificationLevel::Info,
                title: format!("LSP · {server_id}"),
                body_lines: progress_body_lines(track),
                progress: Some(LspNotificationProgress::new(track.percentage)),
                active: true,
                action: None,
            })
        }
        "end" => {
            let mut track = progress_tracks.remove(&token).unwrap_or_default();
            if let Some(message) = parse_optional_progress_text(value.get("message")) {
                track.message = message;
            }
            Some(LspNotification {
                key: progress_notification_key(server_id, root, &token),
                server_id: server_id.to_owned(),
                root: root.map(Path::to_path_buf),
                level: completion_level_for_message(track.message.as_deref()),
                title: format!("LSP · {server_id}"),
                body_lines: progress_body_lines(&track),
                progress: track
                    .percentage
                    .map(|percentage| LspNotificationProgress::new(Some(percentage))),
                active: false,
                action: None,
            })
        }
        _ => None,
    }
}

fn parse_window_message_notification(
    method: &str,
    server_id: &str,
    root: Option<&Path>,
    params: &Value,
) -> Option<LspNotification> {
    // window/logMessage is output-channel traffic only (already in the transport log).
    // Do not promote it to UI toasts — servers such as ols misuse MessageType::Error
    // for benign startup lines like "Starting Odin Language Server …".
    if method != "window/showMessage" {
        return None;
    }
    parse_show_message_notification(server_id, root, params)
}

fn parse_show_message_notification(
    server_id: &str,
    root: Option<&Path>,
    params: &Value,
) -> Option<LspNotification> {
    let level = match params.get("type").and_then(Value::as_u64) {
        Some(1) => LspNotificationLevel::Error,
        Some(2) => LspNotificationLevel::Warning,
        Some(3) | Some(4) => LspNotificationLevel::Info,
        _ => LspNotificationLevel::Info,
    };
    if level != LspNotificationLevel::Error {
        return None;
    }
    let message = params
        .get("message")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())?
        .to_owned();
    let mut lines = vec![message.clone()];
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    Some(LspNotification {
        key: format!(
            "message:{server_id}:{}:{level:?}:{message}",
            notification_root_key(root)
        ),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active: false,
        action: None,
    })
}

fn status_notification_key(server_id: &str, root: Option<&Path>) -> String {
    format!("status:{server_id}:{}", notification_root_key(root))
}

fn parse_copilot_status_notification(
    server_id: &str,
    root: Option<&Path>,
    params: &Value,
) -> Option<LspNotification> {
    let kind = params
        .get("kind")
        .and_then(Value::as_str)
        .unwrap_or("Normal");
    let message = params
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim();
    let level = match kind {
        "Error" => LspNotificationLevel::Error,
        "Warning" => LspNotificationLevel::Warning,
        "Inactive" => LspNotificationLevel::Info,
        _ => LspNotificationLevel::Success,
    };
    if level != LspNotificationLevel::Error {
        return None;
    }
    let mut lines = vec![kind.to_owned()];
    if !message.is_empty() {
        lines.push(message.to_owned());
    }
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    let action = (is_copilot_server(server_id) && kind == "Error")
        .then_some(LspNotificationAction::CopilotSignIn);
    Some(LspNotification {
        key: status_notification_key(server_id, root),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active: matches!(kind, "Error" | "Warning" | "Inactive"),
        action,
    })
}

struct ServerRequestHandling {
    result: Value,
    notification: Option<LspNotification>,
}

fn server_request_response(
    server_id: &str,
    root: Option<&Path>,
    method: Option<&Value>,
    params: Option<&Value>,
    workspace_configuration: Option<&SessionWorkspaceConfiguration>,
) -> ServerRequestHandling {
    let result = match method.and_then(Value::as_str) {
        Some("workspace/configuration") => workspace_configuration
            .map(|workspace_configuration| workspace_configuration.response_for_request(params))
            .unwrap_or_else(|| workspace_configuration_null_response(params)),
        Some("workspace/workspaceFolders") => Value::Array(Vec::new()),
        Some("window/showMessageRequest") => params
            .and_then(|params| params.get("actions"))
            .and_then(Value::as_array)
            .and_then(|actions| actions.first())
            .cloned()
            .unwrap_or(Value::Null),
        Some("window/showDocument") => json!({ "success": false }),
        Some("client/registerCapability")
        | Some("client/unregisterCapability")
        | Some("window/workDoneProgress/create") => Value::Null,
        _ => Value::Null,
    };
    let notification = if matches!(method.and_then(Value::as_str), Some("window/showDocument")) {
        show_document_notification(server_id, root, params)
    } else {
        None
    };
    let result = if notification.is_some() {
        json!({ "success": true })
    } else {
        result
    };
    ServerRequestHandling {
        result,
        notification,
    }
}

fn show_document_notification(
    server_id: &str,
    root: Option<&Path>,
    params: Option<&Value>,
) -> Option<LspNotification> {
    if !is_copilot_server(server_id) {
        return None;
    }
    let uri = params
        .and_then(|params| params.get("uri"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|uri| uri.starts_with("http://") || uri.starts_with("https://"))?
        .to_owned();
    let mut lines = vec!["Opening browser popup".to_owned(), uri.clone()];
    if let Some(root) = root {
        lines.push(root.display().to_string());
    }
    Some(LspNotification {
        key: format!(
            "show-document:{server_id}:{}:{uri}",
            notification_root_key(root)
        ),
        server_id: server_id.to_owned(),
        root: root.map(Path::to_path_buf),
        level: LspNotificationLevel::Info,
        title: format!("LSP · {server_id}"),
        body_lines: lines,
        progress: None,
        active: false,
        action: Some(LspNotificationAction::OpenBrowserPopup { url: uri }),
    })
}

fn parse_publish_diagnostics(params: &Value) -> Option<(PathBuf, Vec<Diagnostic>)> {
    let uri = params.get("uri")?.as_str()?;
    let path = file_uri_to_path(uri)?;
    let diagnostics = params
        .get("diagnostics")
        .and_then(Value::as_array)
        .map(|diagnostics| {
            diagnostics
                .iter()
                .filter_map(parse_diagnostic)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    Some((path, diagnostics))
}

fn parse_diagnostic(value: &Value) -> Option<Diagnostic> {
    let range = value.get("range")?;
    let start = range.get("start")?;
    let end = range.get("end")?;
    let start = TextPoint::new(
        start.get("line")?.as_u64()? as usize,
        start.get("character")?.as_u64()? as usize,
    );
    let end = TextPoint::new(
        end.get("line")?.as_u64()? as usize,
        end.get("character")?.as_u64()? as usize,
    );
    let severity = match value.get("severity").and_then(Value::as_u64).unwrap_or(3) {
        1 => DiagnosticSeverity::Error,
        2 => DiagnosticSeverity::Warning,
        _ => DiagnosticSeverity::Information,
    };
    Some(Diagnostic::new(
        value
            .get("source")
            .and_then(Value::as_str)
            .unwrap_or("lsp")
            .to_owned(),
        value
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        severity,
        TextRange::new(start, end),
    ))
}

fn parse_hover_response(server_id: &str, value: &Value) -> Option<LspHoverContents> {
    let contents = value.get("contents")?;
    let (text, markdown) = hover_text(contents)?;
    let lines = hover_text_lines(&text);
    (!lines.is_empty()).then(|| LspHoverContents::new(server_id, text, lines, markdown))
}

fn parse_signature_help_response(
    server_id: &str,
    value: &Value,
) -> Result<Option<LspSignatureHelpContents>, LspClientError> {
    let signature_help =
        serde_json::from_value::<Option<SignatureHelp>>(value.clone()).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to decode signature help response from `{server_id}`: {error}"
            ))
        })?;
    let Some(signature_help) = signature_help else {
        return Ok(None);
    };
    Ok(signature_help_markdown(&signature_help, None)
        .is_some()
        .then(|| LspSignatureHelpContents::new(server_id, signature_help)))
}

fn parse_definition_response(
    server_id: &str,
    value: &Value,
) -> Result<Vec<LspLocation>, LspClientError> {
    if value.is_null() {
        return Ok(Vec::new());
    }
    let response =
        serde_json::from_value::<GotoDefinitionResponse>(value.clone()).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to decode location response from `{server_id}`: {error}"
            ))
        })?;
    Ok(match response {
        GotoDefinitionResponse::Scalar(location) => location_from_lsp(server_id, &location)
            .into_iter()
            .collect(),
        GotoDefinitionResponse::Array(locations) => locations
            .iter()
            .filter_map(|location| location_from_lsp(server_id, location))
            .collect(),
        GotoDefinitionResponse::Link(links) => links
            .iter()
            .filter_map(|link| location_from_link(server_id, link))
            .collect(),
    })
}

fn parse_reference_response(
    server_id: &str,
    value: &Value,
) -> Result<Vec<LspLocation>, LspClientError> {
    if value.is_null() {
        return Ok(Vec::new());
    }
    let locations = serde_json::from_value::<Vec<Location>>(value.clone()).map_err(|error| {
        LspClientError::Protocol(format!(
            "failed to decode reference response from `{server_id}`: {error}"
        ))
    })?;
    Ok(locations
        .iter()
        .filter_map(|location| location_from_lsp(server_id, location))
        .collect())
}

fn parse_text_edit_response(
    server_id: &str,
    format_kind: &str,
    value: &Value,
) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
    let edits =
        serde_json::from_value::<Option<Vec<TextEdit>>>(value.clone()).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to decode {format_kind} response from `{server_id}`: {error}"
            ))
        })?;
    Ok(edits.map(|edits| edits.iter().map(lsp_text_edit_from_lsp).collect::<Vec<_>>()))
}

fn parse_code_action_response(
    server_id: &str,
    value: &Value,
) -> Result<Vec<LspCodeAction>, LspClientError> {
    if value.is_null() {
        return Ok(Vec::new());
    }
    let Some(actions) = value.as_array() else {
        return Err(LspClientError::Protocol(format!(
            "failed to decode code action response from `{server_id}`: expected an array"
        )));
    };
    Ok(actions
        .iter()
        .filter_map(|action| parse_code_action_item(server_id, action))
        .collect())
}

fn parse_code_action_item(server_id: &str, value: &Value) -> Option<LspCodeAction> {
    let title = value.get("title")?.as_str()?.trim();
    if title.is_empty() {
        return None;
    }
    let kind = value.get("kind").and_then(Value::as_str).map(str::to_owned);
    let disabled_reason = value
        .get("disabled")
        .and_then(|disabled| disabled.get("reason"))
        .and_then(Value::as_str)
        .map(str::to_owned);
    let preferred = value
        .get("isPreferred")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let (document_edits, has_resource_operations) =
        parse_code_action_workspace_edit(value.get("edit"));
    let command_name = parse_code_action_command_name(value);
    Some(LspCodeAction {
        server_id: server_id.to_owned(),
        title: title.to_owned(),
        kind,
        disabled_reason,
        preferred,
        document_edits,
        command_name,
        has_resource_operations,
    })
}

fn parse_code_action_command_name(value: &Value) -> Option<String> {
    match value.get("command") {
        Some(Value::String(command)) => Some(command.to_owned()),
        Some(Value::Object(command)) => command
            .get("command")
            .and_then(Value::as_str)
            .map(str::to_owned),
        _ => None,
    }
}

fn parse_code_action_workspace_edit(value: Option<&Value>) -> (Vec<LspDocumentTextEdits>, bool) {
    let Some(value) = value else {
        return (Vec::new(), false);
    };
    let mut document_edits = Vec::new();
    let mut has_resource_operations = false;

    if let Some(changes) = value.get("changes").and_then(Value::as_object) {
        for (uri, edits_value) in changes {
            let Some(path) = file_uri_to_path(uri) else {
                continue;
            };
            let edits = parse_inline_text_edits(edits_value);
            if edits.is_empty() {
                continue;
            }
            document_edits.push(LspDocumentTextEdits::new(path, edits));
        }
    }

    if let Some(changes) = value.get("documentChanges").and_then(Value::as_array) {
        for change in changes {
            if let Some(document_edit) = parse_code_action_document_change(change) {
                document_edits.push(document_edit);
            } else if change.get("kind").is_some() {
                has_resource_operations = true;
            }
        }
    }

    (document_edits, has_resource_operations)
}

fn parse_code_action_document_change(value: &Value) -> Option<LspDocumentTextEdits> {
    let path = value
        .get("textDocument")
        .and_then(|text_document| text_document.get("uri"))
        .and_then(Value::as_str)
        .and_then(file_uri_to_path)?;
    let edits = parse_inline_text_edits(value.get("edits")?);
    (!edits.is_empty()).then(|| LspDocumentTextEdits::new(path, edits))
}

fn parse_inline_text_edits(value: &Value) -> Vec<LspTextEdit> {
    value
        .as_array()
        .map(|edits| {
            edits
                .iter()
                .filter_map(parse_inline_text_edit)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn parse_inline_text_edit(value: &Value) -> Option<LspTextEdit> {
    let range = parse_inline_text_range(value.get("range")?)?;
    let new_text = value.get("newText")?.as_str()?;
    Some(LspTextEdit::new(range, new_text))
}

fn parse_inline_text_range(value: &Value) -> Option<TextRange> {
    Some(TextRange::new(
        parse_inline_text_point(value.get("start")?)?,
        parse_inline_text_point(value.get("end")?)?,
    ))
}

fn parse_inline_text_point(value: &Value) -> Option<TextPoint> {
    let line = value.get("line").and_then(Value::as_u64)?;
    let character = value.get("character").and_then(Value::as_u64)?;
    Some(TextPoint::new(
        usize::try_from(line).ok()?,
        usize::try_from(character).ok()?,
    ))
}

fn diagnostic_matches_request_range(diagnostic_range: TextRange, request_range: TextRange) -> bool {
    let diagnostic_range = diagnostic_range.normalized();
    let request_range = request_range.normalized();
    if request_range.start() == request_range.end() {
        let point = request_range.start();
        return diagnostic_range.start() <= point && point <= diagnostic_range.end();
    }
    diagnostic_range.start() < request_range.end() && request_range.start() < diagnostic_range.end()
}

fn code_action_params(
    path: &Path,
    range: TextRange,
    diagnostics: &[Diagnostic],
    work_done_progress_params: WorkDoneProgressParams,
) -> Result<CodeActionParams, LspClientError> {
    Ok(CodeActionParams {
        text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
        range: lsp_range_from_text_range(range),
        context: CodeActionContext {
            diagnostics: diagnostics.iter().map(lsp_code_action_diagnostic).collect(),
            only: None,
            trigger_kind: Some(CodeActionTriggerKind::INVOKED),
        },
        work_done_progress_params,
        partial_result_params: PartialResultParams::default(),
    })
}

fn lsp_code_action_diagnostic(diagnostic: &Diagnostic) -> LspDiagnostic {
    LspDiagnostic::new(
        lsp_range_from_text_range(diagnostic.range()),
        Some(lsp_diagnostic_severity(diagnostic.severity())),
        None,
        Some(diagnostic.source().to_owned()),
        diagnostic.message().to_owned(),
        None,
        None,
    )
}

fn lsp_diagnostic_severity(severity: DiagnosticSeverity) -> LspDiagnosticSeverity {
    match severity {
        DiagnosticSeverity::Error => LspDiagnosticSeverity::ERROR,
        DiagnosticSeverity::Warning => LspDiagnosticSeverity::WARNING,
        DiagnosticSeverity::Information => LspDiagnosticSeverity::INFORMATION,
    }
}

fn hover_text(value: &Value) -> Option<(String, bool)> {
    let contents = serde_json::from_value::<HoverContents>(value.clone()).ok()?;
    match contents {
        HoverContents::Scalar(marked_string) => hover_marked_string(marked_string),
        HoverContents::Array(values) => {
            let parts = values
                .into_iter()
                .filter_map(hover_marked_string_markdown_text)
                .collect::<Vec<_>>();
            let text = normalize_hover_text(&parts.join("\n\n"));
            (!text.trim().is_empty()).then_some((text, true))
        }
        HoverContents::Markup(content) => {
            let text = normalize_hover_text(&content.value);
            (!text.trim().is_empty()).then_some((text, content.kind == MarkupKind::Markdown))
        }
    }
}

fn hover_marked_string(marked_string: MarkedString) -> Option<(String, bool)> {
    match marked_string {
        MarkedString::String(text) => {
            let text = normalize_hover_text(&text);
            (!text.trim().is_empty()).then_some((text, true))
        }
        MarkedString::LanguageString(language) => {
            let text =
                normalize_hover_text(&markdown_code_fence(&language.language, &language.value));
            (!text.trim().is_empty()).then_some((text, true))
        }
    }
}

fn hover_marked_string_markdown_text(marked_string: MarkedString) -> Option<String> {
    hover_marked_string(marked_string).map(|(text, _)| text)
}

fn markdown_code_fence(language: &str, value: &str) -> String {
    let value = normalize_hover_text(value);
    let language = language.trim();
    if language.is_empty() {
        format!("```\n{value}\n```")
    } else {
        format!("```{language}\n{value}\n```")
    }
}

fn hover_text_lines(text: &str) -> Vec<String> {
    let text = normalize_hover_text(text);
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut lines = text
        .split('\n')
        .map(str::trim_end)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines
}

fn normalize_hover_text(text: &str) -> String {
    text.replace("\r\n", "\n").replace('\r', "\n")
}

fn signature_help_markdown(
    signature_help: &SignatureHelp,
    language: Option<&str>,
) -> Option<String> {
    if signature_help.signatures.is_empty() {
        return None;
    }
    let active_signature_index = signature_help
        .active_signature
        .map(|index| index as usize)
        .filter(|index| *index < signature_help.signatures.len())
        .unwrap_or(0);
    let active_signature = signature_help.signatures.get(active_signature_index)?;
    let active_parameter_index = active_signature
        .active_parameter
        .or(signature_help.active_parameter)
        .map(|index| index as usize);
    let language = language.unwrap_or_default();
    let multiple_signatures = signature_help.signatures.len() > 1;
    let mut parts = Vec::new();
    for (index, signature) in signature_help.signatures.iter().enumerate() {
        if multiple_signatures {
            let active_marker = if index == active_signature_index {
                " (active)"
            } else {
                ""
            };
            parts.push(format!(
                "**Signature {}/{}{}**",
                index + 1,
                signature_help.signatures.len(),
                active_marker
            ));
        }
        parts.push(markdown_code_fence(language, &signature.label));
        if index == active_signature_index
            && let Some(parameter_documentation) = active_parameter_index
                .and_then(|parameter_index| {
                    signature
                        .parameters
                        .as_ref()
                        .and_then(|parameters| parameters.get(parameter_index))
                })
                .and_then(|parameter| parameter.documentation.as_ref())
        {
            parts.push(documentation_markdown(parameter_documentation));
        }
        if let Some(documentation) = signature.documentation.as_ref() {
            parts.push(documentation_markdown(documentation));
        }
    }
    let text = normalize_hover_text(&parts.join("\n\n"));
    (!text.trim().is_empty()).then_some(text)
}

fn documentation_markdown(documentation: &Documentation) -> String {
    match documentation {
        Documentation::String(text) => normalize_hover_text(text),
        Documentation::MarkupContent(content) => normalize_hover_text(&content.value),
    }
}

fn active_parameter_char_range(
    signature: &lsp_types::SignatureInformation,
    active_parameter_index: usize,
) -> Option<(usize, usize)> {
    let parameter = signature.parameters.as_ref()?.get(active_parameter_index)?;
    match &parameter.label {
        ParameterLabel::Simple(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return None;
            }
            find_substring_char_range(&signature.label, trimmed)
        }
        ParameterLabel::LabelOffsets([start, end]) => {
            let start = *start as usize;
            let end = *end as usize;
            let label_chars = signature.label.chars().count();
            (start <= end && end <= label_chars).then_some((start, end))
        }
    }
}

fn find_substring_char_range(haystack: &str, needle: &str) -> Option<(usize, usize)> {
    let needle_len = needle.chars().count();
    if needle_len == 0 {
        return None;
    }
    let haystack_chars = haystack.chars().collect::<Vec<_>>();
    for start in 0..=haystack_chars.len().saturating_sub(needle_len) {
        if haystack_chars[start..start + needle_len] == needle.chars().collect::<Vec<_>>()[..] {
            return Some((start, start + needle_len));
        }
    }
    None
}

fn location_from_lsp(server_id: &str, location: &Location) -> Option<LspLocation> {
    Some(LspLocation::from_uri(
        server_id,
        location.uri.to_string(),
        text_range_from_lsp_range(&location.range),
    ))
}

fn location_from_link(server_id: &str, link: &LocationLink) -> Option<LspLocation> {
    Some(LspLocation::from_uri(
        server_id,
        link.target_uri.to_string(),
        text_range_from_lsp_range(&link.target_selection_range),
    ))
}

fn lsp_text_edit_from_lsp(edit: &TextEdit) -> LspTextEdit {
    LspTextEdit::new(
        text_range_from_lsp_range(&edit.range),
        edit.new_text.clone(),
    )
}

fn text_range_from_lsp_range(range: &lsp_types::Range) -> TextRange {
    TextRange::new(
        text_point_from_lsp_position(range.start),
        text_point_from_lsp_position(range.end),
    )
}

fn text_point_from_lsp_position(position: Position) -> TextPoint {
    TextPoint::new(position.line as usize, position.character as usize)
}

fn lsp_range_from_text_range(range: TextRange) -> lsp_types::Range {
    lsp_types::Range {
        start: lsp_position_from_text_point(range.start()),
        end: lsp_position_from_text_point(range.end()),
    }
}

fn lsp_position_from_text_point(point: TextPoint) -> Position {
    Position::new(point.line as u32, point.column as u32)
}

fn lsp_formatting_options(options: LspFormattingOptions) -> FormattingOptions {
    FormattingOptions {
        tab_size: options.tab_size(),
        insert_spaces: options.insert_spaces(),
        ..FormattingOptions::default()
    }
}

fn unsupported_lsp_request(error: &LspClientError) -> bool {
    let LspClientError::Protocol(message) = error else {
        return false;
    };
    let lower = message.to_ascii_lowercase();
    lower.contains("-32601")
        || lower.contains("method not found")
        || lower.contains("method not supported")
}

fn sort_locations(locations: &mut Vec<LspLocation>) {
    locations.sort_by(|left, right| {
        left.uri
            .cmp(&right.uri)
            .then_with(|| left.range.start().line.cmp(&right.range.start().line))
            .then_with(|| left.range.start().column.cmp(&right.range.start().column))
            .then_with(|| left.range.end().line.cmp(&right.range.end().line))
            .then_with(|| left.range.end().column.cmp(&right.range.end().column))
    });
    locations.dedup_by(|left, right| left.uri == right.uri && left.range == right.range);
}

fn parse_completion_response(server_id: &str, value: &Value) -> Vec<LspCompletionItem> {
    let empty = Vec::new();
    let items = match value {
        Value::Array(items) => items,
        Value::Object(map) => map.get("items").and_then(Value::as_array).unwrap_or(&empty),
        _ => return Vec::new(),
    };
    items
        .iter()
        .filter_map(|item| parse_completion_item(server_id, item))
        .collect()
}

fn inline_completion_params(
    path: &Path,
    version: i32,
    position: TextPoint,
    options: LspFormattingOptions,
) -> Result<Value, LspClientError> {
    let position = text_document_position_params(path, position)?.position;
    Ok(json!({
        "textDocument": {
            "uri": path_to_uri(path)?,
            "version": version,
        },
        "position": position,
        "context": {
            "triggerKind": 2,
        },
        "formattingOptions": {
            "tabSize": options.tab_size(),
            "insertSpaces": options.insert_spaces(),
        }
    }))
}

fn parse_inline_completion_response(
    server_id: &str,
    root: Option<PathBuf>,
    position: TextPoint,
    value: &Value,
) -> Vec<LspInlineCompletionItem> {
    let empty = Vec::new();
    let items = match value {
        Value::Array(items) => items,
        Value::Object(map) => map.get("items").and_then(Value::as_array).unwrap_or(&empty),
        _ => return Vec::new(),
    };
    items
        .iter()
        .filter_map(|item| parse_inline_completion_item(server_id, root.clone(), position, item))
        .collect::<Vec<_>>()
}

fn parse_inline_completion_item(
    server_id: &str,
    root: Option<PathBuf>,
    position: TextPoint,
    value: &Value,
) -> Option<LspInlineCompletionItem> {
    let insert_text = value.get("insertText")?.as_str()?.replace("\r\n", "\n");
    if insert_text.is_empty() {
        return None;
    }
    let range = value
        .get("range")
        .and_then(parse_inline_text_range)
        .unwrap_or_else(|| TextRange::new(position, position));
    Some(LspInlineCompletionItem::new(
        server_id,
        root,
        insert_text,
        range,
        value.clone(),
    ))
}

fn execute_command_params_from_inline_item(value: &Value) -> Option<Value> {
    let command = parse_lsp_server_command(value.get("command"))?;
    Some(execute_command_params(&command))
}

fn execute_command_params(command: &LspServerCommand) -> Value {
    json!({
        "command": command.command(),
        "arguments": command.arguments(),
    })
}

fn parse_lsp_server_command(value: Option<&Value>) -> Option<LspServerCommand> {
    let value = value?;
    let title = value.get("title").and_then(Value::as_str)?.trim();
    let command = value.get("command").and_then(Value::as_str)?.trim();
    if title.is_empty() || command.is_empty() {
        return None;
    }
    let arguments = value
        .get("arguments")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Some(LspServerCommand::new(
        title.to_owned(),
        command.to_owned(),
        arguments,
    ))
}

fn parse_copilot_sign_in_response(value: &Value) -> Option<CopilotDeviceCodePrompt> {
    let user_code = value.get("userCode").and_then(Value::as_str)?.trim();
    let command = parse_lsp_server_command(value.get("command"))?;
    if user_code.is_empty() {
        return None;
    }
    Some(CopilotDeviceCodePrompt::new(user_code.to_owned(), command))
}
