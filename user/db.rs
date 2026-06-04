use editor_plugin_api::{
    ContextHelpEntry, ContextHelpSpec, DbActionSpec, DbBrowserContext, DbBrowserItemContext,
    DbBrowserItemKind, DbBrowserItemSpec, DbFeatureSpec, PluginAction, PluginBuffer, PluginCommand,
    PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode, buffer_kinds, db_hooks,
    input_hooks,
};

pub const PACKAGE_NAME: &str = "db";
pub const CONNECT_KIND: &str = buffer_kinds::DB_CONNECT;
pub const QUERY_KIND: &str = buffer_kinds::DB_QUERY;
pub const CONNECTIONS_KIND: &str = buffer_kinds::DB_CONNECTIONS;
pub const SCHEMA_KIND: &str = buffer_kinds::DB_SCHEMA;
pub const HISTORY_KIND: &str = buffer_kinds::DB_HISTORY;
pub const SNIPPETS_KIND: &str = buffer_kinds::DB_SNIPPETS;
pub const RESULTS_KIND: &str = buffer_kinds::DB_RESULTS;

pub const CONNECT_BUFFER_NAME: &str = "*db-connect*";
pub const CONNECTIONS_BUFFER_NAME: &str = "*db-connections*";
pub const SCHEMA_BUFFER_NAME: &str = "*db-schema*";
pub const HISTORY_BUFFER_NAME: &str = "*db-history*";
pub const SNIPPETS_BUFFER_NAME: &str = "*db-snippets*";
pub const RESULTS_BUFFER_NAME: &str = "*db-results*";

pub const EXECUTE_CHORD: &str = "Ctrl+c Ctrl+c";

/// Public database feature contract used by first-party and third-party code.
pub fn feature_spec() -> DbFeatureSpec {
    DbFeatureSpec {
        connect_buffer_name: CONNECT_BUFFER_NAME.to_owned(),
        connections_buffer_name: CONNECTIONS_BUFFER_NAME.to_owned(),
        schema_buffer_name: SCHEMA_BUFFER_NAME.to_owned(),
        history_buffer_name: HISTORY_BUFFER_NAME.to_owned(),
        snippets_buffer_name: SNIPPETS_BUFFER_NAME.to_owned(),
        results_buffer_name: RESULTS_BUFFER_NAME.to_owned(),
        execute_chord: EXECUTE_CHORD.to_owned(),
        connect_help: ContextHelpSpec::new(
            "DbConnect",
            "DB Connect",
            vec![
                ContextHelpEntry::new("Enter", "submit", "Submits database connection prompt."),
                ContextHelpEntry::new(
                    "Ctrl+Enter",
                    "submit",
                    "Submits database connection prompt.",
                ),
            ],
        ),
        query_help: ContextHelpSpec::new(
            "DbQuery",
            "DB Query",
            vec![
                ContextHelpEntry::new(
                    EXECUTE_CHORD,
                    "execute sql",
                    "Executes the current statement or selection.",
                ),
                ContextHelpEntry::new(
                    "Ctrl+s",
                    "save snippet",
                    "Saves the current statement or selection as snippet.",
                ),
            ],
        ),
        browser_help: ContextHelpSpec::new(
            "DbBrowser",
            "DB Browser",
            vec![
                ContextHelpEntry::new(
                    "Enter",
                    "activate line",
                    "Runs action attached to current database browser line.",
                ),
                ContextHelpEntry::new("r", "refresh schema", "Refreshes active schema browser."),
            ],
        ),
    }
}

pub fn browser_items(context: &DbBrowserContext) -> Vec<DbBrowserItemSpec> {
    context.items.iter().map(browser_item).collect()
}

