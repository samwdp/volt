//! UI helpers. Mirrors a thin slice of `vim.ui` / location lists.

use super::{Location, with_host};

/// Opens a single location, or a picker when `locations` has more than one entry.
pub fn open_locations(title: &str, locations: &[Location]) {
    let payload = serde_json::json!(
        locations
            .iter()
            .map(|location| serde_json::json!({
                "uri": location.uri,
                "line": location.line,
                "column": location.column,
            }))
            .collect::<Vec<_>>()
    );
    let Ok(encoded) = serde_json::to_string(&payload) else {
        return;
    };
    let _ = with_host(|host| {
        (host.ui_open_locations)(title.into(), encoded.as_str().into());
    });
}
