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
use super::manager::*;
#[allow(unused_imports)]
use super::notifications::*;
#[allow(unused_imports)]
use super::requests::*;
#[allow(unused_imports)]
use super::session::*;
#[allow(unused_imports)]
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
        let sessions = self.ensure_sessions_for_path(path, root, None, false)?;
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
        let handle = self.ensure_session_handle(server_id, session_root)?;
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
}
