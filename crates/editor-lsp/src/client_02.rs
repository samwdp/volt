struct LspSessionHandle {
    key: SessionKey,
    session: LanguageServerSession,
    child: Mutex<Child>,
    writer: Arc<Mutex<ChildStdin>>,
    pending: PendingResponseMap,
    diagnostics: DiagnosticsByPath,
    open_documents: Mutex<BTreeMap<PathBuf, String>>,
    text_document_sync_kind: Mutex<TextDocumentSyncKind>,
    workspace_configuration: Arc<Mutex<SessionWorkspaceConfiguration>>,
    initialization_options: Option<Value>,
    transport_log: TransportLog,
    next_request_id: AtomicU64,
    next_progress_token: AtomicU64,
    disconnected: Arc<AtomicBool>,
    #[cfg(test)]
    fail_next_send: AtomicBool,
    needs_full_document: Mutex<BTreeSet<PathBuf>>,
    completion_resolve_supported: AtomicBool,
}

impl std::fmt::Debug for LspSessionHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LspSessionHandle")
            .field("server_id", &self.key.server_id)
            .field("root", &self.key.root)
            .finish_non_exhaustive()
    }
}

impl Drop for LspSessionHandle {
    fn drop(&mut self) {
        record_transport_event(
            &self.transport_log,
            &self.key.server_id,
            "terminating language server process",
        );
        if let Ok(mut child) = self.child.lock() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

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

    fn session_shared_state(&self) -> LspSessionSharedState {
        LspSessionSharedState {
            transport_log: Arc::clone(&self.transport_log),
            notifications: Arc::clone(&self.notifications),
            diagnostics_generation: Arc::clone(&self.diagnostics_generation),
            dirty_diagnostic_paths: Arc::clone(&self.dirty_diagnostic_paths),
            sessions_generation: Arc::clone(&self.sessions_generation),
        }
    }

    pub fn log_snapshot(&self) -> LspLogSnapshot {
        self.transport_log
            .lock()
            .map(|log| log.snapshot())
            .unwrap_or_default()
    }

    /// Transport log revision without cloning log lines.
    pub fn log_revision(&self) -> u64 {
        self.transport_log
            .lock()
            .map(|log| log.revision())
            .unwrap_or(0)
    }

    /// Clones the transport log only when its revision moved.
    pub fn log_snapshot_if_changed(&self, applied_revision: u64) -> Option<LspLogSnapshot> {
        let Ok(log) = self.transport_log.lock() else {
            return None;
        };
        (log.revision() != applied_revision).then(|| log.snapshot())
    }

    /// Returns a snapshot of recent UI-facing notifications emitted by the LSP client.
    pub fn notification_snapshot(&self) -> LspNotificationSnapshot {
        self.notifications
            .lock()
            .map(|log| log.snapshot())
            .unwrap_or_default()
    }

    /// Notification log revision without cloning entries.
    pub fn notification_revision(&self) -> u64 {
        self.notifications
            .lock()
            .map(|log| log.revision())
            .unwrap_or(0)
    }

    /// Clones UI notifications only when their revision moved.
    pub fn notification_snapshot_if_changed(
        &self,
        applied_revision: u64,
    ) -> Option<LspNotificationSnapshot> {
        let Ok(log) = self.notifications.lock() else {
            return None;
        };
        (log.revision() != applied_revision).then(|| log.snapshot())
    }

    pub fn set_server_settings_override(
        &self,
        server_id: &str,
        root: Option<&Path>,
        settings: Value,
    ) -> Result<bool, LspClientError> {
        self.update_server_settings_override(server_id, root, |_| Some(settings))
    }

    pub fn clear_server_settings_override(
        &self,
        server_id: &str,
        root: Option<&Path>,
    ) -> Result<bool, LspClientError> {
        self.update_server_settings_override(server_id, root, |_| None)
    }

    pub fn set_server_initialization_options_override(
        &self,
        server_id: &str,
        root: Option<&Path>,
        initialization_options: Value,
    ) -> Result<bool, LspClientError> {
        self.update_server_initialization_options_override(server_id, root, |_| {
            Some(initialization_options)
        })
    }

    pub fn clear_server_initialization_options_override(
        &self,
        server_id: &str,
        root: Option<&Path>,
    ) -> Result<bool, LspClientError> {
        self.update_server_initialization_options_override(server_id, root, |_| None)
    }

    pub fn set_csharp_solution_path_override(
        &self,
        root: Option<&Path>,
        solution_path: &Path,
    ) -> Result<bool, LspClientError> {
        self.update_server_settings_override(CSHARP_SERVER_ID, root, |current| {
            with_csharp_solution_path_override(current, solution_path)
        })
    }

    pub fn clear_csharp_solution_path_override(
        &self,
        root: Option<&Path>,
    ) -> Result<bool, LspClientError> {
        self.update_server_settings_override(CSHARP_SERVER_ID, root, |current| {
            without_csharp_solution_path_override(current)
        })
    }

    pub fn has_csharp_solution_path_override(
        &self,
        root: Option<&Path>,
    ) -> Result<bool, LspClientError> {
        self.server_settings_override_contains_key(
            CSHARP_SERVER_ID,
            root,
            Some(CSHARP_WORKSPACE_SECTION),
            "solutionPathOverride",
        )
    }

    pub fn planned_server_root_for_path(
        &self,
        server_id: &str,
        path: &Path,
        workspace_root: Option<&Path>,
    ) -> Result<Option<PathBuf>, LspClientError> {
        self.registry
            .prepare_session_for_path(server_id, path, workspace_root)
            .map(|session| session.root().cloned())
            .map_err(Into::into)
    }

    pub fn csharp_metadata(
        &self,
        root: Option<&Path>,
        uri: &str,
    ) -> Result<Option<Value>, LspClientError> {
        if !is_csharp_metadata_uri(uri) {
            return Ok(None);
        }
        let Some(session) = self.live_session_for_server(CSHARP_SERVER_ID, root)? else {
            return Ok(None);
        };
        session.csharp_metadata(uri)
    }

    pub fn last_synced_revision(&self, path: &Path) -> Option<u64> {
        self.state
            .lock()
            .ok()?
            .tracked_buffers
            .get(path)
            .map(|tracked| tracked.revision)
    }

    pub fn needs_sync(&self, path: &Path, revision: u64) -> bool {
        self.needs_sync_in_workspace(path, revision, None)
    }

    pub fn needs_sync_in_workspace(
        &self,
        path: &Path,
        revision: u64,
        workspace_root: Option<&Path>,
    ) -> bool {
        let Ok(state) = self.state.lock() else {
            return false;
        };
        if let Some(tracked) = state.tracked_buffers.get(path) {
            let has_live_session = tracked.sessions.iter().any(|key| {
                state
                    .sessions
                    .get(key)
                    .map(|session| !session.is_disconnected())
                    .unwrap_or(false)
            });
            if tracked.revision == revision && has_live_session {
                return false;
            }
        }
        let servers = self
            .registry
            .default_enabled_servers_for_path_in_workspace(path, workspace_root);
        if servers.is_empty() {
            return false;
        }
        let failed_server_ids = state
            .start_failures
            .keys()
            .map(|key| key.server_id.as_str())
            .collect::<BTreeSet<_>>();
        servers
            .into_iter()
            .any(|server| !failed_server_ids.contains(server.id()))
    }

    fn update_server_settings_override<F>(
        &self,
        server_id: &str,
        root: Option<&Path>,
        update: F,
    ) -> Result<bool, LspClientError>
    where
        F: FnOnce(Option<Value>) -> Option<Value>,
    {
        let key = SessionKey::new(server_id, root);
        let (session, updated_override) = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            let current = state.settings_overrides.get(&key).cloned();
            let updated_override = update(current.clone());
            if current == updated_override {
                return Ok(false);
            }
            match &updated_override {
                Some(settings) => {
                    state
                        .settings_overrides
                        .insert(key.clone(), settings.clone());
                }
                None => {
                    state.settings_overrides.remove(&key);
                }
            }
            (state.sessions.get(&key).cloned(), updated_override)
        };
        if let Some(session) = session {
            session.set_runtime_settings_override(updated_override.clone())?;
        }
        Ok(true)
    }

