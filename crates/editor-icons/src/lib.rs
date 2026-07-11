//! Bundled Nerd Font icon symbols for the Volt editor.
//!
//! This crate exposes the full Nerd Font icon set as compile-time string constants
//! and a runtime-accessible static slice.  Both the compiled user extension library
//! and the editor shell link against this crate so they agree on the same glyphs.

use std::path::Path;

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

/// Returns the Seti-style icon used for a directory name in file lists.
pub fn seti_directory_icon(name: &str) -> &'static str {
    match name.to_ascii_lowercase().as_str() {
        ".git" => symbols::seti::CUSTOM_FOLDER_GIT,
        ".github" => symbols::seti::CUSTOM_FOLDER_GITHUB,
        "node_modules" => symbols::seti::CUSTOM_FOLDER_NPM,
        ".cargo" | ".config" | ".vscode" => symbols::seti::CUSTOM_FOLDER_CONFIG,
        _ => symbols::seti::CUSTOM_FOLDER,
    }
}

/// Returns the Seti-style icon used for a file path in file lists.
pub fn seti_file_icon(path: &Path) -> &'static str {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let file_name_lower = file_name.to_ascii_lowercase();
    match file_name_lower.as_str() {
        "cargo.toml" => return symbols::seti::CUSTOM_TOML,
        "cargo.lock" | "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" => {
            return symbols::seti::SETI_LOCK;
        }
        "dockerfile" | "docker-compose.yml" | "docker-compose.yaml" => {
            return symbols::seti::SETI_DOCKER;
        }
        "makefile" => return symbols::seti::SETI_MAKEFILE,
        "license" | "license.md" | "copying" => return symbols::seti::SETI_LICENSE,
        "readme" | "readme.md" | "readme.txt" => return symbols::seti::SETI_MARKDOWN,
        ".gitignore" | ".gitattributes" | ".gitmodules" => return symbols::seti::SETI_GIT,
        _ => {}
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase());

    match extension.as_deref() {
        Some("rs") => symbols::seti::SETI_RUST,
        Some("md") | Some("markdown") => symbols::seti::SETI_MARKDOWN,
        Some("toml") => symbols::seti::CUSTOM_TOML,
        Some("json") | Some("jsonc") => symbols::seti::SETI_JSON,
        Some("yaml") | Some("yml") | Some("ini") | Some("cfg") | Some("conf") | Some("env") => {
            symbols::seti::SETI_CONFIG
        }
        Some("html") | Some("htm") => symbols::seti::SETI_HTML,
        Some("css") | Some("scss") | Some("less") => symbols::seti::SETI_CSS,
        Some("js") | Some("mjs") | Some("cjs") | Some("jsx") => symbols::seti::SETI_JAVASCRIPT,
        Some("ts") | Some("tsx") => symbols::seti::SETI_TYPESCRIPT,
        Some("sh") | Some("bash") | Some("zsh") | Some("fish") | Some("ps1") | Some("bat")
        | Some("cmd") => symbols::seti::SETI_SHELL,
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("webp") | Some("svg")
        | Some("ico") | Some("bmp") | Some("tif") | Some("tiff") => symbols::seti::SETI_IMAGE,
        Some("pdf") => symbols::seti::SETI_PDF,
        Some("xml") => symbols::seti::SETI_XML,
        Some("csv") => symbols::seti::SETI_CSV,
        Some("c") | Some("h") => symbols::seti::SETI_C,
        Some("cs") => symbols::seti::SETI_C_SHARP,
        Some("cc") | Some("cpp") | Some("cxx") | Some("hpp") | Some("hh") | Some("hxx") => {
            symbols::seti::SETI_CPP
        }
        Some("go") => symbols::seti::SETI_GO,
        Some("java") => symbols::seti::SETI_JAVA,
        Some("py") | Some("pyi") | Some("pyw") => symbols::seti::SETI_PYTHON,
        Some("zip") | Some("7z") | Some("gz") | Some("xz") | Some("rar") | Some("tar") => {
            symbols::cod::COD_FILE_ZIP
        }
        Some("mp3") | Some("wav") | Some("ogg") | Some("mp4") | Some("mov") | Some("mkv") => {
            symbols::cod::COD_FILE_MEDIA
        }
        Some("lock") => symbols::seti::SETI_LOCK,
        _ => symbols::seti::CUSTOM_DEFAULT,
    }
}
