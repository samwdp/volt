//! Doom-inspired modeline composition: left/right segments with themed parts.

use editor_plugin_api::{ModelinePart, ModelineSegment};

/// Context made available to each user-defined modeline segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelineContext<'a> {
    /// Current modal editing label.
    pub vim_mode: &'a str,
    /// Register currently recording a macro, if any.
    pub recording_macro: Option<char>,
    /// Active workspace display name.
    pub workspace_name: &'a str,
    /// Active buffer display name.
    pub buffer_name: &'a str,
    /// Whether the active buffer has unsaved changes.
    pub buffer_modified: bool,
    /// Active buffer language identifier, if any.
    pub language_id: Option<&'a str>,
    /// 1-based cursor line.
    pub line: usize,
    /// 1-based cursor column.
    pub column: usize,
    /// Attached language server name, if any.
    pub lsp_server: Option<&'a str>,
    /// Active buffer diagnostic summary, if any.
    pub lsp_diagnostics: Option<LspDiagnosticsInfo>,
    /// Whether an ACP client is connected.
    pub acp_connected: bool,
    /// Git modeline info, if available.
    pub git: Option<GitModelineInfo<'a>>,
}

/// Alias kept for statusline module re-exports.
pub type StatuslineContext<'a> = ModelineContext<'a>;

/// Git data surfaced to the modeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitModelineInfo<'a> {
    pub branch: &'a str,
    pub added: usize,
    pub removed: usize,
}

/// Alias kept for statusline module re-exports.
pub type GitStatuslineInfo<'a> = GitModelineInfo<'a>;

/// LSP diagnostics surfaced to the modeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LspDiagnosticsInfo {
    pub errors: usize,
    pub warnings: usize,
}

pub const TOKEN_FOREGROUND: &str = "ui.modeline.foreground";
pub const TOKEN_MUTED: &str = "ui.modeline.muted";
pub const TOKEN_DIAGNOSTIC_ERROR: &str = "ui.diagnostic.error";
pub const TOKEN_DIAGNOSTIC_WARNING: &str = "ui.diagnostic.warning";
pub const TOKEN_GIT_BRANCH: &str = "ui.modeline.git.branch";
pub const TOKEN_GIT_ADDED: &str = "ui.modeline.git.added";
pub const TOKEN_GIT_REMOVED: &str = "ui.modeline.git.removed";

pub const TOKEN_MODE_NORMAL_FG: &str = "ui.modeline.mode.normal.foreground";
pub const TOKEN_MODE_NORMAL_BG: &str = "ui.modeline.mode.normal.background";
pub const TOKEN_MODE_INSERT_FG: &str = "ui.modeline.mode.insert.foreground";
pub const TOKEN_MODE_INSERT_BG: &str = "ui.modeline.mode.insert.background";
pub const TOKEN_MODE_REPLACE_FG: &str = "ui.modeline.mode.replace.foreground";
pub const TOKEN_MODE_REPLACE_BG: &str = "ui.modeline.mode.replace.background";
pub const TOKEN_MODE_VISUAL_FG: &str = "ui.modeline.mode.visual.foreground";
pub const TOKEN_MODE_VISUAL_BG: &str = "ui.modeline.mode.visual.background";

pub const LSP_CONNECTED_ICON: &str = crate::icon_font::symbols::md::MD_LAN_CONNECT;
pub const LSP_ERROR_ICON: &str = crate::icon_font::symbols::cod::COD_ERROR;
pub const LSP_WARNING_ICON: &str = crate::icon_font::symbols::cod::COD_WARNING;

const SEGMENT_GAP: &str = " ";

/// Returns the ordered modeline as plain text (tests / simple callers).
pub fn compose(context: &ModelineContext<'_>) -> String {
    editor_plugin_api::flatten_modeline_text(&compose_modeline(context))
}

