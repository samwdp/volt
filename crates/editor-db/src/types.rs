#![allow(unused_imports)]
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    env, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(test)]
use std::sync::Mutex;

use editor_plugin_api::{
    DbActionSpec, DbBrowserContext, DbBrowserItemContext, DbBrowserItemKind, DbBrowserItemSpec,
    DbBrowserKind,
};
use keyring_core::Entry;
use postgres::{Client as PostgresClient, NoTls, SimpleQueryMessage};
use rusqlite::{Connection as SqliteConnection, types::ValueRef as SqliteValueRef};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tiberius::{Client as SqlServerClient, Config as SqlServerConfig, Row as SqlServerRow};
use tokio::{net::TcpStream, runtime::Runtime};
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[allow(unused_imports)]
use crate::connection::*;
#[allow(unused_imports)]
use crate::engines::*;
#[allow(unused_imports)]
use crate::secrets::*;
#[allow(unused_imports)]
use crate::service::*;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Database sessions, secure remembered connections, schema browsing, and SQL execution.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Stable session identifier for active in-memory database sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DbSessionId(pub(crate) u64);

impl DbSessionId {
    /// Returns the raw numeric identifier.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Supported SQL engines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DbEngine {
    Sqlite,
    Postgres,
    SqlServer,
}

impl DbEngine {
    /// Returns a short user-facing label for the engine.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sqlite => "SQLite",
            Self::Postgres => "PostgreSQL",
            Self::SqlServer => "SQL Server",
        }
    }

    pub(crate) fn dialect_id(self) -> &'static str {
        "sql"
    }

    pub(crate) fn sqls_driver(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite3",
            Self::Postgres => "postgresql",
            Self::SqlServer => "mssql",
        }
    }

    pub(crate) fn preview_sql(self, table: &QualifiedName) -> String {
        let qualified = table.render(self);
        match self {
            Self::SqlServer => format!("SELECT TOP 100 *\nFROM {qualified};"),
            Self::Sqlite | Self::Postgres => {
                format!("SELECT *\nFROM {qualified}\nLIMIT 100;")
            }
        }
    }
}

/// One remembered connection without the underlying secret material.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RememberedConnection {
    pub alias: String,
    pub engine: DbEngine,
    pub display_label: String,
    pub database_name: Option<String>,
    pub host_label: Option<String>,
    pub last_used_epoch_secs: u64,
    pub secret_ref: String,
}

/// Summary of one active session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbSessionSummary {
    pub id: DbSessionId,
    pub alias: String,
    pub engine: DbEngine,
    pub display_label: String,
    pub database_name: Option<String>,
    pub host_label: Option<String>,
    pub remembered: bool,
    pub active: bool,
}

/// Metadata attached to a DB query buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbQueryBufferMeta {
    pub session_id: DbSessionId,
    pub engine: DbEngine,
    pub dialect_id: String,
    pub temp_path: PathBuf,
    pub title: String,
}

/// Result of executing SQL against an attached query buffer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbExecutionOutput {
    pub title: String,
    pub lines: Vec<String>,
    pub row_count: usize,
}

/// One completion candidate sourced from the active DB schema cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbAutocompleteCandidate {
    pub label: String,
    pub replacement: String,
    pub detail: Option<String>,
    pub documentation: Option<String>,
}

/// Action attached to one rendered browser buffer line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbBrowserAction {
    ActivateRemembered {
        alias: String,
    },
    DisconnectSession {
        session_id: DbSessionId,
    },
    OpenTablePreview {
        session_id: DbSessionId,
        table: QualifiedName,
    },
    ExploreRows {
        session_id: DbSessionId,
        table: QualifiedName,
    },
    RefreshSchema {
        session_id: DbSessionId,
    },
    OpenHistoryEntry {
        sql: String,
        session_id: DbSessionId,
    },
    OpenSnippet {
        id: String,
    },
    DeleteSnippet {
        id: String,
    },
    DeleteRemembered {
        alias: String,
    },
}

