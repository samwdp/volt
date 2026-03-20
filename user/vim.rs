use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
};

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
            "vim.delete-char",
            "Deletes the character under the cursor.",
            "editor.vim.edit",
            "delete-char",
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
            "vim.visual-yank",
            "Yanks the current visual selection.",
            "editor.vim.edit",
            "visual-yank",
        ),
        PluginCommand::new(
            "vim.command-line",
            "Opens a Vim-style command picker.",
            vec![PluginAction::emit_hook("ui.picker.open", Some("commands"))],
        ),
    ];

    let key_bindings = vec![
        normal_binding("h", "vim.move-left", PluginKeymapScope::Workspace),
        normal_binding("j", "vim.move-down", PluginKeymapScope::Workspace),
        normal_binding("k", "vim.move-up", PluginKeymapScope::Workspace),
        normal_binding("l", "vim.move-right", PluginKeymapScope::Workspace),
        normal_binding("w", "vim.move-word-forward", PluginKeymapScope::Workspace),
        normal_binding("b", "vim.move-word-backward", PluginKeymapScope::Workspace),
        normal_binding("e", "vim.move-word-end", PluginKeymapScope::Workspace),
        normal_binding("0", "vim.move-line-start", PluginKeymapScope::Workspace),
        normal_binding(
            "^",
            "vim.move-line-first-non-blank",
            PluginKeymapScope::Workspace,
        ),
        normal_binding("$", "vim.move-line-end", PluginKeymapScope::Workspace),
        normal_binding("G", "vim.goto-last-line", PluginKeymapScope::Workspace),
        normal_binding("g", "vim.start-g-prefix", PluginKeymapScope::Workspace),
        normal_binding("f", "vim.start-find-forward", PluginKeymapScope::Workspace),
        normal_binding("F", "vim.start-find-backward", PluginKeymapScope::Workspace),
        normal_binding("t", "vim.start-till-forward", PluginKeymapScope::Workspace),
        normal_binding("T", "vim.start-till-backward", PluginKeymapScope::Workspace),
        normal_binding(";", "vim.repeat-find-next", PluginKeymapScope::Workspace),
        normal_binding(",", "vim.repeat-find-previous", PluginKeymapScope::Workspace),
        normal_binding("i", "vim.enter-insert-mode", PluginKeymapScope::Workspace),
        normal_binding("v", "vim.enter-visual-mode", PluginKeymapScope::Workspace),
        normal_binding("d", "vim.start-delete-operator", PluginKeymapScope::Workspace),
        normal_binding("c", "vim.start-change-operator", PluginKeymapScope::Workspace),
        normal_binding("y", "vim.start-yank-operator", PluginKeymapScope::Workspace),
        normal_binding("a", "vim.append-after-cursor", PluginKeymapScope::Workspace),
        normal_binding("A", "vim.append-line-end", PluginKeymapScope::Workspace),
        normal_binding("I", "vim.insert-line-start", PluginKeymapScope::Workspace),
        normal_binding("o", "vim.open-line-below", PluginKeymapScope::Workspace),
        normal_binding("O", "vim.open-line-above", PluginKeymapScope::Workspace),
        normal_binding("x", "vim.delete-char", PluginKeymapScope::Workspace),
        normal_binding("p", "vim.put-after", PluginKeymapScope::Workspace),
        normal_binding("P", "vim.put-before", PluginKeymapScope::Workspace),
        normal_binding("u", "vim.undo", PluginKeymapScope::Workspace),
        normal_binding(":", "vim.command-line", PluginKeymapScope::Workspace),
        visual_binding("h", "vim.move-left", PluginKeymapScope::Workspace),
        visual_binding("j", "vim.move-down", PluginKeymapScope::Workspace),
        visual_binding("k", "vim.move-up", PluginKeymapScope::Workspace),
        visual_binding("l", "vim.move-right", PluginKeymapScope::Workspace),
        visual_binding("w", "vim.move-word-forward", PluginKeymapScope::Workspace),
        visual_binding("b", "vim.move-word-backward", PluginKeymapScope::Workspace),
        visual_binding("e", "vim.move-word-end", PluginKeymapScope::Workspace),
        visual_binding("0", "vim.move-line-start", PluginKeymapScope::Workspace),
        visual_binding(
            "^",
            "vim.move-line-first-non-blank",
            PluginKeymapScope::Workspace,
        ),
        visual_binding("$", "vim.move-line-end", PluginKeymapScope::Workspace),
        visual_binding("G", "vim.goto-last-line", PluginKeymapScope::Workspace),
        visual_binding("g", "vim.start-g-prefix", PluginKeymapScope::Workspace),
        visual_binding("f", "vim.start-find-forward", PluginKeymapScope::Workspace),
        visual_binding("F", "vim.start-find-backward", PluginKeymapScope::Workspace),
        visual_binding("t", "vim.start-till-forward", PluginKeymapScope::Workspace),
        visual_binding("T", "vim.start-till-backward", PluginKeymapScope::Workspace),
        visual_binding(";", "vim.repeat-find-next", PluginKeymapScope::Workspace),
        visual_binding(",", "vim.repeat-find-previous", PluginKeymapScope::Workspace),
        visual_binding("d", "vim.visual-delete", PluginKeymapScope::Workspace),
        visual_binding("x", "vim.visual-delete", PluginKeymapScope::Workspace),
        visual_binding("c", "vim.visual-change", PluginKeymapScope::Workspace),
        visual_binding("y", "vim.visual-yank", PluginKeymapScope::Workspace),
        PluginKeyBinding::new("Escape", "vim.enter-normal-mode", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Insert),
        PluginKeyBinding::new("Escape", "vim.enter-normal-mode", PluginKeymapScope::Global)
            .with_vim_mode(PluginVimMode::Visual),
        normal_binding("Ctrl+r", "vim.redo", PluginKeymapScope::Workspace),
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

fn visual_binding(chord: &str, command_name: &str, scope: PluginKeymapScope) -> PluginKeyBinding {
    PluginKeyBinding::new(chord, command_name, scope).with_vim_mode(PluginVimMode::Visual)
}
