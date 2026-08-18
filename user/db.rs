use crate::icon_font::symbols::{cod, dev, md};
use editor_plugin_api::{
    ContextHelpEntry, ContextHelpSpec, DbActionSpec, DbBrowserContext, DbBrowserItemContext,
    DbBrowserItemKind, DbBrowserItemSpec, DbBrowserKind, DbFeatureSpec, PluginAction, PluginBuffer,
    PluginBufferLayout, PluginBufferLayoutNode, PluginBufferSection, PluginBufferSectionUpdate,
    PluginBufferSections, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
    PluginVimMode, buffer_kinds, db_hooks, input_hooks, plugin_hooks,
};

pub const PACKAGE_NAME: &str = "db";
pub const CONNECT_KIND: &str = buffer_kinds::DB_CONNECT;
pub const QUERY_KIND: &str = buffer_kinds::DB_QUERY;
pub const CONNECTIONS_KIND: &str = buffer_kinds::DB_CONNECTIONS;
pub const SCHEMA_KIND: &str = buffer_kinds::DB_SCHEMA;
pub const HISTORY_KIND: &str = buffer_kinds::DB_HISTORY;
pub const SNIPPETS_KIND: &str = buffer_kinds::DB_SNIPPETS;
pub const RESULTS_KIND: &str = buffer_kinds::DB_RESULTS;
pub const DASHBOARD_KIND: &str = buffer_kinds::DB_DASHBOARD;
pub const SIDEBAR_KIND: &str = buffer_kinds::DB_SIDEBAR;

pub const CONNECT_BUFFER_NAME: &str = "*db-connect*";
pub const CONNECTIONS_BUFFER_NAME: &str = "*db-connections*";
pub const SCHEMA_BUFFER_NAME: &str = "*db-schema*";
pub const HISTORY_BUFFER_NAME: &str = "*db-history*";
pub const SNIPPETS_BUFFER_NAME: &str = "*db-snippets*";
pub const RESULTS_BUFFER_NAME: &str = "*db-results*";
pub const DASHBOARD_BUFFER_NAME: &str = "*db-dashboard*";
pub const SIDEBAR_BUFFER_NAME: &str = "*db-sidebar*";

pub const EDITOR_SECTION: &str = "Editor";
pub const CONNECTIONS_SECTION: &str = "Connections";
pub const TABLES_SECTION: &str = "Tables";
pub const OUTPUT_SECTION: &str = "Output";
pub const SWITCH_PANE_CHORD: &str = "Ctrl+Tab";

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
        DbBrowserItemKind::Header => header_line(&item.label),
        DbBrowserItemKind::Empty => {
            if item.label.is_empty() {
                String::new()
            } else {
                format!("  {}", item.label)
            }
        }
        DbBrowserItemKind::ActiveConnection | DbBrowserItemKind::RememberedConnection => {
            let active = if item.active { "  · active" } else { "" };
            format!(
                "  {}  {}  ·  {}{active}",
                engine_icon(&item.engine),
                item.label,
                item.engine
            )
        }
        DbBrowserItemKind::Table => format!("  {}  {}", cod::COD_TABLE, item.label),
        DbBrowserItemKind::View => format!("  {}  {}", md::MD_EYE_OUTLINE, item.label),
        DbBrowserItemKind::Index => format!("  {}  {}", cod::COD_SYMBOL_KEY, item.label),
        DbBrowserItemKind::HistoryEntry => format!(
            "  {}  {}  ·  {}",
            engine_icon(&item.engine),
            item.label,
            item.detail
        ),
        DbBrowserItemKind::Snippet => format!(
            "  {}  {}  ·  {}",
            cod::COD_BOOKMARK,
            item.label,
            item.detail
        ),
    };
    DbBrowserItemSpec::new(line, default_action(item))
}

fn header_line(label: &str) -> String {
    if label.is_empty() {
        return String::new();
    }
    if label.starts_with(' ') {
        return label.to_owned();
    }
    format!("{}  {label}", header_icon(label))
}

fn header_icon(label: &str) -> &'static str {
    let lower = label.to_ascii_lowercase();
    if lower.starts_with("tables") {
        cod::COD_TABLE
    } else if lower.starts_with("views") {
        md::MD_EYE_OUTLINE
    } else if lower.starts_with("indexes") {
        cod::COD_SYMBOL_KEY
    } else if lower.starts_with("active sessions") {
        cod::COD_DATABASE
    } else if lower.starts_with("remembered") {
        cod::COD_BOOKMARK
    } else if lower.starts_with("query history") {
        cod::COD_HISTORY
    } else if lower.contains("snippet") {
        cod::COD_BOOKMARK
    } else if lower.starts_with("engine:") {
        engine_icon(
            label
                .split_once(':')
                .map(|(_, rest)| rest.trim())
                .unwrap_or(""),
        )
    } else {
        cod::COD_DATABASE
    }
}

