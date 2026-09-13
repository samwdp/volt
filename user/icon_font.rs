//! Re-exports of the bundled icon-font symbols from the Plugin SDK.
//!
//! Access individual symbol constants via the `symbols` sub-module, e.g.:
//! `user::icon_font::symbols::cod::COD_DIFF_ADDED`
//!
//! Access the full runtime symbol table via `user::icon_font::symbols()`.

pub use editor_plugin_api::symbols;
pub use editor_plugin_api::{IconFontCategory, IconFontSymbol};

/// Returns the complete static list of bundled icon font symbols.
pub fn symbols() -> &'static [IconFontSymbol] {
    editor_plugin_api::all_symbols()
}

/// Looks up a symbol by name (case-insensitive).
pub fn find(name: &str) -> Option<&'static IconFontSymbol> {
    editor_plugin_api::find_symbol(name)
}
