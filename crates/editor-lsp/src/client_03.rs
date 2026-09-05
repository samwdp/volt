impl LspSessionHandle {
    fn start(
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

    fn server_id(&self) -> &str {
        self.session.server_id()
    }

    fn is_disconnected(&self) -> bool {
        self.disconnected.load(Ordering::Acquire)
    }

    fn initialize(&self) -> Result<(), LspClientError> {
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
        #[allow(deprecated)]
        let initialize_params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri,
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

    fn has_open_document(&self, path: &Path) -> bool {
        self.open_documents
            .lock()
            .map(|open_documents| open_documents.contains_key(path))
            .unwrap_or(false)
    }

    fn set_text_document_sync_kind(
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

    fn open_document_text(&self, path: &Path) -> Option<String> {
        self.open_documents
            .lock()
            .ok()
            .and_then(|open_documents| open_documents.get(path).cloned())
    }

    #[cfg(test)]
    fn fail_next_send(&self) {
        self.fail_next_send.store(true, Ordering::SeqCst);
    }

    fn path_needs_full_document(&self, path: &Path) -> bool {
        self.needs_full_document
            .lock()
            .map(|paths| paths.contains(path))
            .unwrap_or(true)
    }

    fn mark_needs_full_document(&self, path: &Path) {
        if let Ok(mut paths) = self.needs_full_document.lock() {
            paths.insert(path.to_path_buf());
        }
    }

    fn clear_needs_full_document(&self, path: &Path) {
        if let Ok(mut paths) = self.needs_full_document.lock() {
            paths.remove(path);
        }
    }

    fn sync_text_document(
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

    fn did_open(&self, path: &Path, version: i32, text: String) -> Result<(), LspClientError> {
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

    fn did_change(
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

    fn did_save(&self, path: &Path) -> Result<(), LspClientError> {
        self.notify_typed::<DidSaveTextDocument>(DidSaveTextDocumentParams {
            text_document: TextDocumentIdentifier::new(path_to_uri(path)?),
            text: None,
        })
    }

    fn did_close(&self, path: &Path) -> Result<(), LspClientError> {
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

    fn did_focus(&self, path: &Path) -> Result<(), LspClientError> {
        self.notify(
            "textDocument/didFocus",
            json!({
                "textDocument": {
                    "uri": path_to_uri(path)?,
                }
            }),
        )
    }

    fn set_runtime_settings_override(
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

    fn workspace_configuration_notification_payload(
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

    fn hover(
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

    fn signature_help(
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

    fn completions(
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

    fn resolve_completion_item(
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

    fn inline_completion(
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

    fn did_show_inline_completion(
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

    fn accept_inline_completion(
        &self,
        item: &LspInlineCompletionItem,
    ) -> Result<(), LspClientError> {
        let Some(params) = execute_command_params_from_inline_item(&item.raw_item) else {
            return Ok(());
        };
        let _ = self.request("workspace/executeCommand", params)?;
        Ok(())
    }

    fn execute_server_command(&self, command: &LspServerCommand) -> Result<(), LspClientError> {
        let _ = self.request("workspace/executeCommand", execute_command_params(command))?;
        Ok(())
    }

    fn copilot_sign_in(&self) -> Result<Option<CopilotDeviceCodePrompt>, LspClientError> {
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

    fn copilot_sign_out(&self) -> Result<(), LspClientError> {
        if !is_copilot_server(self.server_id()) {
            return Ok(());
        }
        let _ = self.request("signOut", json!({}))?;
        Ok(())
    }

    fn csharp_metadata(&self, uri: &str) -> Result<Option<Value>, LspClientError> {
        if !is_csharp_server(self.server_id()) || !is_csharp_metadata_uri(uri) {
            return Ok(None);
        }
        let response = self.request(
            CSHARP_METADATA_REQUEST_METHOD,
            csharp_metadata_request_params(uri),
        )?;
        parse_csharp_metadata_response(uri, &response)
    }

    fn definitions(
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

    fn references(
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

    fn implementations(
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

    fn code_actions(
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

    fn formatting(
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

    fn range_formatting(
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

    fn diagnostics_for_path(&self, path: &Path) -> Vec<Diagnostic> {
        self.diagnostics
            .lock()
            .ok()
            .and_then(|diagnostics| diagnostics.get(path).cloned())
            .unwrap_or_default()
    }

    fn notify_typed<N>(&self, params: N::Params) -> Result<(), LspClientError>
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

    fn request_typed<R>(&self, params: R::Params) -> Result<Value, LspClientError>
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

    fn work_done_progress_params(&self, method: &str) -> WorkDoneProgressParams {
        work_done_progress_params(&self.next_progress_token, method)
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), LspClientError> {
        self.send_message(&json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        }))
    }

    fn request(&self, method: &str, params: Value) -> Result<Value, LspClientError> {
        let timeout = request_timeout_for_method(method);
        self.request_with_timeout(method, params, timeout)
    }

    fn request_with_timeout(
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

    fn send_message(&self, message: &Value) -> Result<(), LspClientError> {
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

struct LspReaderSession {
    server_id: String,
    root: Option<PathBuf>,
    writer: Arc<Mutex<ChildStdin>>,
    pending: PendingResponseMap,
    diagnostics: DiagnosticsByPath,
    workspace_configuration: Arc<Mutex<SessionWorkspaceConfiguration>>,
    disconnected: Arc<AtomicBool>,
    transport_log: TransportLog,
    notifications: NotificationLog,
    diagnostics_generation: Arc<AtomicU64>,
    dirty_diagnostic_paths: Arc<Mutex<BTreeSet<PathBuf>>>,
    sessions_generation: Arc<AtomicU64>,
}

fn spawn_reader_thread(stdout: impl Read + Send + 'static, session: LspReaderSession) {
    thread::spawn(move || {
        let LspReaderSession {
            server_id,
            root,
            writer,
            pending,
            diagnostics,
            workspace_configuration,
            disconnected,
            transport_log,
            notifications,
            diagnostics_generation,
            dirty_diagnostic_paths,
            sessions_generation,
        } = session;
        let mut reader = BufReader::new(stdout);
        let mut progress_tracks = BTreeMap::<String, ProgressTrack>::new();
        loop {
            let message = match read_message(&mut reader) {
                Ok(Some(message)) => message,
                Ok(None) => {
                    record_transport_event(
                        &transport_log,
                        &server_id,
                        "language server closed the transport",
                    );
                    break;
                }
                Err(error) => {
                    record_transport_event(
                        &transport_log,
                        &server_id,
                        format!("transport read error: {error}"),
                    );
                    break;
                }
            };
            record_transport_message(
                &transport_log,
                &server_id,
                LspLogDirection::Incoming,
                &message,
            );
            let Some(object) = message.as_object() else {
                continue;
            };
            if object.contains_key("method") && object.contains_key("id") {
                let workspace_configuration = workspace_configuration
                    .lock()
                    .ok()
                    .map(|workspace_configuration| workspace_configuration.clone());
                let handling = server_request_response(
                    &server_id,
                    root.as_deref(),
                    object.get("method"),
                    object.get("params"),
                    workspace_configuration.as_ref(),
                );
                if let Some(notification) = handling.notification {
                    record_notification(&notifications, notification);
                }
                let id = object.get("id").cloned().unwrap_or(Value::Null);
                let response_message = json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": handling.result,
                });
                if let Err(error) =
                    write_response(&server_id, &transport_log, &writer, response_message)
                {
                    record_transport_event(
                        &transport_log,
                        &server_id,
                        format!("failed to reply to server request: {error}"),
                    );
                    break;
                }
                continue;
            }
            if let Some(id) = object.get("id").and_then(Value::as_u64) {
                let result = if let Some(error) = object.get("error") {
                    Err(LspClientError::Protocol(format!(
                        "language server `{server_id}` returned an error: {error}"
                    )))
                } else {
                    Ok(object.get("result").cloned().unwrap_or(Value::Null))
                };
                if let Ok(mut pending) = pending.lock()
                    && let Some(sender) = pending.remove(&id)
                {
                    let _ = sender.send(result);
                }
                continue;
            }
            if let Some(method) = object.get("method").and_then(Value::as_str) {
                if method == "textDocument/publishDiagnostics"
                    && let Some(params) = object.get("params")
                    && let Some((path, parsed)) = parse_publish_diagnostics(params)
                {
                    record_published_diagnostics(
                        &diagnostics,
                        &dirty_diagnostic_paths,
                        &diagnostics_generation,
                        path,
                        parsed,
                    );
                    continue;
                }
                if method == "$/progress"
                    && let Some(params) = object.get("params")
                    && let Some(notification) = parse_progress_notification(
                        &server_id,
                        root.as_deref(),
                        params,
                        &mut progress_tracks,
                    )
                {
                    record_notification(&notifications, notification);
                    continue;
                }
                if matches!(method, "window/showMessage" | "window/logMessage")
                    && let Some(params) = object.get("params")
                    && let Some(notification) = parse_window_message_notification(
                        method,
                        &server_id,
                        root.as_deref(),
                        params,
                    )
                {
                    record_notification(&notifications, notification);
                    continue;
                }
                if method == "didChangeStatus"
                    && let Some(params) = object.get("params")
                    && let Some(notification) =
                        parse_copilot_status_notification(&server_id, root.as_deref(), params)
                {
                    record_notification(&notifications, notification);
                    continue;
                }
            }
        }
        disconnected.store(true, Ordering::Release);
        note_session_disconnect_diagnostics(&diagnostics, &dirty_diagnostic_paths);
        diagnostics_generation.fetch_add(1, Ordering::Release);
        sessions_generation.fetch_add(1, Ordering::Release);
        record_transport_event(&transport_log, &server_id, "marked session disconnected");
        if let Ok(mut pending) = pending.lock() {
            for sender in pending.values() {
                let _ = sender.send(Err(LspClientError::Disconnected(server_id.clone())));
            }
            pending.clear();
        }
    });
}

fn configure_lsp_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn spawn_lsp_command(
    program: &str,
    args: &[String],
    cwd: Option<&Path>,
    env: &[(String, String)],
) -> std::io::Result<Child> {
    #[cfg(not(windows))]
    let spawn_result = build_lsp_command(program, args, cwd, env, None).spawn();

    #[cfg(windows)]
    let mut spawn_result = build_lsp_command(program, args, cwd, env, None).spawn();
    #[cfg(windows)]
    {
        let should_retry = matches!(
            &spawn_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry {
            for candidate in windows_launch_program_candidates(program) {
                spawn_result = build_lsp_command(&candidate, args, cwd, env, None).spawn();
                match &spawn_result {
                    Ok(_) => break,
                    Err(error) if windows_should_retry_spawn_error(error) => {}
                    Err(_) => break,
                }
            }
        }
        let should_retry_with_fnm = matches!(
            &spawn_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry_with_fnm && let Some(fnm_env) = windows_fnm_environment(cwd, env) {
            for candidate in windows_fnm_launch_program_candidates(program, &fnm_env) {
                spawn_result =
                    build_lsp_command(&candidate, args, cwd, env, Some(&fnm_env)).spawn();
                match &spawn_result {
                    Ok(_) => break,
                    Err(error) if windows_should_retry_spawn_error(error) => {}
                    Err(_) => break,
                }
            }
        }
        let should_retry_with_nvm = matches!(
            &spawn_result,
            Err(error) if windows_should_retry_spawn_error(error)
        );
        if should_retry_with_nvm && let Some(nvm_env) = windows_nvm_environment(cwd, env) {
            for candidate in windows_nvm_launch_program_candidates(program, &nvm_env) {
                spawn_result =
                    build_lsp_command(&candidate, args, cwd, env, Some(&nvm_env)).spawn();
                match &spawn_result {
                    Ok(_) => break,
                    Err(error) if windows_should_retry_spawn_error(error) => {}
                    Err(_) => break,
                }
            }
        }
    }
    spawn_result
}
