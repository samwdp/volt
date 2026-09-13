#![doc = r#"Filename-aware path matching shared across syntax, LSP, and hook dispatch."#]

pub use editor_plugin_api::{
    PathMatcher, PathPattern, PathPatternKind, grammar_install_root, normalize_extension,
    volt_data_dir,
};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Filename-aware path matching shared across syntax, LSP, and hook dispatch.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

#[cfg(test)]
mod tests;
