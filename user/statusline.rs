/// Context made available to each user-defined statusline segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatuslineContext<'a> {
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
    /// Git statusline info, if available.
    pub git: Option<GitStatuslineInfo<'a>>,
}

/// Git data surfaced to the statusline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GitStatuslineInfo<'a> {
    pub branch: &'a str,
    pub added: usize,
    pub removed: usize,
}

/// Function signature used by user-defined statusline segments.
pub type StatuslineSegment = for<'a> fn(&StatuslineContext<'a>) -> Option<String>;

/// Returns the ordered statusline segment list.
///
/// Users can add, remove, or reorder segments by editing this vector and
/// pointing at additional functions in this file.
pub fn segments() -> Vec<StatuslineSegment> {
    vec![
        mode_segment,
        macro_recording_segment,
        workspace_segment,
        filetype_segment,
        buffer_segment,
        git_segment,
        position_segment,
        lsp_segment,
    ]
}

/// Renders the current statusline using the configured segments.
pub fn compose(context: &StatuslineContext<'_>) -> String {
    segments()
        .into_iter()
        .filter_map(|segment| segment(context))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn mode_segment(context: &StatuslineContext<'_>) -> Option<String> {
    Some(context.vim_mode.to_owned())
}

fn macro_recording_segment(context: &StatuslineContext<'_>) -> Option<String> {
    context
        .recording_macro
        .map(|register| format!("@{register}"))
}

fn workspace_segment(context: &StatuslineContext<'_>) -> Option<String> {
    Some(context.workspace_name.to_owned())
}

fn buffer_segment(context: &StatuslineContext<'_>) -> Option<String> {
    let name = context.buffer_name;
    if context.buffer_modified {
        let modified = crate::nerd_font::symbols::cod::COD_DIFF_MODIFIED;
        Some(format!("{modified} {name}"))
    } else {
        Some(name.to_owned())
    }
}

fn filetype_segment(context: &StatuslineContext<'_>) -> Option<String> {
    let language_id = context.language_id?;
    let symbol = match language_id {
        "rust" => crate::nerd_font::symbols::seti::SETI_RUST,
        "markdown" | "markdown-inline" => crate::nerd_font::symbols::seti::SETI_MARKDOWN,
        "gitcommit" => crate::nerd_font::symbols::cod::COD_GIT_COMMIT,
        _ => crate::nerd_font::symbols::cod::COD_FILE,
    };
    Some(symbol.to_owned())
}

fn git_segment(context: &StatuslineContext<'_>) -> Option<String> {
    let git = context.git?;
    let branch = crate::nerd_font::symbols::dev::DEV_GIT_BRANCH;
    let up = crate::nerd_font::symbols::cod::COD_ARROW_UP;
    let down = crate::nerd_font::symbols::cod::COD_ARROW_DOWN;
    Some(format!(
        "{branch} {} {up} {} {down} {}",
        git.branch, git.added, git.removed
    ))
}

fn position_segment(context: &StatuslineContext<'_>) -> Option<String> {
    Some(format!("Ln {}, Col {}", context.line, context.column))
}

fn lsp_segment(context: &StatuslineContext<'_>) -> Option<String> {
    context.lsp_server.map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{StatuslineContext, compose};

    #[test]
    fn compose_joins_the_default_user_segments() {
        let file_icon = crate::nerd_font::symbols::seti::SETI_RUST;
        let statusline = compose(&StatuslineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "*scratch*",
            buffer_modified: false,
            language_id: Some("rust"),
            line: 3,
            column: 9,
            lsp_server: Some("rust-analyzer"),
            git: None,
        });

        assert_eq!(
            statusline,
            format!("NORMAL | default | {file_icon} | *scratch* | Ln 3, Col 9 | rust-analyzer")
        );
    }

    #[test]
    fn compose_skips_empty_optional_segments() {
        let statusline = compose(&StatuslineContext {
            vim_mode: "INSERT",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "*scratch*",
            buffer_modified: false,
            language_id: None,
            line: 1,
            column: 1,
            lsp_server: None,
            git: None,
        });

        assert_eq!(statusline, "INSERT | default | *scratch* | Ln 1, Col 1");
    }

    #[test]
    fn compose_includes_macro_recording_register() {
        let statusline = compose(&StatuslineContext {
            vim_mode: "NORMAL",
            recording_macro: Some('q'),
            workspace_name: "default",
            buffer_name: "*scratch*",
            buffer_modified: false,
            language_id: None,
            line: 1,
            column: 1,
            lsp_server: None,
            git: None,
        });

        assert_eq!(
            statusline,
            "NORMAL | @q | default | *scratch* | Ln 1, Col 1"
        );
    }

    #[test]
    fn compose_includes_filetype_and_modified_icon() {
        let file_icon = crate::nerd_font::symbols::seti::SETI_MARKDOWN;
        let modified = crate::nerd_font::symbols::cod::COD_DIFF_MODIFIED;
        let statusline = compose(&StatuslineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "notes.md",
            buffer_modified: true,
            language_id: Some("markdown"),
            line: 1,
            column: 1,
            lsp_server: None,
            git: None,
        });

        assert_eq!(
            statusline,
            format!("NORMAL | default | {file_icon} | {modified} notes.md | Ln 1, Col 1")
        );
    }

    #[test]
    fn compose_includes_git_segment() {
        let file_icon = crate::nerd_font::symbols::seti::SETI_RUST;
        let branch = crate::nerd_font::symbols::dev::DEV_GIT_BRANCH;
        let up = crate::nerd_font::symbols::cod::COD_ARROW_UP;
        let down = crate::nerd_font::symbols::cod::COD_ARROW_DOWN;
        let statusline = compose(&StatuslineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "main.rs",
            buffer_modified: false,
            language_id: Some("rust"),
            line: 10,
            column: 2,
            lsp_server: None,
            git: Some(super::GitStatuslineInfo {
                branch: "main",
                added: 12,
                removed: 3,
            }),
        });

        assert_eq!(
            statusline,
            format!(
                "NORMAL | default | {file_icon} | main.rs | {branch} main {up} 12 {down} 3 | Ln 10, Col 2"
            )
        );
    }
}
