struct PendingVimSearchRequest {
    due_at: Instant,
    request: VimSearchWorkerRequest,
}

struct VimSearchWorkerRequest {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    text: TextSnapshot,
    direction: VimSearchDirection,
    query: String,
}

struct VimSearchWorkerResult {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    direction: VimSearchDirection,
    query: String,
    data: SearchPickerData,
}

struct VimSearchWorkerState {
    pending: Option<PendingVimSearchRequest>,
    next_request_id: u64,
    request_tx: Sender<VimSearchWorkerRequest>,
    results: Arc<Mutex<Vec<VimSearchWorkerResult>>>,
}

impl VimSearchWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<VimSearchWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(newer_request) = request_rx.try_recv() {
                    request = newer_request;
                }
                let data = vim_search_entries(&request.text, request.direction, &request.query);
                if let Ok(mut results) = worker_results.lock() {
                    results.push(VimSearchWorkerResult {
                        request_id: request.request_id,
                        buffer_id: request.buffer_id,
                        buffer_revision: request.buffer_revision,
                        direction: request.direction,
                        query: request.query,
                        data,
                    });
                    ping_shell_wakeup();
                } else {
                    return;
                }
            }
        });

        Self {
            pending: None,
            next_request_id: 0,
            request_tx,
            results,
        }
    }

    fn clear_pending(&mut self) {
        self.pending = None;
    }

    fn schedule(
        &mut self,
        buffer_id: BufferId,
        buffer_revision: u64,
        text: TextSnapshot,
        direction: VimSearchDirection,
        query: String,
    ) {
        const SEARCH_REFRESH_DEBOUNCE: Duration = Duration::from_millis(100);
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending = Some(PendingVimSearchRequest {
            due_at: Instant::now() + SEARCH_REFRESH_DEBOUNCE,
            request: VimSearchWorkerRequest {
                request_id: self.next_request_id,
                buffer_id,
                buffer_revision,
                text,
                direction,
                query,
            },
        });
    }

    fn dispatch_due(&mut self, now: Instant) {
        let Some(pending) = self.pending.as_ref() else {
            return;
        };
        if now < pending.due_at {
            return;
        }
        let request = self.pending.take().map(|pending| pending.request);
        if let Some(request) = request {
            let _ = self.request_tx.send(request);
        }
    }

    fn next_due_at(&self) -> Option<Instant> {
        self.pending.as_ref().map(|pending| pending.due_at)
    }

    fn take_latest_result(&self) -> Option<VimSearchWorkerResult> {
        let mut results = self.results.lock().ok()?;
        results.drain(..).next_back()
    }
}

struct PendingWorkspaceSearchRequest {
    due_at: Instant,
    request: WorkspaceSearchWorkerRequest,
}

struct WorkspaceSearchWorkerRequest {
    request_id: u64,
    root: PathBuf,
    query: String,
}

struct WorkspaceSearchWorkerResult {
    request_id: u64,
    root: PathBuf,
    query: String,
    data: SearchPickerData,
}

struct WorkspaceSearchWorkerState {
    pending: Option<PendingWorkspaceSearchRequest>,
    next_request_id: u64,
    request_tx: Sender<WorkspaceSearchWorkerRequest>,
    results: Arc<Mutex<Vec<WorkspaceSearchWorkerResult>>>,
}

impl WorkspaceSearchWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<WorkspaceSearchWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(newer_request) = request_rx.try_recv() {
                    request = newer_request;
                }
                let data = workspace_search_entries(&request.root, &request.query);
                if let Ok(mut results) = worker_results.lock() {
                    results.push(WorkspaceSearchWorkerResult {
                        request_id: request.request_id,
                        root: request.root,
                        query: request.query,
                        data,
                    });
                    ping_shell_wakeup();
                } else {
                    return;
                }
            }
        });

        Self {
            pending: None,
            next_request_id: 0,
            request_tx,
            results,
        }
    }

    fn clear_pending(&mut self) {
        self.pending = None;
    }

    fn schedule(&mut self, root: PathBuf, query: String) {
        const SEARCH_REFRESH_DEBOUNCE: Duration = Duration::from_millis(50);
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending = Some(PendingWorkspaceSearchRequest {
            due_at: Instant::now() + SEARCH_REFRESH_DEBOUNCE,
            request: WorkspaceSearchWorkerRequest {
                request_id: self.next_request_id,
                root,
                query,
            },
        });
    }

    fn dispatch_due(&mut self, now: Instant) {
        let Some(pending) = self.pending.as_ref() else {
            return;
        };
        if now < pending.due_at {
            return;
        }
        let request = self.pending.take().map(|pending| pending.request);
        if let Some(request) = request {
            let _ = self.request_tx.send(request);
        }
    }

    fn next_due_at(&self) -> Option<Instant> {
        self.pending.as_ref().map(|pending| pending.due_at)
    }

    fn take_latest_result(&self) -> Option<WorkspaceSearchWorkerResult> {
        let mut results = self.results.lock().ok()?;
        results.drain(..).next_back()
    }
}

