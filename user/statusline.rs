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
    /// 1-based cursor line.
    pub line: usize,
    /// 1-based cursor column.
    pub column: usize,
    /// Attached language server name, if any.
    pub lsp_server: Option<&'a str>,
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
        buffer_segment,
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
    Some(context.buffer_name.to_owned())
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
        let statusline = compose(&StatuslineContext {
            vim_mode: "NORMAL",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "*scratch*",
            line: 3,
            column: 9,
            lsp_server: Some("rust-analyzer"),
        });

        assert_eq!(
            statusline,
            "NORMAL | default | *scratch* | Ln 3, Col 9 | rust-analyzer"
        );
    }

    #[test]
    fn compose_skips_empty_optional_segments() {
        let statusline = compose(&StatuslineContext {
            vim_mode: "INSERT",
            recording_macro: None,
            workspace_name: "default",
            buffer_name: "*scratch*",
            line: 1,
            column: 1,
            lsp_server: None,
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
            line: 1,
            column: 1,
            lsp_server: None,
        });

        assert_eq!(statusline, "NORMAL | @q | default | *scratch* | Ln 1, Col 1");
    }
}
