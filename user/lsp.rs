use crate::icon_font::symbols::md;
use editor_lsp::{LanguageServerRootStrategy, LanguageServerSpec};
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
pub const SERVER_CSHARP_LS: &str = "csharp-ls";
pub const SERVER_TYPESCRIPT_LANGUAGE_SERVER: &str = "typescript-language-server";
pub const SERVER_VSCODE_JSON_LANGUAGE_SERVER: &str = "vscode-json-language-server";
pub const SERVER_TOMBI: &str = "tombi";
pub const SERVER_YAML_LANGUAGE_SERVER: &str = "yaml-language-server";
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
        hook_command(
            "lsp.start-csharp-ls",
            "Starts csharp-ls for the active C# file.",
            HOOK_LSP_START,
            Some(SERVER_CSHARP_LS),
        ),
        hook_command(
            "lsp.start-typescript-language-server",
            "Starts typescript-language-server for the active TS/TSX/JS/JSX file.",
            HOOK_LSP_START,
            Some(SERVER_TYPESCRIPT_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-vscode-json-language-server",
            "Starts vscode-json-language-server for the active JSON file.",
            HOOK_LSP_START,
            Some(SERVER_VSCODE_JSON_LANGUAGE_SERVER),
        ),
        hook_command(
            "lsp.start-tombi",
            "Starts tombi for the active TOML file.",
            HOOK_LSP_START,
            Some(SERVER_TOMBI),
        ),
        hook_command(
            "lsp.start-yaml-language-server",
            "Starts yaml-language-server for the active YAML file.",
            HOOK_LSP_START,
            Some(SERVER_YAML_LANGUAGE_SERVER),
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
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-csharp",
            "lsp.start",
            Some(".cs"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-typescript",
            "lsp.start",
            Some(".ts"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-tsx",
            "lsp.start",
            Some(".tsx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-javascript",
            "lsp.start",
            Some(".js"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-jsx",
            "lsp.start",
            Some(".jsx"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-json",
            "lsp.start",
            Some(".json"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-toml",
            "lsp.start",
            Some(".toml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-yaml",
            "lsp.start",
            Some(".yaml"),
        ),
        PluginHookBinding::new(
            "buffer.file-open",
            "lsp.auto-start-yml",
            "lsp.start",
            Some(".yml"),
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
        LanguageServerSpec::new(
            SERVER_CSHARP_LS,
            "csharp",
            ["cs"],
            "csharp-ls",
            ["--features", "razor-support,metadata-uris"],
        )
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "*.sln",
            "*.csproj",
            "global.json",
            "Directory.Build.props",
            "Directory.Build.targets",
        ]),
        LanguageServerSpec::new(
            SERVER_TYPESCRIPT_LANGUAGE_SERVER,
            "typescript",
            ["ts", "tsx", "js", "jsx"],
            "typescript-language-server",
            ["--stdio"],
        )
        .with_document_language_ids([
            ("tsx", "typescriptreact"),
            ("js", "javascript"),
            ("jsx", "javascriptreact"),
        ])
        .with_root_markers([
            "package.json",
            "tsconfig.json",
            "jsconfig.json",
            "deno.json",
            "deno.jsonc",
        ]),
        LanguageServerSpec::new(
            SERVER_VSCODE_JSON_LANGUAGE_SERVER,
            "json",
            ["json"],
            "vscode-json-language-server",
            ["--stdio"],
        ),
        LanguageServerSpec::new(SERVER_TOMBI, "toml", ["toml"], "tombi", ["lsp"]),
        LanguageServerSpec::new(
            SERVER_YAML_LANGUAGE_SERVER,
            "yaml",
            ["yaml", "yml"],
            "yaml-language-server",
            ["--stdio"],
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
    fn package_registers_rich_language_server_defaults() {
        let package = package();
        let servers = language_servers();
        let ids = servers.iter().map(|server| server.id()).collect::<Vec<_>>();

        assert_eq!(package.name(), "lsp");
        assert!(package.auto_load());
        assert_eq!(package.commands().len(), 14);
        assert_eq!(package.hook_bindings().len(), 12);
        assert_eq!(servers.len(), 7);
        assert!(ids.contains(&SERVER_RUST_ANALYZER));
        assert!(ids.contains(&SERVER_MARKSMAN));
        assert!(ids.contains(&SERVER_CSHARP_LS));
        assert!(ids.contains(&SERVER_TYPESCRIPT_LANGUAGE_SERVER));
        assert!(ids.contains(&SERVER_VSCODE_JSON_LANGUAGE_SERVER));
        assert!(ids.contains(&SERVER_TOMBI));
        assert!(ids.contains(&SERVER_YAML_LANGUAGE_SERVER));

        let rust = servers
            .iter()
            .find(|server| server.id() == SERVER_RUST_ANALYZER)
            .expect("rust-analyzer missing");
        assert_eq!(rust.language_id(), "rust");
        assert!(
            rust.args().is_empty(),
            "rust-analyzer now speaks stdio without a `--stdio` flag"
        );

        let csharp = servers
            .iter()
            .find(|server| server.id() == SERVER_CSHARP_LS)
            .expect("csharp-ls missing");
        assert_eq!(
            csharp.args().iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["--features", "razor-support,metadata-uris"]
        );

        let typescript = servers
            .iter()
            .find(|server| server.id() == SERVER_TYPESCRIPT_LANGUAGE_SERVER)
            .expect("typescript-language-server missing");
        assert_eq!(typescript.language_id(), "typescript");
        assert_eq!(
            typescript
                .file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["ts", "tsx", "js", "jsx"]
        );
        assert_eq!(
            typescript
                .args()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["--stdio"]
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".ts"),
            "typescript"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".tsx"),
            "typescriptreact"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".js"),
            "javascript"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".jsx"),
            "javascriptreact"
        );

        let yaml = servers
            .iter()
            .find(|server| server.id() == SERVER_YAML_LANGUAGE_SERVER)
            .expect("yaml-language-server missing");
        assert_eq!(yaml.language_id(), "yaml");
        assert_eq!(
            yaml.file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["yaml", "yml"]
        );
        assert_eq!(
            yaml.args().iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["--stdio"]
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
