fn parse_completion_item(server_id: &str, value: &Value) -> Option<LspCompletionItem> {
    let label = value.get("label")?.as_str()?.to_owned();
    let kind = value
        .get("kind")
        .and_then(Value::as_u64)
        .and_then(parse_completion_kind);
    // LSP: when textEdit is present, it owns the inserted text and range.
    // Prefer it over insertText so trigger characters (e.g. '.') are not doubled.
    let (insert_text, edit_range) = match value.get("textEdit") {
        Some(text_edit) => {
            let new_text = text_edit
                .get("newText")
                .and_then(Value::as_str)
                .or_else(|| value.get("insertText").and_then(Value::as_str))
                .unwrap_or(&label)
                .to_owned();
            let range = text_edit
                .get("replace")
                .or_else(|| text_edit.get("range"))
                .and_then(parse_inline_text_range);
            (new_text, range)
        }
        None => {
            let insert_text = value
                .get("insertText")
                .and_then(Value::as_str)
                .unwrap_or(&label)
                .to_owned();
            (insert_text, None)
        }
    };
    let detail = value
        .get("detail")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let documentation = value
        .get("documentation")
        .and_then(completion_documentation)
        .or_else(|| detail.clone());
    let has_documentation = documentation.is_some();
    Some(
        LspCompletionItem::new(
            server_id,
            kind,
            label,
            insert_text,
            edit_range,
            detail,
            documentation,
        )
        .with_raw_item(value.clone(), has_documentation),
    )
}

fn completion_documentation(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => Some(text.to_owned()),
        Value::Object(map) => map.get("value").and_then(Value::as_str).map(str::to_owned),
        _ => None,
    }
}

fn parse_completion_kind(kind: u64) -> Option<LspCompletionKind> {
    match kind {
        1 => Some(LspCompletionKind::Text),
        2 => Some(LspCompletionKind::Method),
        3 => Some(LspCompletionKind::Function),
        4 => Some(LspCompletionKind::Constructor),
        5 => Some(LspCompletionKind::Field),
        6 => Some(LspCompletionKind::Variable),
        7 => Some(LspCompletionKind::Class),
        8 => Some(LspCompletionKind::Interface),
        9 => Some(LspCompletionKind::Module),
        10 => Some(LspCompletionKind::Property),
        11 => Some(LspCompletionKind::Unit),
        12 => Some(LspCompletionKind::Value),
        13 => Some(LspCompletionKind::Enum),
        14 => Some(LspCompletionKind::Keyword),
        15 => Some(LspCompletionKind::Snippet),
        16 => Some(LspCompletionKind::Color),
        17 => Some(LspCompletionKind::File),
        18 => Some(LspCompletionKind::Reference),
        19 => Some(LspCompletionKind::Folder),
        20 => Some(LspCompletionKind::EnumMember),
        21 => Some(LspCompletionKind::Constant),
        22 => Some(LspCompletionKind::Struct),
        23 => Some(LspCompletionKind::Event),
        24 => Some(LspCompletionKind::Operator),
        25 => Some(LspCompletionKind::TypeParameter),
        _ => None,
    }
}

fn path_to_file_uri(path: &Path) -> String {
    let raw = path.to_string_lossy().replace('\\', "/");
    let mut uri = String::from("file://");
    if !raw.starts_with('/') {
        uri.push('/');
    }
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b':' | b'-' | b'.' | b'_' | b'~') {
            uri.push(char::from(byte));
        } else {
            uri.push('%');
            uri.push_str(&format!("{byte:02X}"));
        }
    }
    uri
}

fn path_to_uri(path: &Path) -> Result<Uri, LspClientError> {
    path_to_file_uri(path).parse().map_err(|error| {
        LspClientError::Protocol(format!(
            "failed to convert `{}` into a valid file URI: {error}",
            path.display()
        ))
    })
}

fn text_document_position_params(
    path: &Path,
    position: TextPoint,
) -> Result<TextDocumentPositionParams, LspClientError> {
    let line = u32::try_from(position.line).map_err(|_| {
        LspClientError::Protocol(format!(
            "line {} does not fit in LSP position range",
            position.line
        ))
    })?;
    let character = u32::try_from(position.column).map_err(|_| {
        LspClientError::Protocol(format!(
            "column {} does not fit in LSP position range",
            position.column
        ))
    })?;
    Ok(TextDocumentPositionParams {
        text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
        position: Position::new(line, character),
    })
}

fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw = uri.strip_prefix("file://")?;
    let decoded = percent_decode(raw);
    #[cfg(windows)]
    {
        let trimmed = decoded
            .strip_prefix('/')
            .filter(|value| value.as_bytes().get(1) == Some(&b':'))
            .unwrap_or(decoded.as_str());
        Some(PathBuf::from(trimmed.replace('/', "\\")))
    }
    #[cfg(not(windows))]
    {
        Some(PathBuf::from(decoded))
    }
}

fn percent_decode(raw: &str) -> String {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            let high = bytes[index + 1] as char;
            let low = bytes[index + 2] as char;
            let value = [high, low].iter().collect::<String>();
            if let Ok(byte) = u8::from_str_radix(&value, 16) {
                decoded.push(byte);
                index += 3;
                continue;
            }
        }
        decoded.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&decoded).into_owned()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
