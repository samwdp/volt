//! Nerd font symbols and metadata.

pub use nerd_font_symbols as symbols;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NerdFontCategory {
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

impl NerdFontCategory {
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
pub struct NerdFontSymbol {
    pub name: &'static str,
    pub glyph: &'static str,
    pub category: NerdFontCategory,
}

impl NerdFontSymbol {
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

include!(concat!(env!("OUT_DIR"), "/nerd_font_data.rs"));

pub fn symbols() -> &'static [NerdFontSymbol] {
    NERD_FONT_SYMBOLS
}

pub fn find(name: &str) -> Option<&'static NerdFontSymbol> {
    NERD_FONT_SYMBOLS
        .iter()
        .find(|symbol| symbol.name.eq_ignore_ascii_case(name))
}
