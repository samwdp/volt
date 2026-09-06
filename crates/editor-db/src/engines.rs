use std::{collections::BTreeMap, path::Path};

use postgres::{Client as PostgresClient, NoTls, SimpleQueryMessage};
use rusqlite::{Connection as SqliteConnection, types::ValueRef as SqliteValueRef};
use tiberius::{Client as SqlServerClient, Config as SqlServerConfig};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

use crate::connection::*;
use crate::types::*;

/// Follow-up outcome when activating a browser-buffer action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DbActionOutcome {
    ActivatedSession(DbSessionSummary),
    Disconnected,
    OpenPreviewQuery {
        session_id: DbSessionId,
        table: QualifiedName,
    },
    ExploreRows {
        session_id: DbSessionId,
        table: QualifiedName,
    },
    SchemaRefreshed(DbSessionId),
    OpenSql {
        session_id: DbSessionId,
        sql: String,
    },
    SnippetDeleted,
    RememberedDeleted,
}

#[derive(Debug, Clone)]
pub(crate) struct ConnectionDescriptor {
    pub(crate) engine: DbEngine,
    pub(crate) display_label: String,
    pub(crate) database_name: Option<String>,
    pub(crate) host_label: Option<String>,
}

impl ConnectionDescriptor {
    pub(crate) fn from_connection_string(connection_string: &str) -> Result<Self, String> {
        let normalized = connection_string.trim();
        if normalized.is_empty() {
            return Err("connection string is empty".to_owned());
        }
        if looks_like_sql_server_connection_string(normalized) {
            let config = SqlServerConfig::from_ado_string(normalized)
                .map_err(|error| format!("invalid SQL Server connection string: {error}"))?;
            let host_label = parse_key_value(normalized, &["server", "data source"]);
            let database_name = parse_key_value(normalized, &["database", "initial catalog"]);
            return Ok(Self {
                engine: DbEngine::SqlServer,
                display_label: host_label.clone().unwrap_or_else(|| "sqlserver".to_owned()),
                database_name,
                host_label,
            }
            .with_sql_server_defaults(config));
        }
        if looks_like_postgres_connection_string(normalized) {
            let database_name = parse_postgres_keyword(normalized, "dbname")
                .or_else(|| parse_url_database(normalized));
            let host_label =
                parse_postgres_keyword(normalized, "host").or_else(|| parse_url_host(normalized));
            return Ok(Self {
                engine: DbEngine::Postgres,
                display_label: host_label.clone().unwrap_or_else(|| "postgres".to_owned()),
                database_name,
                host_label,
            });
        }
        Ok(Self {
            engine: DbEngine::Sqlite,
            display_label: sqlite_display_label(normalized),
            database_name: sqlite_database_name(normalized),
            host_label: None,
        })
    }

    pub(crate) fn default_alias(&self) -> String {
        match &self.database_name {
            Some(database) if !database.is_empty() => database.clone(),
            _ => sanitize_file_component(&self.display_label),
        }
    }

    pub(crate) fn with_sql_server_defaults(self, _config: SqlServerConfig) -> Self {
        self
    }
}

pub(crate) fn sqlite_index_table(
    connection: &SqliteConnection,
    index_name: &str,
) -> Result<Option<String>, String> {
    let mut statement = connection
        .prepare("SELECT tbl_name FROM sqlite_master WHERE type = 'index' AND name = ?1")
        .map_err(|error| format!("failed to inspect SQLite index `{index_name}`: {error}"))?;
    statement
        .query_row([index_name], |row| row.get::<_, String>(0))
        .map(Some)
        .or_else(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => Ok(None),
            other => Err(format!("failed to resolve SQLite index table: {other}")),
        })
}

pub(crate) fn execute_sqlite(
    connection_string: &str,
    sql: &str,
) -> Result<DbExecutionOutput, String> {
    let path = sqlite_path(connection_string);
    let connection = SqliteConnection::open(&path)
        .map_err(|error| format!("failed to open SQLite database `{path}`: {error}"))?;
    let mut statement = connection
        .prepare(sql)
        .map_err(|error| format!("failed to prepare SQLite query: {error}"))?;
    if statement.column_count() == 0 {
        let changed = connection
            .execute(sql, [])
            .map_err(|error| format!("failed to execute SQLite statement: {error}"))?;
        return Ok(execution_notice(
            "SQLite result",
            rows_affected_line(changed),
            changed,
        ));
    }
    let headers = statement
        .column_names()
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<Vec<_>>();
    let mut rows = statement
        .query([])
        .map_err(|error| format!("failed to run SQLite query: {error}"))?;
    let mut records = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|error| format!("failed to stream SQLite rows: {error}"))?
    {
        let mut values = Vec::new();
        for index in 0..headers.len() {
            values.push(sqlite_value_to_string(row.get_ref(index).map_err(
                |error| format!("failed to read SQLite column value: {error}"),
            )?));
        }
        records.push(values);
    }
    Ok(render_rows("SQLite result", &headers, &records))
}