/// One rendered browser buffer snapshot with line-local actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbBrowserBufferView {
    pub title: String,
    pub lines: Vec<String>,
    pub actions_by_line: Vec<Option<DbBrowserAction>>,
    pub kinds_by_line: Vec<DbBrowserItemKind>,
}

pub type DbBrowserItemRenderer<'a> = dyn Fn(&DbBrowserContext) -> Vec<DbBrowserItemSpec> + 'a;

pub(crate) fn default_db_browser_items(context: &DbBrowserContext) -> Vec<DbBrowserItemSpec> {
    context
        .items
        .iter()
        .map(|item| {
            DbBrowserItemSpec::new(
                default_db_browser_line(item),
                item.default_action.clone().into(),
            )
        })
        .collect()
}

pub(crate) fn default_db_browser_line(item: &DbBrowserItemContext) -> String {
    match item.kind {
        DbBrowserItemKind::Header | DbBrowserItemKind::Empty => item.label.to_string(),
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
    }
}

pub(crate) fn section_count_label(label: &str, count: usize) -> String {
    format!("{label} ({count})")
}

pub(crate) fn push_schema_column_items(
    items: &mut Vec<DbBrowserItemContext>,
    columns: &[DbColumn],
) {
    let name_width = columns
        .iter()
        .map(|column| column.name.chars().count())
        .max()
        .unwrap_or(0);
    for column in columns {
        let nullable = if column.nullable { "  · nullable" } else { "" };
        items.push(DbBrowserItemContext::new(
            DbBrowserItemKind::Header,
            format!(
                "      {:<width$}  {}{nullable}",
                column.name,
                column.data_type,
                width = name_width
            ),
        ));
    }
}

