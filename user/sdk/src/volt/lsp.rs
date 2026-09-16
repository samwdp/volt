//! Language-server requests. Mirrors `vim.lsp` without blocking the UI thread.

use serde_json::{Value, json};

use super::{BufHandle, ClientId, Location, buf, store_callback, ui, with_host};

/// Live language-server ids attached to `buf`.
pub fn get_clients(buf: BufHandle) -> Vec<ClientId> {
    with_host(|host| {
        (host.lsp_client_ids)(buf.0)
            .into_iter()
            .map(|id| ClientId(id.into_string()))
            .collect()
    })
    .unwrap_or_default()
}

/// Sends an LSP request. The callback runs later on the UI thread via host dispatch.
pub fn request(
    client: &ClientId,
    method: &str,
    params: Value,
    callback: impl FnOnce(Result<Value, String>) + Send + 'static,
) {
    let Some(params) = serde_json::to_string(&params).ok() else {
        callback(Err("failed to encode LSP params".to_owned()));
        return;
    };
    let token = store_callback(Box::new(callback));
    let posted = with_host(|host| {
        (host.lsp_request)(
            client.0.as_str().into(),
            method.into(),
            params.as_str().into(),
            token,
        );
    });
    if posted.is_none()
        && let Some(callback) = super::take_callback(token)
    {
        callback(Err("volt host is not installed".to_owned()));
    }
}

/// Jumps to the definition under the cursor. Returns false when no buffer/client is ready.
pub fn goto_definition() -> bool {
    goto_locations("Definitions", "textDocument/definition")
}

/// Jumps to references under the cursor.
pub fn goto_references() -> bool {
    goto_locations("References", "textDocument/references")
}

/// Jumps to implementations under the cursor.
pub fn goto_implementation() -> bool {
    goto_locations("Implementations", "textDocument/implementation")
}

fn goto_locations(title: &'static str, method: &'static str) -> bool {
    let Some(buf) = buf::current() else {
        return false;
    };
    let Some(cursor) = buf::get_cursor(buf) else {
        return false;
    };
    let Some(uri) = buf::get_uri(buf) else {
        return false;
    };
    let Some(client) = get_clients(buf).into_iter().next() else {
        return false;
    };
    let mut params = json!({
        "textDocument": { "uri": uri },
        "position": { "line": cursor.0, "character": cursor.1 }
    });
    if method == "textDocument/references" {
        params["context"] = json!({ "includeDeclaration": true });
    }
    request(&client, method, params, move |result| {
        let Ok(value) = result else {
            return;
        };
        ui::open_locations(title, &parse_locations(&value));
    });
    true
}

pub(crate) fn parse_locations(value: &Value) -> Vec<Location> {
    match value {
        Value::Null => Vec::new(),
        Value::Array(entries) => entries.iter().filter_map(location_from_value).collect(),
        other => location_from_value(other).into_iter().collect(),
    }
}

fn location_from_value(value: &Value) -> Option<Location> {
    let target = value.get("targetUri").or_else(|| value.get("uri"))?;
    let uri = target.as_str()?.to_owned();
    let range = value
        .get("targetSelectionRange")
        .or_else(|| value.get("targetRange"))
        .or_else(|| value.get("range"))?;
    let start = range.get("start")?;
    let line = start.get("line")?.as_u64()?;
    let column = start.get("character")?.as_u64()?;
    Some(Location {
        uri,
        line: u32::try_from(line).ok()?,
        column: u32::try_from(column).ok()?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_locations_reads_location_and_link() {
        let location = json!({
            "uri": "file:///src/main.rs",
            "range": { "start": { "line": 2, "character": 4 }, "end": { "line": 2, "character": 8 } }
        });
        let parsed = parse_locations(&location);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].uri, "file:///src/main.rs");
        assert_eq!(parsed[0].line, 2);
        assert_eq!(parsed[0].column, 4);

        let link = json!({
            "targetUri": "file:///src/lib.rs",
            "targetSelectionRange": {
                "start": { "line": 1, "character": 0 },
                "end": { "line": 1, "character": 3 }
            }
        });
        let parsed = parse_locations(&json!([link]));
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].uri, "file:///src/lib.rs");
        assert_eq!(parsed[0].line, 1);
    }

    #[test]
    fn goto_definition_returns_false_without_host() {
        assert!(!goto_definition());
    }
}
