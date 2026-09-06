use std::{
    collections::{BTreeMap, BTreeSet},
    io::Write,
    path::{Path, PathBuf},
    process::{Child, ChildStdin},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self},
    },
    time::Duration,
};

use editor_buffer::{TextPoint, TextRange};
use lsp_types::{
    ClientInfo, CompletionParams, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, DocumentFormattingParams,
    DocumentRangeFormattingParams, GotoDefinitionParams, HoverParams, InitializeParams,
    InitializeResult, InitializedParams, PartialResultParams, ReferenceContext, ReferenceParams,
    SignatureHelpParams, TextDocumentContentChangeEvent, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentSyncKind, TraceValue, Uri, VersionedTextDocumentIdentifier, WorkDoneProgressParams,
    WorkspaceFolder,
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

use crate::{Diagnostic, LanguageServerSession};

use super::types::*;

pub(crate) struct LspSessionHandle {
    pub(crate) key: SessionKey,
    pub(crate) session: LanguageServerSession,
    pub(crate) child: Mutex<Child>,
    pub(crate) writer: Arc<Mutex<ChildStdin>>,
    pub(crate) pending: PendingResponseMap,
    pub(crate) diagnostics: DiagnosticsByPath,
    pub(crate) open_documents: Mutex<BTreeMap<PathBuf, String>>,
    pub(crate) text_document_sync_kind: Mutex<TextDocumentSyncKind>,
    pub(crate) workspace_configuration: Arc<Mutex<SessionWorkspaceConfiguration>>,
    pub(crate) initialization_options: Option<Value>,
    pub(crate) transport_log: TransportLog,
    pub(crate) next_request_id: AtomicU64,
    pub(crate) next_progress_token: AtomicU64,
    pub(crate) disconnected: Arc<AtomicBool>,
    #[cfg(test)]
    pub(crate) fail_next_send: AtomicBool,
    pub(crate) needs_full_document: Mutex<BTreeSet<PathBuf>>,
    pub(crate) completion_resolve_supported: AtomicBool,
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

impl LspSessionHandle {
    pub(crate) fn start(
        session: LanguageServerSession,
        runtime_override: Option<Value>,
        initialization_options_override: Option<Value>,
        shared: LspSessionSharedState,
    ) -> Result<Arc<Self>, LspClientError> {
        let LspSessionSharedState {
            transport_log,
            notifications,
            diagnostics_generation,
            dirty_diagnostic_paths,
            sessions_generation,
        } = shared;
        let launch = session.launch();
        let launch_program = launch.program().to_owned();
        let launch_args = launch.args().to_vec();
        let launch_cwd = launch.cwd().cloned();
        let launch_env = launch.env().to_vec();
        let mut child = spawn_lsp_command(
            &launch_program,
            &launch_args,
            launch_cwd.as_deref(),
            &launch_env,
        )
        .map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to start language server `{}`: {error}",
                session.server_id()
            ))
        })?;
        let stdin = child.stdin.take().ok_or_else(|| {
            LspClientError::Protocol(format!(
                "language server `{}` is missing stdin pipe",
                session.server_id()
            ))
        })?;
        let stdout = child.stdout.take().ok_or_else(|| {
            LspClientError::Protocol(format!(
                "language server `{}` is missing stdout pipe",
                session.server_id()
            ))
        })?;

        let key = SessionKey::new(session.server_id(), session.root().map(PathBuf::as_path));
        let writer = Arc::new(Mutex::new(stdin));
        let pending = Arc::new(Mutex::new(BTreeMap::new()));
        let diagnostics = Arc::new(Mutex::new(BTreeMap::new()));
        let workspace_configuration = Arc::new(Mutex::new(SessionWorkspaceConfiguration::new(
            &session,
            runtime_override,
        )));
        let initialization_options = initialization_options_for_server(
            session.server_id(),
            initialization_options_override.as_ref(),
        );
        let disconnected = Arc::new(AtomicBool::new(false));
        let pid = child.id();
        let handle = Arc::new(Self {
            key,
            session,
            child: Mutex::new(child),
            writer: Arc::clone(&writer),
            pending: Arc::clone(&pending),
            diagnostics: Arc::clone(&diagnostics),
            open_documents: Mutex::new(BTreeMap::new()),
            text_document_sync_kind: Mutex::new(TextDocumentSyncKind::FULL),
            workspace_configuration: Arc::clone(&workspace_configuration),
            initialization_options,
            transport_log: Arc::clone(&transport_log),
            next_request_id: AtomicU64::new(1),
            next_progress_token: AtomicU64::new(1),
            disconnected: Arc::clone(&disconnected),
            #[cfg(test)]
            fail_next_send: AtomicBool::new(false),
            needs_full_document: Mutex::new(BTreeSet::new()),
            completion_resolve_supported: AtomicBool::new(false),
        });
        record_transport_event(
            &transport_log,
            handle.server_id(),
            launch_summary(
                pid,
                &launch_program,
                &launch_args,
                launch_cwd.as_deref(),
                handle.key.root.as_deref(),
            ),
        );
        record_notification(
            &notifications,
            session_lifecycle_notification(
                handle.server_id(),
                handle.key.root.as_deref(),
                LspNotificationLevel::Info,
                vec!["Starting language server".to_owned()],
                true,
            ),
        );
        spawn_reader_thread(
            stdout,
            LspReaderSession {
                server_id: handle.server_id().to_owned(),
                root: handle.key.root.clone(),
                writer,
                pending,
                diagnostics,
                workspace_configuration,
                disconnected,
                transport_log,
                notifications: Arc::clone(&notifications),
                diagnostics_generation,
                dirty_diagnostic_paths,
                sessions_generation,
            },
        );
        handle.initialize()?;
        record_notification(
            &notifications,
            session_lifecycle_notification(
                handle.server_id(),
                handle.key.root.as_deref(),
                LspNotificationLevel::Success,
                vec!["Ready".to_owned()],
                false,
            ),
        );
        Ok(handle)
    }

    pub(crate) fn server_id(&self) -> &str {
        self.session.server_id()
    }

    pub(crate) fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::Acquire)
    }

    pub(crate) fn initialize(&self) -> Result<(), LspClientError> {
        let root_uri = self
            .session
            .root()
            .map(|root| path_to_uri(root.as_path()))
            .transpose()?;
        let workspace_folders = root_uri.as_ref().map(|uri: &Uri| {
            vec![WorkspaceFolder {
                uri: uri.clone(),
                name: self
                    .session
                    .root()
                    .and_then(|root| root.file_name())
                    .and_then(|name| name.to_str())
                    .unwrap_or("workspace")
                    .to_owned(),
            }]
        });
        let capabilities = client_capabilities()?;
        let initialize_params = InitializeParams {
            process_id: Some(std::process::id()),
            initialization_options: self.initialization_options.clone(),
            capabilities,
            trace: Some(TraceValue::Off),
            workspace_folders,
            client_info: Some(ClientInfo {
                name: "volt".to_owned(),
                version: None,
            }),
            locale: None,
            work_done_progress_params: self.work_done_progress_params(Initialize::METHOD),
            ..InitializeParams::default()
        };
        let initialize_result = self.request_typed::<Initialize>(initialize_params)?;
        let initialize_result: InitializeResult = serde_json::from_value(initialize_result)
            .map_err(|error| {
                LspClientError::Protocol(format!(
                    "failed to decode initialize response for `{}`: {error}",
                    self.server_id()
                ))
            })?;
        self.set_text_document_sync_kind(&initialize_result)?;
        self.completion_resolve_supported.store(
            initialize_result
                .capabilities
                .completion_provider
                .as_ref()
                .and_then(|provider| provider.resolve_provider)
                .unwrap_or(false),
            Ordering::Release,
        );
        self.notify_typed::<Initialized>(InitializedParams {})?;
        if let Some(settings) = self.workspace_configuration_notification_payload(false)? {
            self.notify(
                "workspace/didChangeConfiguration",
                json!({ "settings": settings }),
            )?;
        }
        Ok(())
    }

    pub(crate) fn has_open_document(&self, path: &Path) -> bool {
        self.open_documents
            .lock()
            .map(|open_documents| open_documents.contains_key(path))
            .unwrap_or(false)
    }

    pub(crate) fn set_text_document_sync_kind(
        &self,
        initialize_result: &InitializeResult,
    ) -> Result<(), LspClientError> {
        let kind =
            text_document_sync_kind(initialize_result.capabilities.text_document_sync.clone());
        *self.text_document_sync_kind.lock().map_err(|_| {
            LspClientError::Protocol("LSP text sync kind mutex poisoned".to_owned())
        })? = kind;
        Ok(())
    }

    pub(crate) fn open_document_text(&self, path: &Path) -> Option<String> {
        self.open_documents
            .lock()
            .ok()
            .and_then(|open_documents| open_documents.get(path).cloned())
    }

    #[cfg(test)]
    pub(crate) fn fail_next_send(&self) {
        self.fail_next_send.store(true, Ordering::SeqCst);
    }

    pub(crate) fn path_needs_full_document(&self, path: &Path) -> bool {
        self.needs_full_document
            .lock()
            .map(|paths| paths.contains(path))
            .unwrap_or(true)
    }

    pub(crate) fn mark_needs_full_document(&self, path: &Path) {
        if let Ok(mut paths) = self.needs_full_document.lock() {
            paths.insert(path.to_path_buf());
        }
    }

    pub(crate) fn clear_needs_full_document(&self, path: &Path) {
        if let Ok(mut paths) = self.needs_full_document.lock() {
            paths.remove(path);
        }
    }

    pub(crate) fn sync_text_document(
        &self,
        path: &Path,
        version: i32,
        text: &str,
        incremental_changes: Option<&[TextDocumentContentChangeEvent]>,
    ) -> Result<(), LspClientError> {
        if self.has_open_document(path) {
            self.did_change(path, version, text, incremental_changes)
        } else {
            self.did_open(path, version, text.to_owned())
        }
    }

    pub(crate) fn did_open(
        &self,
        path: &Path,
        version: i32,
        text: String,
    ) -> Result<(), LspClientError> {
        self.notify_typed::<DidOpenTextDocument>(DidOpenTextDocumentParams {
            text_document: TextDocumentItem::new(
                path_to_uri(path)?,
                self.session.document_language_id_for_path(path).to_owned(),
                version,
                text.clone(),
            ),
        })?;
        self.open_documents
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP open documents mutex poisoned".to_owned()))?
            .insert(path.to_path_buf(), text);
        self.clear_needs_full_document(path);
        Ok(())
    }

    pub(crate) fn did_change(
        &self,
        path: &Path,
        version: i32,
        text: &str,
        incremental_changes: Option<&[TextDocumentContentChangeEvent]>,
    ) -> Result<(), LspClientError> {
        let force_full = self.path_needs_full_document(path);
        let used_incremental = !force_full && incremental_changes.is_some();
        let content_changes = {
            let open_documents = self.open_documents.lock().map_err(|_| {
                LspClientError::Protocol("LSP open documents mutex poisoned".to_owned())
            })?;
            let previous_text = open_documents.get(path).ok_or_else(|| {
                LspClientError::Protocol(format!(
                    "cannot send didChange for unopened document `{}`",
                    path.display()
                ))
            })?;
            let sync_kind = *self.text_document_sync_kind.lock().map_err(|_| {
                LspClientError::Protocol("LSP text sync kind mutex poisoned".to_owned())
            })?;
            if sync_kind == TextDocumentSyncKind::INCREMENTAL
                && !force_full
                && let Some(incremental_changes) = incremental_changes
            {
                incremental_changes.to_vec()
            } else {
                vec![text_document_content_change(sync_kind, previous_text, text)]
            }
        };
        let notify_result =
            self.notify_typed::<DidChangeTextDocument>(DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier::new(path_to_uri(path)?, version),
                content_changes,
            });
        match notify_result {
            Ok(()) => {
                self.open_documents
                    .lock()
                    .map_err(|_| {
                        LspClientError::Protocol("LSP open documents mutex poisoned".to_owned())
                    })?
                    .insert(path.to_path_buf(), text.to_owned());
                self.clear_needs_full_document(path);
                Ok(())
            }
            Err(error) => {
                if used_incremental {
                    self.mark_needs_full_document(path);
                }
                Err(error)
            }
        }
    }

    pub(crate) fn did_save(&self, path: &Path) -> Result<(), LspClientError> {
        self.notify_typed::<DidSaveTextDocument>(DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
            text: None,
        })
    }

    pub(crate) fn did_close(&self, path: &Path) -> Result<(), LspClientError> {
        if !self.has_open_document(path) {
            return Ok(());
        }
        self.notify_typed::<DidCloseTextDocument>(DidCloseTextDocumentParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
        })?;
        self.open_documents
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP open documents mutex poisoned".to_owned()))?
            .remove(path);
        self.clear_needs_full_document(path);
        Ok(())
    }

    pub(crate) fn did_focus(&self, path: &Path) -> Result<(), LspClientError> {
        self.notify(
            "textDocument/didFocus",
            json!({
                "textDocument": {
                    "uri": path_to_uri(path)?,
                }
            }),
        )
    }

    pub(crate) fn set_runtime_settings_override(
        &self,
        runtime_override: Option<Value>,
    ) -> Result<(), LspClientError> {
        let payload = {
            let mut workspace_configuration =
                self.workspace_configuration.lock().map_err(|_| {
                    LspClientError::Protocol(
                        "LSP workspace configuration mutex poisoned".to_owned(),
                    )
                })?;
            if !workspace_configuration.set_runtime_override(runtime_override) {
                return Ok(());
            }
            workspace_configuration.did_change_configuration_payload(true)
        };
        if let Some(settings) = payload {
            self.notify(
                "workspace/didChangeConfiguration",
                json!({ "settings": settings }),
            )?;
        }
        Ok(())
    }

    pub(crate) fn workspace_configuration_notification_payload(
        &self,
        include_null_section: bool,
    ) -> Result<Option<Value>, LspClientError> {
        self.workspace_configuration
            .lock()
            .map_err(|_| {
                LspClientError::Protocol("LSP workspace configuration mutex poisoned".to_owned())
            })
            .map(|workspace_configuration| {
                workspace_configuration.did_change_configuration_payload(include_null_section)
            })
    }

    pub(crate) fn hover(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Option<LspHoverContents>, LspClientError> {
        let response = self.request_typed::<HoverRequest>(HoverParams {
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(HoverRequest::METHOD),
        })?;
        Ok(parse_hover_response(self.server_id(), &response))
    }

    pub(crate) fn signature_help(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Option<LspSignatureHelpContents>, LspClientError> {
        let response = match self.request_typed::<SignatureHelpRequest>(SignatureHelpParams {
            context: None,
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(SignatureHelpRequest::METHOD),
        }) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        parse_signature_help_response(self.server_id(), &response)
    }

    pub(crate) fn completions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspCompletionItem>, LspClientError> {
        let response = self.request_typed::<Completion>(CompletionParams {
            text_document_position: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(Completion::METHOD),
            partial_result_params: PartialResultParams::default(),
            context: None,
        })?;
        parse_completion_response(self.server_id(), &response)
            .into_iter()
            .map(|item| self.resolve_completion_item(item))
            .collect()
    }

    pub(crate) fn resolve_completion_item(
        &self,
        item: LspCompletionItem,
    ) -> Result<LspCompletionItem, LspClientError> {
        if item.has_documentation || !self.completion_resolve_supported.load(Ordering::Acquire) {
            return Ok(item);
        }
        let response = match self.request("completionItem/resolve", item.raw_item.clone()) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(item),
            Err(error) => return Err(error),
        };
        let Value::Object(mut resolved) = response else {
            return Err(LspClientError::Protocol(format!(
                "language server `{}` returned a non-object completionItem/resolve response",
                self.server_id()
            )));
        };
        if let Value::Object(initial) = item.raw_item.clone() {
            for (key, value) in initial {
                resolved.entry(key).or_insert(value);
            }
        }
        parse_completion_item(self.server_id(), &Value::Object(resolved)).ok_or_else(|| {
            LspClientError::Protocol(format!(
                "language server `{}` returned an invalid completionItem/resolve response",
                self.server_id()
            ))
        })
    }

    pub(crate) fn inline_completion(
        &self,
        path: &Path,
        version: i32,
        position: TextPoint,
        options: LspFormattingOptions,
    ) -> Result<Option<LspInlineCompletionItem>, LspClientError> {
        let response = match self.request_with_timeout(
            INLINE_COMPLETION_METHOD,
            inline_completion_params(path, version, position, options)?,
            INLINE_COMPLETION_REQUEST_TIMEOUT,
        ) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        Ok(parse_inline_completion_response(
            self.server_id(),
            self.key.root.clone(),
            position,
            &response,
        )
        .into_iter()
        .next())
    }

    pub(crate) fn did_show_inline_completion(
        &self,
        item: &LspInlineCompletionItem,
    ) -> Result<(), LspClientError> {
        self.notify(
            "textDocument/didShowCompletion",
            json!({
                "item": item.raw_item.clone(),
            }),
        )
    }

    pub(crate) fn accept_inline_completion(
        &self,
        item: &LspInlineCompletionItem,
    ) -> Result<(), LspClientError> {
        let Some(params) = execute_command_params_from_inline_item(&item.raw_item) else {
            return Ok(());
        };
        let _ = self.request("workspace/executeCommand", params)?;
        Ok(())
    }

    pub(crate) fn execute_server_command(
        &self,
        command: &LspServerCommand,
    ) -> Result<(), LspClientError> {
        let _ = self.request("workspace/executeCommand", execute_command_params(command))?;
        Ok(())
    }

    pub(crate) fn copilot_sign_in(
        &self,
    ) -> Result<Option<CopilotDeviceCodePrompt>, LspClientError> {
        if !is_copilot_server(self.server_id()) {
            return Ok(None);
        }
        let response = self.request("signIn", json!({}))?;
        let prompt = parse_copilot_sign_in_response(&response).ok_or_else(|| {
            LspClientError::Protocol(format!(
                "language server `{}` returned an invalid Copilot sign-in response",
                self.server_id()
            ))
        })?;
        Ok(Some(prompt))
    }

    pub(crate) fn copilot_sign_out(&self) -> Result<(), LspClientError> {
        if !is_copilot_server(self.server_id()) {
            return Ok(());
        }
        let _ = self.request("signOut", json!({}))?;
        Ok(())
    }

    pub(crate) fn csharp_metadata(&self, uri: &str) -> Result<Option<Value>, LspClientError> {
        if !is_csharp_server(self.server_id()) || !is_csharp_metadata_uri(uri) {
            return Ok(None);
        }
        let response = self.request(
            CSHARP_METADATA_REQUEST_METHOD,
            csharp_metadata_request_params(uri),
        )?;
        parse_csharp_metadata_response(uri, &response)
    }

    pub(crate) fn definitions(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let response = self.request_typed::<GotoDefinition>(GotoDefinitionParams {
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(GotoDefinition::METHOD),
            partial_result_params: PartialResultParams::default(),
        })?;
        parse_definition_response(self.server_id(), &response)
    }

    pub(crate) fn references(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let response = self.request_typed::<References>(ReferenceParams {
            text_document_position: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(References::METHOD),
            partial_result_params: PartialResultParams::default(),
            context: ReferenceContext {
                include_declaration: false,
            },
        })?;
        parse_reference_response(self.server_id(), &response)
    }

    pub(crate) fn implementations(
        &self,
        path: &Path,
        position: TextPoint,
    ) -> Result<Vec<LspLocation>, LspClientError> {
        let response = self.request_typed::<GotoImplementation>(GotoDefinitionParams {
            text_document_position_params: text_document_position_params(path, position)?,
            work_done_progress_params: self.work_done_progress_params(GotoImplementation::METHOD),
            partial_result_params: PartialResultParams::default(),
        })?;
        parse_definition_response(self.server_id(), &response)
    }

    pub(crate) fn code_actions(
        &self,
        path: &Path,
        range: TextRange,
    ) -> Result<Vec<LspCodeAction>, LspClientError> {
        let range = range.normalized();
        let diagnostics = self
            .diagnostics_for_path(path)
            .into_iter()
            .filter(|diagnostic| diagnostic_matches_request_range(diagnostic.range(), range))
            .collect::<Vec<_>>();
        let response = match self.request_typed::<CodeActionRequest>(code_action_params(
            path,
            range,
            &diagnostics,
            self.work_done_progress_params(CodeActionRequest::METHOD),
        )?) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(Vec::new()),
            Err(error) => return Err(error),
        };
        parse_code_action_response(self.server_id(), &response)
    }

    pub(crate) fn formatting(
        &self,
        path: &Path,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let response = match self.request_typed::<Formatting>(DocumentFormattingParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
            options: lsp_formatting_options(options),
            work_done_progress_params: self.work_done_progress_params(Formatting::METHOD),
        }) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        parse_text_edit_response(self.server_id(), "formatting", &response)
    }

    pub(crate) fn range_formatting(
        &self,
        path: &Path,
        range: TextRange,
        options: LspFormattingOptions,
    ) -> Result<Option<Vec<LspTextEdit>>, LspClientError> {
        let response = match self.request_typed::<RangeFormatting>(DocumentRangeFormattingParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
            range: lsp_range_from_text_range(range),
            options: lsp_formatting_options(options),
            work_done_progress_params: self.work_done_progress_params(RangeFormatting::METHOD),
        }) {
            Ok(response) => response,
            Err(error) if unsupported_lsp_request(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
        parse_text_edit_response(self.server_id(), "range formatting", &response)
    }

    pub(crate) fn diagnostics_for_path(&self, path: &Path) -> Vec<Diagnostic> {
        self.diagnostics
            .lock()
            .ok()
            .and_then(|diagnostics| diagnostics.get(path).cloned())
            .unwrap_or_default()
    }

    pub(crate) fn notify_typed<N>(&self, params: N::Params) -> Result<(), LspClientError>
    where
        N: Notification,
        N::Params: Serialize,
    {
        let params = serde_json::to_value(params).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to encode LSP notification params for `{}`: {error}",
                N::METHOD
            ))
        })?;
        self.notify(N::METHOD, params)
    }

    pub(crate) fn request_typed<R>(&self, params: R::Params) -> Result<Value, LspClientError>
    where
        R: Request,
        R::Params: Serialize,
    {
        let params = serde_json::to_value(params).map_err(|error| {
            LspClientError::Protocol(format!(
                "failed to encode LSP request params for `{}`: {error}",
                R::METHOD
            ))
        })?;
        self.request(R::METHOD, params)
    }

    pub(crate) fn work_done_progress_params(&self, method: &str) -> WorkDoneProgressParams {
        work_done_progress_params(&self.next_progress_token, method)
    }

    pub(crate) fn notify(&self, method: &str, params: Value) -> Result<(), LspClientError> {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
    }

    pub(crate) fn request(&self, method: &str, params: Value) -> Result<Value, LspClientError> {
        let timeout = request_timeout_for_method(method);
        self.request_with_timeout(method, params, timeout)
    }

    pub(crate) fn request_with_timeout(
        &self,
        method: &str,
        params: Value,
        timeout: Duration,
    ) -> Result<Value, LspClientError> {
        let id = self.next_request_id.fetch_add(1, Ordering::AcqRel);
        let (sender, receiver) = mpsc::channel();
        self.pending
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP pending map mutex poisoned".to_owned()))?
            .insert(id, sender);
        let message = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        if let Err(error) = self.send_message(&message) {
            if let Ok(mut pending) = self.pending.lock() {
                pending.remove(&id);
            }
            return Err(error);
        }
        match receiver.recv_timeout(timeout) {
            Ok(result) => result,
            Err(_) => {
                if let Ok(mut pending) = self.pending.lock() {
                    pending.remove(&id);
                }
                record_transport_event(
                    &self.transport_log,
                    self.server_id(),
                    format!("timed out waiting for response to `{method}`"),
                );
                Err(LspClientError::Timeout(method.to_owned()))
            }
        }
    }

    pub(crate) fn send_message(&self, message: &Value) -> Result<(), LspClientError> {
        #[cfg(test)]
        if self.fail_next_send.swap(false, Ordering::SeqCst) {
            return Err(LspClientError::Protocol(
                "failed to send language server notification".to_owned(),
            ));
        }
        if self.is_disconnected() {
            record_transport_event(
                &self.transport_log,
                self.server_id(),
                "attempted to write after the server disconnected",
            );
            return Err(LspClientError::Disconnected(self.server_id().to_owned()));
        }
        let encoded = serde_json::to_vec(message).map_err(|error| {
            LspClientError::Protocol(format!("failed to encode JSON-RPC message: {error}"))
        })?;
        let mut writer = self
            .writer
            .lock()
            .map_err(|_| LspClientError::Protocol("LSP writer mutex poisoned".to_owned()))?;
        write!(writer, "Content-Length: {}\r\n\r\n", encoded.len())?;
        writer.write_all(&encoded)?;
        writer.flush()?;
        record_transport_message(
            &self.transport_log,
            self.server_id(),
            LspLogDirection::Outgoing,
            message,
        );
        Ok(())
    }
}