struct FileReloadWorkerRequest {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: PathBuf,
    loaded_fingerprint: Option<BackingFileFingerprint>,
}

enum FileReloadWorkerOutcome {
    Missing,
    Unchanged {
        fingerprint: BackingFileFingerprint,
    },
    Reloaded {
        fingerprint: BackingFileFingerprint,
        text: TextBuffer,
    },
}

struct FileReloadWorkerResult {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: PathBuf,
    outcome: Result<FileReloadWorkerOutcome, String>,
}

enum FileReloadWorkerCommand {
    WatchPath(PathBuf),
    UnwatchPath(PathBuf),
    Reload(FileReloadWorkerRequest),
}

struct FileReloadWorkerState {
    command_tx: Sender<FileReloadWorkerCommand>,
    changed_paths: Arc<Mutex<Vec<PathBuf>>>,
    results: Arc<Mutex<Vec<FileReloadWorkerResult>>>,
    errors: Arc<Mutex<Vec<String>>>,
    watched_paths: HashMap<PathBuf, usize>,
}

impl FileReloadWorkerState {
    fn new() -> Self {
        let (command_tx, command_rx) = mpsc::channel::<FileReloadWorkerCommand>();
        let changed_paths = Arc::new(Mutex::new(Vec::new()));
        let results = Arc::new(Mutex::new(Vec::new()));
        let errors = Arc::new(Mutex::new(Vec::new()));
        let watcher_changed_paths = Arc::clone(&changed_paths);
        let worker_results = Arc::clone(&results);
        let worker_errors = Arc::clone(&errors);
        std::thread::spawn(move || {
            let mut watcher =
                match create_file_reload_watcher(watcher_changed_paths, Arc::clone(&worker_errors))
                {
                    Ok(watcher) => watcher,
                    Err(error) => {
                        push_file_reload_worker_error(
                            &worker_errors,
                            format!("failed to start file watcher: {error}"),
                        );
                        return;
                    }
                };
            while let Ok(command) = command_rx.recv() {
                match command {
                    FileReloadWorkerCommand::WatchPath(path) => {
                        if let Err(error) =
                            watcher.watch(path.as_path(), RecursiveMode::NonRecursive)
                        {
                            push_file_reload_worker_error(
                                &worker_errors,
                                format!("failed to watch `{}`: {error}", path.display()),
                            );
                        }
                    }
                    FileReloadWorkerCommand::UnwatchPath(path) => {
                        if let Err(error) = watcher.unwatch(path.as_path()) {
                            push_file_reload_worker_error(
                                &worker_errors,
                                format!("failed to stop watching `{}`: {error}", path.display()),
                            );
                        }
                    }
                    FileReloadWorkerCommand::Reload(request) => {
                        let result = process_file_reload_request(request);
                        if let Ok(mut results) = worker_results.lock() {
                            results.push(result);
                            ping_shell_wakeup();
                        } else {
                            return;
                        }
                    }
                }
            }
        });

        Self {
            command_tx,
            changed_paths,
            results,
            errors,
            watched_paths: HashMap::new(),
        }
    }

    fn watch_path(&mut self, path: PathBuf) {
        let entry = self.watched_paths.entry(path.clone()).or_default();
        *entry = entry.saturating_add(1);
        if *entry == 1 {
            let _ = self
                .command_tx
                .send(FileReloadWorkerCommand::WatchPath(path));
        }
    }

