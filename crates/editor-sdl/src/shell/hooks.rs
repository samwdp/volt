fn register_shell_hooks(runtime: &mut EditorRuntime) -> Result<(), String> {
    register_hook(runtime, HOOK_MOVE_LEFT, "Moves the active cursor left.")?;
    register_hook(runtime, HOOK_MOVE_DOWN, "Moves the active cursor down.")?;
    register_hook(runtime, HOOK_MOVE_UP, "Moves the active cursor up.")?;
    register_hook(runtime, HOOK_MOVE_RIGHT, "Moves the active cursor right.")?;
    register_hook(
        runtime,
        HOOK_MOVE_WORD_FORWARD,
        "Moves the active cursor to the next word.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_WORD_BACKWARD,
        "Moves the active cursor to the previous word.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_WORD_END,
        "Moves the active cursor to the end of the current or next word.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_BIG_WORD_FORWARD,
        "Moves the active cursor to the next Vim WORD.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_BIG_WORD_BACKWARD,
        "Moves the active cursor to the previous Vim WORD.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_BIG_WORD_END,
        "Moves the active cursor to the end of the current or next Vim WORD.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SENTENCE_FORWARD,
        "Moves the active cursor to the start of the next sentence.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SENTENCE_BACKWARD,
        "Moves the active cursor to the start of the current or previous sentence.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_PARAGRAPH_FORWARD,
        "Moves the active cursor to the start of the next paragraph.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_PARAGRAPH_BACKWARD,
        "Moves the active cursor to the start of the current or previous paragraph.",
    )?;
    register_hook(
        runtime,
        HOOK_MATCH_PAIR,
        "Moves the active cursor to the matching paired delimiter.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_LINE_START,
        "Moves to the start of the current line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_LINE_FIRST_NON_BLANK,
        "Moves to the first non-blank character on the current line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_LINE_END,
        "Moves to the end of the current line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SCREEN_TOP,
        "Moves to the first visible screen line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SCREEN_MIDDLE,
        "Moves to the middle visible screen line.",
    )?;
    register_hook(
        runtime,
        HOOK_MOVE_SCREEN_BOTTOM,
        "Moves to the last visible screen line.",
    )?;
    register_hook(
        runtime,
        HOOK_GOTO_FIRST_LINE,
        "Moves to the first line in the buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_GOTO_LAST_LINE,
        "Moves to the last line in the buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_CURRENT_LINE_TOP,
        "Redraws with the current line at the top of the window.",
    )?;
    register_hook(
        runtime,
        HOOK_CENTER_CURRENT_LINE,
        "Redraws with the current line at the center of the window.",
    )?;
    register_hook(
        runtime,
        HOOK_CURRENT_LINE_BOTTOM,
        "Redraws with the current line at the bottom of the window.",
    )?;
    register_hook(
        runtime,
        HOOK_MODE_INSERT,
        "Switches the shell into insert mode.",
    )?;
    register_hook(
        runtime,
        HOOK_MODE_NORMAL,
        "Switches the shell into normal mode.",
    )?;
    register_hook(runtime, HOOK_VIM_EDIT, "Runs a Vim editing action.")?;
    register_hook(
        runtime,
        HOOK_VIM_COMMAND_LINE,
        "Opens the Vim command line under the active status line.",
    )?;
    register_hook(
        runtime,
        HOOK_BUFFER_SAVE,
        "Saves the active file-backed buffer.",
    )?;
    register_hook(runtime, HOOK_BUFFER_CLOSE, "Closes the active buffer.")?;
    register_hook(
        runtime,
        HOOK_BUFFER_TOGGLE_LINE_WRAP,
        "Toggles automatic line wrapping for the active buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_SAVE,
        "Saves all modified file buffers in the active workspace.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_NEXT,
        "Switches to the next open Project Workspace in open order.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_PREVIOUS,
        "Switches to the previous open Project Workspace in open order.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_MARK,
        "Adds the active Project Workspace root to the Mark List.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_UNMARK,
        "Removes the active Project Workspace root from the Mark List.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_MARKS,
        "Opens the app-wide Mark List as an editable buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_MARKED_1,
        "Jumps to Mark List slot 1 (first Marked Workspace).",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_MARKED_2,
        "Jumps to Mark List slot 2 (second Marked Workspace).",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_MARKED_3,
        "Jumps to Mark List slot 3 (third Marked Workspace).",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_MARKED_4,
        "Jumps to Mark List slot 4 (fourth Marked Workspace).",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WORKTREE_REMOVE,
        "Force-removes the selected Worktree from disk after closing matching Project Workspaces.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_FORMAT,
        "Formats the active buffer or visual selection.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_FORMATTER_REGISTER,
        "Registers a language formatter for workspace.format.",
    )?;
    register_hook(runtime, HOOK_PICKER_OPEN, "Opens a named picker provider.")?;
    register_hook(
        runtime,
        HOOK_PICKER_NEXT,
        "Moves the picker selection down.",
    )?;
    register_hook(
        runtime,
        HOOK_PICKER_PREVIOUS,
        "Moves the picker selection up.",
    )?;
    register_hook(
        runtime,
        HOOK_PICKER_SUBMIT,
        "Executes the selected picker action.",
    )?;
    register_hook(runtime, HOOK_PICKER_CANCEL, "Closes the active picker.")?;
    register_hook(
        runtime,
        HOOK_QUICKFIX_OPEN,
        "Opens current quickfix popup buffer.",
    )?;
    register_hook(runtime, HOOK_QUICKFIX_NEXT, "Opens next quickfix entry.")?;
    register_hook(
        runtime,
        HOOK_QUICKFIX_PREVIOUS,
        "Opens previous quickfix entry.",
    )?;
    register_hook(
        runtime,
        HOOK_QUICKFIX_TOGGLE_MARK,
        "Toggles mark on current quickfix row.",
    )?;
    register_hook(
        runtime,
        HOOK_QUICKFIX_CLEAR_MARKS,
        "Clears all quickfix marks.",
    )?;
    register_hook(runtime, HOOK_QUICKFIX_MARK_ALL, "Marks all quickfix rows.")?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_TRIGGER,
        "Opens autocomplete for the active insert buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_NEXT,
        "Moves to the next autocomplete suggestion.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_PREVIOUS,
        "Moves to the previous autocomplete suggestion.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_ACCEPT,
        "Accepts the selected autocomplete suggestion.",
    )?;
    register_hook(
        runtime,
        HOOK_AUTOCOMPLETE_CANCEL,
        "Closes the active autocomplete window.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_TOGGLE,
        "Shows or closes the hover overlay at the cursor without focusing it.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_FOCUS,
        "Moves focus into the hover overlay at the cursor.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_NEXT,
        "Moves to the next hover provider tab.",
    )?;
    register_hook(
        runtime,
        HOOK_HOVER_PREVIOUS,
        "Moves to the previous hover provider tab.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_DISCONNECT,
        "Disconnects the active ACP session.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PERMISSION_APPROVE,
        "Approves the latest ACP permission request.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PERMISSION_DENY,
        "Denies the latest ACP permission request.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PICK_SESSION,
        "Opens the ACP session picker for the active client.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_NEW_SESSION,
        "Creates a new ACP session for the active client in a new buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PICK_MODE,
        "Opens the ACP mode picker for the active session.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_PICK_MODEL,
        "Opens the ACP model picker for the active session.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_CYCLE_MODE,
        "Cycles to the next ACP session mode.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_SWITCH_PANE,
        "Switches focus between the ACP plan and output panes.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_COMPLETE_SLASH,
        "Opens ACP slash command completion for the active input.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_FOCUS_INPUT,
        "Focuses the ACP input section.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_ZOOM_IN,
        "Zooms the active native image buffer in.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_ZOOM_OUT,
        "Zooms the active native image buffer out.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_ZOOM_RESET,
        "Resets the active native image buffer to its fitted zoom.",
    )?;
    register_hook(
        runtime,
        HOOK_IMAGE_TOGGLE_MODE,
        "Toggles the active SVG image buffer between preview and source mode.",
    )?;
    register_hook(
        runtime,
        HOOK_MARKDOWN_PRETTY_TOGGLE,
        "Toggles Markdown Pretty for the active buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_RAINBOW_PARENS_TOGGLE,
        "Toggles rainbow delimiter highlighting for the active buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_SHOW_PAREN_TOGGLE,
        "Toggles show-paren highlighting for the active buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_PREVIOUS_PAGE,
        "Moves the active PDF buffer to the previous page.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_NEXT_PAGE,
        "Moves the active PDF buffer to the next page.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_ROTATE_CLOCKWISE,
        "Rotates the active PDF page clockwise.",
    )?;
    register_hook(
        runtime,
        HOOK_PDF_DELETE_PAGE,
        "Deletes the active PDF page.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_HALF_PAGE_DOWN,
        "Scrolls down by half a page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_HALF_PAGE_UP,
        "Scrolls up by half a page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_PAGE_DOWN,
        "Scrolls down by a full page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_PAGE_UP,
        "Scrolls up by a full page in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_LINE_DOWN,
        "Scrolls the viewport down by one line in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_SCROLL_LINE_UP,
        "Scrolls the viewport up by one line in Vim normal mode.",
    )?;
    register_hook(
        runtime,
        HOOK_POPUP_TOGGLE,
        "Shows or closes the docked popup window.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_DOCK_TOGGLE,
        "Shows or hides the workspace dock when it is not permanently docked.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_DOCK_PREVIOUS,
        "Moves to the previous workspace in the dock list.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_DOCK_NEXT,
        "Moves to the next workspace in the dock list.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_DOCK_TOGGLE,
        "Shows or hides the ACP dock for the active workspace.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_DOCK_PREVIOUS,
        "Moves to the previous ACP buffer in the dock list.",
    )?;
    register_hook(
        runtime,
        HOOK_ACP_DOCK_NEXT,
        "Moves to the next ACP buffer in the dock list.",
    )?;
    register_hook(runtime, HOOK_POPUP_NEXT, "Cycles to the next popup buffer.")?;
    register_hook(
        runtime,
        HOOK_POPUP_PREVIOUS,
        "Cycles to the previous popup buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_SPLIT_HORIZONTAL,
        "Splits the active workspace horizontally.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_SPLIT_VERTICAL,
        "Splits the active workspace vertically.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_CLOSE,
        "Closes the currently focused split.",
    )?;
    register_hook(
        runtime,
        HOOK_PANE_SWITCH_SPLIT,
        "Swaps the current split positions.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_LEFT,
        "Moves focus to the window on the left.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_DOWN,
        "Moves focus to the window below.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_UP,
        "Moves focus to the window above.",
    )?;
    register_hook(
        runtime,
        HOOK_WORKSPACE_WINDOW_RIGHT,
        "Moves focus to the window on the right.",
    )?;
    register_hook(
        runtime,
        HOOK_GIT_STATUS_OPEN_POPUP,
        "Opens the git status buffer in the popup window.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_OPEN,
        "Opens the browser buffer in a split pane.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_OPEN_BUFFER,
        "Opens the active file in a split browser buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_OPEN_POPUP,
        "Focuses the browser popup after opening it.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_URL,
        "Detects a URL in the active buffer and opens it in a split browser buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_FOCUS_INPUT,
        "Focuses the browser input section.",
    )?;
    register_hook(
        runtime,
        HOOK_BROWSER_SUBMIT,
        "Submits the active browser URL prompt.",
    )?;
    register_hook(
        runtime,
        HOOK_TERMINAL_OPEN_POPUP,
        "Focuses the terminal popup after opening it.",
    )?;
    register_hook(runtime, HOOK_GIT_DIFF_OPEN, "Opens the git diff buffer.")?;
    register_hook(runtime, HOOK_GIT_LOG_OPEN, "Opens the git log buffer.")?;
    register_hook(
        runtime,
        HOOK_GIT_STASH_LIST_OPEN,
        "Opens the git stash list buffer.",
    )?;
    register_hook(runtime, HOOK_OIL_OPEN, "Opens the oil directory buffer.")?;
    register_hook(
        runtime,
        HOOK_OIL_OPEN_PARENT,
        "Opens the oil parent directory buffer.",
    )?;
    register_hook(runtime, HOOK_OIL_ACTION, "Runs an oil buffer action.")?;
    register_hook(
        runtime,
        HOOK_OIL_GIT_WORKTREE,
        "Starts git worktree creation from the active oil buffer.",
    )?;
    register_issues_hooks(runtime)?;
    register_hook(
        runtime,
        HOOK_INPUT_SUBMIT,
        "Submits the active input buffer prompt.",
    )?;
    register_hook(
        runtime,
        HOOK_INPUT_CLEAR,
        "Clears the active input buffer prompt.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_CONNECT,
        "Opens the database connection prompt.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_DISCONNECT,
        "Disconnects the active database session.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_SHOW_TABLES,
        "Opens the active database schema explorer.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_NEW_QUERY_BUFFER,
        "Creates a database SQL query buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_EXECUTE_SQL,
        "Executes SQL in the active database query buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_SHOW_CONNECTIONS,
        "Opens the database connections browser.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_SHOW_HISTORY,
        "Opens the database query history browser.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_SHOW_SNIPPETS,
        "Opens the saved database snippets browser.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_SAVE_SNIPPET,
        "Saves SQL from the active database query buffer as a snippet.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_REFRESH_SCHEMA,
        "Refreshes the active database schema cache.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_ACTIVATE_LINE,
        "Runs the action attached to the current database browser line.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_DASHBOARD,
        "Opens the database dashboard buffer.",
    )?;
    register_hook(
        runtime,
        HOOK_DB_MULTIVIEW,
        "Opens the database sidebar and query buffers.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_EVALUATE,
        "Evaluates the active plugin buffer's input section and writes the output section.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_RUN_COMMAND,
        "Opens the build prompt and streams the workspace build command.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_RERUN_COMMAND,
        "Re-runs the last build command for the active workspace.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_RELOAD_USER_LIBRARY,
        "Reloads user-library derived runtime state after rebuilding `volt-user`.",
    )?;
    register_hook(
        runtime,
        HOOK_PLUGIN_SWITCH_PANE,
        "Switches focus between the active plugin buffer's split panes.",
    )?;
    register_hook(
        runtime,
        HOOK_TREESITTER_RECOMPILE_INSTALLED,
        "Recompiles every currently installed Tree-sitter grammar.",
    )?;

    runtime
        .subscribe_hook(
            HOOK_PLUGIN_EVALUATE,
            "shell.plugin-evaluate",
            |event, runtime| {
                let buffer_id = event.buffer_id.unwrap_or(active_shell_buffer_id(runtime)?);
                evaluate_active_plugin_buffer(runtime, buffer_id)
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PLUGIN_SWITCH_PANE,
            "shell.plugin-switch-pane",
            |event, runtime| switch_active_plugin_pane(runtime, event.buffer_id),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PLUGIN_RUN_COMMAND,
            "shell.plugin-run-command",
            |_event, runtime| open_compile_prompt(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PLUGIN_RERUN_COMMAND,
            "shell.plugin-rerun-command",
            |_, runtime| rerun_compile_command(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PLUGIN_RELOAD_USER_LIBRARY,
            "shell.plugin-reload-user-library",
            |_, runtime| reload_user_library(runtime).map(|_| ()),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_TREESITTER_RECOMPILE_INSTALLED,
            "shell.treesitter-recompile-installed",
            |_, runtime| recompile_installed_tree_sitter_languages(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_LEFT, "shell.move-left", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Left)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_DOWN, "shell.move-down", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Down)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_UP, "shell.move-up", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Up)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_RIGHT, "shell.move-right", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::Right)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_WORD_FORWARD,
            "shell.move-word-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::WordForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_WORD_BACKWARD,
            "shell.move-word-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::WordBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_WORD_END, "shell.move-word-end", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::WordEnd)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_BIG_WORD_FORWARD,
            "shell.move-big-word-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::BigWordForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_BIG_WORD_BACKWARD,
            "shell.move-big-word-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::BigWordBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_BIG_WORD_END,
            "shell.move-big-word-end",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::BigWordEnd)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SENTENCE_FORWARD,
            "shell.move-sentence-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::SentenceForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SENTENCE_BACKWARD,
            "shell.move-sentence-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::SentenceBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_PARAGRAPH_FORWARD,
            "shell.move-paragraph-forward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ParagraphForward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_PARAGRAPH_BACKWARD,
            "shell.move-paragraph-backward",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ParagraphBackward)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MATCH_PAIR, "shell.match-pair", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::MatchPair)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_LINE_START,
            "shell.move-line-start",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::LineStart)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_LINE_FIRST_NON_BLANK,
            "shell.move-line-first-non-blank",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::LineFirstNonBlank)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MOVE_LINE_END, "shell.move-line-end", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::LineEnd)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SCREEN_TOP,
            "shell.move-screen-top",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ScreenTop)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SCREEN_MIDDLE,
            "shell.move-screen-middle",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ScreenMiddle)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MOVE_SCREEN_BOTTOM,
            "shell.move-screen-bottom",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::ScreenBottom)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_GOTO_FIRST_LINE,
            "shell.goto-first-line",
            |_, runtime| {
                apply_motion_command(runtime, ShellMotion::FirstLine)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_GOTO_LAST_LINE, "shell.goto-last-line", |_, runtime| {
            apply_motion_command(runtime, ShellMotion::LastLine)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_CURRENT_LINE_TOP,
            "shell.current-line-top",
            |_, runtime| {
                position_current_line_in_viewport(runtime, 0)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_CENTER_CURRENT_LINE,
            "shell.center-current-line",
            |_, runtime| {
                let offset = {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.viewport_lines().saturating_sub(1) / 2
                };
                position_current_line_in_viewport(runtime, offset)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_CURRENT_LINE_BOTTOM,
            "shell.current-line-bottom",
            |_, runtime| {
                let offset = {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.viewport_lines().saturating_sub(1)
                };
                position_current_line_in_viewport(runtime, offset)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_HALF_PAGE_DOWN,
            "shell.scroll-half-page-down",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::HalfPageDown)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_HALF_PAGE_UP,
            "shell.scroll-half-page-up",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::HalfPageUp)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_PAGE_DOWN,
            "shell.scroll-page-down",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::PageDown)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_SCROLL_PAGE_UP, "shell.scroll-page-up", |_, runtime| {
            apply_scroll_command(runtime, ScrollCommand::PageUp)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SCROLL_LINE_DOWN,
            "shell.scroll-line-down",
            |_, runtime| {
                apply_scroll_command(runtime, ScrollCommand::LineDown)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_SCROLL_LINE_UP, "shell.scroll-line-up", |_, runtime| {
            apply_scroll_command(runtime, ScrollCommand::LineUp)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MODE_INSERT, "shell.enter-insert-mode", |_, runtime| {
            let is_terminal = active_shell_buffer_is_terminal(runtime)?;
            if active_shell_buffer_read_only(runtime)?
                && !active_shell_buffer_has_input(runtime)?
                && !is_terminal
            {
                report_read_only(runtime, "insert mode blocked");
                return Ok(());
            }
            if is_terminal {
                shell_ui_mut(runtime)?.enter_insert_mode();
                return Ok(());
            }
            if active_shell_buffer_has_input(runtime)? {
                let buffer_id = active_shell_buffer_id(runtime)?;
                let buffer = shell_buffer_mut(runtime, buffer_id)?;
                if buffer_is_acp(&buffer.kind) {
                    let _ = buffer.focus_acp_input();
                } else if buffer_is_browser(&buffer.kind) {
                    let _ = buffer.focus_browser_input();
                }
                shell_ui_mut(runtime)?.set_active_vim_target(VimTarget::Input);
            }
            start_change_recording(runtime)?;
            mark_change_finish_on_normal(runtime)?;
            shell_ui_mut(runtime)?.enter_insert_mode();
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_MODE_NORMAL, "shell.enter-normal-mode", |_, runtime| {
            let previous_mode = shell_ui(runtime)?.input_mode();
            let buffer_id = active_shell_buffer_id(runtime)?;
            let is_directory = buffer_is_directory(&shell_buffer(runtime, buffer_id)?.kind);
            let is_dap_locals = matches!(
                &shell_buffer(runtime, buffer_id)?.kind,
                BufferKind::Plugin(kind) if kind == DAP_LOCALS_KIND
            );
            let cursor_point = {
                let buffer = shell_buffer(runtime, buffer_id)?;
                terminal_buffer_cursor_point_for_normal_mode(buffer)
                    .unwrap_or_else(|| buffer.cursor_point())
            };
            let has_input = active_shell_buffer_has_input(runtime)?;
            let targeted_input = active_shell_buffer_vim_targets_input(runtime)?;
            let finish_change = {
                let vim = shell_ui(runtime)?.vim();
                vim.recording_change && vim.finish_change_on_normal
            };
            let visual_snapshot = {
                let (anchor, kind) = {
                    let ui = shell_ui(runtime)?;
                    if ui.input_mode() != InputMode::Visual {
                        (None, ui.vim().visual_kind)
                    } else {
                        (ui.vim().visual_anchor, ui.vim().visual_kind)
                    }
                };
                if let Some(anchor) = anchor {
                    let head = active_shell_buffer_mut(runtime)?.cursor_point();
                    Some((anchor, head, kind))
                } else {
                    None
                }
            };
            apply_pending_block_insert(runtime)?;
            if has_input && previous_mode == InputMode::Normal && targeted_input {
                let ui = shell_ui_mut(runtime)?;
                ui.set_active_vim_target(VimTarget::Buffer);
                ui.enter_normal_mode();
                active_shell_buffer_mut(runtime)?.set_cursor(cursor_point);
                return Ok(());
            }
            shell_ui_mut(runtime)?.enter_normal_mode();
            active_shell_buffer_mut(runtime)?.set_cursor(cursor_point);
            if targeted_input
                && has_input
                && let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut()
                && previous_mode == InputMode::Visual
            {
                input.clear_selection();
            }
            if let Some((anchor, head, kind)) = visual_snapshot {
                store_last_visual_selection(runtime, anchor, head, kind)?;
            }
            if finish_change {
                finish_change_recording(runtime)?;
            }
            if matches!(previous_mode, InputMode::Insert | InputMode::Replace) {
                if is_directory && let Err(error) = apply_directory_edit_queue(runtime, buffer_id) {
                    record_runtime_error(runtime, "oil.directory", error.clone());
                    return Err(error);
                }
                if is_dap_locals && let Err(error) = apply_dap_locals_edits(runtime, buffer_id) {
                    record_runtime_error(runtime, "dap.locals", error.clone());
                    return Err(error);
                }
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_VIM_EDIT, "shell.vim-edit", |event, runtime| {
            let detail = event.detail.as_deref().unwrap_or_default();
            let action = VimEditAction::from_hook_detail(detail)
                .ok_or_else(|| format!("unknown Vim edit action `{detail}`"))?;
            if handle_terminal_vim_edit(runtime, action)? {
                return Ok(());
            }
            if vim_edit_requires_write(action)
                && active_shell_buffer_read_only(runtime)?
                && !vim_edit_targets_input(runtime, action)?
            {
                let action = format!("{detail} blocked");
                report_read_only(runtime, &action);
                return Ok(());
            }
            match action {
                VimEditAction::DeleteChar => {
                    delete_chars(runtime, false)?;
                }
                VimEditAction::DeleteCharBefore => {
                    delete_chars(runtime, true)?;
                }
                VimEditAction::DeleteLineEnd => {
                    start_change_recording(runtime)?;
                    apply_motion_alias(runtime, VimOperator::Delete, ShellMotion::LineEnd)?;
                }
                VimEditAction::ChangeLineEnd => {
                    start_change_recording(runtime)?;
                    apply_motion_alias(runtime, VimOperator::Change, ShellMotion::LineEnd)?;
                }
                VimEditAction::YankLine => {
                    let lines = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
                    apply_linewise_operator(runtime, VimOperator::Yank, lines)?;
                }
                VimEditAction::SubstituteChar => {
                    substitute_chars(runtime)?;
                }
                VimEditAction::SubstituteLine => {
                    let lines = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
                    apply_linewise_operator(runtime, VimOperator::Change, lines)?;
                }
                VimEditAction::ReplaceChar => {
                    start_replace_char(runtime)?;
                }
                VimEditAction::EnterReplaceMode => {
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Replace;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        shell_ui_mut(runtime)?.enter_replace_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    shell_ui_mut(runtime)?.enter_replace_mode();
                }
                VimEditAction::ToggleCase => {
                    toggle_case_chars(runtime)?;
                }
                VimEditAction::ToggleLineComment => {
                    start_change_recording(runtime)?;
                    let count = shell_ui_mut(runtime)?.vim_mut().take_count_or_one();
                    toggle_current_line_comment(runtime, count)?;
                }
                VimEditAction::Append => {
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        let offset = {
                            let state = shell_ui(runtime)?
                                .vim()
                                .multicursor
                                .as_ref()
                                .ok_or_else(|| "multicursor state is missing".to_owned())?;
                            if state.match_text.is_empty() {
                                0
                            } else {
                                state
                                    .cursor_offset
                                    .saturating_add(1)
                                    .min(state.match_text.chars().count())
                            }
                        };
                        set_multicursor_cursor_offset(runtime, offset)?;
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Insert;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            let _ = input.move_right();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.append_after_cursor();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                VimEditAction::AppendLineEnd => {
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        let offset = shell_ui(runtime)?
                            .vim()
                            .multicursor
                            .as_ref()
                            .map(|state| state.match_text.chars().count())
                            .unwrap_or_default();
                        set_multicursor_cursor_offset(runtime, offset)?;
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Insert;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            input.move_line_end();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.append_line_end();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                VimEditAction::InsertLineStart => {
                    if shell_ui(runtime)?.vim().multicursor.is_some() {
                        set_multicursor_cursor_offset(runtime, 0)?;
                        let ui = shell_ui_mut(runtime)?;
                        ui.input_mode = InputMode::Insert;
                        ui.vim_mut().clear_transient();
                        return Ok(());
                    }
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            input.move_line_start();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    active_shell_buffer_mut(runtime)?.insert_line_start();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                VimEditAction::OpenLineBelow => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            input.open_line_below();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    let (buffer_id, indent_size, use_tabs) = {
                        let ui = shell_ui(runtime)?;
                        let buffer_id = active_shell_buffer_id(runtime)?;
                        let language_id =
                            ui.buffer(buffer_id).and_then(|buffer| buffer.language_id());
                        let theme_registry = runtime.services().get::<ThemeRegistry>();
                        (
                            buffer_id,
                            theme_lang_indent(theme_registry, language_id),
                            theme_lang_use_tabs(theme_registry, language_id),
                        )
                    };
                    active_shell_buffer_mut(runtime)?.open_line_below();
                    format_current_line_indent(runtime, buffer_id, indent_size, use_tabs)?;
                    shell_buffer_mut(runtime, buffer_id)?.mark_syntax_dirty();
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                VimEditAction::OpenLineAbove => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    if active_shell_buffer_vim_targets_input(runtime)? {
                        if let Some(input) = active_shell_buffer_mut(runtime)?.input_field_mut() {
                            input.open_line_above();
                        }
                        shell_ui_mut(runtime)?.enter_insert_mode();
                        return Ok(());
                    }
                    let (buffer_id, indent_size, use_tabs, reference_indent) = {
                        let ui = shell_ui(runtime)?;
                        let buffer_id = active_shell_buffer_id(runtime)?;
                        let buffer = ui
                            .buffer(buffer_id)
                            .ok_or_else(|| "active buffer is missing".to_owned())?;
                        let language_id = buffer.language_id();
                        let theme_registry = runtime.services().get::<ThemeRegistry>();
                        let indent_size = theme_lang_indent(theme_registry, language_id);
                        let use_tabs = theme_lang_use_tabs(theme_registry, language_id);
                        let line = buffer.text.line(buffer.cursor_row()).unwrap_or_default();
                        (
                            buffer_id,
                            indent_size,
                            use_tabs,
                            leading_indent_string(&line, indent_size),
                        )
                    };
                    active_shell_buffer_mut(runtime)?.open_line_above();
                    let line_index = shell_buffer(runtime, buffer_id)?.cursor_row();
                    let indent = syntax_indent_for_buffer(
                        runtime,
                        buffer_id,
                        line_index,
                        indent_size,
                        use_tabs,
                    )?
                    .unwrap_or(reference_indent);
                    {
                        let buffer = shell_buffer_mut(runtime, buffer_id)?;
                        apply_line_indent(buffer, line_index, indent_size, &indent);
                        buffer.mark_syntax_dirty();
                    }
                    shell_ui_mut(runtime)?.enter_insert_mode();
                }
                VimEditAction::Undo => {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.undo();
                    buffer.mark_syntax_dirty();
                }
                VimEditAction::Redo => {
                    let buffer = active_shell_buffer_mut(runtime)?;
                    buffer.redo();
                    buffer.mark_syntax_dirty();
                }
                VimEditAction::MulticursorAddNextMatch => {
                    add_next_multicursor_match(runtime)?;
                }
                VimEditAction::MulticursorAddPreviousMatch => {
                    add_previous_multicursor_match(runtime)?;
                }
                VimEditAction::MulticursorSelectAllMatches => {
                    add_next_multicursor_match(runtime)?;
                    while shell_ui(runtime)?.vim().multicursor.is_some() {
                        let before = shell_ui(runtime)?
                            .vim()
                            .multicursor
                            .as_ref()
                            .map(|state| state.ranges.len())
                            .unwrap_or_default();
                        add_next_multicursor_match(runtime)?;
                        let after = shell_ui(runtime)?
                            .vim()
                            .multicursor
                            .as_ref()
                            .map(|state| state.ranges.len())
                            .unwrap_or_default();
                        if after <= before {
                            break;
                        }
                    }
                }
                VimEditAction::EnterVisual => {
                    toggle_visual_mode(runtime)?;
                }
                VimEditAction::EnterVisualLine => {
                    toggle_visual_line_mode(runtime)?;
                }
                VimEditAction::EnterVisualBlock => {
                    toggle_visual_block_mode(runtime)?;
                }
                VimEditAction::StartDeleteOperator => {
                    start_vim_operator(runtime, VimOperator::Delete)?;
                }
                VimEditAction::StartChangeOperator => {
                    start_vim_operator(runtime, VimOperator::Change)?;
                }
                VimEditAction::StartYankOperator => {
                    start_vim_operator(runtime, VimOperator::Yank)?;
                }
                VimEditAction::StartFormatOperator => {
                    start_vim_format(runtime)?;
                }
                VimEditAction::StartGPrefix => {
                    start_vim_g_prefix(runtime)?;
                }
                VimEditAction::StartFindForward => {
                    start_vim_find(runtime, VimFindKind::ForwardTo)?;
                }
                VimEditAction::StartFindBackward => {
                    start_vim_find(runtime, VimFindKind::BackwardTo)?;
                }
                VimEditAction::StartTillForward => {
                    start_vim_find(runtime, VimFindKind::ForwardBefore)?;
                }
                VimEditAction::StartTillBackward => {
                    start_vim_find(runtime, VimFindKind::BackwardAfter)?;
                }
                VimEditAction::RepeatFindNext => {
                    repeat_last_find(runtime, false)?;
                }
                VimEditAction::RepeatFindPrevious => {
                    repeat_last_find(runtime, true)?;
                }
                VimEditAction::StartSearchForward => {
                    open_vim_search_prompt(runtime, VimSearchDirection::Forward)?;
                }
                VimEditAction::StartSearchBackward => {
                    open_vim_search_prompt(runtime, VimSearchDirection::Backward)?;
                }
                VimEditAction::SearchWordForward => {
                    search_word_under_cursor(runtime, VimSearchDirection::Forward)?;
                }
                VimEditAction::SearchWordBackward => {
                    search_word_under_cursor(runtime, VimSearchDirection::Backward)?;
                }
                VimEditAction::RepeatSearchNext => {
                    repeat_vim_search(runtime, false)?;
                }
                VimEditAction::RepeatSearchPrevious => {
                    repeat_vim_search(runtime, true)?;
                }
                VimEditAction::SelectRegister => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::Register);
                }
                VimEditAction::SetMark => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MarkSet);
                }
                VimEditAction::GotoMarkLine => {
                    shell_ui_mut(runtime)?.vim_mut().pending =
                        Some(VimPending::MarkJump { linewise: true });
                }
                VimEditAction::GotoMark => {
                    shell_ui_mut(runtime)?.vim_mut().pending =
                        Some(VimPending::MarkJump { linewise: false });
                }
                VimEditAction::ToggleMacroRecord => {
                    if shell_ui(runtime)?.vim().recording_macro.is_some() {
                        stop_macro_record(runtime)?;
                    } else {
                        shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MacroRecord);
                    }
                }
                VimEditAction::StartMacroPlayback => {
                    shell_ui_mut(runtime)?.vim_mut().pending = Some(VimPending::MacroPlayback);
                }
                VimEditAction::PutAfter => {
                    put_yank(runtime, true)?;
                }
                VimEditAction::PutBefore => {
                    put_yank(runtime, false)?;
                }
                VimEditAction::VisualPutAfter => {
                    put_yank_over_visual_selection(runtime, true)?;
                }
                VimEditAction::VisualPutBefore => {
                    put_yank_over_visual_selection(runtime, false)?;
                }
                VimEditAction::VisualSwapAnchor => {
                    swap_visual_anchor(runtime)?;
                }
                VimEditAction::StartVisualInnerTextObject => {
                    start_visual_text_object(runtime, false)?;
                }
                VimEditAction::StartVisualAroundTextObject => {
                    start_visual_text_object(runtime, true)?;
                }
                VimEditAction::VisualDelete => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Delete)?;
                }
                VimEditAction::VisualChange => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Change)?;
                }
                VimEditAction::VisualReplaceChar => {
                    start_change_recording(runtime)?;
                    shell_ui_mut(runtime)?.vim_mut().pending =
                        Some(VimPending::ReplaceVisualSelection);
                }
                VimEditAction::VisualBlockInsert => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    start_visual_block_insert(runtime, false)?;
                }
                VimEditAction::VisualBlockAppend => {
                    start_change_recording(runtime)?;
                    mark_change_finish_on_normal(runtime)?;
                    start_visual_block_insert(runtime, true)?;
                }
                VimEditAction::VisualFormat => {
                    start_change_recording(runtime)?;
                    emit_workspace_format(runtime)?;
                }
                VimEditAction::VisualToggleComment => {
                    start_change_recording(runtime)?;
                    toggle_visual_selection_comments(runtime)?;
                }
                VimEditAction::VisualYank => {
                    apply_visual_operator(runtime, VimOperator::Yank)?;
                }
                VimEditAction::VisualToggleCase => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::ToggleCase)?;
                }
                VimEditAction::VisualLowercase => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Lowercase)?;
                }
                VimEditAction::VisualUppercase => {
                    start_change_recording(runtime)?;
                    apply_visual_operator(runtime, VimOperator::Uppercase)?;
                }
                VimEditAction::VisualIndent => {
                    start_change_recording(runtime)?;
                    shift_visual_selection(runtime, true)?;
                }
                VimEditAction::VisualOutdent => {
                    start_change_recording(runtime)?;
                    shift_visual_selection(runtime, false)?;
                }
                VimEditAction::VisualJoin => {
                    start_change_recording(runtime)?;
                    join_visual_selection_lines(runtime)?;
                }
                VimEditAction::VisualMoveDown => {
                    start_change_recording(runtime)?;
                    move_visual_selection_lines(runtime, true)?;
                }
                VimEditAction::VisualMoveUp => {
                    start_change_recording(runtime)?;
                    move_visual_selection_lines(runtime, false)?;
                }
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_FORMATTER_REGISTER,
            "shell.formatter-register",
            |event, runtime| {
                let detail = event
                    .detail
                    .as_deref()
                    .ok_or_else(|| "formatter registration hook missing detail".to_owned())?;
                let spec = FormatterSpec::from_hook_detail(detail)?;
                formatter_registry_mut(runtime)?.register(spec)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_FORMAT,
            "shell.workspace-format",
            |_, runtime| {
                format_workspace(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BUFFER_SAVE, "shell.buffer-save", |event, runtime| {
            let workspace_id = event
                .workspace_id
                .or_else(|| active_shell_workspace_id(runtime))
                .or_else(|| runtime.model().active_workspace_id().ok())
                .ok_or_else(|| "buffer.save hook missing workspace".to_owned())?;
            let buffer_id = event
                .buffer_id
                .or_else(|| active_shell_buffer_id(runtime).ok())
                .ok_or_else(|| "buffer.save hook missing buffer".to_owned())?;
            save_buffer(runtime, workspace_id, buffer_id)?;
            // Invalidate only — do not sync-refresh open git-status buffers here.
            // A full `git status` snapshot blocks the UI for hundreds of ms to seconds
            // (:w / <leader>w). Status refreshes on focus / explicit git commands instead.
            let _ = invalidate_git_state_after_save(runtime);
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BUFFER_CLOSE, "shell.buffer-close", |event, runtime| {
            let buffer_id = event.buffer_id.unwrap_or(active_shell_buffer_id(runtime)?);
            close_buffer_with_prompt(runtime, buffer_id)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_BUFFER_TOGGLE_LINE_WRAP,
            "shell.buffer-toggle-line-wrap",
            |event, runtime| {
                let buffer_id = event.buffer_id.unwrap_or(active_shell_buffer_id(runtime)?);
                let buffer = shell_buffer_mut(runtime, buffer_id)?;
                buffer.toggle_line_wrap();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_SAVE,
            "shell.workspace-save",
            |event, runtime| {
                let workspace_id = event
                    .workspace_id
                    .or_else(|| active_shell_workspace_id(runtime))
                    .or_else(|| runtime.model().active_workspace_id().ok())
                    .ok_or_else(|| "workspace.save hook missing workspace".to_owned())?;
                save_workspace(runtime, workspace_id)?;
                let _ = invalidate_git_state_after_save(runtime);
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_WORKSPACE_NEXT, "shell.workspace-next", |_, runtime| {
            cycle_runtime_project_workspace(runtime, CycleDirection::Next)
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_PREVIOUS,
            "shell.workspace-previous",
            |_, runtime| cycle_runtime_project_workspace(runtime, CycleDirection::Previous),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_WORKSPACE_MARK, "shell.workspace-mark", |_, runtime| {
            mark_active_project_workspace(runtime)
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_UNMARK,
            "shell.workspace-unmark",
            |_, runtime| unmark_active_project_workspace(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_MARKS,
            "shell.workspace-marks",
            |_, runtime| open_mark_list(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_MARKED_1,
            "shell.workspace-marked-1",
            |_, runtime| jump_to_marked_workspace_slot(runtime, 0),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_MARKED_2,
            "shell.workspace-marked-2",
            |_, runtime| jump_to_marked_workspace_slot(runtime, 1),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_MARKED_3,
            "shell.workspace-marked-3",
            |_, runtime| jump_to_marked_workspace_slot(runtime, 2),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_MARKED_4,
            "shell.workspace-marked-4",
            |_, runtime| jump_to_marked_workspace_slot(runtime, 3),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WORKTREE_REMOVE,
            "shell.workspace-worktree-remove",
            |_, runtime| worktree_remove_from_one_shot(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_OPEN, "shell.open-picker", |event, runtime| {
            let picker =
                picker::picker_overlay(runtime, event.detail.as_deref().unwrap_or("commands"))?;
            shell_ui_mut(runtime)?.set_picker(picker);
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_VIM_COMMAND_LINE,
            "shell.vim-command-line",
            |_, runtime| open_vim_command_line(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_NEXT, "shell.picker-next", |_, runtime| {
            if let Some(picker) = shell_ui_mut(runtime)?.picker_mut() {
                picker.select_next();
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PICKER_PREVIOUS,
            "shell.picker-previous",
            |_, runtime| {
                if let Some(picker) = shell_ui_mut(runtime)?.picker_mut() {
                    picker.select_previous();
                }
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_CANCEL, "shell.picker-cancel", |_, runtime| {
            if !shell_ui(runtime)?.picker_visible() {
                // Escape reaches this binding through the Popup Minor Mode
                // whenever a popup is focused, picker or not. Without a picker
                // to close, fall through to Normal mode so popup buffers
                // (terminals included) are not stuck in Insert mode.
                let cursor_point = {
                    let buffer_id = active_shell_buffer_id(runtime)?;
                    let buffer = shell_buffer(runtime, buffer_id)?;
                    terminal_buffer_cursor_point_for_normal_mode(buffer)
                };
                shell_ui_mut(runtime)?.enter_normal_mode();
                if let Some(point) = cursor_point {
                    active_shell_buffer_mut(runtime)?.set_cursor(point);
                }
                return Ok(());
            }
            let picker_kind = shell_ui(runtime)?.picker_kind();
            shell_ui_mut(runtime)?.close_picker();
            if let Some(PickerKind::AcpPermission { request_id }) = picker_kind {
                acp::acp_permission_picker_closed(runtime, request_id)?;
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_TRIGGER,
            "shell.autocomplete-trigger",
            |_, runtime| {
                trigger_autocomplete(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_NEXT,
            "shell.autocomplete-next",
            |_, runtime| {
                if let Some(autocomplete) = shell_ui_mut(runtime)?.autocomplete_mut()
                    && autocomplete.is_visible()
                {
                    autocomplete.select_next();
                }
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_PREVIOUS,
            "shell.autocomplete-previous",
            |_, runtime| {
                if let Some(autocomplete) = shell_ui_mut(runtime)?.autocomplete_mut()
                    && autocomplete.is_visible()
                {
                    autocomplete.select_previous();
                }
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_ACCEPT,
            "shell.autocomplete-accept",
            |_, runtime| {
                accept_autocomplete(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_AUTOCOMPLETE_CANCEL,
            "shell.autocomplete-cancel",
            |_, runtime| {
                shell_ui_mut(runtime)?.close_autocomplete();
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_TOGGLE, "shell.hover-toggle", |_, runtime| {
            trigger_hover_toggle(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_FOCUS, "shell.hover-focus", |_, runtime| {
            trigger_hover_focus(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_NEXT, "shell.hover-next", |_, runtime| {
            cycle_hover_provider(runtime, true)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_HOVER_PREVIOUS, "shell.hover-previous", |_, runtime| {
            cycle_hover_provider(runtime, false)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_POPUP_TOGGLE, "shell.popup-toggle", |_, runtime| {
            toggle_runtime_popup(runtime)?;
            sync_active_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_DOCK_TOGGLE,
            "shell.workspace-dock-toggle",
            |_, runtime| {
                toggle_workspace_dock(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_DOCK_PREVIOUS,
            "shell.workspace-dock-previous",
            |_, runtime| {
                cycle_workspace_dock(runtime, false)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_DOCK_NEXT,
            "shell.workspace-dock-next",
            |_, runtime| {
                cycle_workspace_dock(runtime, true)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_DOCK_TOGGLE,
            "shell.acp-dock-toggle",
            |_, runtime| {
                toggle_acp_dock(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_DOCK_PREVIOUS,
            "shell.acp-dock-previous",
            |_, runtime| {
                cycle_acp_dock(runtime, false)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_DOCK_NEXT, "shell.acp-dock-next", |_, runtime| {
            cycle_acp_dock(runtime, true)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_POPUP_NEXT, "shell.popup-next", |_, runtime| {
            if shell_ui(runtime)?.picker_visible() {
                if let Some(picker) = shell_ui_mut(runtime)?.picker_mut() {
                    picker.select_next();
                }
                return Ok(());
            }
            cycle_runtime_popup_buffer(runtime, true)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_POPUP_PREVIOUS, "shell.popup-previous", |_, runtime| {
            if shell_ui(runtime)?.picker_visible() {
                if let Some(picker) = shell_ui_mut(runtime)?.picker_mut() {
                    picker.select_previous();
                }
                return Ok(());
            }
            cycle_runtime_popup_buffer(runtime, false)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PANE_SPLIT_HORIZONTAL,
            "shell.pane-split-horizontal",
            |_, runtime| {
                split_runtime_pane(runtime, PaneSplitDirection::Horizontal)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PANE_SPLIT_VERTICAL,
            "shell.pane-split-vertical",
            |_, runtime| {
                split_runtime_pane(runtime, PaneSplitDirection::Vertical)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PANE_CLOSE, "shell.pane-close", |_, runtime| {
            close_runtime_pane(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PANE_SWITCH_SPLIT,
            "shell.pane-switch-split",
            |_, runtime| {
                switch_runtime_split(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_LEFT,
            "shell.workspace-window-left",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Left)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_DOWN,
            "shell.workspace-window-down",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Down)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_UP,
            "shell.workspace-window-up",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Up)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_WORKSPACE_WINDOW_RIGHT,
            "shell.workspace-window-right",
            |_, runtime| {
                move_workspace_window(runtime, WindowMoveDirection::Right)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_GIT_STATUS_OPEN_POPUP,
            "shell.git-status-open-popup",
            |_, runtime| {
                open_git_status_popup(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BROWSER_OPEN, "shell.browser-open", |_, runtime| {
            open_browser_buffer_in_split(runtime, None)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_BROWSER_OPEN_BUFFER,
            "shell.browser-open-buffer",
            |_, runtime| {
                open_active_buffer_in_browser_split(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_BROWSER_OPEN_POPUP,
            "shell.browser-open-popup",
            |_, runtime| {
                browser::focus_active_browser_popup(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BROWSER_URL, "shell.browser-url", |_, runtime| {
            open_detected_browser_url(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_BROWSER_FOCUS_INPUT,
            "shell.browser-focus-input",
            |_, runtime| {
                focus_browser_input_section(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_TERMINAL_OPEN_POPUP,
            "shell.terminal-open-popup",
            |_, runtime| {
                terminal::focus_active_terminal_popup(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_GIT_DIFF_OPEN, "shell.git-diff-open", |_, runtime| {
            open_git_diff_worktree(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_GIT_LOG_OPEN, "shell.git-log-open", |_, runtime| {
            open_git_log_current(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_GIT_STASH_LIST_OPEN,
            "shell.git-stash-list-open",
            |_, runtime| {
                open_git_stash_list_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_OIL_OPEN, "shell.oil-open", |_, runtime| {
            let root = oil_default_root(runtime)?;
            open_oil_directory(runtime, root)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_OIL_OPEN_PARENT,
            "shell.oil-open-parent",
            |_, runtime| {
                let root = oil_parent_root(runtime)?;
                open_oil_directory(runtime, root)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_OIL_ACTION, "shell.oil-action", |event, runtime| {
            let detail = event
                .detail
                .as_deref()
                .ok_or_else(|| "oil action hook missing detail".to_owned())?;
            let action = OilKeyAction::from_hook_detail(detail)
                .ok_or_else(|| format!("unknown oil action `{detail}`"))?;
            if !execute_oil_action(runtime, action)? {
                return Err("oil action requires an active oil buffer".to_owned());
            }
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_OIL_GIT_WORKTREE,
            "shell.oil-git-worktree",
            |_, runtime| {
                eprintln!("[oil.git-worktree] ui.oil.git-worktree hook received");
                record_runtime_error(
                    runtime,
                    "oil.git-worktree.trace",
                    "ui.oil.git-worktree hook received",
                );
                oil_git_worktree_command(runtime)
            },
        )
        .map_err(|error| error.to_string())?;
    subscribe_issues_hooks(runtime)?;
    runtime
        .subscribe_hook(builtins::PANE_SWITCH, "shell.pane-switch", |_, runtime| {
            shell_ui_mut(runtime)?.close_autocomplete();
            refresh_git_status_if_active_if_due(runtime)?;
            ensure_directory_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            builtins::BUFFER_SWITCH,
            "shell.buffer-switch",
            |_, runtime| {
                shell_ui_mut(runtime)?.close_autocomplete();
                refresh_git_status_if_active_if_due(runtime)?;
                ensure_directory_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_INPUT_SUBMIT, "shell.input-submit", |_, runtime| {
            submit_input_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_BROWSER_SUBMIT, "shell.browser-submit", |_, runtime| {
            browser::submit_browser_input(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_INPUT_CLEAR, "shell.input-clear", |_, runtime| {
            clear_input_buffer(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_DB_CONNECT, "shell.db-connect", |_, runtime| {
            open_db_connect_prompt(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_DB_DISCONNECT, "shell.db-disconnect", |_, runtime| {
            db_service_mut(runtime)?.disconnect(None)?;
            refresh_all_db_browser_buffers(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_DB_SHOW_TABLES, "shell.db-show-tables", |_, runtime| {
            open_db_schema_buffer(runtime, None)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_DB_NEW_QUERY_BUFFER,
            "shell.db-new-query-buffer",
            |_, runtime| {
                open_db_query_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_DB_EXECUTE_SQL, "shell.db-execute-sql", |_, runtime| {
            execute_db_sql(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_DB_SHOW_CONNECTIONS,
            "shell.db-show-connections",
            |_, runtime| {
                open_db_connections_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_DB_SHOW_HISTORY,
            "shell.db-show-history",
            |_, runtime| {
                open_db_history_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_DB_SHOW_SNIPPETS,
            "shell.db-show-snippets",
            |_, runtime| {
                open_db_snippets_buffer(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_DB_SAVE_SNIPPET,
            "shell.db-save-snippet",
            |_, runtime| {
                save_db_snippet(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_DB_REFRESH_SCHEMA,
            "shell.db-refresh-schema",
            |_, runtime| {
                refresh_db_schema(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_DB_ACTIVATE_LINE,
            "shell.db-activate-line",
            |_, runtime| {
                activate_db_browser_line(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_DB_DASHBOARD, "shell.db-dashboard", |_, runtime| {
            open_db_dashboard(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_DB_MULTIVIEW, "shell.db-multiview", |_, runtime| {
            open_db_multiview(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_DISCONNECT, "shell.acp-disconnect", |_, runtime| {
            acp::acp_disconnect(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_PERMISSION_APPROVE,
            "shell.acp-permission-approve",
            |_, runtime| {
                acp::acp_permission_approve(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_PERMISSION_DENY,
            "shell.acp-permission-deny",
            |_, runtime| {
                acp::acp_permission_deny(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_PICK_SESSION,
            "shell.acp-pick-session",
            |_, runtime| {
                acp::acp_pick_session(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_NEW_SESSION,
            "shell.acp-new-session",
            |_, runtime| {
                acp::acp_new_session(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_PICK_MODE, "shell.acp-pick-mode", |_, runtime| {
            acp::acp_pick_mode(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_PICK_MODEL, "shell.acp-pick-model", |_, runtime| {
            acp::acp_pick_model(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_ACP_CYCLE_MODE, "shell.acp-cycle-mode", |_, runtime| {
            acp::acp_cycle_mode(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_SWITCH_PANE,
            "shell.acp-switch-pane",
            |_, runtime| {
                acp::acp_switch_pane(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_COMPLETE_SLASH,
            "shell.acp-complete-slash",
            |_, runtime| {
                acp::acp_complete_slash(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_ACP_FOCUS_INPUT,
            "shell.acp-focus-input",
            |_, runtime| {
                focus_acp_input_section(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_IMAGE_ZOOM_IN, "shell.image-zoom-in", |_, runtime| {
            zoom_active_image_buffer_in(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_IMAGE_ZOOM_OUT, "shell.image-zoom-out", |_, runtime| {
            zoom_active_image_buffer_out(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_IMAGE_ZOOM_RESET,
            "shell.image-zoom-reset",
            |_, runtime| {
                reset_active_image_buffer_zoom(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_IMAGE_TOGGLE_MODE,
            "shell.image-toggle-mode",
            |_, runtime| {
                toggle_active_image_buffer_mode(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_MARKDOWN_PRETTY_TOGGLE,
            "shell.markdown-pretty-toggle",
            |_, runtime| {
                toggle_active_markdown_pretty(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_RAINBOW_PARENS_TOGGLE,
            "shell.rainbow-parens-toggle",
            |_, runtime| {
                toggle_active_rainbow_parens(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_SHOW_PAREN_TOGGLE,
            "shell.show-paren-toggle",
            |_, runtime| {
                toggle_active_show_paren(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PDF_PREVIOUS_PAGE,
            "shell.pdf-previous-page",
            |_, runtime| {
                pdf_previous_page(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PDF_NEXT_PAGE, "shell.pdf-next-page", |_, runtime| {
            pdf_next_page(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PDF_ROTATE_CLOCKWISE,
            "shell.pdf-rotate-clockwise",
            |_, runtime| {
                pdf_rotate_clockwise(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_PDF_DELETE_PAGE,
            "shell.pdf-delete-page",
            |_, runtime| {
                pdf_delete_page(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_PICKER_SUBMIT, "shell.picker-submit", |_, runtime| {
            let (action, query, picker_kind) = {
                let ui = shell_ui_mut(runtime)?;
                let action = ui
                    .picker()
                    .and_then(PickerOverlay::selected_action)
                    .ok_or_else(|| "picker has no selected item".to_owned())?;
                let query = ui
                    .picker()
                    .map(|picker| picker.session().query().to_owned())
                    .unwrap_or_default();
                let picker_kind = ui.picker_kind();
                ui.close_picker();
                (action, query, picker_kind)
            };

            match action {
                PickerAction::NoOp => {}
                PickerAction::ExecuteCommand(command_name) => {
                    runtime
                        .execute_command(&command_name)
                        .map_err(|error| error.to_string())?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::ExecuteCommands(command_names) => {
                    for command_name in command_names {
                        runtime
                            .execute_command(&command_name)
                            .map_err(|error| error.to_string())?;
                    }
                    sync_active_buffer(runtime)?;
                }
                PickerAction::ApplyLspCodeAction {
                    workspace_id,
                    buffer_id,
                    path,
                    code_action,
                } => {
                    apply_lsp_code_action(runtime, workspace_id, buffer_id, &path, &code_action)?;
                }
                PickerAction::FocusBuffer(buffer_id) => {
                    let workspace_id = runtime
                        .model()
                        .active_workspace_id()
                        .map_err(|error| error.to_string())?;
                    runtime
                        .model_mut()
                        .focus_buffer(workspace_id, buffer_id)
                        .map_err(|error| error.to_string())?;
                    shell_ui_mut(runtime)?.focus_buffer(buffer_id);
                    sync_active_buffer(runtime)?;
                }
                PickerAction::CloseBuffer(buffer_id) => {
                    close_buffer_with_prompt(runtime, buffer_id)?;
                }
                PickerAction::CloseBufferSave(buffer_id) => {
                    close_buffer_save(runtime, buffer_id)?;
                }
                PickerAction::CloseBufferDiscard(buffer_id) => {
                    close_buffer_discard(runtime, buffer_id)?;
                }
                PickerAction::OpenFile(path) => {
                    open_workspace_file(runtime, &path)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::OpenFileLocation { path, target } => {
                    open_workspace_file_at(runtime, &path, target)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::OpenLspLocation { location } => {
                    open_lsp_location(runtime, &location)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::OpenAcpClient(client_id) => {
                    acp::open_acp_client(runtime, &client_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::CreateWorkspaceFile { root } => {
                    create_workspace_file_from_query(runtime, &root, &query)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::ActivateTheme(theme_id) => {
                    {
                        let registry = runtime
                            .services_mut()
                            .get_mut::<ThemeRegistry>()
                            .ok_or_else(|| "theme registry service missing".to_owned())?;
                        registry
                            .activate(&theme_id)
                            .map_err(|error| error.to_string())?;
                    }
                    if let Err(error) =
                        write_saved_theme_selection(&active_theme_state_path(), &theme_id)
                    {
                        record_runtime_error(runtime, "theme.save", error);
                    }
                }
                PickerAction::EmitHook { hook, detail } => {
                    let workspace_id = runtime
                        .model()
                        .active_workspace_id()
                        .map_err(|error| error.to_string())?;
                    let mut event = HookEvent::new().with_workspace(workspace_id);
                    if let Some(window_id) = runtime.model().active_window_id() {
                        event = event.with_window(window_id);
                    }
                    if let Ok(workspace) = runtime.model().workspace(workspace_id)
                        && let Some(pane_id) = workspace.active_pane_id()
                    {
                        event = event.with_pane(pane_id);
                        if let Some(buffer_id) = workspace
                            .pane(pane_id)
                            .and_then(|pane| pane.active_buffer())
                        {
                            event = event.with_buffer(buffer_id);
                        }
                    }
                    if let Some(detail) = detail {
                        event = event.with_detail(detail);
                    }
                    runtime
                        .emit_hook(&hook, event)
                        .map_err(|error| error.to_string())?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::UndoTreeNode { buffer_id, node_id } => {
                    apply_undo_tree_node(runtime, buffer_id, node_id)?;
                }
                PickerAction::VimSearch(direction) => {
                    submit_vim_search(runtime, direction, &query)?;
                }
                PickerAction::VimSearchResult { direction, target } => {
                    apply_vim_search_result(runtime, direction, target, &query)?;
                }
                PickerAction::InstallTreeSitterLanguage(language_id) => {
                    install_tree_sitter_language(runtime, &language_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::InstallLanguageServer(server_id) => {
                    tool_install::install_language_server_by_id(runtime, &server_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::InstallDebugAdapter(adapter_id) => {
                    tool_install::install_debug_adapter_by_id(runtime, &adapter_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::CreateWorkspace { name, root } => {
                    open_workspace_from_project(runtime, &name, &root)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::SwitchWorkspace(workspace_id) => {
                    switch_runtime_workspace(runtime, workspace_id)?;
                }
                PickerAction::DeleteWorkspace(workspace_id) => {
                    delete_runtime_workspace(runtime, workspace_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::GitPushRemote(remote) => {
                    push_git_remote(runtime, &remote)?;
                }
                PickerAction::GitFetchRemote(remote) => {
                    fetch_git_remote(runtime, &remote)?;
                }
                PickerAction::GitWorktreeBranch {
                    remote_branch,
                    local_branch,
                } => {
                    open_git_worktree_path_picker(runtime, &remote_branch, &local_branch)?;
                }
                PickerAction::GitWorktreeCreate {
                    remote_branch,
                    local_branch,
                    base_dir,
                } => {
                    create_git_worktree_from_query(
                        runtime,
                        &remote_branch,
                        &local_branch,
                        &base_dir,
                        &query,
                    )?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::GitWorktreeOilBranch {
                    buffer_id,
                    remote_branch,
                    local_branch,
                } => {
                    finish_oil_worktree_branch_selection(
                        runtime,
                        buffer_id,
                        &remote_branch,
                        &local_branch,
                        false,
                    )?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::GitWorktreeOilNewBranch { buffer_id } => {
                    open_git_worktree_new_branch_prompt(runtime, buffer_id)?;
                }
                PickerAction::GitWorktreeDashboardCreate { base_dir } => {
                    open_git_worktree_dashboard_create(runtime, &base_dir)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::GitBranchAction { action, branch } => match action {
                    GitBranchActionKind::Checkout => {
                        checkout_git_branch(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergePlain => {
                        merge_git_plain(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergeEdit => {
                        merge_git_edit(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergeNoCommit => {
                        merge_git_no_commit(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergeSquash => {
                        merge_git_squash(runtime, &branch)?;
                    }
                    GitBranchActionKind::MergePreview => {
                        merge_git_preview(runtime, &branch)?;
                    }
                    GitBranchActionKind::RebaseOnto => {
                        rebase_git_onto(runtime, &branch)?;
                    }
                    GitBranchActionKind::RebaseInteractive => {
                        rebase_git_interactive_onto(runtime, &branch)?;
                    }
                },
                PickerAction::GitCommitAction { action, commit } => match action {
                    GitCommitActionKind::CherryPick => {
                        cherry_pick_git_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::CherryPickNoCommit => {
                        cherry_pick_git_commit_no_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::Revert => {
                        revert_git_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::RevertNoCommit => {
                        revert_git_commit_no_commit(runtime, &commit)?;
                    }
                    GitCommitActionKind::ResetMixed => {
                        reset_git_commit(runtime, &commit, GitResetMode::Mixed)?;
                    }
                    GitCommitActionKind::ResetSoft => {
                        reset_git_commit(runtime, &commit, GitResetMode::Soft)?;
                    }
                    GitCommitActionKind::ResetHard => {
                        reset_git_commit(runtime, &commit, GitResetMode::Hard)?;
                    }
                    GitCommitActionKind::ResetKeep => {
                        reset_git_commit(runtime, &commit, GitResetMode::Keep)?;
                    }
                },
                PickerAction::AcpInsertSlashCommand { buffer_id, command } => {
                    acp::acp_insert_slash_command(runtime, buffer_id, &command)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpInsertFileMention {
                    buffer_id,
                    relative_path,
                } => {
                    acp::acp_insert_file_mention(runtime, buffer_id, &relative_path)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpLoadSession {
                    buffer_id,
                    session_id,
                    session_title,
                } => {
                    acp::acp_load_session(runtime, buffer_id, &session_id, Some(&session_title))?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpSetMode { buffer_id, mode_id } => {
                    acp::acp_set_mode(runtime, buffer_id, &mode_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpSetModel {
                    buffer_id,
                    model_id,
                } => {
                    acp::acp_set_model(runtime, buffer_id, &model_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::AcpResolvePermission {
                    request_id,
                    option_id,
                } => {
                    acp::acp_resolve_permission_option(runtime, request_id, &option_id)?;
                    sync_active_buffer(runtime)?;
                }
                PickerAction::CopyToClipboard(text) => {
                    write_system_clipboard(&text);
                }
                PickerAction::StopLspSession { server_id, root } => {
                    stop_lsp_session(runtime, &server_id, root.as_deref())?;
                }
                PickerAction::RestartLspSession { server_id, root } => {
                    restart_lsp_session(runtime, &server_id, root.as_deref())?;
                }
                PickerAction::StartDapSession {
                    adapter_id,
                    configuration,
                    ask_heuristic_compile,
                } => {
                    let result = continue_dap_start(
                        runtime,
                        &adapter_id,
                        configuration,
                        ask_heuristic_compile,
                    );
                    report_dap_result(runtime, "DAP start failed", result)?;
                }
                PickerAction::ConfirmDapCompile {
                    adapter_id,
                    configuration,
                    command,
                } => {
                    let result = run_dap_prelaunch_then_start(
                        runtime,
                        &adapter_id,
                        configuration,
                        Some(command),
                    );
                    report_dap_result(runtime, "DAP start failed", result)?;
                }
                PickerAction::SkipDapCompile {
                    adapter_id,
                    configuration,
                } => {
                    let result =
                        run_dap_prelaunch_then_start(runtime, &adapter_id, configuration, None);
                    report_dap_result(runtime, "DAP start failed", result)?;
                }
                PickerAction::RemoveDapExpression { expression } => {
                    remove_dap_expression(runtime, &expression)?;
                }
                PickerAction::SwitchDapThread { thread_id } => {
                    switch_dap_thread(runtime, thread_id)?;
                }
                PickerAction::SwitchDapStackFrame { frame_id } => {
                    switch_dap_stack_frame(runtime, frame_id)?;
                }
            }

            if let Some(PickerKind::AcpPermission { request_id }) = picker_kind {
                acp::acp_permission_picker_submitted(runtime, request_id)?;
            }

            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_QUICKFIX_OPEN, "shell.quickfix-open", |_, runtime| {
            quickfix_open_current_list(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(HOOK_QUICKFIX_NEXT, "shell.quickfix-next", |_, runtime| {
            quickfix_select_next(runtime)?;
            Ok(())
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_QUICKFIX_PREVIOUS,
            "shell.quickfix-previous",
            |_, runtime| {
                quickfix_select_previous(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_QUICKFIX_TOGGLE_MARK,
            "shell.quickfix-toggle-mark",
            |_, runtime| {
                quickfix_toggle_mark(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_QUICKFIX_CLEAR_MARKS,
            "shell.quickfix-clear-marks",
            |_, runtime| {
                quickfix_clear_marks(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            HOOK_QUICKFIX_MARK_ALL,
            "shell.quickfix-mark-all",
            |_, runtime| {
                quickfix_mark_all(runtime)?;
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn load_window_icon() -> Result<sdl3::surface::Surface<'static>, ShellError> {
    let image = image::load_from_memory_with_format(WINDOW_ICON_BYTES, image::ImageFormat::Png)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let row_bytes = width as usize * 4;
    // ABGR8888 maps to RGBA byte order on little-endian, matching image::Rgba8 output.
    let mut surface = sdl3::surface::Surface::new(width, height, PixelFormat::ABGR8888)
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let pitch = surface.pitch() as usize;
    if pitch < row_bytes {
        return Err(ShellError::Sdl(format!(
            "icon pitch {pitch} is smaller than row width {row_bytes}"
        )));
    }
    let raw = rgba.into_raw();
    surface.with_lock_mut(|buffer| {
        for row in 0..height as usize {
            let src_start = row * row_bytes;
            let dst_start = row * pitch;
            buffer[dst_start..dst_start + row_bytes]
                .copy_from_slice(&raw[src_start..src_start + row_bytes]);
        }
    });
    Ok(surface)
}

fn register_lsp_status_hooks(runtime: &mut EditorRuntime) -> Result<(), String> {
    if runtime.hooks().contains(HOOK_LSP_START) {
        runtime
            .subscribe_hook(
                HOOK_LSP_START,
                "shell.track-lsp-server",
                |event, runtime| start_lsp_for_active_buffer(runtime, event.detail.as_deref()),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_STOP) {
        runtime
            .subscribe_hook(HOOK_LSP_STOP, "shell.stop-lsp-server", |_, runtime| {
                open_lsp_session_stop_picker(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_RESTART) {
        runtime
            .subscribe_hook(
                HOOK_LSP_RESTART,
                "shell.restart-lsp-server",
                |_, runtime| open_lsp_session_restart_picker(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_LOG) {
        runtime
            .subscribe_hook(HOOK_LSP_LOG, "shell.open-lsp-log", |_, runtime| {
                open_lsp_log_buffer(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_DEFINITION) {
        runtime
            .subscribe_hook(HOOK_LSP_DEFINITION, "shell.lsp-definition", |_, runtime| {
                goto_lsp_definition(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_REFERENCES) {
        runtime
            .subscribe_hook(HOOK_LSP_REFERENCES, "shell.lsp-references", |_, runtime| {
                goto_lsp_references(runtime)
            })
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_IMPLEMENTATION) {
        runtime
            .subscribe_hook(
                HOOK_LSP_IMPLEMENTATION,
                "shell.lsp-implementation",
                |_, runtime| goto_lsp_implementation(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_DIAGNOSTICS) {
        runtime
            .subscribe_hook(
                HOOK_LSP_DIAGNOSTICS,
                "shell.lsp-diagnostics",
                |_, runtime| open_lsp_diagnostics(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_CODE_ACTIONS) {
        runtime
            .subscribe_hook(
                HOOK_LSP_CODE_ACTIONS,
                "shell.lsp-code-actions",
                |_, runtime| open_lsp_code_actions(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_COPILOT_SIGN_IN) {
        runtime
            .subscribe_hook(
                HOOK_LSP_COPILOT_SIGN_IN,
                "shell.lsp-copilot-sign-in",
                |_, runtime| copilot_sign_in_for_active_buffer(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_COPILOT_SIGN_OUT) {
        runtime
            .subscribe_hook(
                HOOK_LSP_COPILOT_SIGN_OUT,
                "shell.lsp-copilot-sign-out",
                |_, runtime| copilot_sign_out_for_active_buffer(runtime),
            )
            .map_err(|error| error.to_string())?;
    }

    if runtime.hooks().contains(HOOK_LSP_INSTALL) {
        runtime
            .subscribe_hook(
                HOOK_LSP_INSTALL,
                "shell.install-lsp-server",
                |event, runtime| {
                    tool_install::handle_lsp_install_hook(runtime, event.detail.as_deref())
                },
            )
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}
