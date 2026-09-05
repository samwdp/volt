fn matches_comment_id(id: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| id == *candidate || id.starts_with(&format!("{candidate}-")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CommentStyle {
    Prefix(&'static str),
    Block {
        open: &'static str,
        close: &'static str,
    },
}

fn comment_style_for_language_path(
    language_id: Option<&str>,
    extension: Option<&str>,
    file_name: Option<&str>,
) -> Option<CommentStyle> {
    let language_id = language_id.map(str::to_ascii_lowercase);
    let extension = extension.map(str::to_ascii_lowercase);
    let file_name = file_name.map(str::to_ascii_lowercase);

    if language_id.as_deref().is_some_and(|id| {
        matches_comment_id(
            id,
            &[
                "rust",
                "c",
                "cpp",
                "csharp",
                "java",
                "javascript",
                "typescript",
                "jsx",
                "tsx",
                "go",
                "zig",
                "swift",
                "kotlin",
                "scala",
                "dart",
                "php",
                "odin",
                "proto",
                "solidity",
            ],
        )
    }) || extension.as_deref().is_some_and(|ext| {
        matches!(
            ext,
            "rs" | "c"
                | "h"
                | "cc"
                | "cpp"
                | "cs"
                | "java"
                | "js"
                | "jsx"
                | "ts"
                | "tsx"
                | "go"
                | "zig"
                | "swift"
                | "kt"
                | "kts"
                | "scala"
                | "dart"
                | "php"
                | "odin"
                | "proto"
                | "sol"
        )
    }) {
        return Some(CommentStyle::Prefix("//"));
    }

    if language_id
        .as_deref()
        .is_some_and(|id| matches_comment_id(id, &["lua", "sql", "haskell"]))
        || extension
            .as_deref()
            .is_some_and(|ext| matches!(ext, "lua" | "sql" | "hs"))
    {
        return Some(CommentStyle::Prefix("--"));
    }

    if language_id.as_deref().is_some_and(|id| {
        matches_comment_id(
            id,
            &[
                "python",
                "bash",
                "shell",
                "sh",
                "zsh",
                "fish",
                "yaml",
                "toml",
                "ruby",
                "perl",
                "r",
                "make",
                "gitcommit",
                "dockerfile",
                "nix",
                "nu",
                "powershell",
                "cmake",
                "elixir",
                "graphql",
                "hcl",
            ],
        )
    }) || extension.as_deref().is_some_and(|ext| {
        matches!(
            ext,
            "py" | "sh"
                | "bash"
                | "zsh"
                | "fish"
                | "yml"
                | "yaml"
                | "toml"
                | "rb"
                | "pl"
                | "r"
                | "mk"
                | "nix"
                | "nu"
                | "ps1"
                | "cmake"
                | "ex"
                | "exs"
                | "gql"
                | "graphql"
                | "graphqls"
                | "hcl"
                | "tf"
                | "nomad"
        )
    }) || file_name.as_deref().is_some_and(|name| {
        matches!(
            name,
            "makefile" | "dockerfile" | ".gitignore" | "cmakelists.txt"
        )
    }) {
        return Some(CommentStyle::Prefix("#"));
    }

    if language_id
        .as_deref()
        .is_some_and(|id| matches_comment_id(id, &["clojure"]))
        || extension
            .as_deref()
            .is_some_and(|ext| matches!(ext, "clj" | "cljs" | "cljc" | "edn"))
    {
        return Some(CommentStyle::Prefix(";"));
    }

    if language_id
        .as_deref()
        .is_some_and(|id| matches_comment_id(id, &["latex"]))
        || extension.as_deref().is_some_and(|ext| {
            matches!(
                ext,
                "tex" | "dtx" | "ins" | "sty" | "cls" | "rd" | "bbx" | "cbx"
            )
        })
    {
        return Some(CommentStyle::Prefix("%"));
    }

    if language_id
        .as_deref()
        .is_some_and(|id| matches_comment_id(id, &["vim"]))
        || extension.as_deref().is_some_and(|ext| ext == "vim")
    {
        return Some(CommentStyle::Prefix("\""));
    }

    if language_id
        .as_deref()
        .is_some_and(|id| matches_comment_id(id, &["css", "scss", "json"]))
        || extension
            .as_deref()
            .is_some_and(|ext| matches!(ext, "css" | "scss" | "json"))
    {
        return Some(CommentStyle::Block {
            open: "/*",
            close: "*/",
        });
    }

    if language_id
        .as_deref()
        .is_some_and(|id| matches_comment_id(id, &["html", "markdown", "xml"]))
        || extension.as_deref().is_some_and(|ext| {
            matches!(
                ext,
                "html" | "htm" | "md" | "markdown" | "xml" | "svg" | "xsd" | "xslt" | "xsl" | "rng"
            )
        })
    {
        return Some(CommentStyle::Block {
            open: "<!--",
            close: "-->",
        });
    }

    None
}

fn comment_style_for_buffer(buffer: &ShellBuffer) -> Option<CommentStyle> {
    comment_style_for_language_path(
        buffer.language_id(),
        buffer
            .path()
            .and_then(Path::extension)
            .and_then(|value| value.to_str()),
        buffer
            .path()
            .and_then(Path::file_name)
            .and_then(|value| value.to_str()),
    )
}

fn comment_toggle_removal_len(content: &str, prefix: &str) -> Option<usize> {
    let rest = content.strip_prefix(prefix)?;
    Some(prefix.chars().count() + usize::from(rest.starts_with(' ')))
}

fn block_comment_toggle_removal_lens(
    content: &str,
    open: &str,
    close: &str,
) -> Option<(usize, usize)> {
    let rest = content.strip_prefix(open)?;
    let start_remove = open.chars().count() + usize::from(rest.starts_with(' '));
    let commented = rest.strip_prefix(' ').unwrap_or(rest);
    let before_close = commented.strip_suffix(close)?;
    let end_remove = close.chars().count() + usize::from(before_close.ends_with(' '));
    Some((start_remove, end_remove))
}

fn toggle_line_comments_in_range(
    runtime: &mut EditorRuntime,
    start_line: usize,
    end_line: usize,
    cursor: TextPoint,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (style, end_line) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let style = comment_style_for_buffer(buffer).ok_or_else(|| {
            let language = buffer.language_id().unwrap_or("this buffer");
            format!("comment toggling is unavailable for `{language}`")
        })?;
        let end_line = end_line.min(buffer.line_count().saturating_sub(1));
        (style, end_line)
    };
    let uncomment = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let mut saw_non_blank = false;
        let mut uncomment = true;
        for line_index in start_line..=end_line {
            let line = buffer.text.line(line_index).unwrap_or_default();
            if line.trim().is_empty() {
                continue;
            }
            saw_non_blank = true;
            let (_, indent_end) = leading_whitespace_info(&line, 1);
            let content = &line[indent_end..];
            uncomment &= match style {
                CommentStyle::Prefix(prefix) => {
                    comment_toggle_removal_len(content, prefix).is_some()
                }
                CommentStyle::Block { open, close } => {
                    block_comment_toggle_removal_lens(content, open, close).is_some()
                }
            };
        }
        if !saw_non_blank {
            return Ok(());
        }
        uncomment
    };
    let buffer = active_shell_buffer_mut(runtime)?;
    let cursor_line = cursor.line.min(buffer.line_count().saturating_sub(1));
    let mut cursor_column = cursor.column.min(buffer.line_len_chars(cursor_line));
    for line_index in start_line..=end_line {
        let line = buffer.text.line(line_index).unwrap_or_default();
        if line.trim().is_empty() {
            continue;
        }
        let (indent_chars, indent_end) = leading_whitespace_info(&line, 1);
        let content = &line[indent_end..];
        match style {
            CommentStyle::Prefix(prefix) => {
                let insert = format!("{prefix} ");
                if uncomment {
                    let Some(remove_chars) = comment_toggle_removal_len(content, prefix) else {
                        continue;
                    };
                    buffer.replace_range(
                        TextRange::new(
                            TextPoint::new(line_index, indent_chars),
                            TextPoint::new(line_index, indent_chars.saturating_add(remove_chars)),
                        ),
                        "",
                    );
                    if line_index == cursor_line && cursor_column > indent_chars {
                        cursor_column = cursor_column.saturating_sub(remove_chars);
                    }
                } else {
                    buffer.replace_range(
                        TextRange::new(
                            TextPoint::new(line_index, indent_chars),
                            TextPoint::new(line_index, indent_chars),
                        ),
                        &insert,
                    );
                    if line_index == cursor_line && cursor_column >= indent_chars {
                        cursor_column = cursor_column.saturating_add(insert.chars().count());
                    }
                }
            }
            CommentStyle::Block { open, close } => {
                if uncomment {
                    let Some((remove_start_chars, remove_end_chars)) =
                        block_comment_toggle_removal_lens(content, open, close)
                    else {
                        continue;
                    };
                    let line_len_chars = line.chars().count();
                    let suffix_start = line_len_chars.saturating_sub(remove_end_chars);
                    buffer.replace_range(
                        TextRange::new(
                            TextPoint::new(line_index, suffix_start),
                            TextPoint::new(line_index, line_len_chars),
                        ),
                        "",
                    );
                    if line_index == cursor_line && cursor_column > suffix_start {
                        cursor_column = suffix_start;
                    }
                    buffer.replace_range(
                        TextRange::new(
                            TextPoint::new(line_index, indent_chars),
                            TextPoint::new(
                                line_index,
                                indent_chars.saturating_add(remove_start_chars),
                            ),
                        ),
                        "",
                    );
                    if line_index == cursor_line && cursor_column > indent_chars {
                        cursor_column = cursor_column.saturating_sub(remove_start_chars);
                    }
                } else {
                    let prefix_insert = format!("{open} ");
                    let suffix_insert = format!(" {close}");
                    let line_len_chars = line.chars().count();
                    buffer.replace_range(
                        TextRange::new(
                            TextPoint::new(line_index, line_len_chars),
                            TextPoint::new(line_index, line_len_chars),
                        ),
                        &suffix_insert,
                    );
                    buffer.replace_range(
                        TextRange::new(
                            TextPoint::new(line_index, indent_chars),
                            TextPoint::new(line_index, indent_chars),
                        ),
                        &prefix_insert,
                    );
                    if line_index == cursor_line && cursor_column >= indent_chars {
                        cursor_column = cursor_column.saturating_add(prefix_insert.chars().count());
                    }
                }
            }
        }
    }
    buffer.set_cursor(TextPoint::new(
        cursor_line,
        cursor_column.min(buffer.line_len_chars(cursor_line)),
    ));
    buffer.mark_syntax_dirty();
    Ok(())
}

fn toggle_current_line_comment(runtime: &mut EditorRuntime, count: usize) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        shell_ui_mut(runtime)?.enter_normal_mode();
        schedule_finish_change(runtime)?;
        return Ok(());
    }
    let cursor = active_shell_buffer_mut(runtime)?.cursor_point();
    let end_line = cursor.line.saturating_add(count.saturating_sub(1)).min(
        active_shell_buffer_mut(runtime)?
            .line_count()
            .saturating_sub(1),
    );
    toggle_line_comments_in_range(runtime, cursor.line, end_line, cursor)?;
    shell_ui_mut(runtime)?.enter_normal_mode();
    apply_directory_edit_queue_if_needed(runtime)?;
    schedule_finish_change(runtime)?;
    Ok(())
}

fn toggle_visual_selection_comments(runtime: &mut EditorRuntime) -> Result<(), String> {
    if active_shell_buffer_vim_targets_input(runtime)? {
        shell_ui_mut(runtime)?.enter_normal_mode();
        schedule_finish_change(runtime)?;
        return Ok(());
    }
    let (start_line, end_line, cursor, anchor, kind) = current_visual_line_span(runtime)?;
    store_last_visual_selection(runtime, anchor, cursor, kind)?;
    let target = {
        let buffer = shell_buffer(runtime, active_shell_buffer_id(runtime)?)?;
        buffer
            .text
            .first_non_blank_in_line(start_line)
            .unwrap_or(TextPoint::new(start_line, 0))
    };
    toggle_line_comments_in_range(runtime, start_line, end_line, target)?;
    shell_ui_mut(runtime)?.enter_normal_mode();
    apply_directory_edit_queue_if_needed(runtime)?;
    schedule_finish_change(runtime)?;
    Ok(())
}
