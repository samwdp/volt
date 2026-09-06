use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::{LanguageServerRegistry, LspError};

use super::session::*;
use super::types::*;

impl LspClientManager {
    pub fn new(registry: LanguageServerRegistry) -> Self {
        Self {
            registry,
            state: Arc::new(Mutex::new(LspClientState::default())),
            transport_log: Arc::new(Mutex::new(LspTransportLog::new(TRANSPORT_LOG_MAX_ENTRIES))),
            notifications: Arc::new(Mutex::new(LspNotificationLog::new(
                NOTIFICATION_LOG_MAX_ENTRIES,
            ))),
            diagnostics_generation: Arc::new(AtomicU64::new(0)),
            dirty_diagnostic_paths: Arc::new(Mutex::new(BTreeSet::new())),
            sessions_generation: Arc::new(AtomicU64::new(0)),
            diagnostics_lookups: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Returns the language-server registry.
    pub fn registry(&self) -> &LanguageServerRegistry {
        &self.registry
    }

    pub(crate) fn live_session_for_server(
        &self,
        server_id: &str,
        root: Option<&Path>,
    ) -> Result<Option<Arc<LspSessionHandle>>, LspClientError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
        if let Some(root) = root {
            let key = SessionKey::new(server_id, Some(root));
            let disconnected = state
                .sessions
                .get(&key)
                .map(|session| session.is_disconnected())
                .unwrap_or(false);
            if disconnected {
                state.sessions.remove(&key);
                return Ok(None);
            }
            return Ok(state.sessions.get(&key).cloned());
        }

        let mut live_sessions = Vec::new();
        let mut stale_keys = Vec::new();
        for (key, session) in &state.sessions {
            if key.server_id != server_id {
                continue;
            }
            if session.is_disconnected() {
                stale_keys.push(key.clone());
                continue;
            }
            live_sessions.push(Arc::clone(session));
        }
        for key in stale_keys {
            state.sessions.remove(&key);
        }

        match live_sessions.len() {
            0 => Ok(None),
            1 => Ok(live_sessions.into_iter().next()),
            _ => Err(LspClientError::Protocol(format!(
                "multiple live `{server_id}` sessions matched the request; provide a workspace root"
            ))),
        }
    }

    pub fn stop_sessions_for_root(&self, root: Option<&Path>) -> Result<(), LspClientError> {
        let root = normalize_session_root(root);
        let session_keys = {
            let state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            state
                .sessions
                .keys()
                .filter(|key| key.root == root)
                .cloned()
                .collect::<BTreeSet<_>>()
        };
        self.shutdown_sessions(&session_keys)
    }

    /// Lists live Language Server Sessions in scope for the active Workspace picker.
    pub fn live_sessions_for_workspace(
        &self,
        open_buffer_paths: &[PathBuf],
        project_workspace_root: Option<&Path>,
    ) -> Result<Vec<LspLiveSession>, LspClientError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
        let mut stale_keys = Vec::new();
        let mut candidates = Vec::new();
        for (key, session) in &state.sessions {
            if session.is_disconnected() {
                stale_keys.push(key.clone());
                continue;
            }
            let tracked_paths = state
                .tracked_buffers
                .iter()
                .filter(|(_, tracked)| tracked.sessions.contains(key))
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>();
            let live = LspLiveSession::new(key.server_id.clone(), key.root.clone());
            if language_server_session_in_workspace_scope(
                live.root(),
                &tracked_paths,
                open_buffer_paths,
                project_workspace_root,
            ) {
                candidates.push(live);
            }
        }
        for key in stale_keys {
            state.sessions.remove(&key);
        }
        candidates.sort();
        Ok(candidates)
    }

    /// Shuts down one live Language Server Session. Returns paths that were tracked to it.
    pub fn stop_session(
        &self,
        server_id: &str,
        root: Option<&Path>,
    ) -> Result<Vec<PathBuf>, LspClientError> {
        let key = SessionKey::new(server_id, root);
        let tracked_paths = {
            let state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            state
                .tracked_buffers
                .iter()
                .filter(|(_, tracked)| tracked.sessions.contains(&key))
                .map(|(path, _)| path.clone())
                .collect::<Vec<_>>()
        };
        for path in &tracked_paths {
            self.close_buffer(path)?;
        }
        let mut keys = BTreeSet::new();
        keys.insert(key);
        self.shutdown_sessions(&keys)?;
        Ok(tracked_paths)
    }

    /// Ensures a live Session exists for the exact server id + root.
    pub fn ensure_session(
        &self,
        server_id: &str,
        root: Option<&Path>,
    ) -> Result<(), LspClientError> {
        let _ = self.ensure_session_handle(server_id, root)?;
        Ok(())
    }

