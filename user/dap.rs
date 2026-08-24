use editor_plugin_api::{
    DebugAdapterRootStrategy, DebugAdapterSpec, DebugAdapterTransport, PluginAction, PluginBuffer,
    PluginBufferLayout, PluginBufferLayoutNode, PluginBufferSection, PluginBufferSections,
    PluginCommand, PluginHookDeclaration, PluginKeyBinding, PluginKeymapScope, PluginPackage,
    PluginVimMode, buffer_kinds, dap_hooks,
};

pub const HOOK_DAP_START: &str = dap_hooks::START;
pub const HOOK_DAP_START_LAST: &str = dap_hooks::START_LAST;
pub const HOOK_DAP_START_RECENT: &str = dap_hooks::START_RECENT;
pub const HOOK_DAP_STOP: &str = dap_hooks::STOP;
pub const HOOK_DAP_RESTART: &str = dap_hooks::RESTART;
pub const HOOK_DAP_CONTINUE: &str = dap_hooks::CONTINUE;
pub const HOOK_DAP_PAUSE: &str = dap_hooks::PAUSE;
pub const HOOK_DAP_STEP: &str = dap_hooks::STEP;
pub const HOOK_DAP_STEP_INTO: &str = dap_hooks::STEP_INTO;
pub const HOOK_DAP_STEP_OUT: &str = dap_hooks::STEP_OUT;
pub const HOOK_DAP_LOG: &str = dap_hooks::LOG;
pub const HOOK_DAP_TOGGLE_BREAKPOINT: &str = dap_hooks::TOGGLE_BREAKPOINT;
pub const HOOK_DAP_DELETE_BREAKPOINT: &str = dap_hooks::DELETE_BREAKPOINT;
pub const HOOK_DAP_OPEN_BREAKPOINTS: &str = dap_hooks::OPEN_BREAKPOINTS;
pub const HOOK_DAP_EXPRESSIONS_ADD: &str = dap_hooks::EXPRESSIONS_ADD;
pub const HOOK_DAP_EXPRESSIONS_REMOVE: &str = dap_hooks::EXPRESSIONS_REMOVE;
pub const HOOK_DAP_EVAL: &str = dap_hooks::EVAL;
pub const HOOK_DAP_EVAL_AT_POINT: &str = dap_hooks::EVAL_AT_POINT;
pub const HOOK_DAP_REPL: &str = dap_hooks::REPL;
pub const HOOK_DAP_SWITCH_THREAD: &str = dap_hooks::SWITCH_THREAD;
pub const HOOK_DAP_SWITCH_STACK_FRAME: &str = dap_hooks::SWITCH_STACK_FRAME;
pub const HOOK_DAP_BREAKPOINT_CONDITION: &str = dap_hooks::BREAKPOINT_CONDITION;
pub const HOOK_DAP_BREAKPOINT_HIT_CONDITION: &str = dap_hooks::BREAKPOINT_HIT_CONDITION;
pub const HOOK_DAP_BREAKPOINT_LOG_MESSAGE: &str = dap_hooks::BREAKPOINT_LOG_MESSAGE;
pub const HOOK_DAP_TOGGLE_VARIABLE: &str = dap_hooks::TOGGLE_VARIABLE;
pub const HOOK_DAP_GOTO_BREAKPOINT: &str = dap_hooks::GOTO_BREAKPOINT;

pub const BREAKPOINTS_KIND: &str = buffer_kinds::DAP_BREAKPOINTS;
pub const LOCALS_KIND: &str = buffer_kinds::DAP_LOCALS;
pub const REPL_KIND: &str = buffer_kinds::DAP_REPL;

pub const LOCALS_SECTION: &str = "Locals";
pub const EXPRESSIONS_SECTION: &str = "Expressions";

