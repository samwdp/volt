#![allow(unused_imports)]
use super::super::*;
#[allow(unused_imports)]
use crate::workspace_roots::*;

use std::{
    collections::{BTreeMap, BTreeSet},
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, SystemTime},
};

use editor_buffer::{TextPoint, TextRange};
use editor_jobs::{ProcessSupervisionMode, supervised_command_if_resolved};
use lsp_types::{
    ClientCapabilities, ClientInfo, CodeActionContext, CodeActionParams, CodeActionTriggerKind,
    CompletionParams, Diagnostic as LspDiagnostic, DiagnosticSeverity as LspDiagnosticSeverity,
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
    DidSaveTextDocumentParams, DocumentFormattingParams, DocumentRangeFormattingParams,
    Documentation, FormattingOptions, GotoDefinitionParams, GotoDefinitionResponse, HoverContents,
    HoverParams, InitializeParams, InitializeResult, InitializedParams, Location, LocationLink,
    MarkedString, MarkupKind, NumberOrString, ParameterLabel, PartialResultParams, Position, Range,
    ReferenceContext, ReferenceParams, SignatureHelp, SignatureHelpParams,
    TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, TextDocumentSyncCapability, TextDocumentSyncKind, TextEdit,
    TraceValue, Uri, VersionedTextDocumentIdentifier, WorkDoneProgressParams, WorkspaceFolder,
    notification::{
        DidChangeTextDocument, DidCloseTextDocument, DidOpenTextDocument, DidSaveTextDocument,
        Initialized, Notification,
    },
    request::{
        CodeActionRequest, Completion, Formatting, GotoDefinition, GotoImplementation,
        HoverRequest, Initialize, RangeFormatting, References, Request, SignatureHelpRequest,
    },
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerSession, LspError,
    LspWorkspaceDiagnostic,
};

#[allow(unused_imports)]
use super::completion::*;
#[allow(unused_imports)]
use super::documents::*;
#[allow(unused_imports)]
use super::manager::*;
#[allow(unused_imports)]
use super::notifications::*;
#[allow(unused_imports)]
use super::session::*;
#[allow(unused_imports)]
use super::types::*;

impl LspClientManager {
    pub(crate) fn session_shared_state(&self) -> LspSessionSharedState {
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

    pub(crate) fn update_server_settings_override<F>(
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

    pub(crate) fn update_server_initialization_options_override<F>(
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

    pub(crate) fn server_settings_override_contains_key(
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

    pub(crate) fn mark_dirty_diagnostic_path(&self, path: &Path) {
        if let Ok(mut dirty) = self.dirty_diagnostic_paths.lock() {
            dirty.insert(path.to_path_buf());
        }
    }

    pub(crate) fn memory_session_handle(
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

    pub(crate) fn tracked_sessions_for_path(
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

    pub(crate) fn tracked_sessions_and_version_for_path(
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
}