    fn update_server_initialization_options_override<F>(
        &self,
        server_id: &str,
        root: Option<&Path>,
        update: F,
    ) -> Result<bool, LspClientError>
    where
        F: FnOnce(Option<Value>) -> Option<Value>,
    {
        let key = SessionKey::new(server_id, root);
        let mut state = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
        let current = state.initialization_options_overrides.get(&key).cloned();
        let updated_override = update(current.clone());
        if current == updated_override {
            return Ok(false);
        }
        match updated_override {
            Some(initialization_options) => {
                state
                    .initialization_options_overrides
                    .insert(key, initialization_options);
            }
            None => {
                state.initialization_options_overrides.remove(&key);
            }
        }
        Ok(true)
    }

    fn server_settings_override_contains_key(
        &self,
        server_id: &str,
        root: Option<&Path>,
        section: Option<&str>,
        key: &str,
    ) -> Result<bool, LspClientError> {
        let session_key = SessionKey::new(server_id, root);
        let settings = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?
            .settings_overrides
            .get(&session_key)
            .cloned();
        Ok(settings_contains_key(
            normalized_workspace_configuration_settings(section, settings).as_ref(),
            key,
        ))
    }

    fn live_session_for_server(
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

    pub fn sync_buffer(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        root: Option<&Path>,
    ) -> Result<Vec<String>, LspClientError> {
        self.sync_buffer_with_edits(path, text, revision, root, None)
    }

    pub fn sync_buffer_with_edits(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        root: Option<&Path>,
        edits: Option<&[editor_buffer::TextEdit]>,
    ) -> Result<Vec<String>, LspClientError> {
        let sessions = self.ensure_sessions_for_path(path, root, None, false)?;
        self.sync_buffer_to_sessions(path, text.into(), revision, sessions, edits)
    }

    pub fn start_buffer_server(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        root: Option<&Path>,
        server_id: &str,
    ) -> Result<Vec<String>, LspClientError> {
        self.start_buffer_server_with_edits(path, text, revision, root, server_id, None)
    }

    pub fn start_buffer_server_with_edits(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        root: Option<&Path>,
        server_id: &str,
        edits: Option<&[editor_buffer::TextEdit]>,
    ) -> Result<Vec<String>, LspClientError> {
        let sessions = self.ensure_sessions_for_path(path, root, Some(server_id), true)?;
        self.sync_buffer_to_sessions(path, text.into(), revision, sessions, edits)
    }

    pub fn save_buffer(&self, path: &Path) -> Result<(), LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        for session in sessions {
            session.did_save(path)?;
        }
        Ok(())
    }

