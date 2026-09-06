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
use crate::types::*;

/// Central DB runtime service used by Volt.
pub struct DbService {
    pub(crate) state_dir: PathBuf,
    pub(crate) state_path: PathBuf,
    pub(crate) query_dir: PathBuf,
    pub(crate) secret_store: Arc<dyn SecretStore>,
    pub(crate) secret_persistence_available: bool,
    pub(crate) persisted: PersistedDbState,
    pub(crate) sessions: BTreeMap<DbSessionId, DbSession>,
    pub(crate) active_session_id: Option<DbSessionId>,
    pub(crate) next_session_id: u64,
    pub(crate) prompt_buffers: HashSet<u64>,
    pub(crate) query_buffers: HashMap<u64, DbQueryBufferMeta>,
    pub(crate) browser_buffers: HashMap<(u64, String), DbBrowserBufferState>,
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
    pub(crate) fn new_with_secret_store(
        state_dir: PathBuf,
        secret_store: Arc<dyn SecretStore>,
    ) -> Result<Self, String> {
        Self::new_with_secret_store_inner(state_dir, secret_store, true)
    }

    pub(crate) fn new_with_secret_store_inner(
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

    pub(crate) fn render_db_browser_section(
        &mut self,
        buffer_key: u64,
        section: &str,
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
        let kinds_by_line = context.items.iter().map(|item| item.kind).collect();
        let view = DbBrowserBufferView {
            title: context.title.to_string(),
            lines,
            actions_by_line: actions.clone(),
            kinds_by_line,
        };
        self.browser_buffers.insert(
            (buffer_key, section.to_owned()),
            DbBrowserBufferState {
                kind,
                title: view.title.clone(),
                actions_by_line: actions,
            },
        );
        Ok(view)
    }

    pub(crate) fn render_db_browser_context(
        &mut self,
        buffer_key: u64,
        kind: DbBrowserBufferKind,
        context: DbBrowserContext,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        self.render_db_browser_section(buffer_key, "", kind, context, renderer)
    }

    /// Returns whether `buffer_key` is the DB connect prompt.
    pub fn is_prompt_buffer(&self, buffer_key: u64) -> bool {
        self.prompt_buffers.contains(&buffer_key)
    }

    /// Detaches any DB metadata for the given buffer.
    pub fn detach_buffer(&mut self, buffer_key: u64) {
        self.prompt_buffers.remove(&buffer_key);
        self.query_buffers.remove(&buffer_key);
        self.browser_buffers
            .retain(|(key, _), _| *key != buffer_key);
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
            "-- {} · {}\n-- Ctrl+c Ctrl+c  execute statement or selection\n-- Ctrl+s         save snippet\n\nSELECT *\nFROM ;\n",
            session.engine.label(),
            session.alias
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

    /// Returns the engine-specific preview query for `table`.
    pub fn preview_sql_for_table(
        &self,
        session_id: DbSessionId,
        table: &QualifiedName,
    ) -> Result<String, String> {
        let session = self
            .sessions
            .get(&session_id)
            .ok_or_else(|| format!("session `{}` is not active", session_id.get()))?;
        Ok(session.engine.preview_sql(table))
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
        self.render_connections_section_with(buffer_key, "", renderer)
    }

    /// Renders the connections browser into a named section of `buffer_key`.
    pub fn render_connections_section_with(
        &mut self,
        buffer_key: u64,
        section: &str,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let active = self.session_summaries();
        let mut items = vec![
            DbBrowserItemContext::new(DbBrowserItemKind::Header, "Database connections"),
            DbBrowserItemContext::new(DbBrowserItemKind::Header, ""),
            DbBrowserItemContext::new(
                DbBrowserItemKind::Header,
                section_count_label("Active sessions", active.len()),
            ),
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
            section_count_label("Remembered connections", self.persisted.remembered.len()),
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
        self.render_db_browser_section(
            buffer_key,
            section,
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
        self.render_schema_section_with(buffer_key, "", session_id, renderer)
    }

    /// Renders the schema browser into a named section of `buffer_key`.
    pub fn render_schema_section_with(
        &mut self,
        buffer_key: u64,
        section: &str,
        session_id: Option<DbSessionId>,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let Some(session_id) = session_id.or(self.active_session_id) else {
            return self.render_db_browser_section(
                buffer_key,
                section,
                DbBrowserBufferKind::Schema { session_id: None },
                DbBrowserContext::new(DbBrowserKind::Schema, "*db-schema*").with_items(vec![
                    DbBrowserItemContext::new(DbBrowserItemKind::Header, "Schema explorer"),
                    DbBrowserItemContext::new(DbBrowserItemKind::Header, ""),
                    DbBrowserItemContext::new(DbBrowserItemKind::Empty, "(no active session)"),
                ]),
                renderer,
            );
        };
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
            DbBrowserItemContext::new(
                DbBrowserItemKind::Header,
                section_count_label("Tables", session.schema_cache.tables.len()),
            ),
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
                push_schema_column_items(&mut items, &table.columns);
            }
        }
        items.push(DbBrowserItemContext::new(DbBrowserItemKind::Header, ""));
        items.push(DbBrowserItemContext::new(
            DbBrowserItemKind::Header,
            section_count_label("Views", session.schema_cache.views.len()),
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
                push_schema_column_items(&mut items, &view.columns);
            }
        }
        items.push(DbBrowserItemContext::new(DbBrowserItemKind::Header, ""));
        items.push(DbBrowserItemContext::new(
            DbBrowserItemKind::Header,
            section_count_label("Indexes", session.schema_cache.indexes.len()),
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
        self.render_db_browser_section(
            buffer_key,
            section,
            DbBrowserBufferKind::Schema {
                session_id: Some(session_id),
            },
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
            DbBrowserItemContext::new(
                DbBrowserItemKind::Header,
                section_count_label("Query history", self.persisted.history.len()),
            ),
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
            DbBrowserItemContext::new(
                DbBrowserItemKind::Header,
                section_count_label("Saved query snippets", self.persisted.snippets.len()),
            ),
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
        self.rerender_browser_section_with(buffer_key, "", renderer)
    }

    /// Re-renders one named browser section for `buffer_key`.
    pub fn rerender_browser_section_with(
        &mut self,
        buffer_key: u64,
        section: &str,
        renderer: &DbBrowserItemRenderer<'_>,
    ) -> Result<DbBrowserBufferView, String> {
        let state = self
            .browser_buffers
            .get(&(buffer_key, section.to_owned()))
            .cloned()
            .ok_or_else(|| format!("buffer `{buffer_key}` is not an attached DB browser buffer"))?;
        match state.kind {
            DbBrowserBufferKind::Connections => {
                self.render_connections_section_with(buffer_key, section, renderer)
            }
            DbBrowserBufferKind::Schema { session_id } => {
                self.render_schema_section_with(buffer_key, section, session_id, renderer)
            }
            DbBrowserBufferKind::History => self.render_history_buffer_with(buffer_key, renderer),
            DbBrowserBufferKind::Snippets => self.render_snippets_buffer_with(buffer_key, renderer),
        }
    }

    /// Returns the action attached to `line_index` for an open browser buffer.
    pub fn browser_action(&self, buffer_key: u64, line_index: usize) -> Option<DbBrowserAction> {
        self.browser_action_in(buffer_key, "", line_index)
    }

    /// Returns the action attached to `line_index` in a named browser section.
    pub fn browser_action_in(
        &self,
        buffer_key: u64,
        section: &str,
        line_index: usize,
    ) -> Option<DbBrowserAction> {
        self.browser_buffers
            .get(&(buffer_key, section.to_owned()))
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

    /// Executes every SQL statement in `sql` and concatenates the results.
    pub fn execute_sql_batch_for_buffer(
        &mut self,
        buffer_key: u64,
        sql: &str,
    ) -> Result<DbExecutionOutput, String> {
        let statements = split_sql_statements(sql);
        if statements.is_empty() {
            return Err("no SQL selected for execution".to_owned());
        }
        if statements.len() == 1 {
            return self.execute_sql_for_buffer(buffer_key, &statements[0]);
        }
        let mut lines = Vec::new();
        let mut row_count = 0usize;
        let mut first_title = None;
        for (index, statement) in statements.iter().enumerate() {
            if index > 0 {
                lines.push(String::new());
            }
            lines.push(format!("-- Query {}", index.saturating_add(1)));
            match self.execute_sql_for_buffer(buffer_key, statement) {
                Ok(output) => {
                    if first_title.is_none() {
                        first_title = Some(output.title.clone());
                    }
                    lines.push(output.title);
                    if !output.lines.is_empty() {
                        lines.push(String::new());
                        lines.extend(output.lines);
                    }
                    row_count = row_count.saturating_add(output.row_count);
                }
                Err(error) => {
                    lines.push("Query failed".to_owned());
                    lines.push(String::new());
                    lines.push(error);
                }
            }
        }
        Ok(DbExecutionOutput {
            title: first_title.unwrap_or_else(|| format!("{} queries", statements.len())),
            lines,
            row_count,
        })
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
            .get(&(buffer_key, String::new()))
            .map(|state| match state.kind {
                DbBrowserBufferKind::Connections => "connections",
                DbBrowserBufferKind::Schema { .. } => "schema",
                DbBrowserBufferKind::History => "history",
                DbBrowserBufferKind::Snippets => "snippets",
            })
    }

    pub(crate) fn session_summary(
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

    pub(crate) fn sqls_connection_config_for_session(
        &self,
        session_id: DbSessionId,
    ) -> Option<Value> {
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

    pub(crate) fn sqls_workspace_settings_for_session(
        &self,
        session_id: DbSessionId,
    ) -> Option<Value> {
        self.sqls_connection_config_for_session(session_id)
            .map(|connection| json!({ "connections": [connection] }))
    }

    pub(crate) fn sqls_initialization_options_for_session(
        &self,
        session_id: DbSessionId,
    ) -> Option<Value> {
        self.sqls_connection_config_for_session(session_id)
            .map(|connection| json!({ "connectionConfig": connection }))
    }

    pub(crate) fn remember_connection(
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

    pub(crate) fn delete_remembered(&mut self, alias: &str) -> Result<(), String> {
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

    pub(crate) fn touch_remembered_entry(&mut self, alias: &str) -> Result<(), String> {
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

    pub(crate) fn record_history(&mut self, session: &DbSession, sql: &str) -> Result<(), String> {
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

    pub(crate) fn save_persisted_state(&self) -> Result<(), String> {
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
