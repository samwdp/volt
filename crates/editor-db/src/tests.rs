
use super::*;
use crate::secrets::InMemorySecretStore;
use crate::types::DbSession;
use editor_plugin_api::{DbBrowserItemKind, DbBrowserItemSpec};
use rusqlite::Connection as SqliteConnection;
use serde_json::json;
use std::{env, fs, path::PathBuf, sync::Arc};

fn temp_state_dir(name: &str) -> PathBuf {
    env::temp_dir().join(format!("volt-editor-db-{name}-{}", unique_suffix()))
}

fn test_service() -> DbService {
    DbService::new_with_secret_store(
        temp_state_dir("service"),
        Arc::new(InMemorySecretStore::default()),
    )
    .expect("test DB service should initialize")
}

fn insert_test_session(
    service: &mut DbService,
    session_id: DbSessionId,
    alias: &str,
    engine: DbEngine,
    connection_string: &str,
) {
    service.sessions.insert(
        session_id,
        DbSession {
            id: session_id,
            alias: alias.to_owned(),
            engine,
            display_label: alias.to_owned(),
            database_name: None,
            host_label: None,
            connection_string: connection_string.to_owned(),
            remembered_secret_ref: None,
            schema_cache: DbSchemaCache::default(),
        },
    );
}

#[test]
fn parse_connect_prompt_supports_session_only_and_remembered_formats() {
    let (alias, connection_string) =
        parse_db_connect_prompt("postgres://localhost/app").expect("session-only prompt");
    assert_eq!(alias, None);
    assert_eq!(connection_string, "postgres://localhost/app");

    let (alias, connection_string) = parse_db_connect_prompt(
        "remember prod :: Server=tcp:db.example.com,1433;Database=app;User ID=sa;Password=secret",
    )
    .expect("remember prompt");
    assert_eq!(alias.as_deref(), Some("prod"));
    assert!(connection_string.contains("Server=tcp:db.example.com,1433"));
}

#[test]
fn connection_descriptor_detects_all_supported_engines() {
    let sqlite = ConnectionDescriptor::from_connection_string("sqlite://C:/data/app.db")
        .expect("sqlite descriptor");
    assert_eq!(sqlite.engine, DbEngine::Sqlite);

    let postgres =
        ConnectionDescriptor::from_connection_string("postgres://volt:secret@localhost:5432/app")
            .expect("postgres descriptor");
    assert_eq!(postgres.engine, DbEngine::Postgres);

    let sql_server = ConnectionDescriptor::from_connection_string(
            "Server=tcp:db.example.com,1433;Database=app;User ID=sa;Password=secret;TrustServerCertificate=true",
        )
        .expect("sql server descriptor");
    assert_eq!(sql_server.engine, DbEngine::SqlServer);
}

#[test]
fn sql_scope_prefers_selection_and_falls_back_to_current_statement() {
    let source = "select * from one;\nselect * from two;\n";
    let selected = sql_scope_from_text(source, 4, Some((0, 17))).expect("selected SQL");
    assert_eq!(selected, "select * from one");

    let current = sql_scope_from_text(source, 22, None).expect("current statement");
    assert_eq!(current, "select * from two");
}

#[test]
fn split_sql_statements_skips_semicolons_inside_quotes_and_comments() {
    let source = "select 'a;b' from t; -- c; d\nupdate t set x = 1;\nselect 2";
    assert_eq!(
        split_sql_statements(source),
        vec![
            "select 'a;b' from t".to_owned(),
            "update t set x = 1".to_owned(),
            "select 2".to_owned(),
        ]
    );
}

#[test]
fn sqls_workspace_settings_preserve_mssql_data_source_name() {
    let mut service = test_service();
    let session_id = DbSessionId(7);
    let connection_string = "Data Source=assetfusion.database.windows.net;Initial Catalog=assetfusion;Integrated Security=False;User ID=assetfusion_admin;Password=secret;Connect Timeout=60;Encrypt=True;TrustServerCertificate=False;ApplicationIntent=ReadWrite;MultiSubnetFailover=False";
    insert_test_session(
        &mut service,
        session_id,
        "assetfusion",
        DbEngine::SqlServer,
        connection_string,
    );
    service.active_session_id = Some(session_id);

    assert_eq!(
        service.sqls_workspace_settings_for_active_session(),
        Some(json!({
            "connections": [
                {
                    "alias": "assetfusion",
                    "driver": "mssql",
                    "dataSourceName": connection_string,
                }
            ]
        }))
    );
}

