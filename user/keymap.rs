//! User-facing keymap configuration (`ui.keymap.*`).

use editor_plugin_api::KeymapConfig;

/// Returns the keymap configuration exported to the host runtime.
pub fn config() -> KeymapConfig {
    let section = crate::config::load().ui.keymap;
    KeymapConfig {
        ambiguous_prefix_timeout_ms: section.ambiguous_prefix_timeout_ms,
    }
}

#[cfg(test)]
mod tests {
    use super::config;

    #[test]
    fn config_defaults_ambiguous_prefix_timeout_to_250ms() {
        assert_eq!(config().ambiguous_prefix_timeout_ms, 250);
    }
}