    fn unwatch_path(&mut self, path: &Path) {
        let Some(count) = self.watched_paths.get_mut(path) else {
            return;
        };
        if *count > 1 {
            *count -= 1;
            return;
        }
        self.watched_paths.remove(path);
        let _ = self
            .command_tx
            .send(FileReloadWorkerCommand::UnwatchPath(path.to_path_buf()));
    }

    fn send(&self, request: FileReloadWorkerRequest) {
        let _ = self
            .command_tx
            .send(FileReloadWorkerCommand::Reload(request));
    }

    fn take_changed_paths(&self) -> Vec<PathBuf> {
        let Ok(mut changed_paths) = self.changed_paths.lock() else {
            return Vec::new();
        };
        changed_paths.drain(..).collect()
    }

    fn take_results(&self) -> Vec<FileReloadWorkerResult> {
        let Ok(mut results) = self.results.lock() else {
            return Vec::new();
        };
        results.drain(..).collect()
    }

    fn take_errors(&self) -> Vec<String> {
        let Ok(mut errors) = self.errors.lock() else {
            return Vec::new();
        };
        errors.drain(..).collect()
    }

    #[cfg(test)]
    fn record_changed_path_for_test(&self, path: PathBuf) {
        if let Ok(mut changed_paths) = self.changed_paths.lock() {
            changed_paths.push(path);
        }
    }
}

fn process_file_reload_request(request: FileReloadWorkerRequest) -> FileReloadWorkerResult {
    let outcome = match BackingFileFingerprint::read(&request.path) {
        Ok(fingerprint) => match request.loaded_fingerprint {
            Some(loaded_fingerprint) if fingerprint == loaded_fingerprint => {
                Ok(FileReloadWorkerOutcome::Unchanged { fingerprint })
            }
            None => Ok(FileReloadWorkerOutcome::Unchanged { fingerprint }),
            Some(_) => match TextBuffer::load_from_path(&request.path) {
                Ok(text) => Ok(FileReloadWorkerOutcome::Reloaded { fingerprint, text }),
                Err(error) if error.kind() == io::ErrorKind::NotFound => {
                    Ok(FileReloadWorkerOutcome::Missing)
                }
                Err(error) => Err(format!(
                    "failed to reload `{}`: {error}",
                    request.path.display()
                )),
            },
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(FileReloadWorkerOutcome::Missing)
        }
        Err(error) => Err(format!(
            "failed to stat `{}`: {error}",
            request.path.display()
        )),
    };

    FileReloadWorkerResult {
        buffer_id: request.buffer_id,
        buffer_revision: request.buffer_revision,
        path: request.path,
        outcome,
    }
}

fn create_file_reload_watcher(
    changed_paths: Arc<Mutex<Vec<PathBuf>>>,
    errors: Arc<Mutex<Vec<String>>>,
) -> notify::Result<RecommendedWatcher> {
    recommended_watcher(move |event: notify::Result<NotifyEvent>| match event {
        Ok(event) => enqueue_file_reload_event(event, &changed_paths),
        Err(error) => {
            push_file_reload_worker_error(
                &errors,
                format!("failed to receive file watcher event: {error}"),
            );
        }
    })
}

fn enqueue_file_reload_event(event: NotifyEvent, changed_paths: &Arc<Mutex<Vec<PathBuf>>>) {
    if !matches!(
        event.kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
    ) {
        return;
    }
    if let Ok(mut queued_paths) = changed_paths.lock() {
        queued_paths.extend(event.paths);
        ping_shell_wakeup();
    }
}

fn push_file_reload_worker_error(errors: &Arc<Mutex<Vec<String>>>, message: String) {
    if let Ok(mut errors) = errors.lock() {
        errors.push(message);
        ping_shell_wakeup();
    }
}

#[derive(Clone)]
struct SyntaxRefreshWorkerRequest {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: Option<PathBuf>,
    buffer_language_id: Option<String>,
    syntax_window: Option<SyntaxLineWindow>,
    rainbow_parens_enabled: bool,
    text: TextBuffer,
}

enum SyntaxWorkerMessage {
    Refresh(Box<SyntaxRefreshWorkerRequest>),
    PreloadBatch {
        language_ids: Vec<String>,
        done: Option<Sender<()>>,
    },
}

