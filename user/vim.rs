use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
};

// Change this to customize the leader key for Vim bindings.
const LEADER_KEY: &str = "Space";

/// Returns the metadata for the Vim bindings package.
pub fn package() -> PluginPackage {
    let commands = vec![
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
        hook_command(
            "vim.enter-visual-mode",
            "Switches the editor into Vim visual mode.",
            "editor.vim.edit",
            "enter-visual",
        ),
        hook_command(
            "vim.enter-visual-line-mode",
            "Switches the editor into Vim linewise visual mode.",
            "editor.vim.edit",
            "enter-visual-line",
        ),
        hook_command(
            "vim.enter-visual-block-mode",
            "Switches the editor into Vim blockwise visual mode.",
            "editor.vim.edit",
            "enter-visual-block",
        ),
        hook_command(
            "vim.delete-char",
            "Deletes the character under the cursor.",
            "editor.vim.edit",
            "delete-char",
        ),
        hook_command(
            "vim.delete-char-before",
            "Deletes the character before the cursor.",
            "editor.vim.edit",
            "delete-char-before",
        ),
        hook_command(
            "vim.delete-line-end",
            "Deletes from the cursor to the end of the line.",
            "editor.vim.edit",
            "delete-line-end",
        ),
        hook_command(
            "vim.change-line-end",
            "Changes from the cursor to the end of the line.",
            "editor.vim.edit",
            "change-line-end",
        ),
        hook_command(
            "vim.yank-line",
            "Yanks the current line.",
            "editor.vim.edit",
            "yank-line",
        ),
        hook_command(
            "vim.substitute-char",
            "Substitutes characters under the cursor and enters insert mode.",
            "editor.vim.edit",
            "substitute-char",
        ),
        hook_command(
            "vim.substitute-line",
            "Substitutes the current line and enters insert mode.",
            "editor.vim.edit",
            "substitute-line",
        ),
        hook_command(
            "vim.replace-char",
            "Replaces characters under the cursor without entering insert mode.",
            "editor.vim.edit",
            "replace-char",
        ),
        hook_command(
            "vim.enter-replace-mode",
            "Enters Vim replace mode.",
            "editor.vim.edit",
            "enter-replace-mode",
        ),
        hook_command(
            "vim.toggle-case",
            "Toggles the case of characters under the cursor.",
            "editor.vim.edit",
            "toggle-case",
        ),
        hook_command(
            "vim.start-delete-operator",
            "Starts a Vim delete operator-pending command.",
            "editor.vim.edit",
            "start-delete-operator",
        ),
        hook_command(
            "vim.start-change-operator",
            "Starts a Vim change operator-pending command.",
            "editor.vim.edit",
            "start-change-operator",
        ),
        hook_command(
            "vim.start-yank-operator",
            "Starts a Vim yank operator-pending command.",
            "editor.vim.edit",
            "start-yank-operator",
        ),
        hook_command(
            "vim.start-format-operator",
            "Starts a Vim format operator.",
            "editor.vim.edit",
            "start-format-operator",
        ),
        hook_command(
            "vim.visual-format",
            "Formats the current visual selection.",
            "editor.vim.edit",
            "visual-format",
        ),
        hook_command(
            "vim.append-after-cursor",
            "Appends after the cursor and enters insert mode.",
            "editor.vim.edit",
            "append",
        ),
        hook_command(
            "vim.append-line-end",
            "Appends at the end of the line and enters insert mode.",
            "editor.vim.edit",
            "append-line-end",
        ),
        hook_command(
            "vim.insert-line-start",
            "Inserts at the first non-blank character on the line.",
            "editor.vim.edit",
            "insert-line-start",
        ),
        hook_command(
            "vim.open-line-below",
            "Opens a new line below and enters insert mode.",
            "editor.vim.edit",
            "open-line-below",
        ),
        hook_command(
            "vim.open-line-above",
            "Opens a new line above and enters insert mode.",
            "editor.vim.edit",
            "open-line-above",
        ),
        hook_command(
            "vim.undo",
            "Undoes the previous change.",
            "editor.vim.edit",
            "undo",
        ),
        hook_command(
            "vim.redo",
            "Redoes the next change.",
            "editor.vim.edit",
            "redo",
        ),
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
        hook_command(
            "vim.start-g-prefix",
            "Starts a Vim g-prefixed motion.",
            "editor.vim.edit",
            "start-g-prefix",
        ),
        hook_command(
            "vim.start-find-forward",
            "Starts a Vim f motion on the current line.",
            "editor.vim.edit",
            "start-find-forward",
        ),
        hook_command(
            "vim.start-find-backward",
            "Starts a Vim F motion on the current line.",
            "editor.vim.edit",
            "start-find-backward",
        ),
        hook_command(
            "vim.start-till-forward",
            "Starts a Vim t motion on the current line.",
            "editor.vim.edit",
            "start-till-forward",
        ),
        hook_command(
            "vim.start-till-backward",
            "Starts a Vim T motion on the current line.",
            "editor.vim.edit",
            "start-till-backward",
        ),
        hook_command(
            "vim.repeat-find-next",
            "Repeats the last Vim find motion forward.",
            "editor.vim.edit",
            "repeat-find-next",
        ),
        hook_command(
            "vim.repeat-find-previous",
            "Repeats the last Vim find motion backward.",
            "editor.vim.edit",
            "repeat-find-previous",
        ),
        hook_command(
            "vim.start-search-forward",
            "Opens a Vim-style forward search prompt.",
            "editor.vim.edit",
            "start-search-forward",
        ),
        hook_command(
            "vim.start-search-backward",
            "Opens a Vim-style backward search prompt.",
            "editor.vim.edit",
            "start-search-backward",
        ),
        hook_command(
            "vim.search-word-forward",
            "Searches forward for the word under the cursor.",
            "editor.vim.edit",
            "search-word-forward",
        ),
        hook_command(
            "vim.search-word-backward",
            "Searches backward for the word under the cursor.",
            "editor.vim.edit",
            "search-word-backward",
        ),
        hook_command(
            "vim.repeat-search-next",
            "Repeats the last Vim search in the same direction.",
            "editor.vim.edit",
            "repeat-search-next",
        ),
        hook_command(
            "vim.repeat-search-previous",
            "Repeats the last Vim search in the opposite direction.",
            "editor.vim.edit",
            "repeat-search-previous",
        ),
        hook_command(
            "vim.select-register",
            "Selects a Vim register for the next operation.",
            "editor.vim.edit",
            "select-register",
        ),
        hook_command(
            "vim.set-mark",
            "Sets a Vim mark at the current cursor.",
            "editor.vim.edit",
            "set-mark",
        ),
        hook_command(
            "vim.goto-mark-line",
            "Jumps to the line of a Vim mark.",
            "editor.vim.edit",
            "goto-mark-line",
        ),
        hook_command(
            "vim.goto-mark",
            "Jumps to the exact position of a Vim mark.",
            "editor.vim.edit",
            "goto-mark",
        ),
        hook_command(
            "vim.toggle-macro-record",
            "Starts or stops Vim macro recording.",
            "editor.vim.edit",
            "toggle-macro-record",
        ),
        hook_command(
            "vim.start-macro-playback",
            "Plays back a recorded Vim macro.",
            "editor.vim.edit",
            "start-macro-playback",
        ),
        hook_command(
            "vim.put-after",
            "Puts the most recent Vim yank after the cursor.",
            "editor.vim.edit",
            "put-after",
        ),
        hook_command(
            "vim.put-before",
            "Puts the most recent Vim yank before the cursor.",
            "editor.vim.edit",
            "put-before",
        ),
        hook_command(
            "vim.visual-delete",
            "Deletes the current visual selection.",
            "editor.vim.edit",
            "visual-delete",
        ),
        hook_command(
            "vim.visual-change",
            "Changes the current visual selection.",
            "editor.vim.edit",
            "visual-change",
        ),
        hook_command(
            "vim.visual-block-insert",
            "Inserts before the visual block selection and enters insert mode.",
            "editor.vim.edit",
            "visual-block-insert",
        ),
        hook_command(
            "vim.visual-block-append",
            "Appends after the visual block selection and enters insert mode.",
            "editor.vim.edit",
            "visual-block-append",
        ),
        hook_command(
            "vim.visual-yank",
            "Yanks the current visual selection.",
            "editor.vim.edit",
            "visual-yank",
        ),
        hook_command(
            "vim.visual-toggle-case",
            "Toggles the case of the current visual selection.",
            "editor.vim.edit",
            "visual-toggle-case",
        ),
        hook_command(
            "vim.visual-lowercase",
            "Lowercases the current visual selection.",
            "editor.vim.edit",
            "visual-lowercase",
        ),
        hook_command(
            "vim.visual-uppercase",
            "Uppercases the current visual selection.",
            "editor.vim.edit",
            "visual-uppercase",
        ),
        hook_command(
            "vim.visual-swap-anchor",
            "Swaps the active and anchor ends of the current visual selection.",
            "editor.vim.edit",
            "visual-swap-anchor",
        ),
        hook_command(
            "vim.start-visual-inner-text-object",
            "Starts a visual-mode inner text object selection.",
            "editor.vim.edit",
            "start-visual-inner-text-object",
        ),
        hook_command(
            "vim.start-visual-around-text-object",
            "Starts a visual-mode around text object selection.",
            "editor.vim.edit",
            "start-visual-around-text-object",
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
        normal_binding("g r r", "lsp.references", PluginKeymapScope::Workspace),
        normal_binding("g i", "lsp.implementation", PluginKeymapScope::Workspace),
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
        normal_binding("-", "oil.open-parent", PluginKeymapScope::Workspace),
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
        normal_binding("Ctrl+b", "vim.scroll-page-up", PluginKeymapScope::Workspace),
        normal_binding(
            "Ctrl+d",
            "vim.scroll-half-page-down",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("Ctrl+r", "vim.redo", PluginKeymapScope::Workspace),
        normal_binding(
            "Ctrl+u",
            "vim.scroll-half-page-up",
            PluginKeymapScope::Workspace,
        ),
        normal_binding(
            "Ctrl+e",
            "vim.scroll-line-down",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("Ctrl+f", "workspace.new", PluginKeymapScope::Workspace),
        normal_binding("Ctrl+y", "vim.scroll-line-up", PluginKeymapScope::Workspace),
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
        visual_binding("Ctrl+b", "vim.scroll-page-up", PluginKeymapScope::Workspace),
        visual_binding(
            "Ctrl+d",
            "vim.scroll-half-page-down",
            PluginKeymapScope::Workspace,
        ),
        visual_binding(
            "Ctrl+u",
            "vim.scroll-half-page-up",
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
        visual_binding("Ctrl+y", "vim.scroll-line-up", PluginKeymapScope::Workspace),
        // Deleting text
        visual_binding("d", "vim.visual-delete", PluginKeymapScope::Workspace),
        visual_binding("x", "vim.visual-delete", PluginKeymapScope::Workspace),
        // Copying and moving text
        visual_binding("y", "vim.visual-yank", PluginKeymapScope::Workspace),
        // Changing text
        visual_binding("c", "vim.visual-change", PluginKeymapScope::Workspace),
        visual_binding("I", "vim.visual-block-insert", PluginKeymapScope::Workspace),
        visual_binding("A", "vim.visual-block-append", PluginKeymapScope::Workspace),
        visual_binding("=", "vim.visual-format", PluginKeymapScope::Workspace),
        visual_binding("u", "vim.visual-lowercase", PluginKeymapScope::Workspace),
        visual_binding("U", "vim.visual-uppercase", PluginKeymapScope::Workspace),
        visual_binding("~", "vim.visual-toggle-case", PluginKeymapScope::Workspace),
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
        leader_binding("s g", "workspace.search", PluginKeymapScope::Workspace),
        // Workspace
        leader_binding("p s", "workspace.switch", PluginKeymapScope::Workspace),
        leader_binding("p d", "workspace.delete", PluginKeymapScope::Workspace),
        // Open
        leader_binding(
            "o p",
            "picker.toggle-popup-window",
            PluginKeymapScope::Workspace,
        ),
        leader_binding("o b", "browser.open", PluginKeymapScope::Workspace),
        leader_binding("o t", "terminal.popup", PluginKeymapScope::Workspace),
        leader_binding("o u", "browser.url", PluginKeymapScope::Workspace),
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

fn normal_binding(chord: &str, command_name: &str, scope: PluginKeymapScope) -> PluginKeyBinding {
    PluginKeyBinding::new(chord, command_name, scope).with_vim_mode(PluginVimMode::Normal)
}

fn leader_binding(chord: &str, command_name: &str, scope: PluginKeymapScope) -> PluginKeyBinding {
    PluginKeyBinding::new(format!("{LEADER_KEY} {chord}"), command_name, scope)
        .with_vim_mode(PluginVimMode::Normal)
}

fn visual_binding(chord: &str, command_name: &str, scope: PluginKeymapScope) -> PluginKeyBinding {
    PluginKeyBinding::new(chord, command_name, scope).with_vim_mode(PluginVimMode::Visual)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    }

    #[test]
    fn package_exports_alt_x_for_command_line() {
        let package = package();
        assert!(package.key_bindings().iter().any(|binding| {
            binding.chord() == "Alt+x" && binding.command_name() == "picker.open-commands"
        }));
    }
}
