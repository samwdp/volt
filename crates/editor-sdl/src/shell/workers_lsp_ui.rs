#[derive(Debug, Clone)]
struct LspUiHoverPayload {
    hovers: Vec<editor_lsp::LspHoverContents>,
    signatures: Vec<editor_lsp::LspSignatureHelpContents>,
}

#[derive(Clone, Copy)]
enum LspUiLocationKind {
    Definition,
    References,
    Implementation,
}

enum LspUiRequestKind {
    Hover {
        signature_point: TextPoint,
        focused: bool,
    },
    Locations {
        title: String,
        kind: LspUiLocationKind,
    },
    Format {
        options: LspFormattingOptions,
        range: Option<TextRange>,
        original_cursor: TextPoint,
        then_save: bool,
        workspace_id: WorkspaceId,
        extension: Option<String>,
        cwd: Option<PathBuf>,
    },
    CodeActions {
        range: TextRange,
        workspace_id: WorkspaceId,
    },
}

struct LspUiWorkerRequest {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: PathBuf,
    text: String,
    root: Option<PathBuf>,
    cursor: TextPoint,
    kind: LspUiRequestKind,
    lsp_client: Arc<LspClientManager>,
    edits: Option<Vec<editor_buffer::TextEdit>>,
}

struct LspUiCodeActionsPayload {
    labels: Vec<String>,
    actions: Vec<LspCodeAction>,
    path: PathBuf,
    workspace_id: WorkspaceId,
}

struct LspUiFormatPayload {
    edits: Option<Vec<LspTextEdit>>,
    original_cursor: TextPoint,
    then_save: bool,
    workspace_id: WorkspaceId,
    path: PathBuf,
    extension: Option<String>,
    cwd: Option<PathBuf>,
    range: Option<TextRange>,
}

struct LspUiWorkerResult {
    buffer_id: BufferId,
    buffer_revision: u64,
    cursor: TextPoint,
    hover: Option<(LspUiHoverPayload, bool)>,
    locations: Option<(String, Vec<editor_lsp::LspLocation>)>,
    format: Option<LspUiFormatPayload>,
    code_actions: Option<LspUiCodeActionsPayload>,
    error: Option<String>,
}

struct LspUiWorkerState {
    request_tx: Sender<LspUiWorkerRequest>,
    results: Arc<Mutex<Vec<LspUiWorkerResult>>>,
}

impl LspUiWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<LspUiWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                let result = run_lsp_ui_request(request);
                if let Ok(mut results) = worker_results.lock() {
                    results.push(result);
                    ping_shell_wakeup();
                } else {
                    return;
                }
            }
        });
        Self {
            request_tx,
            results,
        }
    }

    fn schedule(&mut self, request: LspUiWorkerRequest) {
        let _ = self.request_tx.send(request);
    }

    fn take_results(&mut self) -> Vec<LspUiWorkerResult> {
        let Ok(mut results) = self.results.lock() else {
            return Vec::new();
        };
        results.drain(..).collect()
    }
}

fn lsp_ui_empty_result(request: &LspUiWorkerRequest, error: Option<String>) -> LspUiWorkerResult {
    LspUiWorkerResult {
        buffer_id: request.buffer_id,
        buffer_revision: request.buffer_revision,
        cursor: request.cursor,
        hover: None,
        locations: None,
        format: None,
        code_actions: None,
        error,
    }
}

fn run_lsp_ui_request(request: LspUiWorkerRequest) -> LspUiWorkerResult {
    let sync_error = request
        .lsp_client
        .sync_buffer_with_edits(
            &request.path,
            request.text.clone(),
            request.buffer_revision,
            request.root.as_deref(),
            request.edits.as_deref(),
        )
        .err()
        .map(|error| error.to_string());
    let mut result = lsp_ui_empty_result(&request, sync_error);
    match request.kind {
        LspUiRequestKind::Hover {
            signature_point,
            focused,
        } => {
            let hovers = request
                .lsp_client
                .hover(&request.path, request.cursor)
                .unwrap_or_default();
            let signatures = request
                .lsp_client
                .signature_help(&request.path, signature_point)
                .unwrap_or_default();
            result.hover = Some((LspUiHoverPayload { hovers, signatures }, focused));
        }
        LspUiRequestKind::Locations { title, kind } => {
            let locations = match kind {
                LspUiLocationKind::Definition => {
                    request.lsp_client.definitions(&request.path, request.cursor)
                }
                LspUiLocationKind::References => {
                    request.lsp_client.references(&request.path, request.cursor)
                }
                LspUiLocationKind::Implementation => request
                    .lsp_client
                    .implementations(&request.path, request.cursor),
            };
            match locations {
                Ok(locations) => result.locations = Some((title, locations)),
                Err(error) => result.error = Some(error.to_string()),
            }
        }
        LspUiRequestKind::Format {
            options,
            range,
            original_cursor,
            then_save,
            workspace_id,
            extension,
            cwd,
        } => {
            let edits = match range {
                Some(range) => request
                    .lsp_client
                    .range_formatting(&request.path, range, options),
                None => request.lsp_client.formatting(&request.path, options),
            };
            match edits {
                Ok(edits) => {
                    result.format = Some(LspUiFormatPayload {
                        edits,
                        original_cursor,
                        then_save,
                        workspace_id,
                        path: request.path.clone(),
                        extension,
                        cwd,
                        range,
                    });
                }
                Err(error) => result.error = Some(error.to_string()),
            }
        }
        LspUiRequestKind::CodeActions {
            range,
            workspace_id,
        } => {
            let labels = request.lsp_client.session_labels_for_path(&request.path);
            match request.lsp_client.code_actions(&request.path, range) {
                Ok(actions) => {
                    result.code_actions = Some(LspUiCodeActionsPayload {
                        labels,
                        actions,
                        path: request.path.clone(),
                        workspace_id,
                    });
                }
                Err(error) => result.error = Some(error.to_string()),
            }
        }
    }
    result
}
