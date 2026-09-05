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
fn permission_prompt_lines(request: &RequestPermissionRequest) -> Vec<String> {
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
fn format_acp_status_badge(status: &impl std::fmt::Debug) -> String {
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
fn humanize_debug_label(value: &str) -> String {
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

fn format_permission_option_kind(kind: PermissionOptionKind) -> &'static str {
    match kind {
        PermissionOptionKind::AllowOnce => "allow once",
        PermissionOptionKind::AllowAlways => "allow always",
        PermissionOptionKind::RejectOnce => "reject once",
        PermissionOptionKind::RejectAlways => "reject always",
        _ => "custom",
    }
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

#[cfg(test)]
#[path = "acp_tests.rs"]
mod tests;