pub(crate) fn postgres_columns_by_table(
    client: &mut PostgresClient,
) -> Result<BTreeMap<(String, String), Vec<DbColumn>>, String> {
    let rows = client
        .query(
            "SELECT table_schema, table_name, column_name, data_type, is_nullable \
             FROM information_schema.columns \
             WHERE table_schema NOT IN ('pg_catalog', 'information_schema') \
             ORDER BY table_schema, table_name, ordinal_position",
            &[],
        )
        .map_err(|error| format!("failed to introspect PostgreSQL columns: {error}"))?;
    let mut result = BTreeMap::new();
    for row in rows {
        let key = (row.get::<_, String>(0), row.get::<_, String>(1));
        result.entry(key).or_insert_with(Vec::new).push(DbColumn {
            name: row.get(2),
            data_type: row.get(3),
            nullable: row.get::<_, String>(4).eq_ignore_ascii_case("YES"),
        });
    }
    Ok(result)
}

pub(crate) fn execute_postgres(
    connection_string: &str,
    sql: &str,
) -> Result<DbExecutionOutput, String> {
    let mut client = PostgresClient::connect(connection_string, NoTls)
        .map_err(|error| format!("failed to connect to PostgreSQL: {error}"))?;
    let messages = client
        .simple_query(sql)
        .map_err(|error| format!("failed to execute PostgreSQL query: {error}"))?;
    let mut headers = Vec::<String>::new();
    let mut rows = Vec::<Vec<String>>::new();
    let mut status_lines = Vec::<String>::new();
    for message in messages {
        match message {
            SimpleQueryMessage::Row(row) => {
                if headers.is_empty() {
                    headers = row
                        .columns()
                        .iter()
                        .map(|column| column.name().to_owned())
                        .collect();
                }
                rows.push(
                    (0..row.len())
                        .map(|index| row.get(index).unwrap_or("").to_owned())
                        .collect(),
                );
            }
            SimpleQueryMessage::CommandComplete(count) => {
                status_lines.push(rows_affected_line(count as usize));
            }
            _ => {}
        }
    }
    if !rows.is_empty() {
        return Ok(render_rows("PostgreSQL result", &headers, &rows));
    }
    if status_lines.is_empty() {
        status_lines.push("Statement completed.".to_owned());
    }
    Ok(DbExecutionOutput {
        title: "PostgreSQL result".to_owned(),
        lines: status_lines,
        row_count: 0,
    })
}

pub(crate) fn execute_sql_server(
    connection_string: &str,
    sql: &str,
) -> Result<DbExecutionOutput, String> {
    let runtime = build_tokio_runtime()?;
    runtime.block_on(async move {
        let mut client = connect_sql_server(connection_string).await?;
        let results = client
            .simple_query(sql)
            .await
            .map_err(|error| format!("failed to execute SQL Server query: {error}"))?
            .into_results()
            .await
            .map_err(|error| format!("failed to read SQL Server results: {error}"))?;
        let Some(first_non_empty) = results.iter().find(|rows| !rows.is_empty()) else {
            return Ok(execution_notice(
                "SQL Server result",
                "Statement completed.",
                0,
            ));
        };
        let headers = first_non_empty[0]
            .columns()
            .iter()
            .map(|column| column.name().to_owned())
            .collect::<Vec<_>>();
        let rows = first_non_empty
            .iter()
            .map(sql_server_row_values)
            .collect::<Vec<_>>();
        Ok(render_rows("SQL Server result", &headers, &rows))
    })
}

pub(crate) async fn connect_sql_server(
    connection_string: &str,
) -> Result<SqlServerClient<tokio_util::compat::Compat<TcpStream>>, String> {
    let config = SqlServerConfig::from_ado_string(connection_string)
        .map_err(|error| format!("invalid SQL Server connection string: {error}"))?;
    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(|error| format!("failed to open SQL Server transport: {error}"))?;
    tcp.set_nodelay(true)
        .map_err(|error| format!("failed to configure SQL Server TCP stream: {error}"))?;
    SqlServerClient::connect(config, tcp.compat_write())
        .await
        .map_err(|error| format!("failed to connect to SQL Server: {error}"))
}

pub(crate) fn execution_notice(
    title: &str,
    message: impl Into<String>,
    row_count: usize,
) -> DbExecutionOutput {
    DbExecutionOutput {
        title: title.to_owned(),
        lines: vec![message.into()],
        row_count,
    }
}

pub(crate) fn sqlite_value_to_string(value: SqliteValueRef<'_>) -> String {
    match value {
        SqliteValueRef::Null => "NULL".to_owned(),
        SqliteValueRef::Integer(value) => value.to_string(),
        SqliteValueRef::Real(value) => value.to_string(),
        SqliteValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        SqliteValueRef::Blob(value) => format!("0x{}", hex_string(value)),
    }
}

pub(crate) fn sqlite_path(connection_string: &str) -> String {
    let trimmed = connection_string.trim();
    if let Some(path) = trimmed.strip_prefix("sqlite://") {
        return path.to_owned();
    }
    if let Some(path) = trimmed.strip_prefix("sqlite:") {
        return path.trim_start_matches('/').to_owned();
    }
    trimmed.to_owned()
}

pub(crate) fn sqlite_display_label(connection_string: &str) -> String {
    let path = sqlite_path(connection_string);
    Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or(path)
}

pub(crate) fn sqlite_database_name(connection_string: &str) -> Option<String> {
    let path = sqlite_path(connection_string);
    Path::new(&path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned)
}