/// Stored snippet metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbSnippet {
    pub id: String,
    pub name: String,
    pub sql: String,
    pub engine: DbEngine,
    pub created_at_epoch_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DbHistoryEntry {
    pub(crate) session_alias: String,
    pub(crate) engine: DbEngine,
    pub(crate) sql: String,
    pub(crate) ran_at_epoch_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub(crate) struct PersistedDbState {
    pub(crate) remembered: Vec<RememberedConnection>,
    pub(crate) snippets: Vec<DbSnippet>,
    pub(crate) history: Vec<DbHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum DbBrowserBufferKind {
    Connections,
    Schema { session_id: Option<DbSessionId> },
    History,
    Snippets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbBrowserBufferState {
    pub(crate) kind: DbBrowserBufferKind,
    pub(crate) title: String,
    pub(crate) actions_by_line: Vec<Option<DbBrowserAction>>,
}

#[derive(Debug, Clone)]
pub(crate) struct DbSession {
    pub(crate) id: DbSessionId,
    pub(crate) alias: String,
    pub(crate) engine: DbEngine,
    pub(crate) display_label: String,
    pub(crate) database_name: Option<String>,
    pub(crate) host_label: Option<String>,
    pub(crate) connection_string: String,
    pub(crate) remembered_secret_ref: Option<String>,
    pub(crate) schema_cache: DbSchemaCache,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct DbSchemaCache {
    pub(crate) tables: Vec<DbTable>,
    pub(crate) views: Vec<DbTable>,
    pub(crate) indexes: Vec<DbIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbTable {
    pub(crate) name: QualifiedName,
    pub(crate) columns: Vec<DbColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbColumn {
    pub(crate) name: String,
    pub(crate) data_type: String,
    pub(crate) nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DbIndex {
    pub(crate) name: String,
    pub(crate) table: QualifiedName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub schema: Option<String>,
    pub name: String,
}

impl QualifiedName {
    pub fn render(&self, engine: DbEngine) -> String {
        match (&self.schema, engine) {
            (Some(schema), DbEngine::SqlServer) => {
                format!(
                    "[{}].[{}]",
                    escape_bracket(schema),
                    escape_bracket(&self.name)
                )
            }
            (Some(schema), _) => {
                format!(
                    "\"{}\".\"{}\"",
                    escape_double_quote(schema),
                    escape_double_quote(&self.name)
                )
            }
            (None, DbEngine::SqlServer) => format!("[{}]", escape_bracket(&self.name)),
            (None, _) => format!("\"{}\"", escape_double_quote(&self.name)),
        }
    }

    /// Human-readable `schema.name` form used in titles and autocomplete.
    pub fn display(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{schema}.{}", self.name),
            None => self.name.clone(),
        }
    }
}

pub(crate) fn db_browser_action_from_spec(spec: &DbActionSpec) -> Result<DbBrowserAction, String> {
    match spec {
        DbActionSpec::ActivateRemembered { alias } => Ok(DbBrowserAction::ActivateRemembered {
            alias: alias.to_string(),
        }),
        DbActionSpec::DisconnectSession { session_id } => Ok(DbBrowserAction::DisconnectSession {
            session_id: DbSessionId(*session_id),
        }),
        DbActionSpec::OpenTablePreview {
            session_id,
            schema,
            table,
        } => Ok(DbBrowserAction::OpenTablePreview {
            session_id: DbSessionId(*session_id),
            table: qualified_name_from_spec(
                schema
                    .clone()
                    .into_option()
                    .map(|schema| schema.into_string()),
                table.to_string(),
            )?,
        }),
        DbActionSpec::ExploreRows {
            session_id,
            schema,
            table,
        } => Ok(DbBrowserAction::ExploreRows {
            session_id: DbSessionId(*session_id),
            table: qualified_name_from_spec(
                schema
                    .clone()
                    .into_option()
                    .map(|schema| schema.into_string()),
                table.to_string(),
            )?,
        }),
        DbActionSpec::RefreshSchema { session_id } => Ok(DbBrowserAction::RefreshSchema {
            session_id: DbSessionId(*session_id),
        }),
        DbActionSpec::OpenHistoryEntry { session_id, sql } => {
            Ok(DbBrowserAction::OpenHistoryEntry {
                session_id: DbSessionId(*session_id),
                sql: sql.to_string(),
            })
        }
        DbActionSpec::OpenSnippet { id } => Ok(DbBrowserAction::OpenSnippet { id: id.to_string() }),
        DbActionSpec::DeleteSnippet { id } => {
            Ok(DbBrowserAction::DeleteSnippet { id: id.to_string() })
        }
        DbActionSpec::DeleteRemembered { alias } => Ok(DbBrowserAction::DeleteRemembered {
            alias: alias.to_string(),
        }),
    }
}

pub(crate) fn qualified_name_from_spec(
    schema: Option<String>,
    name: String,
) -> Result<QualifiedName, String> {
    if name.trim().is_empty() {
        return Err("database browser table action requires a table name".to_owned());
    }
    Ok(QualifiedName { schema, name })
}

impl DbEngine {
    pub(crate) fn load_schema(self, connection_string: &str) -> Result<DbSchemaCache, String> {
        match self {
            Self::Sqlite => load_sqlite_schema(connection_string),
            Self::Postgres => load_postgres_schema(connection_string),
            Self::SqlServer => load_sql_server_schema(connection_string),
        }
    }

    pub(crate) fn execute_sql(
        self,
        connection_string: &str,
        sql: &str,
    ) -> Result<DbExecutionOutput, String> {
        match self {
            Self::Sqlite => execute_sqlite(connection_string, sql),
            Self::Postgres => execute_postgres(connection_string, sql),
            Self::SqlServer => execute_sql_server(connection_string, sql),
        }
    }
}

pub(crate) fn load_sqlite_schema(connection_string: &str) -> Result<DbSchemaCache, String> {
    let path = sqlite_path(connection_string);
    let connection = SqliteConnection::open(&path)
        .map_err(|error| format!("failed to open SQLite database `{path}`: {error}"))?;
    let mut cache = DbSchemaCache::default();
    let mut statement = connection
        .prepare(
            "SELECT type, name FROM sqlite_master \
             WHERE type IN ('table', 'view', 'index') \
               AND name NOT LIKE 'sqlite_%' \
             ORDER BY type, name",
        )
        .map_err(|error| format!("failed to introspect SQLite schema: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            let object_type: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((object_type, name))
        })
        .map_err(|error| format!("failed to read SQLite schema objects: {error}"))?;
    for row in rows {
        let (object_type, name) =
            row.map_err(|error| format!("failed to parse SQLite schema row: {error}"))?;
        match object_type.as_str() {
            "table" => cache.tables.push(DbTable {
                name: QualifiedName {
                    schema: None,
                    name: name.clone(),
                },
                columns: load_sqlite_columns(&connection, &name)?,
            }),
            "view" => cache.views.push(DbTable {
                name: QualifiedName {
                    schema: None,
                    name: name.clone(),
                },
                columns: load_sqlite_columns(&connection, &name)?,
            }),
            "index" => cache.indexes.push(DbIndex {
                name: name.clone(),
                table: QualifiedName {
                    schema: None,
                    name: sqlite_index_table(&connection, &name)?.unwrap_or_else(|| name.clone()),
                },
            }),
            _ => {}
        }
    }
    Ok(cache)
}

pub(crate) fn load_sqlite_columns(
    connection: &SqliteConnection,
    table_name: &str,
) -> Result<Vec<DbColumn>, String> {
    let pragma = format!("PRAGMA table_info({})", quote_sqlite_identifier(table_name));
    let mut statement = connection
        .prepare(&pragma)
        .map_err(|error| format!("failed to inspect SQLite columns for `{table_name}`: {error}"))?;
    let rows = statement
        .query_map([], |row| {
            let name: String = row.get(1)?;
            let data_type: String = row.get(2)?;
            let nullable: i64 = row.get(3)?;
            Ok(DbColumn {
                name,
                data_type,
                nullable: nullable == 0,
            })
        })
        .map_err(|error| format!("failed to read SQLite columns for `{table_name}`: {error}"))?;
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("failed to parse SQLite column metadata: {error}"))
}

pub(crate) fn load_postgres_schema(connection_string: &str) -> Result<DbSchemaCache, String> {
    let mut client = PostgresClient::connect(connection_string, NoTls)
        .map_err(|error| format!("failed to connect to PostgreSQL: {error}"))?;
    let mut cache = DbSchemaCache::default();
    let tables = client
        .query(
            "SELECT table_schema, table_name, table_type \
             FROM information_schema.tables \
             WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
             ORDER BY table_schema, table_name",
            &[],
        )
        .map_err(|error| format!("failed to introspect PostgreSQL tables: {error}"))?;
    let columns = postgres_columns_by_table(&mut client)?;
    let indexes = client
        .query(
            "SELECT schemaname, tablename, indexname \
             FROM pg_indexes \
             WHERE schemaname NOT IN ('pg_catalog', 'information_schema') \
             ORDER BY schemaname, tablename, indexname",
            &[],
        )
        .map_err(|error| format!("failed to introspect PostgreSQL indexes: {error}"))?;
    for row in tables {
        let schema: String = row.get(0);
        let name: String = row.get(1);
        let table_type: String = row.get(2);
        let qualified = QualifiedName {
            schema: Some(schema.clone()),
            name: name.clone(),
        };
        let columns = columns
            .get(&(schema.clone(), name.clone()))
            .cloned()
            .unwrap_or_default();
        match table_type.as_str() {
            "VIEW" => cache.views.push(DbTable {
                name: qualified,
                columns,
            }),
            _ => cache.tables.push(DbTable {
                name: qualified,
                columns,
            }),
        }
    }
    for row in indexes {
        cache.indexes.push(DbIndex {
            name: row.get::<_, String>(2),
            table: QualifiedName {
                schema: Some(row.get(0)),
                name: row.get(1),
            },
        });
    }
    Ok(cache)
}

pub(crate) fn load_sql_server_schema(connection_string: &str) -> Result<DbSchemaCache, String> {
    let runtime = build_tokio_runtime()?;
    runtime.block_on(async move {
        let mut client = connect_sql_server(connection_string).await?;
        let table_rows = client
            .simple_query(
                "SELECT TABLE_SCHEMA, TABLE_NAME, TABLE_TYPE \
                 FROM INFORMATION_SCHEMA.TABLES \
                 WHERE TABLE_SCHEMA NOT IN ('INFORMATION_SCHEMA', 'sys') \
                 ORDER BY TABLE_SCHEMA, TABLE_NAME",
            )
            .await
            .map_err(|error| format!("failed to introspect SQL Server tables: {error}"))?
            .into_first_result()
            .await
            .map_err(|error| format!("failed to read SQL Server tables: {error}"))?;
        let column_rows = client
            .simple_query(
                "SELECT TABLE_SCHEMA, TABLE_NAME, COLUMN_NAME, DATA_TYPE, IS_NULLABLE \
                 FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA NOT IN ('INFORMATION_SCHEMA', 'sys') \
                 ORDER BY TABLE_SCHEMA, TABLE_NAME, ORDINAL_POSITION",
            )
            .await
            .map_err(|error| format!("failed to introspect SQL Server columns: {error}"))?
            .into_first_result()
            .await
            .map_err(|error| format!("failed to read SQL Server columns: {error}"))?;
        let index_rows = client
            .simple_query(
                "SELECT s.name AS schema_name, t.name AS table_name, i.name AS index_name \
                 FROM sys.indexes i \
                 JOIN sys.tables t ON i.object_id = t.object_id \
                 JOIN sys.schemas s ON t.schema_id = s.schema_id \
                 WHERE i.name IS NOT NULL \
                 ORDER BY s.name, t.name, i.name",
            )
            .await
            .map_err(|error| format!("failed to introspect SQL Server indexes: {error}"))?
            .into_first_result()
            .await
            .map_err(|error| format!("failed to read SQL Server indexes: {error}"))?;
        let columns = sql_server_columns_by_table(&column_rows);
        let mut cache = DbSchemaCache::default();
        for row in table_rows {
            let schema = sql_server_cell(&row, "TABLE_SCHEMA");
            let name = sql_server_cell(&row, "TABLE_NAME");
            let table_type = sql_server_cell(&row, "TABLE_TYPE");
            let qualified = QualifiedName {
                schema: Some(schema.clone()),
                name: name.clone(),
            };
            let columns = columns
                .get(&(schema.clone(), name.clone()))
                .cloned()
                .unwrap_or_default();
            if table_type.eq_ignore_ascii_case("VIEW") {
                cache.views.push(DbTable {
                    name: qualified,
                    columns,
                });
            } else {
                cache.tables.push(DbTable {
                    name: qualified,
                    columns,
                });
            }
        }
        for row in index_rows {
            cache.indexes.push(DbIndex {
                name: sql_server_cell(&row, "index_name"),
                table: QualifiedName {
                    schema: Some(sql_server_cell(&row, "schema_name")),
                    name: sql_server_cell(&row, "table_name"),
                },
            });
        }
        Ok(cache)
    })
}

pub(crate) fn sql_server_columns_by_table(
    rows: &[SqlServerRow],
) -> BTreeMap<(String, String), Vec<DbColumn>> {
    let mut result = BTreeMap::new();
    for row in rows {
        let key = (
            sql_server_cell(row, "TABLE_SCHEMA"),
            sql_server_cell(row, "TABLE_NAME"),
        );
        result.entry(key).or_insert_with(Vec::new).push(DbColumn {
            name: sql_server_cell(row, "COLUMN_NAME"),
            data_type: sql_server_cell(row, "DATA_TYPE"),
            nullable: sql_server_cell(row, "IS_NULLABLE").eq_ignore_ascii_case("YES"),
        });
    }
    result
}

pub(crate) fn build_tokio_runtime() -> Result<Runtime, String> {
    Runtime::new().map_err(|error| format!("failed to create Tokio runtime: {error}"))
}

pub(crate) fn render_rows(
    title: &str,
    headers: &[String],
    rows: &[Vec<String>],
) -> DbExecutionOutput {
    if headers.is_empty() {
        return execution_notice(title, "No columns returned.", 0);
    }
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|row| {
            (0..headers.len())
                .map(|index| display_cell(row.get(index).map(String::as_str).unwrap_or("")))
                .collect()
        })
        .collect();
    let headers = headers
        .iter()
        .map(|header| display_cell(header))
        .collect::<Vec<_>>();
    let mut widths = headers
        .iter()
        .map(|header| header.chars().count())
        .collect::<Vec<_>>();
    for row in &cells {
        for (index, value) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(value.chars().count());
            }
        }
    }
    let aligns = (0..headers.len())
        .map(|index| {
            if column_is_numeric(&cells, index) {
                CellAlign::Right
            } else {
                CellAlign::Left
            }
        })
        .collect::<Vec<_>>();
    let mut lines = Vec::new();
    lines.push(box_rule(&widths, BoxRuleKind::Top));
    lines.push(box_row(
        &headers,
        &widths,
        &vec![CellAlign::Left; headers.len()],
    ));
    lines.push(box_rule(&widths, BoxRuleKind::Middle));
    for row in &cells {
        lines.push(box_row(row, &widths, &aligns));
    }
    lines.push(box_rule(&widths, BoxRuleKind::Bottom));
    lines.push(String::new());
    lines.push(row_count_footer(cells.len(), headers.len()));
    DbExecutionOutput {
        title: title.to_owned(),
        lines,
        row_count: cells.len(),
    }
}

pub(crate) fn rows_affected_line(count: usize) -> String {
    match count {
        0 => "No rows affected.".to_owned(),
        1 => "1 row affected.".to_owned(),
        n => format!("{n} rows affected."),
    }
}

pub(crate) fn row_count_footer(rows: usize, columns: usize) -> String {
    let row_label = match rows {
        1 => "1 row".to_owned(),
        n => format!("{n} rows"),
    };
    let column_label = match columns {
        1 => "1 column".to_owned(),
        n => format!("{n} columns"),
    };
    format!("{row_label}  ·  {column_label}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CellAlign {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BoxRuleKind {
    Top,
    Middle,
    Bottom,
}

pub(crate) fn box_rule(widths: &[usize], kind: BoxRuleKind) -> String {
    let (left, join, right) = match kind {
        BoxRuleKind::Top => ('┌', '┬', '┐'),
        BoxRuleKind::Middle => ('├', '┼', '┤'),
        BoxRuleKind::Bottom => ('└', '┴', '┘'),
    };
    let mut line = String::new();
    line.push(left);
    for (index, width) in widths.iter().enumerate() {
        if index > 0 {
            line.push(join);
        }
        line.push_str(&"─".repeat(width.saturating_add(2)));
    }
    line.push(right);
    line
}

pub(crate) fn box_row(values: &[String], widths: &[usize], aligns: &[CellAlign]) -> String {
    let mut line = String::from("│");
    for (index, width) in widths.iter().enumerate() {
        let value = values.get(index).map(String::as_str).unwrap_or("");
        let align = aligns.get(index).copied().unwrap_or(CellAlign::Left);
        line.push(' ');
        line.push_str(&pad_cell(value, *width, align));
        line.push(' ');
        line.push('│');
    }
    line
}

pub(crate) fn pad_cell(value: &str, width: usize, align: CellAlign) -> String {
    let pad = width.saturating_sub(value.chars().count());
    match align {
        CellAlign::Left => format!("{value}{}", " ".repeat(pad)),
        CellAlign::Right => format!("{}{value}", " ".repeat(pad)),
    }
}

pub(crate) fn display_cell(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\n' | '\r' | '\t' => ' ',
            other => other,
        })
        .collect()
}

pub(crate) fn column_is_numeric(rows: &[Vec<String>], index: usize) -> bool {
    let mut saw_number = false;
    for row in rows {
        let Some(value) = row.get(index) else {
            continue;
        };
        let trimmed = value.trim();
        if trimmed.is_empty() || trimmed == "NULL" {
            continue;
        }
        if !is_numeric_cell(trimmed) {
            return false;
        }
        saw_number = true;
    }
    saw_number
}

pub(crate) fn is_numeric_cell(value: &str) -> bool {
    let mut seen_digit = false;
    let mut seen_dot = false;
    for (index, character) in value.chars().enumerate() {
        match character {
            '0'..='9' => seen_digit = true,
            '+' | '-' if index == 0 => {}
            '.' if !seen_dot => seen_dot = true,
            _ => return false,
        }
    }
    seen_digit
}

pub(crate) fn sql_server_row_values(row: &SqlServerRow) -> Vec<String> {
    row.cells()
        .map(|(_, value)| sql_server_column_data_to_string(value))
        .collect()
}

pub(crate) fn sql_server_cell(row: &SqlServerRow, column: &str) -> String {
    row.try_get::<&str, _>(column)
        .ok()
        .flatten()
        .map(str::to_owned)
        .or_else(|| {
            row.try_get::<i32, _>(column)
                .ok()
                .flatten()
                .map(|value| value.to_string())
        })
        .or_else(|| {
            row.try_get::<i64, _>(column)
                .ok()
                .flatten()
                .map(|value| value.to_string())
        })
        .unwrap_or_default()
}

pub(crate) fn sql_server_column_data_to_string(value: &tiberius::ColumnData<'static>) -> String {
    match value {
        tiberius::ColumnData::U8(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::I16(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::I32(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::I64(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::F32(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::F64(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::Bit(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::String(value) => {
            value.as_ref().map(|value| value.as_ref().to_owned())
        }
        tiberius::ColumnData::Guid(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::Binary(value) => value
            .as_ref()
            .map(|value| format!("0x{}", hex_string(value))),
        tiberius::ColumnData::Numeric(value) => value.map(|value| value.to_string()),
        tiberius::ColumnData::Xml(value) => value.as_ref().map(|value| value.to_string()),
        other => Some(format!("{other:?}")),
    }
    .unwrap_or_else(|| "NULL".to_owned())
}

pub(crate) fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join("")
}

/// Extracts selected SQL when `selection` is present; otherwise returns the current statement.
pub fn sql_scope_from_text(
    text: &str,
    cursor_char_index: usize,
    selection: Option<(usize, usize)>,
) -> Option<String> {
    if let Some((start, end)) = selection {
        let start = start.min(text.len());
        let end = end.min(text.len());
        if start >= end {
            return None;
        }
        return Some(text[start..end].trim().to_owned()).filter(|sql| !sql.is_empty());
    }
    current_statement(text, cursor_char_index)
}

pub(crate) fn current_statement(text: &str, cursor_char_index: usize) -> Option<String> {
    let cursor = cursor_char_index.min(text.len());
    let bytes = text.as_bytes();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut statement_start = 0usize;
    let mut statement_end = text.len();
    let mut found_current = false;
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        if in_line_comment {
            if byte == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if !in_single && !in_double && byte == b'-' && next == Some(b'-') {
            in_line_comment = true;
            index += 2;
            continue;
        }
        if byte == b'\'' && !in_double {
            in_single = !in_single;
            index += 1;
            continue;
        }
        if byte == b'"' && !in_single {
            in_double = !in_double;
            index += 1;
            continue;
        }
        if byte == b';' && !in_single && !in_double {
            if cursor <= index {
                statement_end = index;
                found_current = true;
                break;
            }
            statement_start = index.saturating_add(1);
        }
        index += 1;
    }
    if !found_current {
        statement_end = text.len();
    }
    let statement = text[statement_start.min(text.len())..statement_end.min(text.len())]
        .trim()
        .to_owned();
    (!statement.is_empty()).then_some(statement)
}

/// Splits `text` into SQL statements using `;` outside quotes and comments.
pub fn split_sql_statements(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut statements = Vec::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut in_line_comment = false;
    let mut start = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        let byte = bytes[index];
        let next = bytes.get(index + 1).copied();
        if in_line_comment {
            if byte == b'\n' {
                in_line_comment = false;
            }
            index += 1;
            continue;
        }
        if !in_single && !in_double && byte == b'-' && next == Some(b'-') {
            in_line_comment = true;
            index += 2;
            continue;
        }
        if byte == b'\'' && !in_double {
            in_single = !in_single;
            index += 1;
            continue;
        }
        if byte == b'"' && !in_single {
            in_double = !in_double;
            index += 1;
            continue;
        }
        if byte == b';' && !in_single && !in_double {
            let statement = text[start..index].trim().to_owned();
            if !statement.is_empty() {
                statements.push(statement);
            }
            index += 1;
            start = skip_sql_trivia(bytes, index);
            index = start;
            continue;
        }
        index += 1;
    }
    let start = skip_sql_trivia(bytes, start.min(bytes.len()));
    let tail = text[start.min(text.len())..].trim().to_owned();
    if !tail.is_empty() {
        statements.push(tail);
    }
    statements
}

pub(crate) fn skip_sql_trivia(bytes: &[u8], mut index: usize) -> usize {
    loop {
        while index < bytes.len() && bytes[index].is_ascii_whitespace() {
            index += 1;
        }
        if index.saturating_add(1) < bytes.len() && bytes[index] == b'-' && bytes[index + 1] == b'-'
        {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        break;
    }
    index
}

pub(crate) fn load_persisted_state(path: &Path) -> Result<PersistedDbState, String> {
    match fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str(&contents)
            .map_err(|error| format!("failed to parse DB metadata `{}`: {error}", path.display())),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(PersistedDbState::default()),
        Err(error) => Err(format!(
            "failed to read DB metadata `{}`: {error}",
            path.display()
        )),
    }
}

pub(crate) fn initialize_native_keyring() -> Result<(), String> {
    use keyring_core::set_default_store;

    #[cfg(target_os = "windows")]
    {
        let store =
            windows_native_keyring_store::Store::new().map_err(|error| error.to_string())?;
        set_default_store(store);
    }
    #[cfg(target_os = "macos")]
    {
        let store = apple_native_keyring_store::keychain::Store::new()
            .map_err(|error| error.to_string())?;
        set_default_store(store);
    }
    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
    {
        let store =
            zbus_secret_service_keyring_store::Store::new().map_err(|error| error.to_string())?;
        set_default_store(store);
    }
    Ok(())
}

pub(crate) fn default_volt_state_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .join("volt")
    } else {
        env::var_os("XDG_STATE_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("state"))
            })
            .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
            .join("volt")
    }
}

pub(crate) fn unix_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

pub(crate) fn redact_error(error: String, connection_string: &str) -> String {
    let mut redacted = error.replace(connection_string, "[redacted connection string]");
    for key in ["password", "pwd", "secret", "token", "access key"] {
        redacted = redact_key_value_segments(&redacted, key);
    }
    redacted
}

pub(crate) fn redact_key_value_segments(input: &str, needle: &str) -> String {
    let mut output = String::new();
    for segment in input.split(';') {
        if let Some((key, _)) = segment.split_once('=')
            && key.trim().eq_ignore_ascii_case(needle)
        {
            if !output.is_empty() {
                output.push(';');
            }
            output.push_str(key);
            output.push_str("=[redacted]");
            continue;
        }
        if !output.is_empty() {
            output.push(';');
        }
        output.push_str(segment);
    }
    output
}

pub(crate) fn quote_sqlite_identifier(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn summarize_sql(sql: &str) -> String {
    let compact = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= 72 {
        compact
    } else {
        let mut shortened = compact.chars().take(69).collect::<String>();
        shortened.push_str("...");
        shortened
    }
}

pub(crate) fn sanitize_file_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned();
    if sanitized.is_empty() {
        "db".to_owned()
    } else {
        sanitized
    }
}

pub(crate) fn escape_bracket(value: &str) -> String {
    value.replace(']', "]]")
}
