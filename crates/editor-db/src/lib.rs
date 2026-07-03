#![doc = r#"Database sessions, secure remembered connections, schema browsing, and SQL execution."#]

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

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Database sessions, secure remembered connections, schema browsing, and SQL execution.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Stable session identifier for active in-memory database sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DbSessionId(u64);

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

    fn dialect_id(self) -> &'static str {
        "sql"
    }

    fn sqls_driver(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite3",
            Self::Postgres => "postgresql",
            Self::SqlServer => "mssql",
        }
    }

    fn preview_sql(self, table: &QualifiedName) -> String {
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
}

pub type DbBrowserItemRenderer<'a> = dyn Fn(&DbBrowserContext) -> Vec<DbBrowserItemSpec> + 'a;

fn default_db_browser_items(context: &DbBrowserContext) -> Vec<DbBrowserItemSpec> {
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

fn default_db_browser_line(item: &DbBrowserItemContext) -> String {
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
struct DbHistoryEntry {
    session_alias: String,
    engine: DbEngine,
    sql: String,
    ran_at_epoch_secs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
struct PersistedDbState {
    remembered: Vec<RememberedConnection>,
    snippets: Vec<DbSnippet>,
    history: Vec<DbHistoryEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DbBrowserBufferKind {
    Connections,
    Schema { session_id: DbSessionId },
    History,
    Snippets,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DbBrowserBufferState {
    kind: DbBrowserBufferKind,
    title: String,
    actions_by_line: Vec<Option<DbBrowserAction>>,
}

#[derive(Debug, Clone)]
struct DbSession {
    id: DbSessionId,
    alias: String,
    engine: DbEngine,
    display_label: String,
    database_name: Option<String>,
    host_label: Option<String>,
    connection_string: String,
    remembered_secret_ref: Option<String>,
    schema_cache: DbSchemaCache,
}

#[derive(Debug, Clone, Default)]
struct DbSchemaCache {
    tables: Vec<DbTable>,
    views: Vec<DbTable>,
    indexes: Vec<DbIndex>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DbTable {
    name: QualifiedName,
    columns: Vec<DbColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DbColumn {
    name: String,
    data_type: String,
    nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DbIndex {
    name: String,
    table: QualifiedName,
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

    fn display(&self) -> String {
        match &self.schema {
            Some(schema) => format!("{schema}.{}", self.name),
            None => self.name.clone(),
        }
    }
}

fn db_browser_action_from_spec(spec: &DbActionSpec) -> Result<DbBrowserAction, String> {
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

fn qualified_name_from_spec(schema: Option<String>, name: String) -> Result<QualifiedName, String> {
    if name.trim().is_empty() {
        return Err("database browser table action requires a table name".to_owned());
    }
    Ok(QualifiedName { schema, name })
}

trait SecretStore: Send + Sync {
    fn set_secret(&self, secret_ref: &str, secret: &str) -> Result<(), String>;
    fn get_secret(&self, secret_ref: &str) -> Result<String, String>;
    fn delete_secret(&self, secret_ref: &str) -> Result<(), String>;
}

#[derive(Debug)]
struct OsSecretStore {
    service_name: String,
}

impl OsSecretStore {
    fn new(service_name: impl Into<String>) -> Result<Self, String> {
        let service_name = service_name.into();
        initialize_native_keyring()?;
        Ok(Self { service_name })
    }

    fn entry(&self, secret_ref: &str) -> Result<Entry, String> {
        Entry::new(&self.service_name, secret_ref).map_err(|error| error.to_string())
    }
}

impl SecretStore for OsSecretStore {
    fn set_secret(&self, secret_ref: &str, secret: &str) -> Result<(), String> {
        self.entry(secret_ref)?
            .set_password(secret)
            .map_err(|error| error.to_string())
    }

    fn get_secret(&self, secret_ref: &str) -> Result<String, String> {
        self.entry(secret_ref)?
            .get_password()
            .map_err(|error| error.to_string())
    }

    fn delete_secret(&self, secret_ref: &str) -> Result<(), String> {
        self.entry(secret_ref)?
            .delete_credential()
            .map_err(|error| error.to_string())
    }
}

#[derive(Debug)]
struct DisabledSecretStore {
    reason: String,
}

impl SecretStore for DisabledSecretStore {
    fn set_secret(&self, _: &str, _: &str) -> Result<(), String> {
        Err(self.reason.clone())
    }

    fn get_secret(&self, _: &str) -> Result<String, String> {
        Err(self.reason.clone())
    }

    fn delete_secret(&self, _: &str) -> Result<(), String> {
        Err(self.reason.clone())
    }
}

#[cfg(test)]
#[derive(Debug, Default)]
struct InMemorySecretStore {
    secrets: Mutex<HashMap<String, String>>,
}

#[cfg(test)]
impl SecretStore for InMemorySecretStore {
    fn set_secret(&self, secret_ref: &str, secret: &str) -> Result<(), String> {
        self.secrets
            .lock()
            .map_err(|_| "secret store lock poisoned".to_owned())?
            .insert(secret_ref.to_owned(), secret.to_owned());
        Ok(())
    }

    fn get_secret(&self, secret_ref: &str) -> Result<String, String> {
        self.secrets
            .lock()
            .map_err(|_| "secret store lock poisoned".to_owned())?
            .get(secret_ref)
            .cloned()
            .ok_or_else(|| format!("secret `{secret_ref}` is missing"))
    }

    fn delete_secret(&self, secret_ref: &str) -> Result<(), String> {
        self.secrets
            .lock()
            .map_err(|_| "secret store lock poisoned".to_owned())?
            .remove(secret_ref);
        Ok(())
    }
}

/// Central DB runtime service used by Volt.
pub struct DbService {
    state_dir: PathBuf,
    state_path: PathBuf,
    query_dir: PathBuf,
    secret_store: Arc<dyn SecretStore>,
    secret_persistence_available: bool,
    persisted: PersistedDbState,
    sessions: BTreeMap<DbSessionId, DbSession>,
    active_session_id: Option<DbSessionId>,
    next_session_id: u64,
    prompt_buffers: HashSet<u64>,
    query_buffers: HashMap<u64, DbQueryBufferMeta>,
    browser_buffers: HashMap<u64, DbBrowserBufferState>,
}

impl DbService {
    /// Creates a DB service rooted at Volt's default state directory.
    pub fn new() -> Result<Self, String> {
        let state_dir = default_volt_state_dir();
        let (secret_store, secret_persistence_available): (Arc<dyn SecretStore>, bool) =
            match OsSecretStore::new("volt.db") {
                Ok(store) => (Arc::new(store), true),
                Err(error) => (
                    Arc::new(DisabledSecretStore {
                        reason: format!(
                            "OS secret storage is unavailable on this machine, so remembered DB connections are disabled: {error}"
                        ),
                    }),
                    false,
                ),
            };
        Self::new_with_secret_store_inner(state_dir, secret_store, secret_persistence_available)
    }

    /// Creates a DB service rooted at `state_dir`.
    #[cfg(test)]
    fn new_with_secret_store(
        state_dir: PathBuf,
        secret_store: Arc<dyn SecretStore>,
    ) -> Result<Self, String> {
        Self::new_with_secret_store_inner(state_dir, secret_store, true)
    }

    fn new_with_secret_store_inner(
        state_dir: PathBuf,
        secret_store: Arc<dyn SecretStore>,
        secret_persistence_available: bool,
    ) -> Result<Self, String> {
        fs::create_dir_all(&state_dir).map_err(|error| {
            format!(
                "failed to create DB state directory `{}`: {error}",
                state_dir.display()
            )
        })?;
        let query_dir = state_dir.join("db").join("queries");
        fs::create_dir_all(&query_dir).map_err(|error| {
            format!(
                "failed to create DB query directory `{}`: {error}",
                query_dir.display()
            )
        })?;
        let state_path = state_dir.join("db").join("state.json");
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create DB metadata directory `{}`: {error}",
                    parent.display()
                )
            })?;
        }
        let persisted = load_persisted_state(&state_path)?;
        Ok(Self {
            state_dir,
            state_path,
            query_dir,
            secret_store,
            secret_persistence_available,
            persisted,
            sessions: BTreeMap::new(),
            active_session_id: None,
            next_session_id: 1,
            prompt_buffers: HashSet::new(),
            query_buffers: HashMap::new(),
            browser_buffers: HashMap::new(),
        })
    }

    /// Attaches a prompt buffer used for `db.connect`.
    pub fn attach_prompt_buffer(&mut self, buffer_key: u64) {
        self.prompt_buffers.insert(buffer_key);
    }

    fn render_db_browser_context(
        &mut self,
        buffer_key: u64,
        kind: DbBrowserBufferKind,
        context: DbBrowserContext,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let specs = renderer(&context);
        if specs.len() != context.items.len() {
            return Err(format!(
                "database browser renderer returned {} rows for {} input rows",
                specs.len(),
                context.items.len()
            ));
        }
        let mut lines = Vec::with_capacity(specs.len());
        let mut actions = Vec::with_capacity(specs.len());
        for spec in specs {
            lines.push(spec.line().to_owned());
            actions.push(spec.action().map(db_browser_action_from_spec).transpose()?);
        }
        let view = DbBrowserBufferView {
            title: context.title.to_string(),
            lines,
            actions_by_line: actions.clone(),
        };
        self.browser_buffers.insert(
            buffer_key,
            DbBrowserBufferState {
                kind,
                title: view.title.clone(),
                actions_by_line: actions,
            },
        );
        Ok(view)
    }

    /// Returns whether `buffer_key` is the DB connect prompt.
    pub fn is_prompt_buffer(&self, buffer_key: u64) -> bool {
        self.prompt_buffers.contains(&buffer_key)
    }

    /// Detaches any DB metadata for the given buffer.
    pub fn detach_buffer(&mut self, buffer_key: u64) {
        self.prompt_buffers.remove(&buffer_key);
        self.query_buffers.remove(&buffer_key);
        self.browser_buffers.remove(&buffer_key);
    }

    /// Returns whether secure remembered-connection persistence is available.
    pub fn secret_persistence_available(&self) -> bool {
        self.secret_persistence_available
    }

    /// Connects from a raw connection string and optionally remembers the secret.
    pub fn connect_raw(
        &mut self,
        connection_string: &str,
        remember_alias: Option<&str>,
    ) -> Result<DbSessionSummary, String> {
        let connection_string = connection_string.trim();
        if connection_string.is_empty() {
            return Err("connection string is empty".to_owned());
        }
        let descriptor = ConnectionDescriptor::from_connection_string(connection_string)?;
        let schema_cache = descriptor
            .engine
            .load_schema(connection_string)
            .map_err(|error| redact_error(error, connection_string))?;
        let remembered_secret_ref = remember_alias.and_then(|alias| {
            self.remember_connection(alias, &descriptor, connection_string)
                .ok()
        });
        let session_id = DbSessionId(self.next_session_id);
        self.next_session_id = self.next_session_id.saturating_add(1);
        let alias = remember_alias
            .map(str::trim)
            .filter(|alias| !alias.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| descriptor.default_alias());
        let display_label = descriptor.display_label.clone();
        let session = DbSession {
            id: session_id,
            alias: alias.clone(),
            engine: descriptor.engine,
            display_label: display_label.clone(),
            database_name: descriptor.database_name.clone(),
            host_label: descriptor.host_label.clone(),
            connection_string: connection_string.to_owned(),
            remembered_secret_ref,
            schema_cache,
        };
        self.sessions.insert(session_id, session);
        self.active_session_id = Some(session_id);
        self.touch_remembered_entry(&alias)?;
        self.session_summary(session_id, true)
    }

    /// Reconnects from a remembered alias.
    pub fn connect_remembered(&mut self, alias: &str) -> Result<DbSessionSummary, String> {
        let remembered = self
            .persisted
            .remembered
            .iter()
            .find(|connection| connection.alias.eq_ignore_ascii_case(alias))
            .cloned()
            .ok_or_else(|| format!("remembered connection `{alias}` was not found"))?;
        let secret = self
            .secret_store
            .get_secret(&remembered.secret_ref)
            .map_err(|error| {
                format!(
                    "failed to read secret for remembered connection `{}`: {error}",
                    remembered.alias
                )
            })?;
        let descriptor = ConnectionDescriptor::from_connection_string(&secret)?;
        let schema_cache = descriptor
            .engine
            .load_schema(&secret)
            .map_err(|error| redact_error(error, &secret))?;
        let session_id = DbSessionId(self.next_session_id);
        self.next_session_id = self.next_session_id.saturating_add(1);
        let session = DbSession {
            id: session_id,
            alias: remembered.alias.clone(),
            engine: descriptor.engine,
            display_label: descriptor.display_label.clone(),
            database_name: descriptor.database_name.clone(),
            host_label: descriptor.host_label.clone(),
            connection_string: secret,
            remembered_secret_ref: Some(remembered.secret_ref.clone()),
            schema_cache,
        };
        self.sessions.insert(session_id, session);
        self.active_session_id = Some(session_id);
        self.touch_remembered_entry(&remembered.alias)?;
        self.session_summary(session_id, true)
    }

    /// Disconnects a specific session or the current active session.
    pub fn disconnect(&mut self, session_id: Option<DbSessionId>) -> Result<(), String> {
        let session_id = session_id
            .or(self.active_session_id)
            .ok_or_else(|| "no active database session".to_owned())?;
        if self.sessions.remove(&session_id).is_none() {
            return Err(format!("session `{}` is not active", session_id.get()));
        }
        if self.active_session_id == Some(session_id) {
            self.active_session_id = self.sessions.keys().next_back().copied();
        }
        Ok(())
    }

    /// Returns summaries for current active sessions.
    pub fn session_summaries(&self) -> Vec<DbSessionSummary> {
        self.sessions
            .keys()
            .filter_map(|session_id| self.session_summary(*session_id, false).ok())
            .collect()
    }

    /// Returns the current active session summary, if any.
    pub fn active_session_summary(&self) -> Option<DbSessionSummary> {
        let session_id = self.active_session_id?;
        self.session_summary(session_id, false).ok()
    }

    /// Returns metadata for an attached DB query buffer.
    pub fn query_buffer_meta(&self, buffer_key: u64) -> Option<&DbQueryBufferMeta> {
        self.query_buffers.get(&buffer_key)
    }

    /// Returns the active DB session associated with the query buffer.
    pub fn query_buffer_session_id(&self, buffer_key: u64) -> Option<DbSessionId> {
        self.query_buffers
            .get(&buffer_key)
            .map(|meta| meta.session_id)
    }

    /// Returns `sqls` workspace settings for the current active DB session.
    pub fn sqls_workspace_settings_for_active_session(&self) -> Option<Value> {
        self.active_session_id
            .and_then(|session_id| self.sqls_workspace_settings_for_session(session_id))
    }

    /// Returns `sqls` workspace settings for the DB session attached to `buffer_key`.
    pub fn sqls_workspace_settings_for_query_buffer(&self, buffer_key: u64) -> Option<Value> {
        self.query_buffer_session_id(buffer_key)
            .and_then(|session_id| self.sqls_workspace_settings_for_session(session_id))
    }

    /// Returns `sqls` initialization options for the DB session attached to `buffer_key`.
    pub fn sqls_initialization_options_for_query_buffer(&self, buffer_key: u64) -> Option<Value> {
        self.query_buffer_session_id(buffer_key)
            .and_then(|session_id| self.sqls_initialization_options_for_session(session_id))
    }

    /// Creates and attaches a query buffer metadata record.
    pub fn attach_query_buffer(
        &mut self,
        buffer_key: u64,
        session_id: Option<DbSessionId>,
        requested_name: Option<&str>,
    ) -> Result<DbQueryBufferMeta, String> {
        let session_id = session_id
            .or(self.active_session_id)
            .ok_or_else(|| "no active database session".to_owned())?;
        let session = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session `{}` is not active", session_id.get()))?;
        let file_name = format!(
            "{}-{}-{}.sql",
            sanitize_file_component(&session.alias),
            session_id.get(),
            unique_suffix()
        );
        let session_query_dir = self.query_dir.join(session_id.get().to_string());
        fs::create_dir_all(&session_query_dir).map_err(|error| {
            format!(
                "failed to create DB query directory `{}`: {error}",
                session_query_dir.display()
            )
        })?;
        let temp_path = session_query_dir.join(file_name);
        let starter = format!(
            "-- {}\n-- {}\n\nSELECT *\nFROM ;\n",
            session.engine.label(),
            session.display_label
        );
        fs::write(&temp_path, starter.as_bytes()).map_err(|error| {
            format!(
                "failed to create DB query buffer `{}`: {error}",
                temp_path.display()
            )
        })?;
        let title = requested_name
            .map(str::to_owned)
            .unwrap_or_else(|| format!("*db-query {}*", session.alias));
        let meta = DbQueryBufferMeta {
            session_id,
            engine: session.engine,
            dialect_id: session.engine.dialect_id().to_owned(),
            temp_path,
            title,
        };
        self.query_buffers.insert(buffer_key, meta.clone());
        Ok(meta)
    }

    /// Builds a query buffer pre-filled with a preview query for `table`.
    pub fn attach_table_preview_query_buffer(
        &mut self,
        buffer_key: u64,
        session_id: DbSessionId,
        table: &QualifiedName,
    ) -> Result<(DbQueryBufferMeta, String), String> {
        let meta = self.attach_query_buffer(
            buffer_key,
            Some(session_id),
            Some(&format!("*db-query {}*", table.display())),
        )?;
        let sql = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session `{}` is not active", session_id.get()))?
            .engine
            .preview_sql(table);
        fs::write(&meta.temp_path, sql.as_bytes()).map_err(|error| {
            format!(
                "failed to seed DB preview query `{}`: {error}",
                meta.temp_path.display()
            )
        })?;
        Ok((meta, sql))
    }

    /// Returns buffer lines and actions for the remembered/active connections browser.
    pub fn render_connections_buffer(
        &mut self,
        buffer_key: u64,
    ) -> Result<DbBrowserBufferView, String> {
        self.render_connections_buffer_with(buffer_key, &default_db_browser_items)
    }

    pub fn render_connections_buffer_with(
        &mut self,
        buffer_key: u64,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let active = self.session_summaries();
        let mut items = vec![
            DbBrowserItemContext::new(DbBrowserItemKind::Header, "Database connections"),
            DbBrowserItemContext::new(DbBrowserItemKind::Header, ""),
            DbBrowserItemContext::new(DbBrowserItemKind::Header, "Active sessions"),
        ];
        if active.is_empty() {
            items.push(DbBrowserItemContext::new(
                DbBrowserItemKind::Empty,
                "(no active sessions)",
            ));
        } else {
            for session in active {
                items.push(
                    DbBrowserItemContext::new(DbBrowserItemKind::ActiveConnection, session.alias)
                        .with_engine(session.engine.label())
                        .with_session_id(session.id.get())
                        .with_active(session.active)
                        .with_default_action(DbActionSpec::disconnect_session(session.id.get())),
                );
            }
        }
        items.push(DbBrowserItemContext::new(DbBrowserItemKind::Header, ""));
        items.push(DbBrowserItemContext::new(
            DbBrowserItemKind::Header,
            "Remembered connections",
        ));
        if self.persisted.remembered.is_empty() {
            items.push(DbBrowserItemContext::new(
                DbBrowserItemKind::Empty,
                "(no remembered connections)",
            ));
        } else {
            for remembered in &self.persisted.remembered {
                let already_active = self
                    .sessions
                    .values()
                    .any(|session| session.alias.eq_ignore_ascii_case(&remembered.alias));
                items.push(
                    DbBrowserItemContext::new(
                        DbBrowserItemKind::RememberedConnection,
                        remembered.alias.clone(),
                    )
                    .with_engine(remembered.engine.label())
                    .with_active(already_active)
                    .with_remembered(true)
                    .with_default_action(DbActionSpec::activate_remembered(
                        remembered.alias.clone(),
                    )),
                );
            }
        }
        self.render_db_browser_context(
            buffer_key,
            DbBrowserBufferKind::Connections,
            DbBrowserContext::new(DbBrowserKind::Connections, "*db-connections*").with_items(items),
            renderer,
        )
    }

    /// Returns buffer lines and actions for the schema browser.
    pub fn render_schema_buffer(
        &mut self,
        buffer_key: u64,
        session_id: Option<DbSessionId>,
    ) -> Result<DbBrowserBufferView, String> {
        self.render_schema_buffer_with(buffer_key, session_id, &default_db_browser_items)
    }

    pub fn render_schema_buffer_with(
        &mut self,
        buffer_key: u64,
        session_id: Option<DbSessionId>,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let session_id = session_id
            .or(self.active_session_id)
            .ok_or_else(|| "no active database session".to_owned())?;
        self.refresh_schema_cache(session_id)?;
        let session = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session `{}` is not active", session_id.get()))?;
        let mut items = vec![
            DbBrowserItemContext::new(
                DbBrowserItemKind::Header,
                format!("Schema explorer: {}", session.alias),
            ),
            DbBrowserItemContext::new(
                DbBrowserItemKind::Header,
                format!("Engine: {}", session.engine.label()),
            ),
            DbBrowserItemContext::new(DbBrowserItemKind::Header, ""),
            DbBrowserItemContext::new(DbBrowserItemKind::Header, "Tables"),
        ];
        if session.schema_cache.tables.is_empty() {
            items.push(DbBrowserItemContext::new(
                DbBrowserItemKind::Empty,
                "(no tables)",
            ));
        } else {
            for table in &session.schema_cache.tables {
                items.push(
                    DbBrowserItemContext::new(DbBrowserItemKind::Table, table.name.display())
                        .with_session_id(session_id.get())
                        .with_table(table.name.schema.clone(), table.name.name.clone())
                        .with_default_action(DbActionSpec::open_table_preview(
                            session_id.get(),
                            table.name.schema.clone(),
                            table.name.name.clone(),
                        )),
                );
                for column in &table.columns {
                    items.push(DbBrowserItemContext::new(
                        DbBrowserItemKind::Header,
                        format!(
                            "  - {}: {}{}",
                            column.name,
                            column.data_type,
                            if column.nullable { " nullable" } else { "" }
                        ),
                    ));
                }
            }
        }
        items.push(DbBrowserItemContext::new(DbBrowserItemKind::Header, ""));
        items.push(DbBrowserItemContext::new(
            DbBrowserItemKind::Header,
            "Views",
        ));
        if session.schema_cache.views.is_empty() {
            items.push(DbBrowserItemContext::new(
                DbBrowserItemKind::Empty,
                "(no views)",
            ));
        } else {
            for view in &session.schema_cache.views {
                items.push(
                    DbBrowserItemContext::new(DbBrowserItemKind::View, view.name.display())
                        .with_session_id(session_id.get())
                        .with_table(view.name.schema.clone(), view.name.name.clone())
                        .with_default_action(DbActionSpec::open_table_preview(
                            session_id.get(),
                            view.name.schema.clone(),
                            view.name.name.clone(),
                        )),
                );
                for column in &view.columns {
                    items.push(DbBrowserItemContext::new(
                        DbBrowserItemKind::Header,
                        format!(
                            "  - {}: {}{}",
                            column.name,
                            column.data_type,
                            if column.nullable { " nullable" } else { "" }
                        ),
                    ));
                }
            }
        }
        items.push(DbBrowserItemContext::new(DbBrowserItemKind::Header, ""));
        items.push(DbBrowserItemContext::new(
            DbBrowserItemKind::Header,
            "Indexes",
        ));
        if session.schema_cache.indexes.is_empty() {
            items.push(DbBrowserItemContext::new(
                DbBrowserItemKind::Empty,
                "(no indexes)",
            ));
        } else {
            for index in &session.schema_cache.indexes {
                items.push(DbBrowserItemContext::new(
                    DbBrowserItemKind::Index,
                    format!("{} -> {}", index.name, index.table.display()),
                ));
            }
        }
        self.render_db_browser_context(
            buffer_key,
            DbBrowserBufferKind::Schema { session_id },
            DbBrowserContext::new(
                DbBrowserKind::Schema,
                format!("*db-schema {}*", session.alias),
            )
            .with_items(items),
            renderer,
        )
    }

    /// Returns buffer lines and actions for recent query history.
    pub fn render_history_buffer(
        &mut self,
        buffer_key: u64,
    ) -> Result<DbBrowserBufferView, String> {
        self.render_history_buffer_with(buffer_key, &default_db_browser_items)
    }

    pub fn render_history_buffer_with(
        &mut self,
        buffer_key: u64,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let mut items = vec![
            DbBrowserItemContext::new(DbBrowserItemKind::Header, "Query history"),
            DbBrowserItemContext::new(DbBrowserItemKind::Header, ""),
        ];
        if self.persisted.history.is_empty() {
            items.push(DbBrowserItemContext::new(
                DbBrowserItemKind::Empty,
                "(no history)",
            ));
        } else {
            for entry in self.persisted.history.iter().rev() {
                let mut item = DbBrowserItemContext::new(
                    DbBrowserItemKind::HistoryEntry,
                    entry.session_alias.clone(),
                )
                .with_engine(entry.engine.label())
                .with_detail(summarize_sql(&entry.sql))
                .with_sql(entry.sql.clone());
                if let Some(session_id) = self
                    .sessions
                    .values()
                    .find(|session| session.alias.eq_ignore_ascii_case(&entry.session_alias))
                    .map(|session| session.id)
                {
                    item = item.with_session_id(session_id.get()).with_default_action(
                        DbActionSpec::open_history_entry(session_id.get(), entry.sql.clone()),
                    );
                }
                items.push(item);
            }
        }
        self.render_db_browser_context(
            buffer_key,
            DbBrowserBufferKind::History,
            DbBrowserContext::new(DbBrowserKind::History, "*db-history*").with_items(items),
            renderer,
        )
    }

    /// Returns buffer lines and actions for saved snippets.
    pub fn render_snippets_buffer(
        &mut self,
        buffer_key: u64,
    ) -> Result<DbBrowserBufferView, String> {
        self.render_snippets_buffer_with(buffer_key, &default_db_browser_items)
    }

    pub fn render_snippets_buffer_with(
        &mut self,
        buffer_key: u64,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let mut items = vec![
            DbBrowserItemContext::new(DbBrowserItemKind::Header, "Saved query snippets"),
            DbBrowserItemContext::new(DbBrowserItemKind::Header, ""),
        ];
        if self.persisted.snippets.is_empty() {
            items.push(DbBrowserItemContext::new(
                DbBrowserItemKind::Empty,
                "(no snippets)",
            ));
        } else {
            for snippet in &self.persisted.snippets {
                items.push(
                    DbBrowserItemContext::new(DbBrowserItemKind::Snippet, snippet.name.clone())
                        .with_engine(snippet.engine.label())
                        .with_detail(summarize_sql(&snippet.sql))
                        .with_sql(snippet.sql.clone())
                        .with_id(snippet.id.clone())
                        .with_default_action(DbActionSpec::open_snippet(snippet.id.clone())),
                );
            }
        }
        self.render_db_browser_context(
            buffer_key,
            DbBrowserBufferKind::Snippets,
            DbBrowserContext::new(DbBrowserKind::Snippets, "*db-snippets*").with_items(items),
            renderer,
        )
    }

    /// Re-renders a previously attached browser buffer using its stored browser kind.
    pub fn rerender_browser_buffer(
        &mut self,
        buffer_key: u64,
    ) -> Result<DbBrowserBufferView, String> {
        self.rerender_browser_buffer_with(buffer_key, &default_db_browser_items)
    }

    pub fn rerender_browser_buffer_with(
        &mut self,
        buffer_key: u64,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let state = self
            .browser_buffers
            .get(&buffer_key)
            .cloned()
            .ok_or_else(|| format!("buffer `{buffer_key}` is not an attached DB browser buffer"))?;
        match state.kind {
            DbBrowserBufferKind::Connections => {
                self.render_connections_buffer_with(buffer_key, renderer)
            }
            DbBrowserBufferKind::Schema { session_id } => {
                self.render_schema_buffer_with(buffer_key, Some(session_id), renderer)
            }
            DbBrowserBufferKind::History => self.render_history_buffer_with(buffer_key, renderer),
            DbBrowserBufferKind::Snippets => self.render_snippets_buffer_with(buffer_key, renderer),
        }
    }

    /// Returns the action attached to `line_index` for an open browser buffer.
    pub fn browser_action(&self, buffer_key: u64, line_index: usize) -> Option<DbBrowserAction> {
        self.browser_buffers
            .get(&buffer_key)
            .and_then(|state| state.actions_by_line.get(line_index))
            .cloned()
            .flatten()
    }

    /// Activates one browser action and returns a follow-up action result.
    pub fn activate_browser_action(
        &mut self,
        action: DbBrowserAction,
    ) -> Result<DbActionOutcome, String> {
        match action {
            DbBrowserAction::ActivateRemembered { alias } => {
                let session = self.connect_remembered(&alias)?;
                Ok(DbActionOutcome::ActivatedSession(session))
            }
            DbBrowserAction::DisconnectSession { session_id } => {
                self.disconnect(Some(session_id))?;
                Ok(DbActionOutcome::Disconnected)
            }
            DbBrowserAction::OpenTablePreview { session_id, table } => {
                Ok(DbActionOutcome::OpenPreviewQuery { session_id, table })
            }
            DbBrowserAction::ExploreRows { session_id, table } => {
                Ok(DbActionOutcome::ExploreRows { session_id, table })
            }
            DbBrowserAction::RefreshSchema { session_id } => {
                self.refresh_schema_cache(session_id)?;
                Ok(DbActionOutcome::SchemaRefreshed(session_id))
            }
            DbBrowserAction::OpenHistoryEntry { sql, session_id } => {
                Ok(DbActionOutcome::OpenSql { session_id, sql })
            }
            DbBrowserAction::OpenSnippet { id } => {
                let snippet = self
                    .persisted
                    .snippets
                    .iter()
                    .find(|snippet| snippet.id == id)
                    .cloned()
                    .ok_or_else(|| format!("snippet `{id}` was not found"))?;
                let session_id = self
                    .active_session_id
                    .ok_or_else(|| "no active database session".to_owned())?;
                Ok(DbActionOutcome::OpenSql {
                    session_id,
                    sql: snippet.sql,
                })
            }
            DbBrowserAction::DeleteSnippet { id } => {
                self.persisted.snippets.retain(|snippet| snippet.id != id);
                self.save_persisted_state()?;
                Ok(DbActionOutcome::SnippetDeleted)
            }
            DbBrowserAction::DeleteRemembered { alias } => {
                self.delete_remembered(&alias)?;
                Ok(DbActionOutcome::RememberedDeleted)
            }
        }
    }

    /// Refreshes schema state for an active session.
    pub fn refresh_schema_cache(&mut self, session_id: DbSessionId) -> Result<(), String> {
        let (alias, connection_string, engine) = {
            let session = self
                .sessions
                .get(&session_id)
                .ok_or_else(|| format!("session `{}` is not active", session_id.get()))?;
            (
                session.alias.clone(),
                session.connection_string.clone(),
                session.engine,
            )
        };
        let schema_cache = engine
            .load_schema(&connection_string)
            .map_err(|error| redact_error(error, &connection_string))?;
        let session = self
            .sessions
            .get_mut(&session_id)
            .ok_or_else(|| format!("session `{}` is not active", session_id.get()))?;
        session.schema_cache = schema_cache;
        self.touch_remembered_entry(&alias)?;
        Ok(())
    }

    /// Executes `sql` for the buffer's attached session and records history.
    pub fn execute_sql_for_buffer(
        &mut self,
        buffer_key: u64,
        sql: &str,
    ) -> Result<DbExecutionOutput, String> {
        let meta = self
            .query_buffers
            .get(&buffer_key)
            .cloned()
            .ok_or_else(|| "active buffer is not a DB query buffer".to_owned())?;
        let session = self
            .sessions
            .get(&meta.session_id)
            .ok_or_else(|| format!("session `{}` is not active", meta.session_id.get()))?
            .clone();
        let sql = sql.trim();
        if sql.is_empty() {
            return Err("no SQL selected for execution".to_owned());
        }
        let output = session
            .engine
            .execute_sql(&session.connection_string, sql)
            .map_err(|error| redact_error(error, &session.connection_string))?;
        self.record_history(&session, sql)?;
        Ok(output)
    }

    /// Returns schema-derived autocomplete candidates for a DB query buffer.
    pub fn autocomplete_candidates_for_buffer(
        &self,
        buffer_key: u64,
    ) -> Vec<DbAutocompleteCandidate> {
        let Some(meta) = self.query_buffers.get(&buffer_key) else {
            return Vec::new();
        };
        let Some(session) = self.sessions.get(&meta.session_id) else {
            return Vec::new();
        };
        let mut seen = HashSet::new();
        let mut candidates = Vec::new();
        for table in session
            .schema_cache
            .tables
            .iter()
            .chain(session.schema_cache.views.iter())
        {
            let table_name = table.name.name.clone();
            if seen.insert(("table".to_owned(), table_name.clone())) {
                candidates.push(DbAutocompleteCandidate {
                    label: table.name.display(),
                    replacement: table_name,
                    detail: Some("table".to_owned()),
                    documentation: Some(format!(
                        "{} columns: {}",
                        table.name.display(),
                        table
                            .columns
                            .iter()
                            .map(|column| column.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                });
            }
            let qualified = table.name.display();
            if seen.insert(("qualified-table".to_owned(), qualified.clone())) {
                candidates.push(DbAutocompleteCandidate {
                    label: qualified.clone(),
                    replacement: qualified.clone(),
                    detail: Some("qualified table".to_owned()),
                    documentation: Some(format!("{} ({})", qualified, session.engine.label())),
                });
            }
            for column in &table.columns {
                let key = format!("{}::{}", table.name.display(), column.name);
                if seen.insert(("column".to_owned(), key)) {
                    candidates.push(DbAutocompleteCandidate {
                        label: column.name.clone(),
                        replacement: column.name.clone(),
                        detail: Some(format!("{} column", table.name.display())),
                        documentation: Some(format!(
                            "{}.{}: {}{}",
                            table.name.display(),
                            column.name,
                            column.data_type,
                            if column.nullable { " nullable" } else { "" }
                        )),
                    });
                }
            }
        }
        candidates
    }

    /// Saves a named snippet sourced from the attached query buffer.
    pub fn save_snippet(
        &mut self,
        buffer_key: u64,
        name: &str,
        sql: &str,
    ) -> Result<DbSnippet, String> {
        let meta = self
            .query_buffers
            .get(&buffer_key)
            .cloned()
            .ok_or_else(|| "active buffer is not a DB query buffer".to_owned())?;
        let snippet = DbSnippet {
            id: format!("snippet-{}", unique_suffix()),
            name: name.trim().to_owned(),
            sql: sql.trim().to_owned(),
            engine: meta.engine,
            created_at_epoch_secs: unix_epoch_secs(),
        };
        if snippet.name.is_empty() {
            return Err("snippet name is empty".to_owned());
        }
        if snippet.sql.is_empty() {
            return Err("snippet SQL is empty".to_owned());
        }
        self.persisted.snippets.push(snippet.clone());
        self.save_persisted_state()?;
        Ok(snippet)
    }

    /// Returns state dir used by the service.
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Returns the current browser buffer kind for `buffer_key`.
    pub fn browser_buffer_kind(&self, buffer_key: u64) -> Option<&str> {
        self.browser_buffers
            .get(&buffer_key)
            .map(|state| match state.kind {
                DbBrowserBufferKind::Connections => "connections",
                DbBrowserBufferKind::Schema { .. } => "schema",
                DbBrowserBufferKind::History => "history",
                DbBrowserBufferKind::Snippets => "snippets",
            })
    }

    fn session_summary(
        &self,
        session_id: DbSessionId,
        force_active: bool,
    ) -> Result<DbSessionSummary, String> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session `{}` is not active", session_id.get()))?;
        Ok(DbSessionSummary {
            id: session.id,
            alias: session.alias.clone(),
            engine: session.engine,
            display_label: session.display_label.clone(),
            database_name: session.database_name.clone(),
            host_label: session.host_label.clone(),
            remembered: session.remembered_secret_ref.is_some(),
            active: force_active || self.active_session_id == Some(session_id),
        })
    }

    fn sqls_connection_config_for_session(&self, session_id: DbSessionId) -> Option<Value> {
        let session = self.sessions.get(&session_id)?;
        let alias = if session.alias.trim().is_empty() {
            session.display_label.clone()
        } else {
            session.alias.clone()
        };
        Some(json!({
            "alias": alias,
            "driver": session.engine.sqls_driver(),
            "dataSourceName": session.connection_string.clone(),
        }))
    }

    fn sqls_workspace_settings_for_session(&self, session_id: DbSessionId) -> Option<Value> {
        self.sqls_connection_config_for_session(session_id)
            .map(|connection| json!({ "connections": [connection] }))
    }

    fn sqls_initialization_options_for_session(&self, session_id: DbSessionId) -> Option<Value> {
        self.sqls_connection_config_for_session(session_id)
            .map(|connection| json!({ "connectionConfig": connection }))
    }

    fn remember_connection(
        &mut self,
        alias: &str,
        descriptor: &ConnectionDescriptor,
        connection_string: &str,
    ) -> Result<String, String> {
        let alias = alias.trim();
        if alias.is_empty() {
            return Err("remembered connection alias is empty".to_owned());
        }
        let previous_secret_ref = self
            .persisted
            .remembered
            .iter()
            .find(|existing| existing.alias.eq_ignore_ascii_case(alias))
            .map(|existing| existing.secret_ref.clone());
        let secret_ref = format!("db-{}-{}", sanitize_file_component(alias), unique_suffix());
        self.secret_store
            .set_secret(&secret_ref, connection_string)
            .map_err(|error| {
                format!("failed to persist secret for remembered connection `{alias}`: {error}")
            })?;
        let remembered = RememberedConnection {
            alias: alias.to_owned(),
            engine: descriptor.engine,
            display_label: descriptor.display_label.clone(),
            database_name: descriptor.database_name.clone(),
            host_label: descriptor.host_label.clone(),
            last_used_epoch_secs: unix_epoch_secs(),
            secret_ref: secret_ref.clone(),
        };
        self.persisted
            .remembered
            .retain(|existing| !existing.alias.eq_ignore_ascii_case(alias));
        self.persisted.remembered.push(remembered);
        if let Some(previous_secret_ref) = previous_secret_ref {
            let _ = self.secret_store.delete_secret(&previous_secret_ref);
        }
        self.save_persisted_state()?;
        Ok(secret_ref)
    }

    fn delete_remembered(&mut self, alias: &str) -> Result<(), String> {
        let Some(remembered) = self
            .persisted
            .remembered
            .iter()
            .find(|connection| connection.alias.eq_ignore_ascii_case(alias))
            .cloned()
        else {
            return Err(format!("remembered connection `{alias}` was not found"));
        };
        self.secret_store
            .delete_secret(&remembered.secret_ref)
            .map_err(|error| {
                format!(
                    "failed to delete secret for remembered connection `{}`: {error}",
                    remembered.alias
                )
            })?;
        self.persisted
            .remembered
            .retain(|connection| !connection.alias.eq_ignore_ascii_case(alias));
        self.save_persisted_state()
    }

    fn touch_remembered_entry(&mut self, alias: &str) -> Result<(), String> {
        let mut touched = false;
        for remembered in &mut self.persisted.remembered {
            if remembered.alias.eq_ignore_ascii_case(alias) {
                remembered.last_used_epoch_secs = unix_epoch_secs();
                touched = true;
            }
        }
        if touched {
            self.save_persisted_state()?;
        }
        Ok(())
    }

    fn record_history(&mut self, session: &DbSession, sql: &str) -> Result<(), String> {
        self.persisted.history.push(DbHistoryEntry {
            session_alias: session.alias.clone(),
            engine: session.engine,
            sql: sql.to_owned(),
            ran_at_epoch_secs: unix_epoch_secs(),
        });
        if self.persisted.history.len() > 100 {
            let keep = 100usize;
            let drop_count = self.persisted.history.len().saturating_sub(keep);
            self.persisted.history.drain(0..drop_count);
        }
        self.save_persisted_state()
    }

    fn save_persisted_state(&self) -> Result<(), String> {
        let payload = serde_json::to_string_pretty(&self.persisted)
            .map_err(|error| format!("failed to encode DB metadata: {error}"))?;
        fs::write(&self.state_path, payload).map_err(|error| {
            format!(
                "failed to write DB metadata `{}`: {error}",
                self.state_path.display()
            )
        })
    }
}

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
        return Ok(DbExecutionOutput {
            title: "SQLite result".to_owned(),
            lines: vec![format!(
                "Statement executed successfully. Rows affected: {changed}."
            )],
            row_count: changed,
        });
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
                status_lines.push(format!(
                    "Statement executed successfully. Rows affected: {count}."
                ));
            }
            _ => {}
        }
    }
    if !rows.is_empty() {
        return Ok(render_rows("PostgreSQL result", &headers, &rows));
    }
    if status_lines.is_empty() {
        status_lines.push("Statement executed successfully.".to_owned());
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
            return Ok(DbExecutionOutput {
                title: "SQL Server result".to_owned(),
                lines: vec!["Statement executed successfully.".to_owned()],
                row_count: 0,
            });
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
        return DbExecutionOutput {
            title: title.to_owned(),
            lines: vec!["Query returned no columns.".to_owned()],
            row_count: 0,
        };
    }
    let mut widths = headers
        .iter()
        .map(|header| header.chars().count())
        .collect::<Vec<_>>();
    for row in rows {
        for (index, value) in row.iter().enumerate() {
            if let Some(width) = widths.get_mut(index) {
                *width = (*width).max(value.chars().count());
            }
        }
    }
    let mut lines = Vec::new();
    lines.push(pad_row(headers, &widths));
    lines.push(
        widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>()
            .join("-+-"),
    );
    for row in rows {
        lines.push(pad_row(row, &widths));
    }
    lines.push(String::new());
    lines.push(format!("Rows: {}", rows.len()));
    DbExecutionOutput {
        title: title.to_owned(),
        lines,
        row_count: rows.len(),
    }
}

fn pad_row(values: &[String], widths: &[usize]) -> String {
    values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let width = widths
                .get(index)
                .copied()
                .unwrap_or_else(|| value.chars().count());
            format!("{value:<width$}")
        })
        .collect::<Vec<_>>()
        .join(" | ")
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

fn looks_like_sql_server_connection_string(connection_string: &str) -> bool {
    let lower = connection_string.to_ascii_lowercase();
    lower.starts_with("server=")
        || lower.starts_with("data source=")
        || lower.starts_with("jdbc:sqlserver://")
        || lower.starts_with("sqlserver://")
        || lower.contains(";trustservercertificate=")
}

fn looks_like_postgres_connection_string(connection_string: &str) -> bool {
    let lower = connection_string.to_ascii_lowercase();
    lower.starts_with("postgres://")
        || lower.starts_with("postgresql://")
        || lower.contains("host=") && (lower.contains("user=") || lower.contains("dbname="))
}

fn parse_key_value(connection_string: &str, keys: &[&str]) -> Option<String> {
    connection_string
        .split(';')
        .filter_map(|segment| segment.split_once('='))
        .find_map(|(key, value)| {
            keys.iter()
                .any(|expected| key.trim().eq_ignore_ascii_case(expected))
                .then(|| value.trim().to_owned())
        })
}

fn parse_postgres_keyword(connection_string: &str, key: &str) -> Option<String> {
    connection_string
        .split_whitespace()
        .filter_map(|segment| segment.split_once('='))
        .find_map(|(candidate, value)| {
            candidate
                .trim()
                .eq_ignore_ascii_case(key)
                .then(|| value.trim_matches('\'').to_owned())
        })
}

fn parse_url_database(connection_string: &str) -> Option<String> {
    let (_, rest) = connection_string.split_once("://")?;
    let path = rest.split('/').nth(1)?;
    let db = path.split('?').next().unwrap_or(path);
    (!db.is_empty()).then(|| db.to_owned())
}

fn parse_url_host(connection_string: &str) -> Option<String> {
    let (_, rest) = connection_string.split_once("://")?;
    let host_port = rest.split('/').next()?;
    let host_port = host_port.rsplit('@').next().unwrap_or(host_port);
    let host = host_port.split(':').next().unwrap_or(host_port);
    (!host.is_empty()).then(|| host.to_owned())
}

fn sqlite_path(connection_string: &str) -> String {
    let trimmed = connection_string.trim();
    if let Some(path) = trimmed.strip_prefix("sqlite://") {
        return path.to_owned();
    }
    if let Some(path) = trimmed.strip_prefix("sqlite:") {
        return path.trim_start_matches('/').to_owned();
    }
    trimmed.to_owned()
}

fn sqlite_display_label(connection_string: &str) -> String {
    let path = sqlite_path(connection_string);
    Path::new(&path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_owned)
        .unwrap_or(path)
}

fn sqlite_database_name(connection_string: &str) -> Option<String> {
    let path = sqlite_path(connection_string);
    Path::new(&path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .map(str::to_owned)
}

fn quote_sqlite_identifier(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn summarize_sql(sql: &str) -> String {
    let compact = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= 72 {
        compact
    } else {
        let mut shortened = compact.chars().take(69).collect::<String>();
        shortened.push_str("...");
        shortened
    }
}

fn sanitize_file_component(value: &str) -> String {
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

fn escape_bracket(value: &str) -> String {
    value.replace(']', "]]")
}

fn escape_double_quote(value: &str) -> String {
    value.replace('"', "\"\"")
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let postgres = ConnectionDescriptor::from_connection_string(
            "postgres://volt:secret@localhost:5432/app",
        )
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
}
