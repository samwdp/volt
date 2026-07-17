use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    VimActionSpec, VimEditAction,
};

// Change this to customize the leader key for Vim bindings.
const LEADER_KEY: &str = "Space";

/// Returns the metadata for the Vim bindings package.
pub fn package() -> PluginPackage {
    let mut commands = vec![
        hook_command(
            "vim.move-left",
            "Moves the cursor left in Vim normal mode.",
            "editor.cursor.move-left",
            "left",
        ),
        hook_command(
            "vim.move-down",
            "Moves the cursor down in Vim normal mode.",
            "editor.cursor.move-down",
            "down",
        ),
        hook_command(
            "vim.move-up",
            "Moves the cursor up in Vim normal mode.",
            "editor.cursor.move-up",
            "up",
        ),
        hook_command(
            "vim.move-right",
            "Moves the cursor right in Vim normal mode.",
            "editor.cursor.move-right",
            "right",
        ),
        hook_command(
            "vim.move-word-forward",
            "Moves to the next word boundary in Vim normal mode.",
            "editor.cursor.move-word-forward",
            "word-forward",
        ),
        hook_command(
            "vim.move-word-backward",
            "Moves to the previous word boundary in Vim normal mode.",
            "editor.cursor.move-word-backward",
            "word-backward",
        ),
        hook_command(
            "vim.move-word-end",
            "Moves to the end of the current or next word in Vim normal mode.",
            "editor.cursor.move-word-end",
            "word-end",
        ),
        hook_command(
            "vim.move-big-word-forward",
            "Moves to the next Vim WORD boundary in normal mode.",
            "editor.cursor.move-big-word-forward",
            "big-word-forward",
        ),
        hook_command(
            "vim.move-big-word-backward",
            "Moves to the previous Vim WORD boundary in normal mode.",
            "editor.cursor.move-big-word-backward",
            "big-word-backward",
        ),
        hook_command(
            "vim.move-big-word-end",
            "Moves to the end of the current or next Vim WORD in normal mode.",
            "editor.cursor.move-big-word-end",
            "big-word-end",
        ),
        hook_command(
            "vim.move-sentence-forward",
            "Moves to the start of the next sentence in Vim normal mode.",
            "editor.cursor.move-sentence-forward",
            "sentence-forward",
        ),
        hook_command(
            "vim.move-sentence-backward",
            "Moves to the start of the current or previous sentence in Vim normal mode.",
            "editor.cursor.move-sentence-backward",
            "sentence-backward",
        ),
        hook_command(
            "vim.move-paragraph-forward",
            "Moves to the start of the next paragraph in Vim normal mode.",
            "editor.cursor.move-paragraph-forward",
            "paragraph-forward",
        ),
        hook_command(
            "vim.move-paragraph-backward",
            "Moves to the start of the current or previous paragraph in Vim normal mode.",
            "editor.cursor.move-paragraph-backward",
            "paragraph-backward",
        ),
        hook_command(
            "vim.match-pair",
            "Moves to the matching paired delimiter.",
            "editor.cursor.match-pair",
            "match-pair",
        ),
        hook_command(
            "vim.move-line-start",
            "Moves to the start of the current line.",
            "editor.cursor.move-line-start",
            "line-start",
        ),
        hook_command(
            "vim.move-line-first-non-blank",
            "Moves to the first non-blank character on the current line.",
            "editor.cursor.move-line-first-non-blank",
            "line-first-non-blank",
        ),
        hook_command(
            "vim.move-line-end",
            "Moves to the end of the current line.",
            "editor.cursor.move-line-end",
            "line-end",
        ),
        hook_command(
            "vim.move-screen-top",
            "Moves to the first visible screen line.",
            "editor.cursor.move-screen-top",
            "screen-top",
        ),
        hook_command(
            "vim.move-screen-middle",
            "Moves to the middle visible screen line.",
            "editor.cursor.move-screen-middle",
            "screen-middle",
        ),
        hook_command(
            "vim.move-screen-bottom",
            "Moves to the last visible screen line.",
            "editor.cursor.move-screen-bottom",
            "screen-bottom",
        ),
        hook_command(
            "vim.goto-first-line",
            "Moves to the first line in the buffer.",
            "editor.cursor.goto-first-line",
            "first-line",
        ),
        hook_command(
            "vim.goto-last-line",
            "Moves to the last line in the buffer.",
            "editor.cursor.goto-last-line",
            "last-line",
        ),
        hook_command(
            "vim.current-line-top",
            "Redraws with the current line at the top of the window.",
            "editor.vim.current-line-top",
            "current-line-top",
        ),
        hook_command(
            "vim.center-current-line",
            "Redraws with the current line at the center of the window.",
            "editor.vim.center-current-line",
            "center-current-line",
        ),
        hook_command(
            "vim.current-line-bottom",
            "Redraws with the current line at the bottom of the window.",
            "editor.vim.current-line-bottom",
            "current-line-bottom",
        ),
        hook_command(
            "vim.enter-insert-mode",
            "Switches the editor into Vim insert mode.",
            "editor.mode.insert",
            "insert",
        ),
        hook_command(
            "vim.enter-normal-mode",
            "Switches the editor into Vim normal mode.",
            "editor.mode.normal",
            "normal",
        ),
        vim_edit_command(
            "vim.enter-visual-mode",
            "Switches the editor into Vim visual mode.",
            VimEditAction::EnterVisual,
        ),
        vim_edit_command(
            "vim.enter-visual-line-mode",
            "Switches the editor into Vim linewise visual mode.",
            VimEditAction::EnterVisualLine,
        ),
        vim_edit_command(
            "vim.enter-visual-block-mode",
            "Switches the editor into Vim blockwise visual mode.",
            VimEditAction::EnterVisualBlock,
        ),
        vim_edit_command(
            "vim.delete-char",
            "Deletes the character under the cursor.",
            VimEditAction::DeleteChar,
        ),
        vim_edit_command(
            "vim.delete-char-before",
            "Deletes the character before the cursor.",
            VimEditAction::DeleteCharBefore,
        ),
        vim_edit_command(
            "vim.delete-line-end",
            "Deletes from the cursor to the end of the line.",
            VimEditAction::DeleteLineEnd,
        ),
        vim_edit_command(
            "vim.change-line-end",
            "Changes from the cursor to the end of the line.",
            VimEditAction::ChangeLineEnd,
        ),
        vim_edit_command(
            "vim.yank-line",
            "Yanks the current line.",
            VimEditAction::YankLine,
        ),
        vim_edit_command(
            "vim.substitute-char",
            "Substitutes characters under the cursor and enters insert mode.",
            VimEditAction::SubstituteChar,
        ),
        vim_edit_command(
            "vim.substitute-line",
            "Substitutes the current line and enters insert mode.",
            VimEditAction::SubstituteLine,
        ),
        vim_edit_command(
            "vim.replace-char",
            "Replaces characters under the cursor without entering insert mode.",
            VimEditAction::ReplaceChar,
        ),
        vim_edit_command(
            "vim.enter-replace-mode",
            "Enters Vim replace mode.",
            VimEditAction::EnterReplaceMode,
        ),
        vim_edit_command(
            "vim.toggle-case",
            "Toggles the case of characters under the cursor.",
            VimEditAction::ToggleCase,
        ),
        vim_edit_command(
            "vim.start-delete-operator",
            "Starts a Vim delete operator-pending command.",
            VimEditAction::StartDeleteOperator,
        ),
        vim_edit_command(
            "vim.start-change-operator",
            "Starts a Vim change operator-pending command.",
            VimEditAction::StartChangeOperator,
        ),
        vim_edit_command(
            "vim.start-yank-operator",
            "Starts a Vim yank operator-pending command.",
            VimEditAction::StartYankOperator,
        ),
        vim_edit_command(
            "vim.start-format-operator",
            "Starts a Vim format operator.",
            VimEditAction::StartFormatOperator,
        ),
        vim_edit_command(
            "vim.visual-format",
            "Formats the current visual selection.",
            VimEditAction::VisualFormat,
        ),
        vim_edit_command(
            "vim.toggle-line-comment",
            "Toggles a line comment on the current line.",
            VimEditAction::ToggleLineComment,
        ),
        vim_edit_command(
            "vim.visual-toggle-comment",
            "Toggles line comments across the current visual selection.",
            VimEditAction::VisualToggleComment,
        ),
        vim_edit_command(
            "vim.append-after-cursor",
            "Appends after the cursor and enters insert mode.",
            VimEditAction::Append,
        ),
        vim_edit_command(
            "vim.append-line-end",
            "Appends at the end of the line and enters insert mode.",
            VimEditAction::AppendLineEnd,
        ),
        vim_edit_command(
            "vim.insert-line-start",
            "Inserts at the first non-blank character on the line.",
            VimEditAction::InsertLineStart,
        ),
        vim_edit_command(
            "vim.open-line-below",
            "Opens a new line below and enters insert mode.",
            VimEditAction::OpenLineBelow,
        ),
        vim_edit_command(
            "vim.open-line-above",
            "Opens a new line above and enters insert mode.",
            VimEditAction::OpenLineAbove,
        ),
        vim_edit_command(
            "vim.undo",
            "Undoes the previous change.",
            VimEditAction::Undo,
        ),
        vim_edit_command("vim.redo", "Redoes the next change.", VimEditAction::Redo),
        hook_command(
            "vim.scroll-half-page-down",
            "Scrolls down by half a page in Vim normal mode.",
            "editor.vim.scroll-half-page-down",
            "scroll-half-page-down",
        ),
        hook_command(
            "vim.scroll-half-page-up",
            "Scrolls up by half a page in Vim normal mode.",
            "editor.vim.scroll-half-page-up",
            "scroll-half-page-up",
        ),
        hook_command(
            "vim.scroll-page-down",
            "Scrolls down by a full page in Vim normal mode.",
            "editor.vim.scroll-page-down",
            "scroll-page-down",
        ),
        hook_command(
            "vim.scroll-page-up",
            "Scrolls up by a full page in Vim normal mode.",
            "editor.vim.scroll-page-up",
            "scroll-page-up",
        ),
        hook_command(
            "vim.scroll-line-down",
            "Scrolls the window down by one line in Vim normal mode.",
            "editor.vim.scroll-line-down",
            "scroll-line-down",
        ),
        hook_command(
            "vim.scroll-line-up",
            "Scrolls the window up by one line in Vim normal mode.",
            "editor.vim.scroll-line-up",
            "scroll-line-up",
        ),
        vim_edit_command(
            "vim.start-g-prefix",
            "Starts a Vim g-prefixed motion.",
            VimEditAction::StartGPrefix,
        ),
        vim_edit_command(
            "vim.start-find-forward",
            "Starts a Vim f motion on the current line.",
            VimEditAction::StartFindForward,
        ),
        vim_edit_command(
            "vim.start-find-backward",
            "Starts a Vim F motion on the current line.",
            VimEditAction::StartFindBackward,
        ),
        vim_edit_command(
            "vim.start-till-forward",
            "Starts a Vim t motion on the current line.",
            VimEditAction::StartTillForward,
        ),
        vim_edit_command(
            "vim.start-till-backward",
            "Starts a Vim T motion on the current line.",
            VimEditAction::StartTillBackward,
        ),
        vim_edit_command(
            "vim.repeat-find-next",
            "Repeats the last Vim find motion forward.",
            VimEditAction::RepeatFindNext,
        ),
        vim_edit_command(
            "vim.repeat-find-previous",
            "Repeats the last Vim find motion backward.",
            VimEditAction::RepeatFindPrevious,
        ),
        vim_edit_command(
            "vim.start-search-forward",
            "Opens a Vim-style forward search prompt.",
            VimEditAction::StartSearchForward,
        ),
        vim_edit_command(
            "vim.start-search-backward",
            "Opens a Vim-style backward search prompt.",
            VimEditAction::StartSearchBackward,
        ),
        vim_edit_command(
            "vim.search-word-forward",
            "Searches forward for the word under the cursor.",
            VimEditAction::SearchWordForward,
        ),
        vim_edit_command(
            "vim.search-word-backward",
            "Searches backward for the word under the cursor.",
            VimEditAction::SearchWordBackward,
        ),
        vim_edit_command(
            "vim.repeat-search-next",
            "Repeats the last Vim search in the same direction.",
            VimEditAction::RepeatSearchNext,
        ),
        vim_edit_command(
            "vim.repeat-search-previous",
            "Repeats the last Vim search in the opposite direction.",
            VimEditAction::RepeatSearchPrevious,
        ),
        vim_edit_command(
            "vim.select-register",
            "Selects a Vim register for the next operation.",
            VimEditAction::SelectRegister,
        ),
        vim_edit_command(
            "vim.set-mark",
            "Sets a Vim mark at the current cursor.",
            VimEditAction::SetMark,
        ),
        vim_edit_command(
            "vim.goto-mark-line",
            "Jumps to the line of a Vim mark.",
            VimEditAction::GotoMarkLine,
        ),
        vim_edit_command(
            "vim.goto-mark",
            "Jumps to the exact position of a Vim mark.",
            VimEditAction::GotoMark,
        ),
        vim_edit_command(
            "vim.toggle-macro-record",
            "Starts or stops Vim macro recording.",
            VimEditAction::ToggleMacroRecord,
        ),
        vim_edit_command(
            "vim.start-macro-playback",
            "Plays back a recorded Vim macro.",
            VimEditAction::StartMacroPlayback,
        ),
        vim_edit_command(
            "vim.put-after",
            "Puts the most recent Vim yank after the cursor.",
            VimEditAction::PutAfter,
        ),
        vim_edit_command(
            "vim.put-before",
            "Puts the most recent Vim yank before the cursor.",
            VimEditAction::PutBefore,
        ),
        vim_edit_command(
            "vim.visual-put-after",
            "Replaces the current visual selection with the most recent Vim yank.",
            VimEditAction::VisualPutAfter,
        ),
        vim_edit_command(
            "vim.visual-put-before",
            "Replaces the current visual selection with the most recent Vim yank before the cursor.",
            VimEditAction::VisualPutBefore,
        ),
        vim_edit_command(
            "vim.visual-delete",
            "Deletes the current visual selection.",
            VimEditAction::VisualDelete,
        ),
        vim_edit_command(
            "vim.visual-change",
            "Changes the current visual selection.",
            VimEditAction::VisualChange,
        ),
        vim_edit_command(
            "vim.visual-replace-char",
            "Replaces each character in the current visual selection with the next typed character.",
            VimEditAction::VisualReplaceChar,
        ),
        vim_edit_command(
            "vim.visual-block-insert",
            "Inserts before the visual block selection and enters insert mode.",
            VimEditAction::VisualBlockInsert,
        ),
        vim_edit_command(
            "vim.visual-block-append",
            "Appends after the visual block selection and enters insert mode.",
            VimEditAction::VisualBlockAppend,
        ),
        vim_edit_command(
            "vim.visual-yank",
            "Yanks the current visual selection.",
            VimEditAction::VisualYank,
        ),
        vim_edit_command(
            "vim.visual-toggle-case",
            "Toggles the case of the current visual selection.",
            VimEditAction::VisualToggleCase,
        ),
        vim_edit_command(
            "vim.visual-lowercase",
            "Lowercases the current visual selection.",
            VimEditAction::VisualLowercase,
        ),
        vim_edit_command(
            "vim.visual-uppercase",
            "Uppercases the current visual selection.",
            VimEditAction::VisualUppercase,
        ),
        vim_edit_command(
            "vim.visual-indent",
            "Indents each selected line in visual mode.",
            VimEditAction::VisualIndent,
        ),
        vim_edit_command(
            "vim.visual-outdent",
            "Outdents each selected line in visual mode.",
            VimEditAction::VisualOutdent,
        ),
        vim_edit_command(
            "vim.visual-join",
            "Joins the selected visual lines into a single line.",
            VimEditAction::VisualJoin,
        ),
        vim_edit_command(
            "vim.visual-move-down",
            "Moves selected visual lines down one line and reindents them.",
            VimEditAction::VisualMoveDown,
        ),
        vim_edit_command(
            "vim.visual-move-up",
            "Moves selected visual lines up one line and reindents them.",
            VimEditAction::VisualMoveUp,
        ),
        vim_edit_command(
            "vim.visual-swap-anchor",
            "Swaps the active and anchor ends of the current visual selection.",
            VimEditAction::VisualSwapAnchor,
        ),
        vim_edit_command(
            "vim.start-visual-inner-text-object",
            "Starts a visual-mode inner text object selection.",
            VimEditAction::StartVisualInnerTextObject,
        ),
        vim_edit_command(
            "vim.start-visual-around-text-object",
            "Starts a visual-mode around text object selection.",
            VimEditAction::StartVisualAroundTextObject,
        ),
        PluginCommand::new(
            "vim.command-line",
            "Opens a Vim-style command line.",
            vec![PluginAction::emit_hook(
                "editor.vim.command-line",
                None::<&str>,
            )],
        ),
    ];
    commands.extend(crate::commandline::commands());

    let key_bindings = vec![
        // Left-right motions
        normal_binding(
            crate::hover::TOGGLE_CHORD,
            "hover.toggle",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("h", "vim.move-left", PluginKeymapScope::Workspace),
        normal_binding("l", "vim.move-right", PluginKeymapScope::Workspace),
        normal_binding("0", "vim.move-line-start", PluginKeymapScope::Workspace),
        normal_binding(
            "^",
            "vim.move-line-first-non-blank",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("$", "vim.move-line-end", PluginKeymapScope::Workspace),
        // Various motions
        normal_binding("%", "vim.match-pair", PluginKeymapScope::Workspace),
        normal_binding(
            "(",
            "vim.move-sentence-backward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            ")",
            "vim.move-sentence-forward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "{",
            "vim.move-paragraph-backward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "}",
            "vim.move-paragraph-forward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("H", "vim.move-screen-top", PluginKeymapScope::Workspace),
        normal_binding("M", "vim.move-screen-middle", PluginKeymapScope::Workspace),
        normal_binding("L", "vim.move-screen-bottom", PluginKeymapScope::Workspace),
        normal_binding("f", "vim.start-find-forward", PluginKeymapScope::Workspace),
        normal_binding("F", "vim.start-find-backward", PluginKeymapScope::Workspace),
        normal_binding("t", "vim.start-till-forward", PluginKeymapScope::Workspace),
        normal_binding("T", "vim.start-till-backward", PluginKeymapScope::Workspace),
        normal_binding(";", "vim.repeat-find-next", PluginKeymapScope::Workspace),
        normal_binding(
            ",",
            "vim.repeat-find-previous",
            PluginKeymapScope::Workspace,
        ),
        // Up-down motions
        normal_binding("j", "vim.move-down", PluginKeymapScope::Workspace),
        normal_binding("k", "vim.move-up", PluginKeymapScope::Workspace),
        normal_binding("g", "vim.start-g-prefix", PluginKeymapScope::Workspace),
        normal_binding("g d", "lsp.definition", PluginKeymapScope::Workspace),
        normal_binding("g r", "lsp.references", PluginKeymapScope::Workspace),
        normal_binding("g r r", "lsp.references", PluginKeymapScope::Workspace),
        normal_binding("g i", "lsp.implementation", PluginKeymapScope::Workspace),
        normal_binding(
            "g q",
            "vim.start-format-operator",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("G", "vim.goto-last-line", PluginKeymapScope::Workspace),
        // Text object motions
        normal_binding("w", "vim.move-word-forward", PluginKeymapScope::Workspace),
        normal_binding(
            "W",
            "vim.move-big-word-forward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("e", "vim.move-word-end", PluginKeymapScope::Workspace),
        normal_binding("E", "vim.move-big-word-end", PluginKeymapScope::Workspace),
        normal_binding("b", "vim.move-word-backward", PluginKeymapScope::Workspace),
        normal_binding(
            "B",
            "vim.move-big-word-backward",
            PluginKeymapScope::Workspace,
        ),
        // Pattern searches
        normal_binding(
            "/",
            "vim.start-search-forward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "?",
            "vim.start-search-backward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("\\", "pane.split-vertical", PluginKeymapScope::Workspace),
        normal_binding("|", "pane.split-horizontal", PluginKeymapScope::Workspace),
        normal_binding("-", "oil.open-directory", PluginKeymapScope::Workspace),
        normal_binding("_", "oil.open-parent", PluginKeymapScope::Workspace),
        normal_binding("*", "vim.search-word-forward", PluginKeymapScope::Workspace),
        normal_binding(
            "#",
            "vim.search-word-backward",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("n", "vim.repeat-search-next", PluginKeymapScope::Workspace),
        normal_binding(
            "N",
            "vim.repeat-search-previous",
            PluginKeymapScope::Workspace,
        ),
        // Registers and macros
        normal_binding("\"", "vim.select-register", PluginKeymapScope::Workspace),
        normal_binding("q", "vim.toggle-macro-record", PluginKeymapScope::Workspace),
        normal_binding(
            "@",
            "vim.start-macro-playback",
            PluginKeymapScope::Workspace,
        ),
        // Marks
        normal_binding("m", "vim.set-mark", PluginKeymapScope::Workspace),
        normal_binding("'", "vim.goto-mark-line", PluginKeymapScope::Workspace),
        normal_binding("`", "vim.goto-mark", PluginKeymapScope::Workspace),
        // Inserting text
        normal_binding("a", "vim.append-after-cursor", PluginKeymapScope::Workspace),
        normal_binding("A", "vim.append-line-end", PluginKeymapScope::Workspace),
        normal_binding("i", "vim.enter-insert-mode", PluginKeymapScope::Workspace),
        normal_binding("I", "vim.insert-line-start", PluginKeymapScope::Workspace),
        normal_binding("o", "vim.open-line-below", PluginKeymapScope::Workspace),
        normal_binding("O", "vim.open-line-above", PluginKeymapScope::Workspace),
        // Deleting text
        normal_binding("x", "vim.delete-char", PluginKeymapScope::Workspace),
        normal_binding("X", "vim.delete-char-before", PluginKeymapScope::Workspace),
        normal_binding(
            "d",
            "vim.start-delete-operator",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("D", "vim.delete-line-end", PluginKeymapScope::Workspace),
        // Copying and moving text
        normal_binding("y", "vim.start-yank-operator", PluginKeymapScope::Workspace),
        normal_binding("Y", "vim.yank-line", PluginKeymapScope::Workspace),
        normal_binding("p", "vim.put-after", PluginKeymapScope::Workspace),
        normal_binding("P", "vim.put-before", PluginKeymapScope::Workspace),
        // Changing text
        normal_binding(
            "c",
            "vim.start-change-operator",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("C", "vim.change-line-end", PluginKeymapScope::Workspace),
        normal_binding("s", "vim.substitute-char", PluginKeymapScope::Workspace),
        normal_binding("S", "vim.substitute-line", PluginKeymapScope::Workspace),
        normal_binding("r", "vim.replace-char", PluginKeymapScope::Workspace),
        normal_binding("R", "vim.enter-replace-mode", PluginKeymapScope::Workspace),
        normal_binding("~", "vim.toggle-case", PluginKeymapScope::Workspace),
        normal_binding(
            "=",
            "vim.start-format-operator",
            PluginKeymapScope::Workspace,
        ),
        // Visual mode
        normal_binding("v", "vim.enter-visual-mode", PluginKeymapScope::Workspace),
        normal_binding(
            "V",
            "vim.enter-visual-line-mode",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "Ctrl+v",
            "vim.enter-visual-block-mode",
            PluginKeymapScope::Workspace,
        ),
        // Undo/Redo commands
        normal_binding("u", "vim.undo", PluginKeymapScope::Workspace),
        normal_binding_commands(
            "Ctrl+d",
            &["vim.scroll-half-page-down", "vim.center-current-line"],
            PluginKeymapScope::Workspace,
        ),
        normal_binding("Ctrl+r", "vim.redo", PluginKeymapScope::Workspace),
        normal_binding_commands(
            "Ctrl+u",
            &["vim.scroll-half-page-up", "vim.center-current-line"],
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "Ctrl+e",
            "vim.scroll-line-down",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "Ctrl+f",
            "vim.scroll-page-down",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("Ctrl+b", "vim.scroll-page-up", PluginKeymapScope::Workspace),
        normal_binding("Ctrl+y", "vim.scroll-line-up", PluginKeymapScope::Workspace),
        normal_binding(
            "z Enter",
            "vim.current-line-top",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("z t", "vim.current-line-top", PluginKeymapScope::Workspace),
        normal_binding(
            "z .",
            "vim.center-current-line",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "z z",
            "vim.center-current-line",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "z -",
            "vim.current-line-bottom",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "z b",
            "vim.current-line-bottom",
            PluginKeymapScope::Workspace,
        ),
        // Window navigation
        normal_binding(
            "Ctrl+h",
            "workspace.window-left",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "Ctrl+j",
            "workspace.window-down",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "Ctrl+k",
            "workspace.window-up",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "Ctrl+l",
            "workspace.window-right",
            PluginKeymapScope::Workspace,
        ),
        // Window navigation (popups)
        normal_binding("Ctrl+h", "workspace.window-left", PluginKeymapScope::Popup),
        normal_binding("Ctrl+j", "workspace.window-down", PluginKeymapScope::Popup),
        normal_binding("Ctrl+k", "workspace.window-up", PluginKeymapScope::Popup),
        normal_binding("Ctrl+l", "workspace.window-right", PluginKeymapScope::Popup),
        // Insert mode keys
        PluginKeyBinding::new("Escape", "vim.enter-normal-mode", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new("Escape", "vim.enter-normal-mode", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Normal),
        // Command-line editing
        normal_binding(":", "vim.command-line", PluginKeymapScope::Workspace),
        normal_binding(
            "Alt+x",
            "picker.open-commands",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("Ctrl+.", "workspace.list-files", PluginKeymapScope::Global),
        // Visual mode
        visual_binding("v", "vim.enter-visual-mode", PluginKeymapScope::Workspace),
        visual_binding(
            "V",
            "vim.enter-visual-line-mode",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "Ctrl+v",
            "vim.enter-visual-block-mode",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("o", "vim.visual-swap-anchor", PluginKeymapScope::Workspace),
        PluginKeyBinding::new("Escape", "vim.enter-normal-mode", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Visual),
        // Left-right motions
        visual_binding("h", "vim.move-left", PluginKeymapScope::Workspace),
        visual_binding("l", "vim.move-right", PluginKeymapScope::Workspace),
        visual_binding("0", "vim.move-line-start", PluginKeymapScope::Workspace),
        visual_binding(
            "^",
            "vim.move-line-first-non-blank",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("$", "vim.move-line-end", PluginKeymapScope::Workspace),
        // Various motions
        visual_binding("%", "vim.match-pair", PluginKeymapScope::Workspace),
        visual_binding(
            "(",
            "vim.move-sentence-backward",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            ")",
            "vim.move-sentence-forward",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "{",
            "vim.move-paragraph-backward",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "}",
            "vim.move-paragraph-forward",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("H", "vim.move-screen-top", PluginKeymapScope::Workspace),
        visual_binding("M", "vim.move-screen-middle", PluginKeymapScope::Workspace),
        visual_binding("L", "vim.move-screen-bottom", PluginKeymapScope::Workspace),
        visual_binding("f", "vim.start-find-forward", PluginKeymapScope::Workspace),
        visual_binding("F", "vim.start-find-backward", PluginKeymapScope::Workspace),
        visual_binding("t", "vim.start-till-forward", PluginKeymapScope::Workspace),
        visual_binding("T", "vim.start-till-backward", PluginKeymapScope::Workspace),
        visual_binding(";", "vim.repeat-find-next", PluginKeymapScope::Workspace),
        visual_binding(
            ",",
            "vim.repeat-find-previous",
            PluginKeymapScope::Workspace,
        ),
        // Up-down motions
        visual_binding("j", "vim.move-down", PluginKeymapScope::Workspace),
        visual_binding("k", "vim.move-up", PluginKeymapScope::Workspace),
        visual_binding("g", "vim.start-g-prefix", PluginKeymapScope::Workspace),
        visual_binding(
            "g c",
            "vim.visual-toggle-comment",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("G", "vim.goto-last-line", PluginKeymapScope::Workspace),
        // Text object motions
        visual_binding("w", "vim.move-word-forward", PluginKeymapScope::Workspace),
        visual_binding(
            "W",
            "vim.move-big-word-forward",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("e", "vim.move-word-end", PluginKeymapScope::Workspace),
        visual_binding("E", "vim.move-big-word-end", PluginKeymapScope::Workspace),
        visual_binding("b", "vim.move-word-backward", PluginKeymapScope::Workspace),
        visual_binding(
            "B",
            "vim.move-big-word-backward",
            PluginKeymapScope::Workspace,
        ),
        // Pattern searches
        visual_binding(
            "/",
            "vim.start-search-forward",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "?",
            "vim.start-search-backward",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("n", "vim.repeat-search-next", PluginKeymapScope::Workspace),
        visual_binding(
            "N",
            "vim.repeat-search-previous",
            PluginKeymapScope::Workspace,
        ),
        // Registers
        visual_binding("\"", "vim.select-register", PluginKeymapScope::Workspace),
        visual_binding_commands(
            "Ctrl+d",
            &["vim.scroll-half-page-down", "vim.center-current-line"],
            PluginKeymapScope::Workspace,
        ),
        visual_binding_commands(
            "Ctrl+u",
            &["vim.scroll-half-page-up", "vim.center-current-line"],
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "Ctrl+e",
            "vim.scroll-line-down",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "Ctrl+f",
            "vim.scroll-page-down",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("Ctrl+b", "vim.scroll-page-up", PluginKeymapScope::Workspace),
        visual_binding("Ctrl+y", "vim.scroll-line-up", PluginKeymapScope::Workspace),
        visual_binding(
            "z Enter",
            "vim.current-line-top",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("z t", "vim.current-line-top", PluginKeymapScope::Workspace),
        visual_binding(
            "z .",
            "vim.center-current-line",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "z z",
            "vim.center-current-line",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "z -",
            "vim.current-line-bottom",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "z b",
            "vim.current-line-bottom",
            PluginKeymapScope::Workspace,
        ),
        // Deleting text
        visual_binding("d", "vim.visual-delete", PluginKeymapScope::Workspace),
        visual_binding("x", "vim.visual-delete", PluginKeymapScope::Workspace),
        // Copying and moving text
        visual_binding("y", "vim.visual-yank", PluginKeymapScope::Workspace),
        visual_binding("p", "vim.visual-put-after", PluginKeymapScope::Workspace),
        visual_binding("P", "vim.visual-put-before", PluginKeymapScope::Workspace),
        // Changing text
        visual_binding("c", "vim.visual-change", PluginKeymapScope::Workspace),
        visual_binding("s", "vim.visual-change", PluginKeymapScope::Workspace),
        visual_binding("r", "vim.visual-replace-char", PluginKeymapScope::Workspace),
        visual_binding("I", "vim.visual-block-insert", PluginKeymapScope::Workspace),
        visual_binding("A", "vim.visual-block-append", PluginKeymapScope::Workspace),
        visual_binding(">", "vim.visual-indent", PluginKeymapScope::Workspace),
        visual_binding("<", "vim.visual-outdent", PluginKeymapScope::Workspace),
        visual_binding("J", "vim.visual-move-down", PluginKeymapScope::Workspace),
        visual_binding("K", "vim.visual-move-up", PluginKeymapScope::Workspace),
        visual_binding("=", "vim.visual-format", PluginKeymapScope::Workspace),
        visual_binding("g J", "vim.visual-join", PluginKeymapScope::Workspace),
        visual_binding("g q", "vim.visual-format", PluginKeymapScope::Workspace),
        visual_binding("u", "vim.visual-lowercase", PluginKeymapScope::Workspace),
        visual_binding("g u", "vim.visual-lowercase", PluginKeymapScope::Workspace),
        visual_binding("U", "vim.visual-uppercase", PluginKeymapScope::Workspace),
        visual_binding("g U", "vim.visual-uppercase", PluginKeymapScope::Workspace),
        visual_binding("~", "vim.visual-toggle-case", PluginKeymapScope::Workspace),
        visual_binding(
            "g ~",
            "vim.visual-toggle-case",
            PluginKeymapScope::Workspace,
        ),
        // Text objects (only in Visual mode or after an operator)
        visual_binding(
            "i",
            "vim.start-visual-inner-text-object",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "a",
            "vim.start-visual-around-text-object",
            PluginKeymapScope::Workspace,
        ),
        // Leader bindings
        leader_binding("w", "buffer.save", PluginKeymapScope::Workspace),
        leader_binding("w n", "workspace.next", PluginKeymapScope::Workspace),
        leader_binding("w p", "workspace.previous", PluginKeymapScope::Workspace),
        leader_binding("w +", "workspace.mark", PluginKeymapScope::Workspace),
        leader_binding("w -", "workspace.unmark", PluginKeymapScope::Workspace),
        leader_binding("w m", "workspace.marks", PluginKeymapScope::Workspace),
        leader_binding("W", "workspace.save", PluginKeymapScope::Workspace),
        // acp
        leader_binding("a a", "acp.pick-client", PluginKeymapScope::Workspace),
        leader_binding("a n", "acp.new-session", PluginKeymapScope::Workspace),
        leader_binding("a p", "acp.pick-session", PluginKeymapScope::Workspace),
        // buffer
        leader_binding("b b", "picker.open-buffers", PluginKeymapScope::Workspace),
        leader_binding("d w", "pane.close", PluginKeymapScope::Workspace),
        leader_binding("d b", "buffer.close", PluginKeymapScope::Workspace),
        leader_binding("b k", "buffer.close-picker", PluginKeymapScope::Workspace),
        // Git
        leader_binding("g s", "git.status-open", PluginKeymapScope::Workspace),
        leader_binding(
            "f n",
            "picker.open-icon-fonts",
            PluginKeymapScope::Workspace,
        ),
        leader_binding("s d", "lsp.diagnostics", PluginKeymapScope::Workspace),
        leader_binding("s g", "workspace.search", PluginKeymapScope::Workspace),
        // Quickfix
        normal_binding("[ q", "quickfix.previous", PluginKeymapScope::Workspace),
        normal_binding("] q", "quickfix.next", PluginKeymapScope::Workspace),
        leader_binding("q o", "quickfix.open", PluginKeymapScope::Workspace),
        // Workspace
        leader_binding("p n", "workspace.new", PluginKeymapScope::Workspace),
        leader_binding("p s", "workspace.switch", PluginKeymapScope::Workspace),
        leader_binding("p d", "workspace.dashboard", PluginKeymapScope::Workspace),
        leader_binding("p k", "workspace.delete", PluginKeymapScope::Workspace),
        // Open
        leader_binding(
            "o p",
            "picker.toggle-popup-window",
            PluginKeymapScope::Workspace,
        ),
        leader_binding("o b", "browser.open", PluginKeymapScope::Workspace),
        leader_binding("o t", "terminal.open", PluginKeymapScope::Workspace),
        leader_binding("o T", "terminal.popup", PluginKeymapScope::Workspace),
        leader_binding("o u", "browser.url", PluginKeymapScope::Workspace),
        leader_binding("f w", "browser.open-buffer", PluginKeymapScope::Workspace),
        leader_binding("q m", "quickfix.toggle-mark", PluginKeymapScope::Popup),
    ];

    PluginPackage::new(
        "vim",
        true,
        "Modal bindings, motions, operators, and command ergonomics.",
    )
    .with_commands(commands)
    .with_key_bindings(key_bindings)
}

fn hook_command(name: &str, description: &str, hook_name: &str, detail: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, Some(detail))],
    )
}

fn vim_edit_command(name: &str, description: &str, action: VimActionSpec) -> PluginCommand {
    hook_command(name, description, "editor.vim.edit", action.hook_detail())
}

fn normal_binding(chord: &str, command_name: &str, scope: PluginKeymapScope) -> PluginKeyBinding {
    PluginKeyBinding::new(chord, command_name, scope).with_vim_mode(PluginVimMode::Normal)
}

fn normal_binding_commands(
    chord: &str,
    command_names: &[&str],
    scope: PluginKeymapScope,
) -> PluginKeyBinding {
    PluginKeyBinding::new_many(chord, command_names.iter().copied(), scope)
        .with_vim_mode(PluginVimMode::Normal)
}

fn leader_binding(chord: &str, command_name: &str, scope: PluginKeymapScope) -> PluginKeyBinding {
    PluginKeyBinding::new(format!("{LEADER_KEY} {chord}"), command_name, scope)
        .with_vim_mode(PluginVimMode::Normal)
}

fn visual_binding(chord: &str, command_name: &str, scope: PluginKeymapScope) -> PluginKeyBinding {
    PluginKeyBinding::new(chord, command_name, scope).with_vim_mode(PluginVimMode::Visual)
}

fn visual_binding_commands(
    chord: &str,
    command_names: &[&str],
    scope: PluginKeymapScope,
) -> PluginKeyBinding {
    PluginKeyBinding::new_many(chord, command_names.iter().copied(), scope)
        .with_vim_mode(PluginVimMode::Visual)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_vim_edit_actions_as_typed_details() {
        let package = package();
        let command = package
            .commands()
            .iter()
            .find(|command| command.name() == "vim.delete-char")
            .expect("delete char command");
        assert_eq!(
            command.actions()[0].hook().and_then(|hook| hook.detail()),
            Some(VimEditAction::DeleteChar.hook_detail())
        );
    }

    #[test]
    fn package_exports_lsp_navigation_bindings() {
        let package = package();
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == "g d"
                    && binding.command_name() == "lsp.definition")
        );
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == "g r"
                    && binding.command_name() == "lsp.references")
        );
        assert!(package.key_bindings().iter().any(
            |binding| binding.chord() == "g r r" && binding.command_name() == "lsp.references"
        ));
        assert!(
            package
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == "g i"
                    && binding.command_name() == "lsp.implementation")
        );
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Space s d"
                && binding.command_name() == "lsp.diagnostics"
                && binding.scope() == PluginKeymapScope::Workspace
                && binding.vim_mode() == PluginVimMode::Normal
        }));
    }

    #[test]
    fn package_exports_alt_x_for_command_line() {
        let package = package();
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Alt+x" && binding.command_name() == "picker.open-commands"
        }));
    }

    #[test]
    fn package_exports_quickfix_bindings() {
        let package = package();
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "[ q"
                && binding.command_name() == "quickfix.previous"
                && binding.scope() == PluginKeymapScope::Workspace
                && binding.vim_mode() == PluginVimMode::Normal
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "] q"
                && binding.command_name() == "quickfix.next"
                && binding.scope() == PluginKeymapScope::Workspace
                && binding.vim_mode() == PluginVimMode::Normal
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Space q o"
                && binding.command_name() == "quickfix.open"
                && binding.scope() == PluginKeymapScope::Workspace
                && binding.vim_mode() == PluginVimMode::Normal
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Space q m"
                && binding.command_name() == "quickfix.toggle-mark"
                && binding.scope() == PluginKeymapScope::Popup
                && binding.vim_mode() == PluginVimMode::Normal
        }));
    }

    #[test]
    fn package_exports_workspace_cycle_bindings() {
        let package = package();
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Space w n"
                && binding.command_name() == "workspace.next"
                && binding.scope() == PluginKeymapScope::Workspace
                && binding.vim_mode() == PluginVimMode::Normal
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Space w p"
                && binding.command_name() == "workspace.previous"
                && binding.scope() == PluginKeymapScope::Workspace
                && binding.vim_mode() == PluginVimMode::Normal
        }));
    }

    #[test]
    fn package_exports_mark_list_bindings() {
        let package = package();
        for (chord, command) in [
            ("Space w +", "workspace.mark"),
            ("Space w -", "workspace.unmark"),
            ("Space w m", "workspace.marks"),
        ] {
            assert!(package.key_bindings().iter().any(|binding| {
                binding.chord() == chord
                    && binding.command_name() == command
                    && binding.scope() == PluginKeymapScope::Workspace
                    && binding.vim_mode() == PluginVimMode::Normal
            }));
        }
    }

    #[test]
    fn package_exports_canonical_paging_bindings() {
        let package = package();
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+f"
                && binding.command_name() == "vim.scroll-page-down"
                && binding.vim_mode() == PluginVimMode::Normal
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+d"
                && binding.vim_mode() == PluginVimMode::Normal
                && binding
                    .command_names()
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    == vec!["vim.scroll-half-page-down", "vim.center-current-line"]
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+u"
                && binding.vim_mode() == PluginVimMode::Normal
                && binding
                    .command_names()
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    == vec!["vim.scroll-half-page-up", "vim.center-current-line"]
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+b"
                && binding.command_name() == "vim.scroll-page-up"
                && binding.vim_mode() == PluginVimMode::Normal
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+f"
                && binding.command_name() == "vim.scroll-page-down"
                && binding.vim_mode() == PluginVimMode::Visual
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+d"
                && binding.vim_mode() == PluginVimMode::Visual
                && binding
                    .command_names()
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    == vec!["vim.scroll-half-page-down", "vim.center-current-line"]
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+u"
                && binding.vim_mode() == PluginVimMode::Visual
                && binding
                    .command_names()
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>()
                    == vec!["vim.scroll-half-page-up", "vim.center-current-line"]
        }));
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Ctrl+b"
                && binding.command_name() == "vim.scroll-page-up"
                && binding.vim_mode() == PluginVimMode::Visual
        }));
        for (chord, command_name, vim_mode) in [
            ("z Enter", "vim.current-line-top", PluginVimMode::Normal),
            ("z t", "vim.current-line-top", PluginVimMode::Normal),
            ("z .", "vim.center-current-line", PluginVimMode::Normal),
            ("z z", "vim.center-current-line", PluginVimMode::Normal),
            ("z -", "vim.current-line-bottom", PluginVimMode::Normal),
            ("z b", "vim.current-line-bottom", PluginVimMode::Normal),
            ("z Enter", "vim.current-line-top", PluginVimMode::Visual),
            ("z t", "vim.current-line-top", PluginVimMode::Visual),
            ("z .", "vim.center-current-line", PluginVimMode::Visual),
            ("z z", "vim.center-current-line", PluginVimMode::Visual),
            ("z -", "vim.current-line-bottom", PluginVimMode::Visual),
            ("z b", "vim.current-line-bottom", PluginVimMode::Visual),
        ] {
            assert!(package.key_bindings().iter().any(|binding| {
                binding.chord() == chord
                    && binding.command_name() == command_name
                    && binding.vim_mode() == vim_mode
            }));
        }
    }

    #[test]
    fn package_exports_g_prefix_aliases_for_format_and_case() {
        let package = package();
        for (chord, command_name, vim_mode) in [
            ("g c", "vim.visual-toggle-comment", PluginVimMode::Visual),
            ("g J", "vim.visual-join", PluginVimMode::Visual),
            ("g q", "vim.start-format-operator", PluginVimMode::Normal),
            ("g q", "vim.visual-format", PluginVimMode::Visual),
            ("g u", "vim.visual-lowercase", PluginVimMode::Visual),
            ("g U", "vim.visual-uppercase", PluginVimMode::Visual),
            ("g ~", "vim.visual-toggle-case", PluginVimMode::Visual),
        ] {
            assert!(package.key_bindings().iter().any(|binding| {
                binding.chord() == chord
                    && binding.command_name() == command_name
                    && binding.vim_mode() == vim_mode
            }));
        }
    }

    #[test]
    fn package_exports_visual_put_bindings() {
        let package = package();
        for (chord, command_name) in [
            ("p", "vim.visual-put-after"),
            ("P", "vim.visual-put-before"),
            ("r", "vim.visual-replace-char"),
            (">", "vim.visual-indent"),
            ("<", "vim.visual-outdent"),
            ("J", "vim.visual-move-down"),
            ("K", "vim.visual-move-up"),
        ] {
            assert!(package.key_bindings().iter().any(|binding| {
                binding.chord() == chord
                    && binding.command_name() == command_name
                    && binding.vim_mode() == PluginVimMode::Visual
            }));
        }
    }

    #[test]
    fn package_exports_command_line_alias_commands() {
        let package = package();
        let names = package
            .commands()
            .iter()
            .map(|command| command.name())
            .collect::<Vec<_>>();
        for name in ["q", "write", "wq", "split", "vsplit", "commands", "term"] {
            assert!(names.contains(&name), "missing command-line alias `{name}`");
        }
    }
}