struct SyntaxRefreshWorkerResult {
    buffer_id: BufferId,
    buffer_revision: u64,
    path: Option<PathBuf>,
    buffer_language_id: Option<String>,
    language_id: Option<String>,
    syntax_window: Option<SyntaxLineWindow>,
    compute_elapsed: Duration,
    highlight_span_count: usize,
    syntax_result: Option<Result<IndexedSyntaxLines, String>>,
}

#[derive(Debug, Default, Clone, Copy)]
struct SyntaxRefreshStats {
    changed: bool,
    worker_compute: Duration,
    result_count: usize,
    highlight_spans: usize,
}

const SYNTAX_REFRESH_WORKER_STACK_SIZE: usize = 64 * 1024 * 1024;
const SYNTAX_REFRESH_REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

struct SyntaxRefreshWorkerState {
    request_tx: Option<Sender<SyntaxWorkerMessage>>,
    results: Arc<Mutex<Vec<SyntaxRefreshWorkerResult>>>,
    configs: Vec<LanguageConfiguration>,
    install_root: Option<PathBuf>,
    query_asset_root: Option<PathBuf>,
}

impl SyntaxRefreshWorkerState {
    fn disabled() -> Self {
        Self {
            request_tx: None,
            results: Arc::new(Mutex::new(Vec::new())),
            configs: Vec::new(),
            install_root: None,
            query_asset_root: None,
        }
    }

    fn configure(
        &mut self,
        configs: Vec<LanguageConfiguration>,
        install_root: PathBuf,
        query_asset_root: Option<PathBuf>,
    ) {
        self.configs = configs;
        self.install_root = Some(install_root);
        self.query_asset_root = query_asset_root;
        // Drop the sender so the existing worker exits; the next send/preload restarts it
        // with the updated language configs.
        self.request_tx = None;
        self.results = Arc::new(Mutex::new(Vec::new()));
    }

    fn ensure_worker(&mut self) -> bool {
        if self.request_tx.is_some() {
            return true;
        }
        let Some(install_root) = self.install_root.clone() else {
            return false;
        };
        let query_asset_root = self.query_asset_root.clone();
        let configs = self.configs.clone();
        let (request_tx, request_rx) = mpsc::channel::<SyntaxWorkerMessage>();
        let worker_results = Arc::clone(&self.results);
        let spawn_result = std::thread::Builder::new()
            .name("volt-syntax-refresh".to_owned())
            .stack_size(SYNTAX_REFRESH_WORKER_STACK_SIZE)
            .spawn(move || {
                let mut registry = SyntaxRegistry::with_install_root(install_root);
                registry.set_query_asset_root(query_asset_root);
                for config in configs {
                    if let Err(error) = registry.register(config) {
                        eprintln!("failed to register syntax worker language: {error}");
                    }
                }
                let mut parse_sessions = BTreeMap::<BufferId, SyntaxParseSession>::new();
                while let Ok(first) = request_rx.recv() {
                    let mut refreshes = BTreeMap::<BufferId, SyntaxRefreshWorkerRequest>::new();
                    let mut preload_batches = Vec::<(Vec<String>, Option<Sender<()>>)>::new();
                    match first {
                        SyntaxWorkerMessage::Refresh(request) => {
                            refreshes.insert(request.buffer_id, *request);
                        }
                        SyntaxWorkerMessage::PreloadBatch { language_ids, done } => {
                            preload_batches.push((language_ids, done));
                        }
                    }
                    while let Ok(message) = request_rx.try_recv() {
                        match message {
                            SyntaxWorkerMessage::Refresh(request) => {
                                refreshes.insert(request.buffer_id, *request);
                            }
                            SyntaxWorkerMessage::PreloadBatch { language_ids, done } => {
                                preload_batches.push((language_ids, done));
                            }
                        }
                    }
                    // Finish queued preloads before refresh so a cold language still
                    // loads on the worker, not the UI thread. Callers do not wait.
                    for (language_ids, done) in preload_batches {
                        for language_id in language_ids {
                            if let Err(error) = registry.preload_language(&language_id) {
                                eprintln!(
                                    "tree-sitter worker prewarm failed for `{language_id}`: {error}"
                                );
                            }
                        }
                        if let Some(done) = done {
                            let _ = done.send(());
                        }
                    }
                    for request in refreshes.into_values() {
                        let result = process_syntax_refresh_request(
                            &mut registry,
                            &mut parse_sessions,
                            request,
                        );
                        if let Ok(mut results) = worker_results.lock() {
                            results.push(result);
                            ping_shell_wakeup();
                        } else {
                            return;
                        }
                    }
                }
            });
        match spawn_result {
            Ok(_) => {
                self.request_tx = Some(request_tx);
                true
            }
            Err(error) => {
                eprintln!("failed to start syntax refresh worker: {error}");
                false
            }
        }
    }

