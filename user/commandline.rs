use editor_plugin_api::{PluginAction, PluginCommand};

/// Returns whether the Vim-style command line is enabled.
pub const fn enabled() -> bool {
    true
}

/// Returns the user-facing Ex-style command aliases exposed through `:`.
pub fn commands() -> Vec<PluginCommand> {
    let mut commands = Vec::new();
    commands.extend(hook_aliases(
        &["q", "quit"],
        "Closes the currently focused split.",
        "ui.pane.close",
        None,
    ));
    commands.extend(hook_aliases(
        &["w", "write"],
        "Saves the active file-backed buffer.",
        "buffer.save",
        None,
    ));
    commands.extend(hook_aliases(
        &["wa", "wall"],
        "Saves all modified file buffers in the active workspace.",
        "workspace.save",
        None,
    ));
    commands.extend(action_aliases(
        &["wq", "x", "xit"],
        "Saves the active buffer and closes the currently focused split.",
        &[
            PluginAction::emit_hook("buffer.save", None::<&str>),
            PluginAction::emit_hook("ui.pane.close", None::<&str>),
        ],
    ));
    commands.extend(hook_aliases(
        &["bd", "bdelete"],
        "Closes the active buffer.",
        "buffer.close",
        None,
    ));
    commands.extend(picker_aliases(
        &["b", "buffer", "ls", "buffers"],
        "Opens the buffer picker popup.",
        "buffers",
    ));
    commands.extend(picker_aliases(
        &["e", "edit", "files", "find"],
        "Lists the current workspace files that are visible to Git.",
        "workspace.files",
    ));
    commands.extend(picker_aliases(
        &["projects", "project"],
        "Creates or focuses a workspace from the project picker.",
        "workspace.projects",
    ));
    commands.extend(picker_aliases(
        &["search"],
        "Searches text across files in the active workspace.",
        "workspace.search",
    ));
    commands.extend(picker_aliases(
        &["commands"],
        "Opens the command picker popup.",
        "commands",
    ));
    commands.extend(hook_aliases(
        &["split", "sp"],
        "Splits the active workspace horizontally.",
        "ui.pane.split-horizontal",
        None,
    ));
    commands.extend(hook_aliases(
        &["vsplit", "vs"],
        "Splits the active workspace vertically.",
        "ui.pane.split-vertical",
        None,
    ));
    commands.extend(hook_aliases(
        &["format"],
        "Formats the active file buffer.",
        "workspace.format",
        None,
    ));
    commands.extend(action_aliases(
        &["term", "terminal"],
        "Opens a popup-hosted terminal buffer.",
        &[PluginAction::open_buffer(
            "*terminal-popup*",
            "terminal",
            Some("Terminal"),
        )],
    ));
    commands
}

fn hook_aliases(
    names: &[&str],
    description: &str,
    hook_name: &str,
    detail: Option<&str>,
) -> Vec<PluginCommand> {
    action_aliases(
        names,
        description,
        &[PluginAction::emit_hook(hook_name, detail)],
    )
}

fn picker_aliases(names: &[&str], description: &str, provider: &str) -> Vec<PluginCommand> {
    hook_aliases(names, description, "ui.picker.open", Some(provider))
}

fn action_aliases(
    names: &[&str],
    description: &str,
    actions: &[PluginAction],
) -> Vec<PluginCommand> {
    names
        .iter()
        .map(|name| PluginCommand::new(*name, description, actions.to_vec()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{commands, enabled};
    use std::collections::BTreeSet;

    #[test]
    fn command_line_exports_commands_when_enabled() {
        if enabled() {
            assert!(!commands().is_empty());
        }
    }

    #[test]
    fn command_line_commands_have_unique_names() {
        let commands = commands();
        let mut names = BTreeSet::new();
        for command in &commands {
            assert!(!command.name().is_empty());
            assert!(
                names.insert(command.name().to_owned()),
                "duplicate command-line alias `{}`",
                command.name()
            );
            assert!(
                !command.actions().is_empty(),
                "command-line alias `{}` should do something",
                command.name()
            );
        }
    }
}
