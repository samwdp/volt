//! User-facing ligature configuration.

use editor_plugin_api::LigatureConfig;

/// Global toggle for OpenType coding-ligature shaping in the SDL shell.
pub const ENABLED: bool = true;

/// Returns the ligature configuration exported to the host runtime.
pub const fn config() -> LigatureConfig {
    LigatureConfig { enabled: ENABLED }
}

#[cfg(test)]
mod tests {
    use super::{ENABLED, config};

    #[test]
    fn config_exposes_current_ligature_setting() {
        assert_eq!(config().enabled, ENABLED);
    }
}
