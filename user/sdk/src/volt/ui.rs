//! UI helpers. Mirrors a thin slice of `vim.ui` / location lists.

use super::{Location, with_host};

/// Opens a single location, or a picker when `locations` has more than one entry.
pub fn open_locations(title: &str, locations: &[Location]) {
    open_locations_from(title, None, None, locations);
}

/// Opens locations and records which server document the jump started from.
pub fn open_locations_from(
    title: &str,
    server_id: Option<&str>,
    origin_uri: Option<&str>,
    locations: &[Location],
) {
    let entries = locations
        .iter()
        .map(|location| {
            serde_json::json!({
                "uri": location.uri,
                "line": location.line,
                "column": location.column,
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "serverId": server_id,
        "originUri": origin_uri,
        "locations": entries,
    });
    let Ok(encoded) = serde_json::to_string(&payload) else {
        return;
    };
    let _ = with_host(|host| {
        (host.ui_open_locations)(title.into(), encoded.as_str().into());
    });
}