/// Returns doom-style left/right modeline segments for host paint.
pub fn compose_modeline(context: &ModelineContext<'_>) -> Vec<ModelineSegment> {
    let mut left = Vec::new();
    left.push(mode_segment(context));
    if context.acp_connected {
        left.push(ModelineSegment::left(vec![ModelinePart::fg(
            crate::icon_font::symbols::fa::FA_CONNECTDEVELOP,
            TOKEN_FOREGROUND,
        )]));
    }
    if let Some(register) = context.recording_macro {
        left.push(ModelineSegment::left(vec![ModelinePart::fg(
            format!("@{register}"),
            TOKEN_FOREGROUND,
        )]));
    }
    left.push(ModelineSegment::left(vec![ModelinePart::fg(
        context.workspace_name,
        TOKEN_MUTED,
    )]));
    if let Some(filetype) = filetype_symbol(context.language_id) {
        left.push(ModelineSegment::left(vec![ModelinePart::fg(
            filetype,
            TOKEN_FOREGROUND,
        )]));
    }
    left.push(buffer_segment(context));
    if let Some(git) = git_segment(context) {
        left.push(git);
    }

    let mut right = Vec::new();
    right.push(ModelineSegment::right(vec![ModelinePart::fg(
        format!("Ln {}, Col {}", context.line, context.column),
        TOKEN_MUTED,
    )]));
    if let Some(lsp) = lsp_segment(context) {
        right.push(lsp);
    }

    left.into_iter().chain(right).collect()
}

fn mode_segment(context: &ModelineContext<'_>) -> ModelineSegment {
    let (foreground, background) = mode_tokens(context.vim_mode);
    ModelineSegment::left(vec![ModelinePart::new(
        format!(" {} ", context.vim_mode),
        foreground,
        Some(background.into()),
    )])
}

fn mode_tokens(vim_mode: &str) -> (&'static str, &'static str) {
    let normalized = vim_mode.trim();
    let mode = normalized
        .strip_prefix("MC ")
        .unwrap_or(normalized)
        .to_ascii_uppercase();
    match mode.as_str() {
        "INSERT" => (TOKEN_MODE_INSERT_FG, TOKEN_MODE_INSERT_BG),
        "REPLACE" => (TOKEN_MODE_REPLACE_FG, TOKEN_MODE_REPLACE_BG),
        "VISUAL" => (TOKEN_MODE_VISUAL_FG, TOKEN_MODE_VISUAL_BG),
        _ => (TOKEN_MODE_NORMAL_FG, TOKEN_MODE_NORMAL_BG),
    }
}

fn buffer_segment(context: &ModelineContext<'_>) -> ModelineSegment {
    let name = context.buffer_name;
    let text = if context.buffer_modified {
        let modified = crate::icon_font::symbols::cod::COD_DIFF_MODIFIED;
        format!("{modified} {name}")
    } else {
        name.to_string()
    };
    ModelineSegment::left(vec![ModelinePart::fg(text, TOKEN_FOREGROUND)])
}

fn filetype_symbol(language_id: Option<&str>) -> Option<&'static str> {
    let language_id = language_id?;
    Some(match language_id {
        "c" => crate::icon_font::symbols::seti::SETI_C,
        "cpp" => crate::icon_font::symbols::seti::SETI_CPP,
        "css" => crate::icon_font::symbols::seti::SETI_CSS,
        "csharp" => crate::icon_font::symbols::seti::SETI_C_SHARP,
        "go" => crate::icon_font::symbols::seti::SETI_GO,
        "html" => crate::icon_font::symbols::seti::SETI_HTML,
        "javascript" | "jsx" => crate::icon_font::symbols::seti::SETI_JAVASCRIPT,
        "json" => crate::icon_font::symbols::seti::SETI_JSON,
        "rust" => crate::icon_font::symbols::seti::SETI_RUST,
        "python" => crate::icon_font::symbols::seti::SETI_PYTHON,
        "markdown" | "markdown-inline" => crate::icon_font::symbols::seti::SETI_MARKDOWN,
        "toml" => crate::icon_font::symbols::seti::CUSTOM_TOML,
        "sql" => crate::icon_font::symbols::cod::COD_DATABASE,
        "typescript" | "tsx" => crate::icon_font::symbols::seti::SETI_TYPESCRIPT,
        "zig" => crate::icon_font::symbols::seti::SETI_ZIG,
        "gitcommit" => crate::icon_font::symbols::cod::COD_GIT_COMMIT,
        _ => crate::icon_font::symbols::cod::COD_FILE,
    })
}

fn git_segment(context: &ModelineContext<'_>) -> Option<ModelineSegment> {
    let git = context.git?;
    let branch = crate::icon_font::symbols::dev::DEV_GIT_BRANCH;
    let up = crate::icon_font::symbols::cod::COD_ARROW_UP;
    let down = crate::icon_font::symbols::cod::COD_ARROW_DOWN;
    Some(ModelineSegment::left(vec![
        ModelinePart::fg(format!("{branch} {}", git.branch), TOKEN_GIT_BRANCH),
        ModelinePart::fg(format!("{up} {}", git.added), TOKEN_GIT_ADDED),
        ModelinePart::fg(format!("{down} {}", git.removed), TOKEN_GIT_REMOVED),
    ]))
}