#[test]
fn sqls_workspace_settings_for_query_buffer_use_attached_session() {
    let mut service = test_service();
    let active_session_id = DbSessionId(1);
    let attached_session_id = DbSessionId(2);
    insert_test_session(
        &mut service,
        active_session_id,
        "sqlite-main",
        DbEngine::Sqlite,
        "sqlite://main.db",
    );
    insert_test_session(
        &mut service,
        attached_session_id,
        "warehouse",
        DbEngine::Postgres,
        "postgres://volt:secret@localhost:5432/warehouse",
    );
    service.active_session_id = Some(active_session_id);
    service.query_buffers.insert(
        42,
        DbQueryBufferMeta {
            session_id: attached_session_id,
            engine: DbEngine::Postgres,
            dialect_id: "sql".to_owned(),
            temp_path: service.query_dir.join("warehouse.sql"),
            title: "*db-query warehouse*".to_owned(),
        },
    );

    assert_eq!(
        service.sqls_workspace_settings_for_query_buffer(42),
        Some(json!({
            "connections": [
                {
                    "alias": "warehouse",
                    "driver": "postgresql",
                    "dataSourceName": "postgres://volt:secret@localhost:5432/warehouse",
                }
            ]
        }))
    );
}

#[test]
fn sqls_initialization_options_for_query_buffer_use_attached_session() {
    let mut service = test_service();
    let active_session_id = DbSessionId(1);
    let attached_session_id = DbSessionId(2);
    insert_test_session(
        &mut service,
        active_session_id,
        "sqlite-main",
        DbEngine::Sqlite,
        "sqlite://main.db",
    );
    insert_test_session(
        &mut service,
        attached_session_id,
        "warehouse",
        DbEngine::Postgres,
        "postgres://volt:secret@localhost:5432/warehouse",
    );
    service.active_session_id = Some(active_session_id);
    service.query_buffers.insert(
        42,
        DbQueryBufferMeta {
            session_id: attached_session_id,
            engine: DbEngine::Postgres,
            dialect_id: "sql".to_owned(),
            temp_path: service.query_dir.join("warehouse.sql"),
            title: "*db-query warehouse*".to_owned(),
        },
    );

    assert_eq!(
        service.sqls_initialization_options_for_query_buffer(42),
        Some(json!({
            "connectionConfig": {
                "alias": "warehouse",
                "driver": "postgresql",
                "dataSourceName": "postgres://volt:secret@localhost:5432/warehouse",
            }
        }))
    );
}

#[test]
fn db_browser_renderer_customizes_rows_and_preserves_actions() {
    let mut service = test_service();
    let session_id = DbSessionId(7);
    insert_test_session(
        &mut service,
        session_id,
        "local",
        DbEngine::Sqlite,
        "sqlite://local.db",
    );
    service.active_session_id = Some(session_id);

    let view = service
        .render_connections_buffer_with(42, &|context| {
            context
                .items
                .iter()
                .map(|item| {
                    let line = if item.kind == DbBrowserItemKind::ActiveConnection {
                        format!("custom {}", item.label)
                    } else {
                        item.label.to_string()
                    };
                    DbBrowserItemSpec::new(line, item.default_action.clone().into())
                })
                .collect()
        })
        .expect("connections render");

    assert!(view.lines.iter().any(|line| line == "custom local"));
    let line_index = view
        .lines
        .iter()
        .position(|line| line == "custom local")
        .expect("custom line");
    assert!(matches!(
        service.browser_action(42, line_index),
        Some(DbBrowserAction::DisconnectSession { session_id: id }) if id == session_id
    ));
}

#[test]
fn db_browser_renderer_rejects_row_count_mismatch() {
    let mut service = test_service();
    let error = service
        .render_connections_buffer_with(42, &|_| Vec::new())
        .expect_err("mismatched renderer should fail");
    assert!(error.contains("database browser renderer returned"));
}

#[test]
fn render_rows_draws_boxed_table_and_right_aligns_numbers() {
    let output = render_rows(
        "SQLite result",
        &["id".to_owned(), "name".to_owned()],
        &[
            vec!["1".to_owned(), "Ada".to_owned()],
            vec!["12".to_owned(), "Grace".to_owned()],
        ],
    );
    assert_eq!(
        output.lines,
        vec![
            "┌────┬───────┐".to_owned(),
            "│ id │ name  │".to_owned(),
            "├────┼───────┤".to_owned(),
            "│  1 │ Ada   │".to_owned(),
            "│ 12 │ Grace │".to_owned(),
            "└────┴───────┘".to_owned(),
            String::new(),
            "2 rows  ·  2 columns".to_owned(),
        ]
    );
    assert_eq!(output.row_count, 2);
}

