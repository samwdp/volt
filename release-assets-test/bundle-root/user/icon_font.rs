//! Re-exports of the bundled icon-font symbols from `editor-icons`.
//!
//! Access individual symbol constants via the `symbols` sub-module, e.g.:
//! `user::icon_font::symbols::cod::COD_DIFF_ADDED`
//!
//! Access the full runtime symbol table via `user::icon_font::symbols()`.

pub use editor_icons::symbols;
pub use editor_icons::{IconFontCategory, IconFontSymbol};

/// Returns the complete static list of bundled icon font symbols.
pub fn symbols() -> &'static [IconFontSymbol] {
    editor_icons::all_symbols()
}

/// Looks up a symbol by name (case-insensitive).
pub fn find(name: &str) -> Option<&'static IconFontSymbol> {
    editor_icons::find_symbol(name)
}
