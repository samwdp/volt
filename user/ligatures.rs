//! User-facing ligature configuration.

use editor_plugin_api::LigatureConfig;

/// Returns the ligature configuration exported to the host runtime.
pub fn config() -> LigatureConfig {
    LigatureConfig {
        enabled: crate::config::load().ui.ligatures_enabled,
    }
}

#[cfg(test)]
mod tests {
    use super::config;

    #[test]
    fn config_exposes_current_ligature_setting() {
        assert!(config().enabled);
    }
}
