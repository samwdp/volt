use editor_plugin_api::{
    PluginAction, PluginBuffer, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
    PluginVimMode,
};

pub const PACKAGE_NAME: &str = "db";
pub const CONNECT_KIND: &str = "db-connect";
pub const QUERY_KIND: &str = "db-query";
pub const CONNECTIONS_KIND: &str = "db-connections";
pub const SCHEMA_KIND: &str = "db-schema";
pub const HISTORY_KIND: &str = "db-history";
pub const SNIPPETS_KIND: &str = "db-snippets";
pub const RESULTS_KIND: &str = "db-results";

pub const HOOK_CONNECT: &str = "db.connect";
pub const HOOK_DISCONNECT: &str = "db.disconnect";
pub const HOOK_SHOW_TABLES: &str = "db.show-tables";
pub const HOOK_NEW_QUERY_BUFFER: &str = "db.new-query-buffer";
pub const HOOK_EXECUTE_SQL: &str = "db.execute-sql";
pub const HOOK_SHOW_CONNECTIONS: &str = "db.show-connections";
pub const HOOK_SHOW_HISTORY: &str = "db.show-history";
pub const HOOK_SHOW_SNIPPETS: &str = "db.show-snippets";
pub const HOOK_SAVE_SNIPPET: &str = "db.save-snippet";
pub const HOOK_REFRESH_SCHEMA: &str = "db.refresh-schema";
pub const HOOK_ACTIVATE_LINE: &str = "db.activate-line";

pub const CONNECT_BUFFER_NAME: &str = "*db-connect*";
pub const CONNECTIONS_BUFFER_NAME: &str = "*db-connections*";
pub const SCHEMA_BUFFER_NAME: &str = "*db-schema*";
pub const HISTORY_BUFFER_NAME: &str = "*db-history*";
pub const SNIPPETS_BUFFER_NAME: &str = "*db-snippets*";
pub const RESULTS_BUFFER_NAME: &str = "*db-results*";

pub const EXECUTE_CHORD: &str = "Ctrl+c Ctrl+c";

/// Returns the database explorer package metadata.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        PACKAGE_NAME,
        true,
        "Database sessions, schema browsing, and SQL query buffers.",
    )
    .with_commands(vec![
        hook_command(
            "db.connect",
            "Prompts for a raw database connection string.",
            HOOK_CONNECT,
            None,
        ),
        hook_command(
            "db.disconnect",
            "Disconnects the active database session.",
            HOOK_DISCONNECT,
            None,
        ),
        hook_command(
            "db.show-tables",
            "Opens the schema explorer for the active database session.",
            HOOK_SHOW_TABLES,
            None,
        ),
        hook_command(
            "db.new-query-buffer",
            "Creates a SQL query buffer bound to the active database session.",
            HOOK_NEW_QUERY_BUFFER,
            None,
        ),
        hook_command(
            "db.execute-sql",
            "Executes the selected SQL or current statement in the active DB query buffer.",
            HOOK_EXECUTE_SQL,
            None,
        ),
        hook_command(
            "db.show-connections",
            "Shows active and remembered database connections.",
            HOOK_SHOW_CONNECTIONS,
            None,
        ),
        hook_command(
            "db.show-history",
            "Shows recent executed SQL for active database sessions.",
            HOOK_SHOW_HISTORY,
            None,
        ),
        hook_command(
            "db.show-snippets",
            "Shows saved SQL snippets.",
            HOOK_SHOW_SNIPPETS,
            None,
        ),
        hook_command(
            "db.save-snippet",
            "Saves the selected SQL or current statement as a snippet.",
            HOOK_SAVE_SNIPPET,
            None,
        ),
        hook_command(
            "db.refresh-schema",
            "Refreshes schema cache for the active database session.",
            HOOK_REFRESH_SCHEMA,
            None,
        ),
        hook_command(
            "db.activate-line",
            "Runs the action attached to the current database browser line.",
            HOOK_ACTIVATE_LINE,
            None,
        ),
    ])
    .with_buffers(vec![
        PluginBuffer::new(CONNECT_KIND, connect_buffer_lines()).with_key_bindings(vec![
            PluginKeyBinding::new("Enter", "ui.input.submit", PluginKeymapScope::Popup)
                .with_vim_mode(PluginVimMode::Insert),
            PluginKeyBinding::new("Ctrl+Enter", "ui.input.submit", PluginKeymapScope::Popup)
                .with_vim_mode(PluginVimMode::Insert),
        ]),
        PluginBuffer::new(QUERY_KIND, query_buffer_lines()).with_key_bindings(vec![
            PluginKeyBinding::new(
                EXECUTE_CHORD,
                "db.execute-sql",
                PluginKeymapScope::Workspace,
            ),
            PluginKeyBinding::new("Ctrl+s", "db.save-snippet", PluginKeymapScope::Workspace),
        ]),
        readonly_browser_buffer(CONNECTIONS_KIND, false),
        readonly_browser_buffer(SCHEMA_KIND, true),
        readonly_browser_buffer(HISTORY_KIND, false),
        readonly_browser_buffer(SNIPPETS_KIND, false),
        PluginBuffer::new(RESULTS_KIND, vec!["Query results appear here.".to_owned()]),
    ])
}

fn readonly_browser_buffer(kind: &'static str, include_refresh: bool) -> PluginBuffer {
    let mut key_bindings = vec![
        PluginKeyBinding::new("Enter", "db.activate-line", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
    ];
    if include_refresh {
        key_bindings.push(
            PluginKeyBinding::new("r", "db.refresh-schema", PluginKeymapScope::Workspace)
                .with_vim_mode(PluginVimMode::Normal),
        );
    }
    PluginBuffer::new(kind, vec!["Loading...".to_owned()]).with_key_bindings(key_bindings)
}

fn connect_buffer_lines() -> Vec<String> {
    vec![
        "Connect to SQLite, PostgreSQL, or SQL Server.".to_owned(),
        "Paste a raw connection string into the prompt below.".to_owned(),
        "Use `remember <alias> :: <connection string>` to save securely when OS keyring support is available."
            .to_owned(),
    ]
}

fn query_buffer_lines() -> Vec<String> {
    vec![
        "-- SQL query buffer".to_owned(),
        "-- Press Ctrl+c Ctrl+c to execute the current statement or selection.".to_owned(),
        String::new(),
        "SELECT 1;".to_owned(),
    ]
}

fn hook_command(
    name: &str,
    description: &str,
    hook_name: &str,
    detail: Option<&str>,
) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, detail)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_required_commands() {
        let package = package();
        for command_name in [
            "db.connect",
            "db.disconnect",
            "db.show-tables",
            "db.new-query-buffer",
            "db.execute-sql",
            "db.show-connections",
            "db.show-history",
            "db.show-snippets",
            "db.save-snippet",
        ] {
            assert!(
                package
                    .commands()
                    .iter()
                    .any(|command| command.name() == command_name),
                "missing command `{command_name}`",
            );
        }
    }

    #[test]
    fn query_buffer_exports_execute_chord() {
        let package = package();
        let query_buffer = package
            .buffers()
            .iter()
            .find(|buffer| buffer.kind() == QUERY_KIND)
            .expect("query buffer should be exported");
        assert!(
            query_buffer
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == EXECUTE_CHORD),
        );
    }
}