fn browser_item(item: &DbBrowserItemContext) -> DbBrowserItemSpec {
    let line = match item.kind {
        DbBrowserItemKind::Header => item.label.to_string(),
        DbBrowserItemKind::Empty => item.label.to_string(),
        DbBrowserItemKind::ActiveConnection | DbBrowserItemKind::RememberedConnection => {
            format!(
                "{} {}{}",
                item.engine,
                item.label,
                if item.active { " [active]" } else { "" }
            )
        }
        DbBrowserItemKind::Table => format!("▦ {}", item.label),
        DbBrowserItemKind::View => format!("◫ {}", item.label),
        DbBrowserItemKind::Index => format!("◎ {}", item.label),
        DbBrowserItemKind::HistoryEntry | DbBrowserItemKind::Snippet => {
            format!("{} {} :: {}", item.engine, item.label, item.detail)
        }
    };
    DbBrowserItemSpec::new(line, default_action(item))
}

fn default_action(item: &DbBrowserItemContext) -> Option<DbActionSpec> {
    item.default_action.clone().into()
}

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
            db_hooks::CONNECT,
            None,
        ),
        hook_command(
            "db.disconnect",
            "Disconnects the active database session.",
            db_hooks::DISCONNECT,
            None,
        ),
        hook_command(
            "db.show-tables",
            "Opens the schema explorer for the active database session.",
            db_hooks::SHOW_TABLES,
            None,
        ),
        hook_command(
            "db.new-query-buffer",
            "Creates a SQL query buffer bound to the active database session.",
            db_hooks::NEW_QUERY_BUFFER,
            None,
        ),
        hook_command(
            "db.execute-sql",
            "Executes the selected SQL or current statement in the active DB query buffer.",
            db_hooks::EXECUTE_SQL,
            None,
        ),
        hook_command(
            "db.show-connections",
            "Shows active and remembered database connections.",
            db_hooks::SHOW_CONNECTIONS,
            None,
        ),
        hook_command(
            "db.show-history",
            "Shows recent executed SQL for active database sessions.",
            db_hooks::SHOW_HISTORY,
            None,
        ),
        hook_command(
            "db.show-snippets",
            "Shows saved SQL snippets.",
            db_hooks::SHOW_SNIPPETS,
            None,
        ),
        hook_command(
            "db.save-snippet",
            "Saves the selected SQL or current statement as a snippet.",
            db_hooks::SAVE_SNIPPET,
            None,
        ),
        hook_command(
            "db.refresh-schema",
            "Refreshes schema cache for the active database session.",
            db_hooks::REFRESH_SCHEMA,
            None,
        ),
        hook_command(
            "db.activate-line",
            "Runs the action attached to the current database browser line.",
            db_hooks::ACTIVATE_LINE,
            None,
        ),
    ])
    .with_buffers(vec![
        PluginBuffer::new(CONNECT_KIND, connect_buffer_lines()).with_key_bindings(vec![
            PluginKeyBinding::new("Enter", input_hooks::SUBMIT, PluginKeymapScope::Popup)
                .with_vim_mode(PluginVimMode::Insert),
            PluginKeyBinding::new("Ctrl+Enter", input_hooks::SUBMIT, PluginKeymapScope::Popup)
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
        PluginBuffer::new(RESULTS_KIND, vec!["Query results appear here.".to_owned()])
            .with_line_wrap(false),
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

    #[test]
    fn results_buffer_disables_line_wrap_by_default() {
        let package = package();
        let results_buffer = package
            .buffers()
            .iter()
            .find(|buffer| buffer.kind() == RESULTS_KIND)
            .expect("results buffer should be exported");

        assert!(!results_buffer.line_wrap());
    }

    #[test]
    fn browser_items_shape_table_rows_from_user_config() {
        let context = DbBrowserContext::new(
            editor_plugin_api::DbBrowserKind::Schema,
            "*db-schema local*",
        )
        .with_items(vec![
            DbBrowserItemContext::new(DbBrowserItemKind::Table, "users")
                .with_default_action(DbActionSpec::open_table_preview(7, None::<String>, "users")),
        ]);
        let items = browser_items(&context);
        assert_eq!(items[0].line(), "▦ users");
        assert!(matches!(
            items[0].action(),
            Some(DbActionSpec::OpenTablePreview { session_id: 7, .. })
        ));
    }
}
