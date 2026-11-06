use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, atomic::AtomicU64},
};

use editor_jobs::WorkspaceId;

use super::session::*;
use super::types::*;

impl LspClientManager {
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
        self.sync_buffer_with_edits_for_workspace(path, text, revision, root, edits, None)
    }

    /// Syncs a buffer and tags live Sessions for the editor Workspace.
    pub fn sync_buffer_with_edits_for_workspace(
        &self,
        path: &Path,
        text: impl Into<String>,
        revision: u64,
        root: Option<&Path>,
        edits: Option<&[editor_buffer::TextEdit]>,
        workspace_id: Option<u64>,
    ) -> Result<Vec<String>, LspClientError> {
        let workspace_id = workspace_id.map(WorkspaceId::from_raw);
        let sessions = self.ensure_sessions_for_path(path, root, None, false, workspace_id)?;
        self.sync_buffer_to_sessions(path, text.into(), revision, sessions, edits)
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
        let handle = self.ensure_session_handle(server_id, session_root, None)?;
        self.sync_buffer_to_sessions(path, text.into(), revision, vec![handle], edits)
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

    pub(crate) fn sync_buffer_to_sessions(
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
            let external_uri = state
                .external_documents
                .get(path)
                .map(|document| document.uri.clone());
            let tracked = state
                .tracked_buffers
                .entry(path.to_path_buf())
                .or_insert_with(TrackedBufferState::default);
            tracked.sessions = session_keys;
            if let Some(uri) = external_uri.as_deref() {
                for session in &sessions {
                    session.bind_external_document(path, uri);
                }
            }
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
            self.schedule_pulled_diagnostics(Arc::clone(session), path.to_path_buf());
        }

        Ok(labels)
    }

    /// Drains project-init / capability-registration refresh flags and pulls diagnostics.
    ///
    /// Safe to call from the shell refresh loop: work runs on a background thread.
    pub fn refresh_pull_diagnostics_if_needed(&self) {
        let sessions = self
            .state
            .lock()
            .map(|state| state.sessions.values().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let mut work = Vec::new();
        for session in sessions {
            if session.is_disconnected() || !session.take_diagnostic_refresh_requested() {
                continue;
            }
            for path in session.open_document_paths() {
                work.push((Arc::clone(&session), path));
            }
        }
        if work.is_empty() {
            return;
        }
        let dirty_diagnostic_paths = Arc::clone(&self.dirty_diagnostic_paths);
        let diagnostics_generation = Arc::clone(&self.diagnostics_generation);
        std::thread::spawn(move || {
            for (session, path) in work {
                record_session_pulled_diagnostics(
                    &session,
                    &path,
                    &dirty_diagnostic_paths,
                    &diagnostics_generation,
                );
            }
        });
    }

    fn schedule_pulled_diagnostics(&self, session: Arc<LspSessionHandle>, path: PathBuf) {
        if session.diagnostic_identifiers().is_empty() {
            return;
        }
        let dirty_diagnostic_paths = Arc::clone(&self.dirty_diagnostic_paths);
        let diagnostics_generation = Arc::clone(&self.diagnostics_generation);
        std::thread::spawn(move || {
            record_session_pulled_diagnostics(
                &session,
                &path,
                &dirty_diagnostic_paths,
                &diagnostics_generation,
            );
        });
    }
}

fn record_session_pulled_diagnostics(
    session: &LspSessionHandle,
    path: &Path,
    dirty_diagnostic_paths: &Arc<Mutex<BTreeSet<PathBuf>>>,
    diagnostics_generation: &AtomicU64,
) {
    if let Ok(Some(diagnostics)) = session.pull_document_diagnostics(path) {
        record_published_diagnostics(
            &session.diagnostics,
            dirty_diagnostic_paths,
            diagnostics_generation,
            path.to_path_buf(),
            diagnostics,
        );
    }
}
