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
use super::requests::*;
#[allow(unused_imports)]
use super::session::*;
#[allow(unused_imports)]
use super::types::*;

impl LspClientManager {
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
}
