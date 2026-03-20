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
            "vim.delete-char",
            "Deletes the character under the cursor.",
            "editor.vim.edit",
            "delete-char",
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
        normal_binding("i", "vim.enter-insert-mode", PluginKeymapScope::Workspace),
        normal_binding("a", "vim.append-after-cursor", PluginKeymapScope::Workspace),
        normal_binding("A", "vim.append-line-end", PluginKeymapScope::Workspace),
        normal_binding("I", "vim.insert-line-start", PluginKeymapScope::Workspace),
        normal_binding("o", "vim.open-line-below", PluginKeymapScope::Workspace),
        normal_binding("O", "vim.open-line-above", PluginKeymapScope::Workspace),
        normal_binding("x", "vim.delete-char", PluginKeymapScope::Workspace),
        normal_binding("u", "vim.undo", PluginKeymapScope::Workspace),
        normal_binding(":", "vim.command-line", PluginKeymapScope::Workspace),
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