/// Returns the metadata for the DAP integration package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "dap",
        true,
        "Debug adapter integration and Rust debugging session defaults.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "dap.start",
            "Starts a Workspace Debug Session using the preferred adapter for the active file.",
            vec![PluginAction::emit_hook(HOOK_DAP_START, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.stop",
            "Stops the live Debug Session for the active Workspace.",
            vec![PluginAction::emit_hook(HOOK_DAP_STOP, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.restart",
            "Restarts the live Debug Session with the same Debug Configuration.",
            vec![PluginAction::emit_hook(HOOK_DAP_RESTART, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.continue",
            "Continues execution of the stopped Debug Session thread.",
            vec![PluginAction::emit_hook(HOOK_DAP_CONTINUE, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.pause",
            "Pauses the live Debug Session.",
            vec![PluginAction::emit_hook(HOOK_DAP_PAUSE, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.step",
            "Steps over the current line in the Debug Session.",
            vec![PluginAction::emit_hook(HOOK_DAP_STEP, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.step-into",
            "Steps into the current call in the Debug Session.",
            vec![PluginAction::emit_hook(HOOK_DAP_STEP_INTO, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.step-out",
            "Steps out of the current function in the Debug Session.",
            vec![PluginAction::emit_hook(HOOK_DAP_STEP_OUT, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.log",
            "Opens the DAP transport log buffer.",
            vec![PluginAction::emit_hook(HOOK_DAP_LOG, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.toggle-breakpoint",
            "Toggles a Breakpoint on the current line.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_TOGGLE_BREAKPOINT,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.delete-breakpoint",
            "Deletes a Breakpoint on the current line.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_DELETE_BREAKPOINT,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.breakpoints",
            "Opens the Workspace Breakpoints list.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_OPEN_BREAKPOINTS,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.expressions-add",
            "Adds a Watch Expression to the Expressions section.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_EXPRESSIONS_ADD,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.expressions-remove",
            "Removes a Watch Expression from the Expressions section.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_EXPRESSIONS_REMOVE,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.eval",
            "Evaluates an expression once without adding a Watch.",
            vec![PluginAction::emit_hook(HOOK_DAP_EVAL, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.eval-at-point",
            "Evaluates the identifier at point once without adding a Watch.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_EVAL_AT_POINT,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.repl",
            "Opens the debug REPL Popup.",
            vec![PluginAction::emit_hook(HOOK_DAP_REPL, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.switch-thread",
            "Switches the active Debug Session thread.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_SWITCH_THREAD,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.switch-stack-frame",
            "Switches the active Debug Session stack frame.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_SWITCH_STACK_FRAME,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.breakpoint-condition",
            "Sets or clears the Breakpoint condition on the current line.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_BREAKPOINT_CONDITION,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.breakpoint-hit-condition",
            "Sets or clears the Breakpoint hit condition on the current line.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_BREAKPOINT_HIT_CONDITION,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.breakpoint-log-message",
            "Sets or clears the Breakpoint log message (logpoint) on the current line.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_BREAKPOINT_LOG_MESSAGE,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.toggle-variable",
            "Expands or collapses the Locals or Watch Expression under the cursor.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_TOGGLE_VARIABLE,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.goto-breakpoint",
            "Jumps to the source of the Breakpoint under the cursor.",
            vec![PluginAction::emit_hook(
                HOOK_DAP_GOTO_BREAKPOINT,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dap.start-codelldb",
            "Starts a Rust Debug Session with the codelldb adapter.",
            vec![PluginAction::emit_hook(HOOK_DAP_START, Some("codelldb"))],
        ),
        PluginCommand::new(
            "dap.start-gdb",
            "Starts a Debug Session with the gdb adapter.",
            vec![PluginAction::emit_hook(HOOK_DAP_START, Some("gdb"))],
        ),
        PluginCommand::new(
            "dap.start-sharpdbg",
            "Starts a C# Debug Session with the sharpdbg adapter.",
            vec![PluginAction::emit_hook(HOOK_DAP_START, Some("sharpdbg"))],
        ),
        PluginCommand::new(
            "dap.start-last",
            "Replays the last successful Debug Configuration.",
            vec![PluginAction::emit_hook(HOOK_DAP_START_LAST, None::<&str>)],
        ),
        PluginCommand::new(
            "dap.start-recent",
            "Opens a picker of recent Debug Configurations.",
            vec![PluginAction::emit_hook(HOOK_DAP_START_RECENT, None::<&str>)],
        ),
    ])
    .with_hook_declarations(vec![
        PluginHookDeclaration::new(
            HOOK_DAP_START,
            "Starts a Workspace-scoped Debug Session through the DAP client.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_START_LAST,
            "Starts a Debug Session using the last successful Debug Configuration.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_START_RECENT,
            "Opens a picker of recent Debug Configurations to start.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_STOP,
            "Performs Debug Stop for the active Workspace Session.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_RESTART,
            "Restarts the active Workspace Session with the same Debug Configuration.",
        ),
        PluginHookDeclaration::new(HOOK_DAP_CONTINUE, "Continues the stopped Debug Session."),
        PluginHookDeclaration::new(HOOK_DAP_PAUSE, "Pauses the live Debug Session."),
        PluginHookDeclaration::new(HOOK_DAP_STEP, "Steps over in the Debug Session."),
        PluginHookDeclaration::new(HOOK_DAP_STEP_INTO, "Steps into in the Debug Session."),
        PluginHookDeclaration::new(HOOK_DAP_STEP_OUT, "Steps out in the Debug Session."),
        PluginHookDeclaration::new(HOOK_DAP_LOG, "Opens the DAP transport log buffer."),
        PluginHookDeclaration::new(
            HOOK_DAP_TOGGLE_BREAKPOINT,
            "Toggles a Workspace Breakpoint at the current line.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_DELETE_BREAKPOINT,
            "Deletes the Workspace Breakpoint at the current line.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_OPEN_BREAKPOINTS,
            "Opens the Workspace Breakpoints surface.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_EXPRESSIONS_ADD,
            "Adds a Watch Expression shown under Locals.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_EXPRESSIONS_REMOVE,
            "Removes a Watch Expression from the Expressions section.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_EVAL,
            "Evaluates an expression once without adding a Watch.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_EVAL_AT_POINT,
            "Evaluates the identifier at point once without adding a Watch.",
        ),
        PluginHookDeclaration::new(HOOK_DAP_REPL, "Opens the debug REPL Popup."),
        PluginHookDeclaration::new(
            HOOK_DAP_SWITCH_THREAD,
            "Switches the active Debug Session thread and refreshes Locals/watches.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_SWITCH_STACK_FRAME,
            "Switches the active stack frame and refreshes Locals/watches.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_BREAKPOINT_CONDITION,
            "Sets or clears the Breakpoint condition and syncs to the adapter.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_BREAKPOINT_HIT_CONDITION,
            "Sets or clears the Breakpoint hit condition and syncs to the adapter.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_BREAKPOINT_LOG_MESSAGE,
            "Sets or clears the Breakpoint log message and syncs to the adapter.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_TOGGLE_VARIABLE,
            "Toggles expansion of a structured Locals or Watch Expression row.",
        ),
        PluginHookDeclaration::new(
            HOOK_DAP_GOTO_BREAKPOINT,
            "Opens the source of the Breakpoint under the cursor.",
        ),
    ])
    .with_buffers(vec![
        PluginBuffer::new(
            BREAKPOINTS_KIND,
            vec!["No Breakpoints in this Workspace.".to_owned()],
        )
        .with_key_bindings(vec![
            PluginKeyBinding::new("Enter", "dap.goto-breakpoint", PluginKeymapScope::Workspace)
                .with_vim_mode(PluginVimMode::Normal),
        ]),
        PluginBuffer::new(
            LOCALS_KIND,
            vec!["No locals".to_owned(), "Watch Expressions".to_owned()],
        )
        .with_sections(locals_sections())
        .with_key_bindings(vec![
            PluginKeyBinding::new("Enter", "dap.toggle-variable", PluginKeymapScope::Workspace)
                .with_vim_mode(PluginVimMode::Normal),
            PluginKeyBinding::new("l", "dap.toggle-variable", PluginKeymapScope::Workspace)
                .with_vim_mode(PluginVimMode::Normal),
        ]),
        PluginBuffer::new(
            REPL_KIND,
            vec!["(debug REPL — enter expression below)".to_owned()],
        ),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new(
            "Space d a",
            "dap.toggle-breakpoint",
            PluginKeymapScope::Workspace,
        ),
        PluginKeyBinding::new("F5", "dap.start", PluginKeymapScope::Global),
        PluginKeyBinding::new("F5", "dap.continue", PluginKeymapScope::Dap),
        PluginKeyBinding::new("F10", "dap.step", PluginKeymapScope::Dap),
        PluginKeyBinding::new("F11", "dap.step-into", PluginKeymapScope::Dap),
    ])
}

/// Locals pane: adapter locals above a Watch Expressions header (insert a line to watch).
/// Expressions below mirrors those watches.
pub fn locals_sections() -> PluginBufferSections {
    PluginBufferSections::new(vec![
        PluginBufferSection::new(LOCALS_SECTION)
            .with_writable(true)
            .with_min_lines(4)
            .with_initial_lines(vec!["No locals".to_owned(), "Watch Expressions".to_owned()]),
        PluginBufferSection::new(EXPRESSIONS_SECTION)
            .with_min_lines(2)
            .with_initial_lines(vec!["No watches".to_owned()]),
    ])
    .with_layout(PluginBufferLayout::rows(vec![
        PluginBufferLayoutNode::section(LOCALS_SECTION, 3),
        PluginBufferLayoutNode::section(EXPRESSIONS_SECTION, 1),
    ]))
}

/// Returns DAP adapter specifications compiled into the user library.
pub fn debug_adapters() -> Vec<DebugAdapterSpec> {
    vec![
        DebugAdapterSpec::new("codelldb", "rust", ["rs"], "codelldb", ["--port", "13000"])
            .with_transport(DebugAdapterTransport::Tcp {
                host: "127.0.0.1".to_owned(),
                port: 13000,
            })
            .with_preference(100)
            .with_root_markers(["Cargo.toml", "rust-project.json"])
            .with_root_strategy(DebugAdapterRootStrategy::MarkersOrWorkspace),
        DebugAdapterSpec::new(
            "gdb",
            "rust",
            ["rs", "c", "cpp", "cc", "h", "hpp"],
            "gdb",
            ["-i=dap"],
        )
        .with_preference(50)
        .with_root_markers(["Cargo.toml", "Makefile", "CMakeLists.txt"])
        .with_root_strategy(DebugAdapterRootStrategy::MarkersOrWorkspace),
        DebugAdapterSpec::new(
            "sharpdbg",
            "csharp",
            ["cs"],
            "sharpdbg",
            ["--interpreter=vscode"],
        )
        .with_preference(100)
        .with_root_markers(["*.sln", "*.csproj"])
        .with_root_strategy(DebugAdapterRootStrategy::MarkersOrWorkspace),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locals_buffer_declares_locals_and_expressions_sections() {
        let sections = locals_sections();
        let names: Vec<_> = sections
            .sections()
            .iter()
            .map(|section| section.name().to_owned())
            .collect();
        assert_eq!(
            names,
            vec![LOCALS_SECTION.to_owned(), EXPRESSIONS_SECTION.to_owned()]
        );
        let layout = sections.layout().expect("locals layout");
        assert_eq!(layout.children().len(), 2);
        assert!(
            sections.sections()[0].writable(),
            "Locals section must be writable so Watch Expressions can be inserted as lines"
        );
    }

    #[test]
    fn package_exports_debug_layout_buffers() {
        let package = package();
        assert!(package.buffer(BREAKPOINTS_KIND).is_some());
        assert!(
            package
                .buffer(LOCALS_KIND)
                .and_then(|buffer| buffer.sections().cloned())
                .is_some()
        );
        assert!(package.buffer(REPL_KIND).is_some());
        let locals_chords: Vec<_> = package
            .buffer(LOCALS_KIND)
            .expect("locals")
            .key_bindings()
            .iter()
            .map(|binding| (binding.chord(), binding.command_name()))
            .collect();
        assert!(
            locals_chords.contains(&("Enter", "dap.toggle-variable")),
            "Locals Enter must toggle expansion: {locals_chords:?}"
        );
        let bp_chords: Vec<_> = package
            .buffer(BREAKPOINTS_KIND)
            .expect("breakpoints")
            .key_bindings()
            .iter()
            .map(|binding| (binding.chord(), binding.command_name()))
            .collect();
        assert!(
            bp_chords.contains(&("Enter", "dap.goto-breakpoint")),
            "Breakpoints Enter must jump: {bp_chords:?}"
        );
    }

    #[test]
    fn package_exports_stepping_and_restart_commands() {
        let package = package();
        let names: Vec<_> = package
            .commands()
            .iter()
            .map(|command| command.name().to_owned())
            .collect();
        for expected in [
            "dap.continue",
            "dap.pause",
            "dap.step",
            "dap.step-into",
            "dap.step-out",
            "dap.restart",
        ] {
            assert!(
                names.iter().any(|name| name == expected),
                "missing command {expected}"
            );
        }
    }

    #[test]
    fn package_exports_polish_commands() {
        let package = package();
        let names: Vec<_> = package
            .commands()
            .iter()
            .map(|command| command.name().to_owned())
            .collect();
        for expected in [
            "dap.expressions-add",
            "dap.expressions-remove",
            "dap.eval",
            "dap.eval-at-point",
            "dap.repl",
            "dap.switch-thread",
            "dap.switch-stack-frame",
            "dap.breakpoint-condition",
            "dap.breakpoint-hit-condition",
            "dap.breakpoint-log-message",
            "dap.toggle-variable",
            "dap.goto-breakpoint",
            "dap.log",
        ] {
            assert!(
                names.iter().any(|name| name == expected),
                "missing command {expected}"
            );
        }
    }

    #[test]
    fn package_exports_start_family_commands() {
        let package = package();
        let names: Vec<_> = package
            .commands()
            .iter()
            .map(|command| command.name().to_owned())
            .collect();
        for expected in [
            "dap.start",
            "dap.start-codelldb",
            "dap.start-gdb",
            "dap.start-sharpdbg",
            "dap.start-last",
            "dap.start-recent",
        ] {
            assert!(
                names.iter().any(|name| name == expected),
                "missing command {expected}"
            );
        }
    }

    #[test]
    fn adapter_preferences_match_language_defaults() {
        let adapters = debug_adapters();
        let rust: Vec<_> = adapters
            .iter()
            .filter(|adapter| adapter.file_extensions().iter().any(|ext| ext == "rs"))
            .collect();
        assert_eq!(rust[0].id(), "codelldb");
        assert!(rust[0].preference() > rust[1].preference());
        assert_eq!(rust[1].id(), "gdb");

        let csharp = adapters
            .iter()
            .find(|adapter| adapter.id() == "sharpdbg")
            .expect("sharpdbg");
        assert!(csharp.file_extensions().iter().any(|ext| ext == "cs"));
        assert_eq!(csharp.preference(), 100);
        assert_eq!(
            csharp.args(),
            &["--interpreter=vscode".to_owned()],
            "sharpdbg requires --interpreter=vscode for DAP stdio"
        );

        let cpp_gdb = adapters
            .iter()
            .find(|adapter| {
                adapter.id() == "gdb"
                    && adapter
                        .file_extensions()
                        .iter()
                        .any(|ext| ext == "cpp" || ext == "c")
            })
            .expect("gdb for c/c++");
        assert_eq!(cpp_gdb.preference(), 50);
    }

    #[test]
    fn package_exports_workspace_and_dap_mode_keybindings() {
        let package = package();
        let bindings: Vec<_> = package
            .key_bindings()
            .iter()
            .map(|binding| (binding.chord(), binding.command_name(), binding.scope()))
            .collect();
        assert!(
            bindings.contains(&(
                "Space d a",
                "dap.toggle-breakpoint",
                PluginKeymapScope::Workspace
            )),
            "Workspace <leader> da must toggle Breakpoint"
        );
        assert!(
            bindings.contains(&("F5", "dap.start", PluginKeymapScope::Global)),
            "F5 must start a Debug Session when DAP Mode is inactive"
        );
        assert!(
            bindings.contains(&("F5", "dap.continue", PluginKeymapScope::Dap)),
            "DAP Mode F5 must continue"
        );
        assert!(
            bindings.contains(&("F10", "dap.step", PluginKeymapScope::Dap)),
            "DAP Mode F10 must step over"
        );
        assert!(
            bindings.contains(&("F11", "dap.step-into", PluginKeymapScope::Dap)),
            "DAP Mode F11 must step into"
        );
    }
}