#[test]
fn sqlite_query_execution_and_schema_cache_work() {
    let state_dir = temp_state_dir("sqlite");
    let db_path = state_dir.join("app.db");
    fs::create_dir_all(&state_dir).expect("state dir");
    let connection = SqliteConnection::open(&db_path).expect("sqlite open");
    connection
        .execute_batch(
            "CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT NOT NULL);\n\
                 CREATE INDEX idx_users_name ON users(name);\n\
                 INSERT INTO users(name) VALUES ('Ada'), ('Grace');",
        )
        .expect("seed sqlite");
    let mut service =
        DbService::new_with_secret_store(state_dir, Arc::new(InMemorySecretStore::default()))
            .expect("service");
    let session = service
        .connect_raw(&format!("sqlite://{}", db_path.display()), Some("local"))
        .expect("connect sqlite");
    assert_eq!(session.engine, DbEngine::Sqlite);

    let schema_view = service
        .render_schema_buffer(10, Some(session.id))
        .expect("schema view");
    assert!(schema_view.lines.iter().any(|line| line.contains("users")));
    assert!(
        schema_view
            .lines
            .iter()
            .any(|line| line.contains("idx_users_name"))
    );

    let query_meta = service
        .attach_query_buffer(20, Some(session.id), None)
        .expect("query");
    assert_eq!(query_meta.engine, DbEngine::Sqlite);
    let result = service
        .execute_sql_for_buffer(20, "SELECT name FROM users ORDER BY id;")
        .expect("execute sqlite");
    assert!(result.lines.iter().any(|line| line.contains("Ada")));
    assert!(result.lines.iter().any(|line| line.contains("Grace")));

    let candidates = service.autocomplete_candidates_for_buffer(20);
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.replacement == "users")
    );
    assert!(
        candidates
            .iter()
            .any(|candidate| candidate.replacement == "name")
    );
}

#[test]
fn sqlite_batch_execution_concatenates_query_results() {
    let state_dir = temp_state_dir("sqlite-batch");
    let db_path = state_dir.join("app.db");
    fs::create_dir_all(&state_dir).expect("state dir");
    let connection = SqliteConnection::open(&db_path).expect("sqlite open");
    connection
        .execute_batch(
            "CREATE TABLE users(id INTEGER PRIMARY KEY, name TEXT NOT NULL);\n\
                 INSERT INTO users(name) VALUES ('Ada'), ('Grace');",
        )
        .expect("seed sqlite");
    let mut service =
        DbService::new_with_secret_store(state_dir, Arc::new(InMemorySecretStore::default()))
            .expect("service");
    let session = service
        .connect_raw(&format!("sqlite://{}", db_path.display()), Some("local"))
        .expect("connect sqlite");
    service
        .attach_query_buffer(20, Some(session.id), None)
        .expect("query");
    let result = service
        .execute_sql_batch_for_buffer(
            20,
            "SELECT name FROM users WHERE id = 1;\nSELECT name FROM users WHERE id = 2;",
        )
        .expect("batch execute");
    assert!(result.lines.iter().any(|line| line.contains("-- Query 1")));
    assert!(result.lines.iter().any(|line| line.contains("-- Query 2")));
    assert!(result.lines.iter().any(|line| line.contains("Ada")));
    assert!(result.lines.iter().any(|line| line.contains("Grace")));
}

#[test]
fn remembered_connections_store_metadata_separately_from_secret() {
    let mut service = test_service();
    let session = service
        .connect_raw("sqlite://memory.db", Some("memdb"))
        .expect("remember sqlite");
    assert_eq!(session.alias, "memdb");
    assert_eq!(service.persisted.remembered.len(), 1);
    let remembered = &service.persisted.remembered[0];
    assert_eq!(remembered.alias, "memdb");
    assert!(remembered.secret_ref.starts_with("db-memdb-"));
    let metadata_json = fs::read_to_string(&service.state_path).expect("read state");
    assert!(!metadata_json.contains("sqlite://memory.db"));
    let secret = service
        .secret_store
        .get_secret(&remembered.secret_ref)
        .expect("secret");
    assert_eq!(secret, "sqlite://memory.db");
}

#[test]
fn snippets_and_history_persist() {
    let mut service = test_service();
    let session = service
        .connect_raw("sqlite://history.db", Some("history"))
        .expect("remember sqlite");
    service
        .attach_query_buffer(33, Some(session.id), None)
        .expect("query");
    service
        .save_snippet(33, "List users", "SELECT * FROM users;")
        .expect("save snippet");
    let snippets = service.render_snippets_buffer(44).expect("render snippets");
    assert!(
        snippets
            .lines
            .iter()
            .any(|line| line.contains("List users"))
    );

    let session_record = service
        .sessions
        .get(&session.id)
        .expect("session exists")
        .clone();
    service
        .record_history(&session_record, "SELECT * FROM users;")
        .expect("history");
    let history = service.render_history_buffer(55).expect("render history");
    assert!(
        history
            .lines
            .iter()
            .any(|line| line.contains("SELECT * FROM users"))
    );
}