    fn is_configured(&self) -> bool {
        self.install_root.is_some()
    }

    #[cfg(test)]
    fn has_live_worker(&self) -> bool {
        self.request_tx.is_some()
    }

    fn preload_languages<I, S>(&mut self, language_ids: I) -> bool
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let language_ids = language_ids.into_iter().map(Into::into).collect::<Vec<_>>();
        if language_ids.is_empty() {
            return true;
        }
        self.send_preload_batch(language_ids, None)
    }

    fn send_preload_batch(&mut self, language_ids: Vec<String>, done: Option<Sender<()>>) -> bool {
        if !self.ensure_worker() {
            return false;
        }
        let message = SyntaxWorkerMessage::PreloadBatch { language_ids, done };
        let sent = self
            .request_tx
            .as_ref()
            .cloned()
            .and_then(|tx| tx.send(message).ok());
        if sent.is_none() {
            self.request_tx = None;
            return false;
        }
        true
    }

    #[cfg(test)]
    fn wait_for_pending_preloads(&mut self, timeout: Duration) -> bool {
        let (done_tx, done_rx) = mpsc::channel();
        if !self.send_preload_batch(Vec::new(), Some(done_tx)) {
            return false;
        }
        done_rx.recv_timeout(timeout).is_ok()
    }

    fn send(&mut self, request: SyntaxRefreshWorkerRequest) -> bool {
        if !self.ensure_worker() {
            return false;
        }
        let message = SyntaxWorkerMessage::Refresh(Box::new(request.clone()));
        if self
            .request_tx
            .as_ref()
            .cloned()
            .and_then(|tx| tx.send(message).ok())
            .is_some()
        {
            return true;
        }
        self.request_tx = None;
        if !self.ensure_worker() {
            return false;
        }
        self.request_tx
            .as_ref()
            .cloned()
            .and_then(|tx| {
                tx.send(SyntaxWorkerMessage::Refresh(Box::new(request)))
                    .ok()
            })
            .is_some()
    }

    fn take_results(&self) -> Vec<SyntaxRefreshWorkerResult> {
        let Ok(mut results) = self.results.lock() else {
            return Vec::new();
        };
        results.drain(..).collect()
    }
}

fn process_syntax_refresh_request(
    registry: &mut SyntaxRegistry,
    parse_sessions: &mut BTreeMap<BufferId, SyntaxParseSession>,
    request: SyntaxRefreshWorkerRequest,
) -> SyntaxRefreshWorkerResult {
    let started = Instant::now();
    let mut parse_session = parse_sessions.remove(&request.buffer_id);
    let (language_id, syntax_result) = compute_buffer_syntax(
        registry,
        request.path.as_deref(),
        &request.text,
        request.buffer_language_id.as_deref(),
        request.syntax_window,
        &mut parse_session,
    );
    if let Some(parse_session) = parse_session {
        parse_sessions.insert(request.buffer_id, parse_session);
    }
    let (highlight_span_count, syntax_result) = match syntax_result {
        Some(Ok(snapshot)) => {
            let highlight_span_count = snapshot.highlight_spans.len();
            (
                highlight_span_count,
                Some(Ok(index_syntax_lines_with_rainbow_parens(
                    snapshot,
                    &request.text,
                    request.rainbow_parens_enabled,
                ))),
            )
        }
        Some(Err(error)) => (0, Some(Err(error.to_string()))),
        None => (0, None),
    };
    SyntaxRefreshWorkerResult {
        buffer_id: request.buffer_id,
        buffer_revision: request.buffer_revision,
        path: request.path,
        buffer_language_id: request.buffer_language_id,
        language_id,
        syntax_window: request.syntax_window,
        compute_elapsed: started.elapsed(),
        highlight_span_count,
        syntax_result,
    }
}
