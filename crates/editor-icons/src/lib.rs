//! Bundled Nerd Font icon symbols for the Volt editor.
//!
//! This crate exposes the full Nerd Font icon set as compile-time string constants
//! and a runtime-accessible static slice.  Both the compiled user extension library
//! and the editor shell link against this crate so they agree on the same glyphs.

/// Individual icon symbol sub-modules (codicons, devicons, material-design, …).
pub mod symbols {
    pub use crate::nerd_font_symbols::*;
}

#[path = "../nerd_font_symbols/mod.rs"]
mod nerd_font_symbols;

/// One entry in the bundled icon font symbol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconFontSymbol {
    pub name: &'static str,
    pub glyph: &'static str,
    pub category: IconFontCategory,
}

impl IconFontSymbol {
    /// Returns a `"category:name"` identifier string for this symbol.
    pub fn id(&self) -> String {
        format!("{}:{}", self.category.id(), self.name)
    }

    /// Returns the Unicode codepoint label(s) for this symbol's glyph.
    pub fn codepoint_label(&self) -> String {
        self.glyph
            .chars()
            .map(|c| format!("U+{:04X}", c as u32))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Icon font category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconFontCategory {
    Cod,
    Dev,
    Fa,
    Fae,
    Iec,
    Logos,
    Md,
    Oct,
    Ple,
    Pom,
    Seti,
    Weather,
}

impl IconFontCategory {
    pub fn id(self) -> &'static str {
        match self {
            Self::Cod => "cod",
            Self::Dev => "dev",
            Self::Fa => "fa",
            Self::Fae => "fae",
            Self::Iec => "iec",
            Self::Logos => "logos",
            Self::Md => "md",
            Self::Oct => "oct",
            Self::Ple => "ple",
            Self::Pom => "pom",
            Self::Seti => "seti",
            Self::Weather => "weather",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Cod => "Codicons",
            Self::Dev => "Devicons",
            Self::Fa => "Font Awesome",
            Self::Fae => "Font Awesome Extension",
            Self::Iec => "IEC Power Symbols",
            Self::Logos => "Font Logos",
            Self::Md => "Material Design",
            Self::Oct => "Octicons",
            Self::Ple => "Powerline Extra",
            Self::Pom => "Pomicons",
            Self::Seti => "Seti",
            Self::Weather => "Weather",
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/icon_font_data.rs"));

/// Returns the complete static list of bundled icon font symbols.
pub fn all_symbols() -> &'static [IconFontSymbol] {
    ICON_FONT_SYMBOLS
}

/// Looks up a symbol by name (case-insensitive).
pub fn find_symbol(name: &str) -> Option<&'static IconFontSymbol> {
    ICON_FONT_SYMBOLS
        .iter()
        .find(|sym| sym.name.eq_ignore_ascii_case(name))
}
