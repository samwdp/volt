use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    buffer_kinds, issues_hooks,
};

pub const PACKAGE_NAME: &str = "issues";
pub const BOARD_KIND: &str = buffer_kinds::ISSUES_BOARD;
pub const BOARD_BUFFER_NAME: &str = "*issues-board*";

fn hook_command(name: &str, description: &str, hook: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook, None::<&str>)],
    )
}

fn hook_command_detail(name: &str, description: &str, hook: &str, detail: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook, Some(detail))],
    )
}

/// Returns the metadata for workspace Issues commands.
pub fn package() -> PluginPackage {
    let commands = vec![
        hook_command(
            "issues.open-board",
            "Opens the Issue Board listing active Issues.",
            issues_hooks::BOARD_OPEN,
        ),
        hook_command(
            "issues.create",
            "Creates an Issue from a title prompt (no Code Reference required).",
            issues_hooks::CREATE,
        ),
        hook_command(
            "issues.scan",
            "Runs Issue Scan over the workspace tree without blocking the UI.",
            issues_hooks::SCAN,
        ),
        hook_command(
            "issues.capture-focused",
            "Captures unlinked TODO/FIXME comments in the focused file.",
            issues_hooks::CAPTURE_FOCUSED,
        ),
        hook_command(
            "issues.activate-line",
            "Opens the Issue under the Issue Board cursor.",
            issues_hooks::ACTIVATE_LINE,
        ),
        hook_command(
            "issues.board-toggle-closed",
            "Toggles showing Closed Issues on the Issue Board.",
            issues_hooks::TOGGLE_CLOSED,
        ),
        hook_command_detail(
            "issues.set-status.open",
            "Sets the selected Issue Status to Open.",
            issues_hooks::SET_STATUS,
            "Open",
        ),
        hook_command_detail(
            "issues.set-status.planning",
            "Sets the selected Issue Status to Planning.",
            issues_hooks::SET_STATUS,
            "Planning",
        ),
        hook_command_detail(
            "issues.set-status.in-progress",
            "Sets the selected Issue Status to In Progress.",
            issues_hooks::SET_STATUS,
            "In Progress",
        ),
        hook_command_detail(
            "issues.set-status.closed",
            "Sets the selected Issue Status to Closed.",
            issues_hooks::SET_STATUS,
            "Closed",
        ),
        hook_command(
            "issues.place",
            "Places a Code Reference for the selected Issue at the cursor.",
            issues_hooks::PLACE,
        ),
        hook_command(
            "issues.open-from-ref",
            "Opens the Issue linked by the Code Reference under the cursor.",
            issues_hooks::OPEN_FROM_REF,
        ),
        hook_command(
            "issues.jump-refs",
            "Jumps to Code References for the selected Issue.",
            issues_hooks::JUMP_REFS,
        ),
    ];

    let key_bindings = vec![
        PluginKeyBinding::new(
            "Space i b",
            "issues.open-board",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Space i c", "issues.create", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Space i s", "issues.scan", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            "Space i o",
            "issues.activate-line",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            "Space i f",
            "issues.open-from-ref",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new(
            "Space i j",
            "issues.jump-refs",
            PluginKeymapScope::Workspace,
        )
        .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Space i p", "issues.place", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
    ];

    PluginPackage::new(
        PACKAGE_NAME,
        true,
        "Workspace Issue Store, Board, Capture, Place, and Issue Scan.",
    )
    .with_commands(commands)
    .with_key_bindings(key_bindings)
}