    pub(crate) fn ensure_session_handle(
        &self,
        server_id: &str,
        root: Option<&Path>,
    ) -> Result<Arc<LspSessionHandle>, LspClientError> {
        let key = SessionKey::new(server_id, root);
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            if let Some(existing) = state.sessions.get(&key).cloned() {
                if existing.is_disconnected() {
                    state.sessions.remove(&key);
                } else {
                    return Ok(existing);
                }
            }
            state.start_failures.remove(&key);
        }
        let session = self
            .registry
            .prepare_session(server_id, key.root.clone())
            .map_err(LspClientError::from)?;
        let (runtime_override, initialization_options_override) = {
            let state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            (
                state.settings_overrides.get(&key).cloned(),
                state.initialization_options_overrides.get(&key).cloned(),
            )
        };
        let handle = LspSessionHandle::start(
            session,
            runtime_override,
            initialization_options_override,
            self.session_shared_state(),
        )?;
        let mut state = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
        state.sessions.insert(key, Arc::clone(&handle));
        self.sessions_generation.fetch_add(1, Ordering::Release);
        Ok(handle)
    }

    pub(crate) fn shutdown_sessions(
        &self,
        session_keys: &BTreeSet<SessionKey>,
    ) -> Result<(), LspClientError> {
        if session_keys.is_empty() {
            return Ok(());
        }
        let mut dirty_paths = Vec::new();
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            for key in session_keys {
                if let Some(session) = state.sessions.remove(key)
                    && let Ok(diagnostics) = session.diagnostics.lock()
                {
                    dirty_paths.extend(diagnostics.keys().cloned());
                }
                state.start_failures.remove(key);
            }
            for tracked in state.tracked_buffers.values_mut() {
                for key in session_keys {
                    tracked.sessions.remove(key);
                }
            }
        }
        if let Ok(mut dirty) = self.dirty_diagnostic_paths.lock() {
            dirty.extend(dirty_paths);
        }
        self.diagnostics_generation.fetch_add(1, Ordering::Release);
        self.sessions_generation.fetch_add(1, Ordering::Release);
        Ok(())
    }

    pub(crate) fn ensure_sessions_for_path(
        &self,
        path: &Path,
        root: Option<&Path>,
        preferred_server_id: Option<&str>,
        force_retry: bool,
    ) -> Result<Vec<Arc<LspSessionHandle>>, LspClientError> {
        let mut handles = if preferred_server_id.is_none() {
            self.tracked_sessions_for_path(path)?
        } else {
            Vec::new()
        };
        let mut handled_keys = handles
            .iter()
            .map(|handle| handle.key.clone())
            .collect::<BTreeSet<_>>();
        let session_plans = if let Some(server_id) = preferred_server_id {
            vec![
                self.registry
                    .prepare_session_for_path(server_id, path, root)?,
            ]
        } else {
            match self.registry.prepare_sessions_for_path(path, root) {
                Ok(session_plans) => session_plans,
                Err(LspError::UnknownExtension(_)) if !handles.is_empty() => Vec::new(),
                Err(error) => return Err(error.into()),
            }
        };

        for session in session_plans {
            let key = SessionKey::new(session.server_id(), session.root().map(PathBuf::as_path));
            if handled_keys.contains(&key) {
                continue;
            }
            let (existing, runtime_override, initialization_options_override, start_failed) = {
                let mut state = self
                    .state
                    .lock()
                    .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
                if force_retry {
                    state.start_failures.remove(&key);
                    if state
                        .sessions
                        .get(&key)
                        .map(|session| session.is_disconnected())
                        .unwrap_or(false)
                    {
                        state.sessions.remove(&key);
                    }
                }
                (
                    state.sessions.get(&key).cloned(),
                    state.settings_overrides.get(&key).cloned(),
                    state.initialization_options_overrides.get(&key).cloned(),
                    state.start_failures.contains_key(&key),
                )
            };
            if let Some(existing) = existing {
                handled_keys.insert(key);
                handles.push(existing);
                continue;
            }
            if start_failed {
                continue;
            }
            match LspSessionHandle::start(
                session,
                runtime_override,
                initialization_options_override,
                self.session_shared_state(),
            ) {
                Ok(handle) => {
                    self.state
                        .lock()
                        .map_err(|_| {
                            LspClientError::Protocol("LSP state mutex poisoned".to_owned())
                        })?
                        .sessions
                        .insert(key, Arc::clone(&handle));
                    self.sessions_generation.fetch_add(1, Ordering::Release);
                    handled_keys.insert(handle.key.clone());
                    handles.push(handle);
                }
                Err(error) => {
                    record_transport_event(
                        &self.transport_log,
                        &key.server_id,
                        format!("failed to start language server: {error}"),
                    );
                    record_notification(
                        &self.notifications,
                        session_lifecycle_notification(
                            &key.server_id,
                            key.root.as_deref(),
                            LspNotificationLevel::Error,
                            vec![
                                "Failed to start language server".to_owned(),
                                error.to_string(),
                            ],
                            false,
                        ),
                    );
                    self.state
                        .lock()
                        .map_err(|_| {
                            LspClientError::Protocol("LSP state mutex poisoned".to_owned())
                        })?
                        .start_failures
                        .insert(key.clone(), error.to_string());
                    if preferred_server_id.is_some() {
                        return Err(error);
                    }
                }
            }
        }
        Ok(handles)
    }
}
