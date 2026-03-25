//! Bundled icon-font symbols and metadata.

pub use crate::icon_font_symbols as symbols;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IconFontSymbol {
    pub name: &'static str,
    pub glyph: &'static str,
    pub category: IconFontCategory,
}

impl IconFontSymbol {
    pub fn id(&self) -> String {
        format!("{}:{}", self.category.id(), self.name)
    }

    pub fn codepoint_label(&self) -> String {
        let codepoints = self
            .glyph
            .chars()
            .map(|character| format!("U+{:04X}", character as u32))
            .collect::<Vec<_>>();
        codepoints.join(" ")
    }
}

include!(concat!(env!("OUT_DIR"), "/icon_font_data.rs"));

pub fn symbols() -> &'static [IconFontSymbol] {
    ICON_FONT_SYMBOLS
}

pub fn find(name: &str) -> Option<&'static IconFontSymbol> {
    ICON_FONT_SYMBOLS
        .iter()
        .find(|symbol| symbol.name.eq_ignore_ascii_case(name))
}
