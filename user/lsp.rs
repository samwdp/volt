use crate::icon_font::symbols::md;
use editor_lsp::LanguageServerSpec;
use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding, PluginHookDeclaration, PluginPackage,
};

pub const HOOK_LSP_START: &str = "lsp.server-start";
pub const HOOK_LSP_STOP: &str = "lsp.server-stop";
pub const HOOK_LSP_RESTART: &str = "lsp.server-restart";
pub const HOOK_LSP_LOG: &str = "lsp.open-log";
pub const HOOK_LSP_DEFINITION: &str = "lsp.goto-definition";
pub const HOOK_LSP_REFERENCES: &str = "lsp.goto-references";
pub const HOOK_LSP_IMPLEMENTATION: &str = "lsp.goto-implementation";
pub const SERVER_RUST_ANALYZER: &str = "rust-analyzer";
pub const SERVER_MARKSMAN: &str = "marksman";
pub const SHOW_BUFFER_DIAGNOSTICS: bool = true;
pub const DIAGNOSTIC_LINE_LIMIT: usize = 8;
pub const DIAGNOSTIC_ICON: &str = md::MD_ALERT_CIRCLE_OUTLINE;

/// Returns the metadata for the LSP integration package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lsp",
        true,
        "Language server integration, lifecycle commands, and startup hooks.",
    )
    .with_commands(vec![
        hook_command(
            "lsp.start",
            "Starts the language servers registered for the active file.",
            HOOK_LSP_START,
            None,
        ),
        hook_command(
            "lsp.stop",
            "Stops the language servers attached to the active file.",
            HOOK_LSP_STOP,
            None,
        ),
        hook_command(
            "lsp.restart",
            "Restarts the language servers for the active file.",
            HOOK_LSP_RESTART,
            None,
        ),
        hook_command(
            "lsp.log",
            "Opens the live LSP transport log buffer.",
            HOOK_LSP_LOG,
            None,
        ),
        hook_command(
            "lsp.definition",
            "Jumps to the LSP definition under the cursor.",
            HOOK_LSP_DEFINITION,
            None,
        ),
        hook_command(
            "lsp.references",
            "Finds LSP references for the symbol under the cursor.",
            HOOK_LSP_REFERENCES,
            None,
        ),
        hook_command(
            "lsp.implementation",
            "Jumps to LSP implementations for the symbol under the cursor.",
            HOOK_LSP_IMPLEMENTATION,
            None,
        ),
        hook_command(
            "lsp.start-rust-analyzer",
            "Starts rust-analyzer for the active Rust file.",
            HOOK_LSP_START,
            Some(SERVER_RUST_ANALYZER),
        ),
        hook_command(
            "lsp.start-marksman",
            "Starts marksman for the active Markdown file.",
            HOOK_LSP_START,
            Some(SERVER_MARKSMAN),
        ),
    ])
    .with_hook_declarations(vec![
        PluginHookDeclaration::new(
            HOOK_LSP_START,
            "Runs after an LSP start command is triggered.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_STOP,
            "Runs after an LSP stop command is triggered.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_RESTART,
            "Runs after an LSP restart command is triggered.",
        ),
        PluginHookDeclaration::new(HOOK_LSP_LOG, "Opens the live LSP transport log buffer."),
        PluginHookDeclaration::new(
            HOOK_LSP_DEFINITION,
            "Navigates to the LSP definition under the cursor.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_REFERENCES,
            "Lists LSP references for the symbol under the cursor.",
        ),
        PluginHookDeclaration::new(
            HOOK_LSP_IMPLEMENTATION,
            "Navigates to LSP implementations for the symbol under the cursor.",
        ),
    ])
    .with_hook_bindings(vec![
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-rust",
            "lsp.start",
            Some(".rs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-markdown",
            "lsp.start",
            Some(".md"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-markdown-long",
            "lsp.start",
            Some(".markdown"),
        ),
    ])
}

/// Returns LSP server specifications compiled into the user library.
pub fn language_servers() -> Vec<LanguageServerSpec> {
    vec![
        LanguageServerSpec::new(
            SERVER_RUST_ANALYZER,
            "rust",
            ["rs"],
            "rust-analyzer",
            std::iter::empty::<&str>(),
        )
        .with_root_markers(["Cargo.toml", "rust-project.json"]),
        LanguageServerSpec::new(
            SERVER_MARKSMAN,
            "markdown",
            ["md", "markdown"],
            "marksman",
            ["server"],
        ),
    ]
}

fn hook_command(
    name: &str,
    description: &str,
    hook_name: &str,
    detail: Option<&str>,
) -> PluginCommand {
    let mut actions = Vec::new();
    if let Some(detail) = detail {
        actions.push(PluginAction::log_message(format!(
            "Starting language server `{detail}` from the user LSP package."
        )));
    }
    actions.push(PluginAction::emit_hook(hook_name, detail));
    PluginCommand::new(name, description, actions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_registers_rust_auto_start() {
        let package = package();
        let servers = language_servers();

        assert_eq!(package.name(), "lsp");
        assert!(package.auto_load());
        assert_eq!(package.commands().len(), 9);
        assert_eq!(package.hook_bindings().len(), 3);
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].id(), SERVER_RUST_ANALYZER);
        assert_eq!(servers[0].language_id(), "rust");
        assert!(
            servers[0].args().is_empty(),
            "rust-analyzer now speaks stdio without a `--stdio` flag"
        );
        assert_eq!(servers[1].id(), SERVER_MARKSMAN);
        assert_eq!(servers[1].language_id(), "markdown");
        assert_eq!(
            servers[1]
                .args()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["server"]
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "lsp.stop")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "lsp.restart")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "lsp.log")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "lsp.definition")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "lsp.references")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "lsp.implementation")
        );
    }
}
