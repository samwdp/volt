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
struct ConnectionDescriptor {
    engine: DbEngine,
    display_label: String,
    database_name: Option<String>,
    host_label: Option<String>,
}

impl ConnectionDescriptor {
    fn from_connection_string(connection_string: &str) -> Result<Self, String> {
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

    fn default_alias(&self) -> String {
        match &self.database_name {
            Some(database) if !database.is_empty() => database.clone(),
            _ => sanitize_file_component(&self.display_label),
        }
    }

    fn with_sql_server_defaults(self, _config: SqlServerConfig) -> Self {
        self
    }
}

impl DbEngine {
    fn load_schema(self, connection_string: &str) -> Result<DbSchemaCache, String> {
        match self {
            Self::Sqlite => load_sqlite_schema(connection_string),
            Self::Postgres => load_postgres_schema(connection_string),
            Self::SqlServer => load_sql_server_schema(connection_string),
        }
    }

    fn execute_sql(self, connection_string: &str, sql: &str) -> Result<DbExecutionOutput, String> {
        match self {
            Self::Sqlite => execute_sqlite(connection_string, sql),
            Self::Postgres => execute_postgres(connection_string, sql),
            Self::SqlServer => execute_sql_server(connection_string, sql),
        }
    }
}

fn load_sqlite_schema(connection_string: &str) -> Result<DbSchemaCache, String> {
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

fn load_sqlite_columns(
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

fn sqlite_index_table(
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

fn execute_sqlite(connection_string: &str, sql: &str) -> Result<DbExecutionOutput, String> {
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

fn load_postgres_schema(connection_string: &str) -> Result<DbSchemaCache, String> {
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

fn postgres_columns_by_table(
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

fn execute_postgres(connection_string: &str, sql: &str) -> Result<DbExecutionOutput, String> {
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

fn load_sql_server_schema(connection_string: &str) -> Result<DbSchemaCache, String> {
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

fn sql_server_columns_by_table(rows: &[SqlServerRow]) -> BTreeMap<(String, String), Vec<DbColumn>> {
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

fn execute_sql_server(connection_string: &str, sql: &str) -> Result<DbExecutionOutput, String> {
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

async fn connect_sql_server(
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

fn build_tokio_runtime() -> Result<Runtime, String> {
    Runtime::new().map_err(|error| format!("failed to create Tokio runtime: {error}"))
}

fn render_rows(title: &str, headers: &[String], rows: &[Vec<String>]) -> DbExecutionOutput {
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

fn execution_notice(
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

fn rows_affected_line(count: usize) -> String {
    match count {
        0 => "No rows affected.".to_owned(),
        1 => "1 row affected.".to_owned(),
        n => format!("{n} rows affected."),
    }
}

fn row_count_footer(rows: usize, columns: usize) -> String {
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
enum CellAlign {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BoxRuleKind {
    Top,
    Middle,
    Bottom,
}

fn box_rule(widths: &[usize], kind: BoxRuleKind) -> String {
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

fn box_row(values: &[String], widths: &[usize], aligns: &[CellAlign]) -> String {
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

fn pad_cell(value: &str, width: usize, align: CellAlign) -> String {
    let pad = width.saturating_sub(value.chars().count());
    match align {
        CellAlign::Left => format!("{value}{}", " ".repeat(pad)),
        CellAlign::Right => format!("{}{value}", " ".repeat(pad)),
    }
}

fn display_cell(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\n' | '\r' | '\t' => ' ',
            other => other,
        })
        .collect()
}

fn column_is_numeric(rows: &[Vec<String>], index: usize) -> bool {
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

fn is_numeric_cell(value: &str) -> bool {
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

fn sql_server_row_values(row: &SqlServerRow) -> Vec<String> {
    row.cells()
        .map(|(_, value)| sql_server_column_data_to_string(value))
        .collect()
}

fn sql_server_cell(row: &SqlServerRow, column: &str) -> String {
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

fn sql_server_column_data_to_string(value: &tiberius::ColumnData<'static>) -> String {
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

fn sqlite_value_to_string(value: SqliteValueRef<'_>) -> String {
    match value {
        SqliteValueRef::Null => "NULL".to_owned(),
        SqliteValueRef::Integer(value) => value.to_string(),
        SqliteValueRef::Real(value) => value.to_string(),
        SqliteValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        SqliteValueRef::Blob(value) => format!("0x{}", hex_string(value)),
    }
}

fn hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join("")
}

fn parse_connect_prompt(input: &str) -> Result<(Option<String>, String), String> {
    let trimmed = input.trim();
    if let Some(rest) = trimmed.strip_prefix("remember ") {
        let Some((alias, connection_string)) = rest.split_once("::") else {
            return Err(
                "remembered connections must use `remember <alias> :: <connection string>`"
                    .to_owned(),
            );
        };
        let alias = alias.trim();
        if alias.is_empty() {
            return Err("remembered connection alias is empty".to_owned());
        }
        let connection_string = connection_string.trim();
        if connection_string.is_empty() {
            return Err("remembered connection string is empty".to_owned());
        }
        return Ok((Some(alias.to_owned()), connection_string.to_owned()));
    }
    Ok((None, trimmed.to_owned()))
}

/// Parses one DB connect prompt payload.
pub fn parse_db_connect_prompt(input: &str) -> Result<(Option<String>, String), String> {
    parse_connect_prompt(input)
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

fn current_statement(text: &str, cursor_char_index: usize) -> Option<String> {
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

fn skip_sql_trivia(bytes: &[u8], mut index: usize) -> usize {
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

fn load_persisted_state(path: &Path) -> Result<PersistedDbState, String> {
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

fn initialize_native_keyring() -> Result<(), String> {
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

fn default_volt_state_dir() -> PathBuf {
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

fn unix_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

fn redact_error(error: String, connection_string: &str) -> String {
    let mut redacted = error.replace(connection_string, "[redacted connection string]");
    for key in ["password", "pwd", "secret", "token", "access key"] {
        redacted = redact_key_value_segments(&redacted, key);
    }
    redacted
}

fn redact_key_value_segments(input: &str, needle: &str) -> String {
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