fn lsp_segment(context: &ModelineContext<'_>) -> Option<ModelineSegment> {
    let server = context.lsp_server;
    let diagnostics = context.lsp_diagnostics;
    if server.is_none() && diagnostics.is_none() {
        return None;
    }

    let mut parts = Vec::new();
    if let Some(server) = server {
        parts.push(ModelinePart::fg(
            format!("{LSP_CONNECTED_ICON} {server}"),
            TOKEN_FOREGROUND,
        ));
    }
    if let Some(diagnostics) = diagnostics {
        if diagnostics.errors > 0 {
            parts.push(ModelinePart::fg(
                format!("{LSP_ERROR_ICON} {}", diagnostics.errors),
                TOKEN_DIAGNOSTIC_ERROR,
            ));
        }
        if diagnostics.warnings > 0 {
            parts.push(ModelinePart::fg(
                format!("{LSP_WARNING_ICON} {}", diagnostics.warnings),
                TOKEN_DIAGNOSTIC_WARNING,
            ));
        }
    }
    Some(ModelineSegment::right(parts))
}

/// Joins segment texts with a single space (used by tests asserting layout order).
pub fn compose_spaced(context: &ModelineContext<'_>) -> String {
    let mut out = String::new();
    for segment in compose_modeline(context) {
        let segment_text = segment
            .parts
            .iter()
            .map(|part| part.text.as_str())
            .collect::<Vec<_>>()
            .join(SEGMENT_GAP);
        if segment_text.is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push_str(SEGMENT_GAP);
        }
        out.push_str(&segment_text);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{
        ModelineContext, TOKEN_GIT_ADDED, TOKEN_GIT_REMOVED, TOKEN_MODE_INSERT_BG,
        TOKEN_MODE_NORMAL_BG, TOKEN_MODE_REPLACE_BG, TOKEN_MODE_VISUAL_BG, compose_modeline,
        compose_spaced, mode_tokens,
    };
    use editor_plugin_api::ModelineAlignment;

    #[test]
    fn compose_joins_default_left_and_right_segments() {
        let file_icon = crate::icon_font::symbols::seti::SETI_RUST;
        let lsp_icon = super::LSP_CONNECTED_ICON;
        let text = compose_spaced(&ModelineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "*scratch*",
            buffer_modified: false,
            language_id: Some("rust"),
            line: 3,
            column: 9,
            lsp_server: Some("rust-analyzer"),
            lsp_diagnostics: None,
            acp_connected: false,
            git: None,
        });

        assert_eq!(
            text,
            format!(" NORMAL  default {file_icon} *scratch* Ln 3, Col 9 {lsp_icon} rust-analyzer")
        );
    }

    #[test]
    fn compose_skips_empty_optional_segments() {
        let text = compose_spaced(&ModelineContext {
            vim_mode: "INSERT",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "*scratch*",
            buffer_modified: false,
            language_id: None,
            line: 1,
            column: 1,
            lsp_server: None,
            lsp_diagnostics: None,
            acp_connected: false,
            git: None,
        });

        assert_eq!(text, " INSERT  default *scratch* Ln 1, Col 1");
    }

    #[test]
    fn compose_includes_macro_recording_register() {
        let text = compose_spaced(&ModelineContext {
            vim_mode: "NORMAL",
            recording_macro: Some('q'),
            workspace_name: "default",
            buffer_name: "*scratch*",
            buffer_modified: false,
            language_id: None,
            line: 1,
            column: 1,
            lsp_server: None,
            lsp_diagnostics: None,
            acp_connected: false,
            git: None,
        });

        assert_eq!(text, " NORMAL  @q default *scratch* Ln 1, Col 1");
    }

    #[test]
    fn compose_includes_filetype_and_modified_icon() {
        let file_icon = crate::icon_font::symbols::seti::SETI_MARKDOWN;
        let modified = crate::icon_font::symbols::cod::COD_DIFF_MODIFIED;
        let text = compose_spaced(&ModelineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "notes.md",
            buffer_modified: true,
            language_id: Some("markdown"),
            line: 1,
            column: 1,
            lsp_server: None,
            lsp_diagnostics: None,
            acp_connected: false,
            git: None,
        });

        assert_eq!(
            text,
            format!(" NORMAL  default {file_icon} {modified} notes.md Ln 1, Col 1")
        );
    }

    #[test]
    fn compose_includes_git_multipart_segment() {
        let file_icon = crate::icon_font::symbols::seti::SETI_RUST;
        let branch = crate::icon_font::symbols::dev::DEV_GIT_BRANCH;
        let up = crate::icon_font::symbols::cod::COD_ARROW_UP;
        let down = crate::icon_font::symbols::cod::COD_ARROW_DOWN;
        let segments = compose_modeline(&ModelineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "main.rs",
            buffer_modified: false,
            language_id: Some("rust"),
            line: 10,
            column: 2,
            lsp_server: None,
            lsp_diagnostics: None,
            acp_connected: false,
            git: Some(super::GitModelineInfo {
                branch: "main",
                added: 12,
                removed: 3,
            }),
        });

        let git = segments
            .iter()
            .find(|segment| {
                segment.parts.len() == 3
                    && segment.parts[1].foreground == TOKEN_GIT_ADDED
                    && segment.parts[2].foreground == TOKEN_GIT_REMOVED
            })
            .expect("git segment");
        assert_eq!(git.alignment, ModelineAlignment::Left);
        assert_eq!(git.parts[0].text, format!("{branch} main"));
        assert_eq!(git.parts[1].text, format!("{up} 12"));
        assert_eq!(git.parts[2].text, format!("{down} 3"));

        let text = compose_spaced(&ModelineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "main.rs",
            buffer_modified: false,
            language_id: Some("rust"),
            line: 10,
            column: 2,
            lsp_server: None,
            lsp_diagnostics: None,
            acp_connected: false,
            git: Some(super::GitModelineInfo {
                branch: "main",
                added: 12,
                removed: 3,
            }),
        });
        assert_eq!(
            text,
            format!(
                " NORMAL  default {file_icon} main.rs {branch} main {up} 12 {down} 3 Ln 10, Col 2"
            )
        );
    }

    #[test]
    fn compose_places_position_and_lsp_on_the_right() {
        let segments = compose_modeline(&ModelineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "main.rs",
            buffer_modified: false,
            language_id: Some("rust"),
            line: 10,
            column: 2,
            lsp_server: Some("rust-analyzer"),
            lsp_diagnostics: Some(super::LspDiagnosticsInfo {
                errors: 3,
                warnings: 1,
            }),
            acp_connected: false,
            git: None,
        });

        assert!(
            segments
                .iter()
                .any(|segment| segment.alignment == ModelineAlignment::Right
                    && segment.parts.iter().any(|part| part.text.contains("Ln 10")))
        );
        assert!(segments.iter().any(|segment| {
            segment.alignment == ModelineAlignment::Right
                && segment
                    .parts
                    .iter()
                    .any(|part| part.foreground == super::TOKEN_DIAGNOSTIC_ERROR)
        }));
    }

    #[test]
    fn mode_tokens_map_all_vim_modes_including_multicursor() {
        assert_eq!(
            mode_tokens("NORMAL"),
            (super::TOKEN_MODE_NORMAL_FG, TOKEN_MODE_NORMAL_BG)
        );
        assert_eq!(
            mode_tokens("INSERT"),
            (super::TOKEN_MODE_INSERT_FG, TOKEN_MODE_INSERT_BG)
        );
        assert_eq!(
            mode_tokens("REPLACE"),
            (super::TOKEN_MODE_REPLACE_FG, TOKEN_MODE_REPLACE_BG)
        );
        assert_eq!(
            mode_tokens("VISUAL"),
            (super::TOKEN_MODE_VISUAL_FG, TOKEN_MODE_VISUAL_BG)
        );
        assert_eq!(
            mode_tokens("MC INSERT"),
            (super::TOKEN_MODE_INSERT_FG, TOKEN_MODE_INSERT_BG)
        );
        let mode = compose_modeline(&ModelineContext {
            vim_mode: "MC VISUAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "*scratch*",
            buffer_modified: false,
            language_id: None,
            line: 1,
            column: 1,
            lsp_server: None,
            lsp_diagnostics: None,
            acp_connected: false,
            git: None,
        });
        assert_eq!(
            mode[0].parts[0].background.as_deref(),
            Some(TOKEN_MODE_VISUAL_BG)
        );
    }
}