    pub fn close_buffer(&self, path: &Path) -> Result<(), LspClientError> {
        let sessions = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            let Some(tracked) = state.tracked_buffers.remove(path) else {
                return Ok(());
            };
            tracked
                .sessions
                .iter()
                .filter_map(|key| state.sessions.get(key).cloned())
                .collect::<Vec<_>>()
        };
        for session in sessions {
            session.did_close(path)?;
        }
        Ok(())
    }

    pub fn stop_buffer(&self, path: &Path) -> Result<(), LspClientError> {
        let session_keys = {
            let state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            state
                .tracked_buffers
                .get(path)
                .map(|tracked| tracked.sessions.clone())
                .unwrap_or_default()
        };
        self.close_buffer(path)?;
        self.shutdown_sessions(&session_keys)
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

    /// Stops one Session, then starts the same server id + root again.
    pub fn restart_session(
        &self,
        server_id: &str,
        root: Option<&Path>,
    ) -> Result<Vec<PathBuf>, LspClientError> {
        let tracked_paths = self.stop_session(server_id, root)?;
        self.ensure_session(server_id, root)?;
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

    /// Syncs a buffer onto an exact Language Server Session (server id + root).
    pub fn sync_buffer_onto_session(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        server_id: &str,
        session_root: Option<&Path>,
    ) -> Result<Vec<String>, LspClientError> {
        self.sync_buffer_onto_session_with_edits(
            path,
            text,
            revision,
            server_id,
            session_root,
            None,
        )
    }

    /// Syncs a buffer onto an exact Language Server Session, with an optional edit chain.
    pub fn sync_buffer_onto_session_with_edits(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        server_id: &str,
        session_root: Option<&Path>,
        edits: Option<&[editor_buffer::TextEdit]>,
    ) -> Result<Vec<String>, LspClientError> {
        let handle = self.ensure_session_handle(server_id, session_root)?;
        self.sync_buffer_to_sessions(path, text.into(), revision, vec![handle], edits)
    }

    fn ensure_session_handle(
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

    fn shutdown_sessions(&self, session_keys: &BTreeSet<SessionKey>) -> Result<(), LspClientError> {
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

    pub fn restart_buffer(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        root: Option<&Path>,
        preferred_server_id: Option<&str>,
    ) -> Result<Vec<String>, LspClientError> {
        let text = text.into();
        self.stop_buffer(path)?;
        if let Some(server_id) = preferred_server_id {
            return self.start_buffer_server(path, text, revision, root, server_id);
        }
        self.sync_buffer(path, text, revision, root)
    }

    pub fn supports_path(&self, path: &Path) -> bool {
        self.supports_path_in_workspace(path, None)
    }

    pub fn supports_path_in_workspace(&self, path: &Path, workspace_root: Option<&Path>) -> bool {
        !self
            .registry
            .default_enabled_servers_for_path_in_workspace(path, workspace_root)
            .is_empty()
    }

    pub fn registered_server_ids_for_path(&self, path: &Path) -> Vec<String> {
        self.registered_server_ids_for_path_in_workspace(path, None)
    }

    pub fn registered_server_ids_for_path_in_workspace(
        &self,
        path: &Path,
        workspace_root: Option<&Path>,
    ) -> Vec<String> {
        self.registry
            .servers_for_path(path)
            .into_iter()
            .filter(|server| {
                server.activation_markers().is_empty()
                    || self
                        .registry
                        .prepare_session_for_path(server.id(), path, workspace_root)
                        .is_ok()
            })
            .map(|server| server.id().to_owned())
            .collect()
    }

    pub fn diagnostics_for_path(&self, path: &Path) -> Vec<Diagnostic> {
        self.diagnostics_lookups.fetch_add(1, Ordering::Relaxed);
        let sessions = self.tracked_sessions_for_path(path).unwrap_or_default();
        let mut diagnostics = Vec::new();
        for session in sessions {
            diagnostics.extend(session.diagnostics_for_path(path));
        }
        diagnostics.sort_by_key(|diagnostic| {
            (
                diagnostic.range().start().line,
                diagnostic.range().start().column,
                diagnostic.severity() as u8,
            )
        });
        diagnostics
    }

    /// Monotonic generation bumped whenever published diagnostics change.
    pub fn diagnostics_generation(&self) -> u64 {
        self.diagnostics_generation.load(Ordering::Acquire)
    }

    /// Monotonic generation bumped when live Language Server Sessions are added or removed.
    pub fn sessions_generation(&self) -> u64 {
        self.sessions_generation.load(Ordering::Acquire)
    }

    /// Number of `diagnostics_for_path` lookups. Host tests use this to assert apply skip.
    pub fn diagnostics_for_path_lookups(&self) -> u64 {
        self.diagnostics_lookups.load(Ordering::Relaxed)
    }

    /// Paths whose published diagnostics changed since the previous take.
    pub fn take_dirty_diagnostic_paths(&self) -> BTreeSet<PathBuf> {
        self.dirty_diagnostic_paths
            .lock()
            .map(|mut dirty| std::mem::take(&mut *dirty))
            .unwrap_or_default()
    }

    /// Attaches an in-memory Language Server Session with diagnostics for `path`.
    ///
    /// Does not spawn the real language-server program. Used by host tests to
    /// drive the shell diagnostic apply seam.
    pub fn attach_memory_session(
        &self,
        server_id: &str,
        path: &Path,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<(), LspClientError> {
        let handle = if let Some(existing) = self.live_session_for_server(server_id, None)? {
            if let Ok(mut guard) = existing.diagnostics.lock() {
                guard.insert(path.to_path_buf(), diagnostics);
            }
            existing
        } else {
            self.memory_session_handle(server_id, path, diagnostics)?
        };
        let key = handle.key.clone();
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            state.sessions.insert(key.clone(), Arc::clone(&handle));
            let tracked = state.tracked_buffers.entry(path.to_path_buf()).or_default();
            tracked.sessions.insert(key);
            if tracked.revision == 0 {
                tracked.revision = 1;
                tracked.version = 1;
            }
        }
        self.mark_dirty_diagnostic_path(path);
        self.diagnostics_generation.fetch_add(1, Ordering::Release);
        self.sessions_generation.fetch_add(1, Ordering::Release);
        Ok(())
    }

    /// Records diagnostics as if a Language Server Session published them for `path`.
    pub fn apply_published_diagnostics(
        &self,
        path: &Path,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<(), LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let Some(session) = sessions.first() else {
            return Err(LspClientError::Protocol(format!(
                "no live Language Server Session tracks `{}`",
                path.display()
            )));
        };
        record_published_diagnostics(
            &session.diagnostics,
            &self.dirty_diagnostic_paths,
            &self.diagnostics_generation,
            path.to_path_buf(),
            diagnostics,
        );
        Ok(())
    }

    /// Marks live Sessions for `path` disconnected and bumps diagnostics generation.
    pub fn disconnect_memory_sessions_for_path(&self, path: &Path) -> Result<(), LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        if sessions.is_empty() {
            return Ok(());
        }
        for session in &sessions {
            note_session_disconnect_diagnostics(&session.diagnostics, &self.dirty_diagnostic_paths);
            session.disconnected.store(true, Ordering::Release);
        }
        self.diagnostics_generation.fetch_add(1, Ordering::Release);
        self.sessions_generation.fetch_add(1, Ordering::Release);
        Ok(())
    }

    /// Records a transport-log event without cloning a snapshot.
    pub fn record_transport_log_event(&self, server_id: &str, message: impl Into<String>) {
        record_transport_event(&self.transport_log, server_id, message);
    }

    /// Records a `window/showMessage`-style UI notification.
    pub fn record_show_message(&self, server_id: &str, message: impl Into<String>) {
        let server_id = server_id.to_owned();
        let message = message.into();
        record_notification(
            &self.notifications,
            LspNotification {
                key: format!("window:{server_id}:{message}"),
                server_id: server_id.clone(),
                root: None,
                level: LspNotificationLevel::Info,
                title: format!("LSP · {server_id}"),
                body_lines: vec![message],
                progress: None,
                active: false,
                action: None,
            },
        );
    }

    fn mark_dirty_diagnostic_path(&self, path: &Path) {
        if let Ok(mut dirty) = self.dirty_diagnostic_paths.lock() {
            dirty.insert(path.to_path_buf());
        }
    }

    fn memory_session_handle(
        &self,
        server_id: &str,
        path: &Path,
        diagnostics: Vec<Diagnostic>,
    ) -> Result<Arc<LspSessionHandle>, LspClientError> {
        let session = self
            .registry
            .prepare_session_for_path(server_id, path, None)?;
        let workspace_configuration = Arc::new(Mutex::new(SessionWorkspaceConfiguration::new(
            &session, None,
        )));
        let (child, writer) = spawn_inert_child().map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to spawn inert Language Server Session for `{server_id}`: {error}"
            ))
        })?;
        let mut diagnostics_by_path = BTreeMap::new();
        diagnostics_by_path.insert(path.to_path_buf(), diagnostics);
        Ok(Arc::new(LspSessionHandle {
            key: SessionKey::new(server_id, session.root().map(PathBuf::as_path)),
            session,
            child: Mutex::new(child),
            writer: Arc::new(Mutex::new(writer)),
            pending: Arc::new(Mutex::new(BTreeMap::new())),
            diagnostics: Arc::new(Mutex::new(diagnostics_by_path)),
            open_documents: Mutex::new(BTreeMap::new()),
            text_document_sync_kind: Mutex::new(TextDocumentSyncKind::FULL),
            workspace_configuration,
            initialization_options: None,
            transport_log: Arc::clone(&self.transport_log),
            next_request_id: AtomicU64::new(1),
            next_progress_token: AtomicU64::new(1),
            disconnected: Arc::new(AtomicBool::new(false)),
            #[cfg(test)]
            fail_next_send: AtomicBool::new(false),
            needs_full_document: Mutex::new(BTreeSet::new()),
            completion_resolve_supported: AtomicBool::new(false),
        }))
    }

    pub fn workspace_diagnostics(&self) -> Vec<LspWorkspaceDiagnostic> {
        let sessions = self
            .state
            .lock()
            .map(|state| state.sessions.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut diagnostics = Vec::new();
        for session in sessions {
            if session.is_disconnected() {
                continue;
            }
            let server_id = session.key.server_id.clone();
            let snapshot = session
                .diagnostics
                .lock()
                .map(|diagnostics_by_path| {
                    let mut entries = Vec::new();
                    for (path, path_diagnostics) in diagnostics_by_path.iter() {
                        for diagnostic in path_diagnostics {
                            entries.push(LspWorkspaceDiagnostic::new(
                                server_id.clone(),
                                path.clone(),
                                diagnostic.clone(),
                            ));
                        }
                    }
                    entries
                })
                .unwrap_or_default();
            diagnostics.extend(snapshot);
        }
        diagnostics
    }

    pub fn hover(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspHoverContents>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut results = Vec::new();
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            if let Some(hover) = session.hover(path, position)? {
                results.push(hover);
            }
        }
        Ok(results)
    }

    pub fn signature_help(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspSignatureHelpContents>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut results = Vec::new();
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            if let Some(signature_help) = session.signature_help(path, position)? {
                results.push(signature_help);
            }
        }
        Ok(results)
    }

    pub fn completions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspCompletionItem>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut items = Vec::new();
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            items.extend(session.completions(path, position)?);
        }
        Ok(items)
    }

    pub fn inline_completion(
        &self,
        path: &Path,
        position: TextPoint,
        options: LspFormattingOptions,
    ) -> Result<Option<LspInlineCompletionItem>, LspClientError> {
        let (version, sessions) = self.tracked_sessions_and_version_for_path(path)?;
        for session in sessions {
            if !is_copilot_server(session.server_id()) {
                continue;
            }
            session.did_focus(path)?;
            if let Some(item) = session.inline_completion(path, version, position, options)? {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }

    pub fn did_show_inline_completion(
        &self,
        item: &LspInlineCompletionItem,
    ) -> Result<(), LspClientError> {
        if let Some(session) =
            self.live_session_for_server(&item.server_id, item.root.as_deref())?
        {
            session.did_show_inline_completion(item)?;
        }
        Ok(())
    }

    pub fn execute_server_command(
        &self,
        server_id: &str,
        root: Option<&Path>,
        command: &LspServerCommand,
    ) -> Result<(), LspClientError> {
        let session = self
            .live_session_for_server(server_id, root)?
            .ok_or_else(|| {
                LspClientError::Protocol(format!("language server `{server_id}` is not running"))
            })?;
        session.execute_server_command(command)
    }

    pub fn copilot_sign_in(
        &self,
        root: Option<&Path>,
    ) -> Result<Option<CopilotDeviceCodePrompt>, LspClientError> {
        let Some(session) = self.live_session_for_server(COPILOT_SERVER_ID, root)? else {
            return Ok(None);
        };
        session.copilot_sign_in()
    }

    pub fn copilot_sign_out(&self, root: Option<&Path>) -> Result<bool, LspClientError> {
        let Some(session) = self.live_session_for_server(COPILOT_SERVER_ID, root)? else {
            return Ok(false);
        };
        session.copilot_sign_out()?;
        Ok(true)
    }

    pub fn accept_inline_completion(
        &self,
        item: &LspInlineCompletionItem,
    ) -> Result<(), LspClientError> {
        if let Some(session) =
            self.live_session_for_server(&item.server_id, item.root.as_deref())?
        {
            session.accept_inline_completion(item)?;
        }
        Ok(())
    }

    pub fn definitions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut locations = Vec::new();
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            locations.extend(session.definitions(path, position)?);
        }
        sort_locations(&mut locations);
        Ok(locations)
    }

    pub fn references(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut locations = Vec::new();
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            locations.extend(session.references(path, position)?);
        }
        sort_locations(&mut locations);
        Ok(locations)
    }

    pub fn implementations(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut locations = Vec::new();
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            locations.extend(session.implementations(path, position)?);
        }
        sort_locations(&mut locations);
        Ok(locations)
    }

    pub fn code_actions(
        &self,
        path: &Path,
        range: TextRange,
    ) -> Result<Vec<LspCodeAction>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        let mut actions = Vec::new();
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            actions.extend(session.code_actions(path, range)?);
        }
        actions.sort_by(|left, right| {
            right
                .preferred
                .cmp(&left.preferred)
                .then_with(|| left.title.cmp(&right.title))
                .then_with(|| left.server_id.cmp(&right.server_id))
                .then_with(|| left.kind.cmp(&right.kind))
        });
        Ok(actions)
    }

    pub fn formatting(
        &self,
        path: &Path,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            if let Some(edits) = session.formatting(path, options)? {
                return Ok(Some(edits));
            }
        }
        Ok(None)
    }

    pub fn range_formatting(
        &self,
        path: &Path,
        range: TextRange,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let sessions = self.tracked_sessions_for_path(path)?;
        for session in sessions {
            if is_copilot_server(session.server_id()) {
                continue;
            }
            if let Some(edits) = session.range_formatting(path, range, options)? {
                return Ok(Some(edits));
            }
        }
        Ok(None)
    }

    pub fn session_labels_for_path(&self, path: &Path) -> Vec<String> {
        let mut labels = self
            .tracked_sessions_for_path(path)
            .unwrap_or_default()
            .into_iter()
            .map(|session| session.server_id().to_owned())
            .collect::<Vec<_>>();
        labels.sort();
        labels.dedup();
        labels
    }

    pub fn has_live_sessions_for_path(&self, path: &Path) -> bool {
        self.tracked_sessions_for_path(path)
            .map(|sessions| !sessions.is_empty())
            .unwrap_or(false)
    }

    fn sync_buffer_to_sessions(
        &self,
        path: &Path,
        text: String,
        revision: u64,
        sessions: Vec<Arc<LspSessionHandle>>,
        edits: Option<&[editor_buffer::TextEdit]>,
    ) -> Result<Vec<String>, LspClientError> {
        if sessions.is_empty() {
            if let Ok(mut state) = self.state.lock() {
                state.tracked_buffers.remove(path);
            }
            return Ok(Vec::new());
        }

        let session_keys = sessions
            .iter()
            .map(|session| session.key.clone())
            .collect::<BTreeSet<_>>();
        let mut labels = sessions
            .iter()
            .map(|session| session.server_id().to_owned())
            .collect::<Vec<_>>();
        labels.sort();
        labels.dedup();
        let (version, last_revision) = {
            let mut state = self
                .state
                .lock()
                .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
            let last_revision = state
                .tracked_buffers
                .get(path)
                .map(|tracked| tracked.revision);
            let already_synced = last_revision == Some(revision)
                && sessions.iter().all(|session| {
                    session.has_open_document(path) && !session.path_needs_full_document(path)
                });
            let tracked = state
                .tracked_buffers
                .entry(path.to_path_buf())
                .or_insert_with(TrackedBufferState::default);
            tracked.sessions = session_keys;
            if already_synced {
                return Ok(labels);
            }
            tracked.version = tracked.version.saturating_add(1).max(1);
            tracked.revision = revision;
            (tracked.version, last_revision)
        };

        let usable_edits = usable_edit_chain(edits, last_revision, revision);
        let previous_text = sessions
            .iter()
            .find_map(|session| session.open_document_text(path));
        let incremental_changes = match (previous_text.as_deref(), usable_edits) {
            (Some(previous_text), Some(edits)) => {
                incremental_content_changes(previous_text, &text, edits)
            }
            _ => None,
        };

        for session in &sessions {
            session.sync_text_document(path, version, &text, incremental_changes.as_deref())?;
        }

        Ok(labels)
    }

    fn tracked_sessions_for_path(
        &self,
        path: &Path,
    ) -> Result<Vec<Arc<LspSessionHandle>>, LspClientError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
        let Some(tracked) = state.tracked_buffers.get(path).cloned() else {
            return Ok(Vec::new());
        };
        let mut sessions = Vec::new();
        let mut stale_keys = Vec::new();
        for key in tracked.sessions {
            if let Some(session) = state.sessions.get(&key) {
                if session.is_disconnected() {
                    stale_keys.push(key);
                } else {
                    sessions.push(Arc::clone(session));
                }
            }
        }
        for key in stale_keys {
            state.sessions.remove(&key);
        }
        Ok(sessions)
    }

    fn tracked_sessions_and_version_for_path(
        &self,
        path: &Path,
    ) -> Result<(i32, Vec<Arc<LspSessionHandle>>), LspClientError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP state mutex poisoned".to_owned()))?;
        let Some(tracked) = state.tracked_buffers.get(path).cloned() else {
            return Ok((0, Vec::new()));
        };
        let mut sessions = Vec::new();
        let mut stale_keys = Vec::new();
        for key in tracked.sessions {
            if let Some(session) = state.sessions.get(&key) {
                if session.is_disconnected() {
                    stale_keys.push(key);
                } else {
                    sessions.push(Arc::clone(session));
                }
            }
        }
        for key in stale_keys {
            state.sessions.remove(&key);
        }
        Ok((tracked.version, sessions))
    }

    fn ensure_sessions_for_path(
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