fn engine_icon(engine: &str) -> &'static str {
    match engine.trim() {
        "SQLite" => dev::DEV_SQLITE,
        "PostgreSQL" => dev::DEV_POSTGRESQL,
        "SQL Server" => dev::DEV_MICROSOFTSQLSERVER,
        _ => cod::COD_DATABASE,
    }
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
        hook_command(
            "db.dashboard",
            "Opens the database dashboard buffer with connections, tables, editor, and output.",
            db_hooks::DASHBOARD,
            None,
        ),
        hook_command(
            "db.multiview",
            "Opens a database sidebar plus query buffers, with output in a popup.",
            db_hooks::MULTIVIEW,
            None,
        ),
        PluginCommand::new(
            "db.switch-pane",
            "Switch focus between database dashboard sections.",
            vec![PluginAction::emit_hook(
                plugin_hooks::SWITCH_PANE,
                None::<&str>,
            )],
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
        PluginBuffer::new(DASHBOARD_KIND, query_buffer_lines())
            .with_sections(dashboard_sections())
            .with_evaluate_target_section(OUTPUT_SECTION)
            .with_key_bindings(dashboard_key_bindings()),
        PluginBuffer::new(SIDEBAR_KIND, vec!["Loading...".to_owned()])
            .with_sections(sidebar_sections())
            .with_key_bindings(browser_key_bindings(true)),
    ])
}

fn dashboard_key_bindings() -> Vec<PluginKeyBinding> {
    let mut bindings = vec![
        PluginKeyBinding::new(
            EXECUTE_CHORD,
            "db.execute-sql",
            PluginKeymapScope::Workspace,
        ),
        PluginKeyBinding::new("Ctrl+s", "db.save-snippet", PluginKeymapScope::Workspace),
        PluginKeyBinding::new(
            SWITCH_PANE_CHORD,
            "db.switch-pane",
            PluginKeymapScope::Workspace,
        ),
    ];
    bindings.extend(browser_key_bindings(true));
    bindings
}

fn browser_key_bindings(include_refresh: bool) -> Vec<PluginKeyBinding> {
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
    key_bindings
}

/// Dashboard layout: connections/tables on the left, editor/output on the right.
pub fn dashboard_sections() -> PluginBufferSections {
    PluginBufferSections::new(vec![
        PluginBufferSection::new(EDITOR_SECTION)
            .with_writable(true)
            .with_initial_lines(query_buffer_lines()),
        PluginBufferSection::new(CONNECTIONS_SECTION)
            .with_min_lines(4)
            .with_browser_kind(DbBrowserKind::Connections),
        PluginBufferSection::new(TABLES_SECTION)
            .with_min_lines(8)
            .with_browser_kind(DbBrowserKind::Schema),
        PluginBufferSection::new(OUTPUT_SECTION)
            .with_min_lines(4)
            .with_initial_lines(vec!["(press Ctrl+c Ctrl+c to execute)".to_owned()])
            .with_update(PluginBufferSectionUpdate::Replace),
    ])
    .with_layout(PluginBufferLayout::columns(vec![
        PluginBufferLayoutNode::rows(
            1,
            vec![
                PluginBufferLayoutNode::section(CONNECTIONS_SECTION, 1),
                PluginBufferLayoutNode::section(TABLES_SECTION, 3),
            ],
        ),
        PluginBufferLayoutNode::rows(
            3,
            vec![
                PluginBufferLayoutNode::section(EDITOR_SECTION, 3),
                PluginBufferLayoutNode::section(OUTPUT_SECTION, 2),
            ],
        ),
    ]))
}

/// Sidebar layout used by `db.multiview`: connections above tables.
pub fn sidebar_sections() -> PluginBufferSections {
    PluginBufferSections::new(vec![
        PluginBufferSection::new(CONNECTIONS_SECTION)
            .with_min_lines(4)
            .with_browser_kind(DbBrowserKind::Connections),
        PluginBufferSection::new(TABLES_SECTION)
            .with_min_lines(8)
            .with_browser_kind(DbBrowserKind::Schema),
    ])
    .with_layout(PluginBufferLayout::rows(vec![
        PluginBufferLayoutNode::section(CONNECTIONS_SECTION, 1),
        PluginBufferLayoutNode::section(TABLES_SECTION, 3),
    ]))
}

fn readonly_browser_buffer(kind: &'static str, include_refresh: bool) -> PluginBuffer {
    PluginBuffer::new(kind, vec!["Loading...".to_owned()])
        .with_key_bindings(browser_key_bindings(include_refresh))
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
        "-- SQL query".to_owned(),
        format!("-- {EXECUTE_CHORD}  execute statement or selection"),
        "-- Ctrl+s         save snippet".to_owned(),
        String::new(),
        "SELECT *".to_owned(),
        "FROM ;".to_owned(),
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
            "db.dashboard",
            "db.multiview",
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
    fn dashboard_buffer_declares_nested_layout_and_execute_chord() {
        let package = package();
        let dashboard = package
            .buffers()
            .iter()
            .find(|buffer| buffer.kind() == DASHBOARD_KIND)
            .expect("dashboard buffer should be exported");
        let sections = dashboard.sections().expect("dashboard sections");
        assert!(sections.layout().is_some());
        assert_eq!(
            sections
                .items()
                .iter()
                .map(PluginBufferSection::name)
                .collect::<Vec<_>>(),
            vec![
                EDITOR_SECTION,
                CONNECTIONS_SECTION,
                TABLES_SECTION,
                OUTPUT_SECTION
            ]
        );
        assert!(
            dashboard
                .key_bindings()
                .iter()
                .any(|binding| binding.chord() == EXECUTE_CHORD),
        );
        assert_eq!(dashboard.evaluate_target_section(), Some(OUTPUT_SECTION));
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
        assert_eq!(
            items[0].line(),
            format!("  {}  users", crate::icon_font::symbols::cod::COD_TABLE)
        );
        assert!(matches!(
            items[0].action(),
            Some(DbActionSpec::OpenTablePreview { session_id: 7, .. })
        ));
    }
}
