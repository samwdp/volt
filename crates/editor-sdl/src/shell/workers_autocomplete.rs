#[derive(Debug, Clone)]
struct VimSearchMatch {
    point: TextPoint,
    char_index: usize,
    span: usize,
    line_text: String,
}

#[derive(Debug, Clone)]
struct AutocompleteBufferRequest {
    buffer_id: BufferId,
    buffer_revision: u64,
    text: TextSnapshot,
    plugin_kind: Option<String>,
    db_candidates: Vec<DbAutocompleteCandidate>,
    path: Option<PathBuf>,
    root: Option<PathBuf>,
    cursor: TextPoint,
    query: AutocompleteQuery,
    providers: Vec<AutocompleteProviderSpec>,
    lsp_client: Option<Arc<LspClientManager>>,
    edits: Option<Vec<TextEdit>>,
    token_edits_from: Option<u64>,
    token_edits: Option<Vec<TextEdit>>,
}

struct PendingAutocompleteRequest {
    due_at: Instant,
    request: AutocompleteWorkerRequest,
}

#[derive(Debug, Clone)]
struct LspSyncWorkerRequest {
    path: PathBuf,
    revision: u64,
    text: TextSnapshot,
    root: Option<PathBuf>,
    lsp_client: Arc<LspClientManager>,
    preferred_server_id: Option<String>,
    edits: Option<Vec<TextEdit>>,
}

struct PendingLspSyncRequest {
    due_at: Instant,
    request: LspSyncWorkerRequest,
}

#[derive(Debug, Clone)]
struct InlineCompletionWorkerRequest {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    text: TextSnapshot,
    path: PathBuf,
    root: Option<PathBuf>,
    cursor: TextPoint,
    options: LspFormattingOptions,
    lsp_client: Arc<LspClientManager>,
    edits: Option<Vec<TextEdit>>,
}

struct PendingInlineCompletionRequest {
    due_at: Instant,
    request: InlineCompletionWorkerRequest,
}

struct InlineCompletionWorkerResult {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    cursor: TextPoint,
    item: Option<LspInlineCompletionItem>,
    error: Option<String>,
}

struct InlineCompletionWorkerState {
    pending: Option<PendingInlineCompletionRequest>,
    next_request_id: u64,
    request_tx: Sender<InlineCompletionWorkerRequest>,
    results: Arc<Mutex<Vec<InlineCompletionWorkerResult>>>,
}

impl InlineCompletionWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<InlineCompletionWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(newer_request) = request_rx.try_recv() {
                    request = newer_request;
                }
                let result = request
                    .lsp_client
                    .sync_buffer_with_edits(
                        &request.path,
                        request.text.text(),
                        request.buffer_revision,
                        request.root.as_deref(),
                        request.edits.as_deref(),
                    )
                    .and_then(|_| {
                        request.lsp_client.inline_completion(
                            &request.path,
                            request.cursor,
                            request.options,
                        )
                    });
                let (item, error) = match result {
                    Ok(item) => (item, None),
                    Err(error) => (None, Some(error.to_string())),
                };
                if let Ok(mut results) = worker_results.lock() {
                    results.push(InlineCompletionWorkerResult {
                        request_id: request.request_id,
                        buffer_id: request.buffer_id,
                        buffer_revision: request.buffer_revision,
                        cursor: request.cursor,
                        item,
                        error,
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

    fn schedule(&mut self, mut request: InlineCompletionWorkerRequest) {
        let debounce = if cfg!(test) {
            Duration::from_millis(0)
        } else {
            Duration::from_millis(120)
        };
        self.next_request_id = self.next_request_id.saturating_add(1);
        request.request_id = self.next_request_id;
        self.pending = Some(PendingInlineCompletionRequest {
            due_at: Instant::now() + debounce,
            request,
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

    fn take_latest_result(&self) -> Option<InlineCompletionWorkerResult> {
        let mut results = self.results.lock().ok()?;
        results.drain(..).next_back()
    }
}

#[derive(Debug)]
struct LspSyncWorkerResult {
    path: PathBuf,
    revision: u64,
    error: Option<String>,
}

struct LspSyncWorkerState {
    pending: BTreeMap<PathBuf, PendingLspSyncRequest>,
    requested_revisions: BTreeMap<PathBuf, u64>,
    request_tx: Sender<LspSyncWorkerRequest>,
    results: Arc<Mutex<Vec<LspSyncWorkerResult>>>,
}

impl LspSyncWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<LspSyncWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                let mut latest_by_path = BTreeMap::new();
                latest_by_path.insert(request.path.clone(), request);
                while let Ok(newer_request) = request_rx.try_recv() {
                    latest_by_path.insert(newer_request.path.clone(), newer_request);
                }
                for request in latest_by_path.into_values() {
                    let sync_result = match request.preferred_server_id.as_deref() {
                        Some(server_id) => request.lsp_client.start_buffer_server_with_edits(
                            &request.path,
                            request.text.text(),
                            request.revision,
                            request.root.as_deref(),
                            server_id,
                            request.edits.as_deref(),
                        ),
                        None => request.lsp_client.sync_buffer_with_edits(
                            &request.path,
                            request.text.text(),
                            request.revision,
                            request.root.as_deref(),
                            request.edits.as_deref(),
                        ),
                    };
                    let error = sync_result.err().map(|error| error.to_string());
                    if let Ok(mut results) = worker_results.lock() {
                        results.push(LspSyncWorkerResult {
                            path: request.path,
                            revision: request.revision,
                            error,
                        });
                        ping_shell_wakeup();
                    } else {
                        return;
                    }
                }
            }
        });

        Self {
            pending: BTreeMap::new(),
            requested_revisions: BTreeMap::new(),
            request_tx,
            results,
        }
    }

    fn has_request(&self, path: &Path, revision: u64) -> bool {
        self.pending
            .get(path)
            .map(|pending| pending.request.revision >= revision)
            .unwrap_or(false)
            || self
                .requested_revisions
                .get(path)
                .map(|requested_revision| *requested_revision >= revision)
                .unwrap_or(false)
    }

    fn schedule(&mut self, request: LspSyncWorkerRequest, typing_active: bool) {
        if self
            .requested_revisions
            .get(request.path.as_path())
            .map(|requested_revision| *requested_revision >= request.revision)
            .unwrap_or(false)
        {
            return;
        }
        let due_at = if typing_active {
            Instant::now() + LSP_SYNC_TYPING_IDLE_THRESHOLD
        } else {
            Instant::now()
        };
        self.pending.insert(
            request.path.clone(),
            PendingLspSyncRequest { due_at, request },
        );
    }

    fn cancel_path(&mut self, path: &Path) {
        self.pending.remove(path);
        self.requested_revisions.remove(path);
    }

    fn dispatch_due(&mut self, now: Instant) -> Result<(), String> {
        let due_paths = self
            .pending
            .iter()
            .filter(|(_, pending)| pending.due_at <= now)
            .map(|(path, _)| path.clone())
            .collect::<Vec<_>>();
        for path in due_paths {
            let Some(pending) = self.pending.remove(path.as_path()) else {
                continue;
            };
            if self
                .requested_revisions
                .get(path.as_path())
                .map(|requested_revision| *requested_revision >= pending.request.revision)
                .unwrap_or(false)
            {
                continue;
            }
            self.requested_revisions
                .insert(path.clone(), pending.request.revision);
            if self.request_tx.send(pending.request).is_err() {
                self.requested_revisions.remove(path.as_path());
                return Err(format!(
                    "failed to send LSP sync request for `{}`: worker disconnected",
                    path.display()
                ));
            }
        }
        Ok(())
    }

    fn next_due_at(&self) -> Option<Instant> {
        self.pending.values().map(|pending| pending.due_at).min()
    }

    fn take_results(&mut self) -> Vec<LspSyncWorkerResult> {
        let Ok(mut results) = self.results.lock() else {
            return Vec::new();
        };
        let drained = results.drain(..).collect::<Vec<_>>();
        for result in &drained {
            if self.requested_revisions.get(result.path.as_path()).copied() == Some(result.revision)
            {
                self.requested_revisions.remove(result.path.as_path());
            }
        }
        drained
    }
}

struct AutocompleteWorkerRequest {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    text: TextSnapshot,
    plugin_kind: Option<String>,
    db_candidates: Vec<DbAutocompleteCandidate>,
    path: Option<PathBuf>,
    root: Option<PathBuf>,
    cursor: TextPoint,
    query: AutocompleteQuery,
    providers: Vec<AutocompleteProviderSpec>,
    lsp_client: Option<Arc<LspClientManager>>,
    edits: Option<Vec<TextEdit>>,
    token_edits_from: Option<u64>,
    token_edits: Option<Vec<TextEdit>>,
}

struct AutocompleteWorkerResult {
    request_id: u64,
    buffer_id: BufferId,
    buffer_revision: u64,
    query: AutocompleteQuery,
    entries: Vec<AutocompleteEntry>,
}

struct AutocompleteWorkerState {
    pending: Option<PendingAutocompleteRequest>,
    next_request_id: u64,
    request_tx: Sender<AutocompleteWorkerRequest>,
    results: Arc<Mutex<Vec<AutocompleteWorkerResult>>>,
    token_map_key: Arc<Mutex<Option<(u64, u64)>>>,
}

impl AutocompleteWorkerState {
    fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<AutocompleteWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        let token_map_key = Arc::new(Mutex::new(None));
        let worker_token_map_key = Arc::clone(&token_map_key);
        std::thread::spawn(move || {
            let mut token_cache = AutocompleteTokenCache::default();
            while let Ok(mut request) = request_rx.recv() {
                while let Ok(newer_request) = request_rx.try_recv() {
                    request = newer_request;
                }
                let entries = autocomplete_entries(&request, &mut token_cache);
                if let Some(key) = token_cache.key()
                    && let Ok(mut slot) = worker_token_map_key.lock()
                {
                    *slot = Some(key);
                }
                if let Ok(mut results) = worker_results.lock() {
                    results.push(AutocompleteWorkerResult {
                        request_id: request.request_id,
                        buffer_id: request.buffer_id,
                        buffer_revision: request.buffer_revision,
                        query: request.query,
                        entries,
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
            token_map_key,
        }
    }

    fn token_map_key(&self) -> Option<(u64, u64)> {
        self.token_map_key.lock().ok().and_then(|guard| *guard)
    }

    fn clear_pending(&mut self) {
        self.pending = None;
    }

    fn schedule(&mut self, request: AutocompleteBufferRequest) {
        let debounce = if cfg!(test) {
            Duration::from_millis(0)
        } else {
            Duration::from_millis(45)
        };
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.pending = Some(PendingAutocompleteRequest {
            due_at: Instant::now() + debounce,
            request: AutocompleteWorkerRequest {
                request_id: self.next_request_id,
                buffer_id: request.buffer_id,
                buffer_revision: request.buffer_revision,
                text: request.text,
                plugin_kind: request.plugin_kind,
                db_candidates: request.db_candidates,
                path: request.path,
                root: request.root,
                cursor: request.cursor,
                query: request.query,
                providers: request.providers,
                lsp_client: request.lsp_client,
                edits: request.edits,
                token_edits_from: request.token_edits_from,
                token_edits: request.token_edits,
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

    fn take_latest_result(&self) -> Option<AutocompleteWorkerResult> {
        let mut results = self.results.lock().ok()?;
        results.drain(..).next_back()
    }
}

#[derive(Debug)]
struct RankedAutocompleteEntry {
    entry: AutocompleteEntry,
    score: i64,
    provider_index: usize,
}

fn autocomplete_entries(
    request: &AutocompleteWorkerRequest,
    token_cache: &mut AutocompleteTokenCache,
) -> Vec<AutocompleteEntry> {
    let mut ranked = Vec::new();
    let mut satisfied_or_groups = BTreeSet::new();
    for (provider_index, provider) in request.providers.iter().enumerate() {
        if provider
            .or_group
            .as_ref()
            .is_some_and(|group| satisfied_or_groups.contains(group))
        {
            continue;
        }
        let entries = match provider.kind {
            AutocompleteProviderKind::Buffer => {
                token_cache.refresh(
                    request.buffer_id.get(),
                    request.buffer_revision,
                    &request.text,
                    request.token_edits_from,
                    request.token_edits.as_deref(),
                );
                buffer_autocomplete_entries(token_cache.counts(), &request.query, provider)
            }
            AutocompleteProviderKind::Database => db_autocomplete_entries(
                &request.plugin_kind,
                &request.db_candidates,
                &request.query,
                provider,
            ),
            AutocompleteProviderKind::Lsp => {
                lsp_autocomplete_entries(request, &request.query, provider)
            }
            AutocompleteProviderKind::Manual => {
                manual_autocomplete_entries(&request.plugin_kind, &request.query, provider)
            }
        };
        if !entries.is_empty()
            && let Some(group) = provider.or_group.as_ref()
        {
            satisfied_or_groups.insert(group.clone());
        }
        ranked.extend(
            entries
                .into_iter()
                .map(|(entry, score)| RankedAutocompleteEntry {
                    entry,
                    score,
                    provider_index,
                }),
        );
    }
    ranked.sort_by(|left, right| {
        left.provider_index
            .cmp(&right.provider_index)
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| {
                left.entry
                    .replacement
                    .chars()
                    .count()
                    .cmp(&right.entry.replacement.chars().count())
            })
            .then_with(|| left.entry.replacement.cmp(&right.entry.replacement))
    });
    ranked.into_iter().map(|entry| entry.entry).collect()
}

fn buffer_autocomplete_entries(
    counts: &BTreeMap<String, usize>,
    query: &AutocompleteQuery,
    provider: &AutocompleteProviderSpec,
) -> Vec<(AutocompleteEntry, i64)> {
    let prefix_lower = query.prefix.to_ascii_lowercase();
    counts
        .iter()
        .filter_map(|(token, frequency)| {
            let token_lower = token.to_ascii_lowercase();
            if !prefix_lower.is_empty() && !token_lower.starts_with(&prefix_lower) {
                return None;
            }
            if !query.token.is_empty() && token == &query.token {
                return None;
            }
            let score = autocomplete_score(token, *frequency, query);
            Some((
                AutocompleteEntry {
                    provider_id: provider.id.clone(),
                    provider_label: provider.label.clone(),
                    provider_icon: provider.icon.clone(),
                    item_icon: provider.item_icon.clone(),
                    label: token.clone(),
                    replacement: token.clone(),
                    replace_range: None,
                    detail: None,
                    documentation: None,
                },
                score,
            ))
        })
        .collect()
}

fn lsp_kind_icon(kind: Option<editor_lsp::LspCompletionKind>) -> &'static str {
    use editor_icons::symbols::cod::*;
    use editor_lsp::LspCompletionKind;
    match kind {
        Some(LspCompletionKind::Text) => COD_TEXT_SIZE,
        Some(LspCompletionKind::Method)
        | Some(LspCompletionKind::Function)
        | Some(LspCompletionKind::Constructor) => COD_SYMBOL_METHOD,
        Some(LspCompletionKind::Field) => COD_SYMBOL_FIELD,
        Some(LspCompletionKind::Variable) => COD_SYMBOL_VARIABLE,
        Some(LspCompletionKind::Class) => COD_SYMBOL_CLASS,
        Some(LspCompletionKind::Interface) => COD_SYMBOL_INTERFACE,
        Some(LspCompletionKind::Module) => COD_SYMBOL_NAMESPACE,
        Some(LspCompletionKind::Property) => COD_SYMBOL_PROPERTY,
        Some(LspCompletionKind::Unit) => COD_SYMBOL_RULER,
        Some(LspCompletionKind::Value) => COD_SYMBOL_NUMERIC,
        Some(LspCompletionKind::Enum) => COD_SYMBOL_ENUM,
        Some(LspCompletionKind::Keyword) => COD_SYMBOL_KEYWORD,
        Some(LspCompletionKind::Snippet) => COD_SYMBOL_SNIPPET,
        Some(LspCompletionKind::Color) => COD_SYMBOL_COLOR,
        Some(LspCompletionKind::File) => COD_FILE,
        Some(LspCompletionKind::Reference) => COD_REFERENCES,
        Some(LspCompletionKind::Folder) => COD_FOLDER,
        Some(LspCompletionKind::EnumMember) => COD_SYMBOL_ENUM_MEMBER,
        Some(LspCompletionKind::Constant) => COD_SYMBOL_CONSTANT,
        Some(LspCompletionKind::Struct) => COD_SYMBOL_STRUCTURE,
        Some(LspCompletionKind::Event) => COD_SYMBOL_EVENT,
        Some(LspCompletionKind::Operator) => COD_SYMBOL_OPERATOR,
        Some(LspCompletionKind::TypeParameter) => COD_SYMBOL_PARAMETER,
        None => COD_SYMBOL_MISC,
    }
}

fn lsp_autocomplete_entries(
    request: &AutocompleteWorkerRequest,
    query: &AutocompleteQuery,
    provider: &AutocompleteProviderSpec,
) -> Vec<(AutocompleteEntry, i64)> {
    let Some(path) = request.path.as_deref() else {
        return Vec::new();
    };
    let Some(lsp_client) = request.lsp_client.as_ref() else {
        return Vec::new();
    };
    let text = request.text.text();
    let completions = lsp_client
        .sync_buffer_with_edits(
            path,
            text,
            request.buffer_revision,
            request.root.as_deref(),
            request.edits.as_deref(),
        )
        .ok()
        .and_then(|_| lsp_client.completions(path, request.cursor).ok())
        .unwrap_or_default();
    let prefix_lower = query.prefix.to_ascii_lowercase();
    completions
        .into_iter()
        .filter_map(|item| {
            let replacement = item.insert_text().to_owned();
            let label = item.label().to_owned();
            let candidate = if label.is_empty() {
                replacement.clone()
            } else {
                label.clone()
            };
            let candidate_lower = candidate.to_ascii_lowercase();
            if !prefix_lower.is_empty() && !candidate_lower.starts_with(&prefix_lower) {
                return None;
            }
            if !query.token.is_empty() && replacement == query.token {
                return None;
            }
            Some((
                AutocompleteEntry {
                    provider_id: provider.id.clone(),
                    provider_label: provider.label.clone(),
                    provider_icon: provider.icon.clone(),
                    item_icon: lsp_kind_icon(item.kind()).to_owned(),
                    label: candidate.clone(),
                    replacement,
                    replace_range: item.edit_range(),
                    detail: item.detail().map(str::to_owned),
                    documentation: item.documentation().map(str::to_owned),
                },
                autocomplete_score(&candidate, 2, query) + 40,
            ))
        })
        .collect()
}

fn manual_autocomplete_entries(
    plugin_kind: &Option<String>,
    query: &AutocompleteQuery,
    provider: &AutocompleteProviderSpec,
) -> Vec<(AutocompleteEntry, i64)> {
    if provider.buffer_kind.as_ref() != plugin_kind.as_ref() {
        return Vec::new();
    }
    let prefix_lower = query.prefix.to_ascii_lowercase();
    provider
        .items
        .iter()
        .filter_map(|item| {
            let label_lower = item.label.to_ascii_lowercase();
            let replacement_lower = item.replacement.to_ascii_lowercase();
            if !prefix_lower.is_empty()
                && !label_lower.starts_with(&prefix_lower)
                && !replacement_lower.starts_with(&prefix_lower)
            {
                return None;
            }
            if !query.token.is_empty() && item.replacement == query.token {
                return None;
            }
            Some((
                AutocompleteEntry {
                    provider_id: provider.id.clone(),
                    provider_label: provider.label.clone(),
                    provider_icon: provider.icon.clone(),
                    item_icon: provider.item_icon.clone(),
                    label: item.label.clone(),
                    replacement: item.replacement.clone(),
                    replace_range: None,
                    detail: item.detail.clone(),
                    documentation: item.documentation.clone(),
                },
                autocomplete_score(&item.replacement, 1, query) + 80,
            ))
        })
        .collect()
}

fn db_autocomplete_entries(
    plugin_kind: &Option<String>,
    candidates: &[DbAutocompleteCandidate],
    query: &AutocompleteQuery,
    provider: &AutocompleteProviderSpec,
) -> Vec<(AutocompleteEntry, i64)> {
    if provider.buffer_kind.as_ref() != plugin_kind.as_ref() {
        return Vec::new();
    }
    let prefix_lower = query.prefix.to_ascii_lowercase();
    candidates
        .iter()
        .filter_map(|candidate| {
            let label_lower = candidate.label.to_ascii_lowercase();
            let replacement_lower = candidate.replacement.to_ascii_lowercase();
            if !prefix_lower.is_empty()
                && !label_lower.starts_with(&prefix_lower)
                && !replacement_lower.starts_with(&prefix_lower)
            {
                return None;
            }
            if !query.token.is_empty() && candidate.replacement == query.token {
                return None;
            }
            Some((
                AutocompleteEntry {
                    provider_id: provider.id.clone(),
                    provider_label: provider.label.clone(),
                    provider_icon: provider.icon.clone(),
                    item_icon: provider.item_icon.clone(),
                    label: candidate.label.clone(),
                    replacement: candidate.replacement.clone(),
                    replace_range: None,
                    detail: candidate.detail.clone(),
                    documentation: candidate.documentation.clone(),
                },
                autocomplete_score(&candidate.replacement, 1, query) + 100,
            ))
        })
        .collect()
}

fn autocomplete_score(token: &str, frequency: usize, query: &AutocompleteQuery) -> i64 {
    let starts_with_exact_case =
        usize::from(!query.prefix.is_empty() && token.starts_with(&query.prefix));
    (frequency as i64 * 100)
        + (starts_with_exact_case as i64 * 24)
        + (query.prefix.chars().count() as i64 * 8)
        - token.chars().count() as i64
}

fn autocomplete_query(snapshot: &TextSnapshot, allow_empty: bool) -> Option<AutocompleteQuery> {
    let cursor = snapshot.cursor();
    let line = snapshot.line(cursor.line)?;
    let characters = line.chars().collect::<Vec<_>>();
    let cursor_col = cursor.column.min(characters.len());
    let mut start = cursor_col;
    while start > 0 && is_completion_word_char(characters[start - 1]) {
        start -= 1;
    }
    let mut end = cursor_col;
    while end < characters.len() && is_completion_word_char(characters[end]) {
        end += 1;
    }
    let allow_empty = allow_empty || is_member_access_completion_point(&characters, cursor_col);
    if !allow_empty && start == cursor_col && end == cursor_col {
        return None;
    }
    let prefix = characters[start..cursor_col].iter().collect::<String>();
    if !allow_empty && prefix.is_empty() {
        return None;
    }
    let token = characters[start..end].iter().collect::<String>();
    Some(AutocompleteQuery {
        prefix,
        token,
        replace_range: TextRange::new(
            TextPoint::new(cursor.line, start),
            TextPoint::new(cursor.line, end),
        ),
    })
}

fn normalize_completion_replacement(
    snapshot: &TextSnapshot,
    replace_range: TextRange,
    replacement: &str,
) -> String {
    // Servers sometimes include the already-typed trigger in insertText without a
    // textEdit range. Strip it when that trigger sits immediately before the edit.
    if let Some(stripped) = replacement.strip_prefix('.')
        && char_immediately_before(snapshot, replace_range.start()) == Some('.')
    {
        return stripped.to_owned();
    }
    if let Some(stripped) = replacement.strip_prefix("->")
        && chars_immediately_before(snapshot, replace_range.start(), 2).as_deref() == Some("->")
    {
        return stripped.to_owned();
    }
    replacement.to_owned()
}

fn char_immediately_before(snapshot: &TextSnapshot, point: TextPoint) -> Option<char> {
    if point.column == 0 {
        return None;
    }
    let line = snapshot.line(point.line)?;
    line.chars().nth(point.column - 1)
}

fn chars_immediately_before(
    snapshot: &TextSnapshot,
    point: TextPoint,
    count: usize,
) -> Option<String> {
    if point.column < count {
        return None;
    }
    let line = snapshot.line(point.line)?;
    let characters = line.chars().collect::<Vec<_>>();
    if point.column > characters.len() {
        return None;
    }
    Some(
        characters[point.column - count..point.column]
            .iter()
            .collect(),
    )
}

fn is_member_access_completion_point(characters: &[char], cursor_col: usize) -> bool {
    if cursor_col == 0 {
        return false;
    }
    matches!(characters.get(cursor_col.saturating_sub(1)), Some('.'))
        || (cursor_col >= 2
            && matches!(characters.get(cursor_col - 2), Some('-'))
            && matches!(characters.get(cursor_col - 1), Some('>')))
}

fn completion_token_at_cursor(buffer: &ShellBuffer) -> Option<(TextRange, String)> {
    let cursor = buffer.cursor_point();
    let line = buffer.text.line(cursor.line)?;
    let characters = line.chars().collect::<Vec<_>>();
    let cursor_col = cursor.column.min(characters.len());
    let token_col =
        if cursor_col < characters.len() && is_completion_word_char(characters[cursor_col]) {
            cursor_col
        } else if cursor_col > 0 && is_completion_word_char(characters[cursor_col - 1]) {
            cursor_col - 1
        } else {
            return None;
        };
    let mut start = token_col;
    while start > 0 && is_completion_word_char(characters[start - 1]) {
        start -= 1;
    }
    let mut end = token_col + 1;
    while end < characters.len() && is_completion_word_char(characters[end]) {
        end += 1;
    }
    let token = characters[start..end].iter().collect::<String>();
    Some((
        TextRange::new(
            TextPoint::new(cursor.line, start),
            TextPoint::new(cursor.line, end),
        ),
        token,
    ))
}

fn hover_signature_request_point(buffer: &ShellBuffer) -> TextPoint {
    let cursor = buffer.cursor_point();
    let Some((token_range, _)) = completion_token_at_cursor(buffer) else {
        return cursor;
    };
    if cursor < token_range.start() || cursor > token_range.end() {
        return cursor;
    }
    hover_signature_call_point_after_token(buffer, token_range.end()).unwrap_or(cursor)
}

fn hover_signature_call_point_after_token(
    buffer: &ShellBuffer,
    token_end: TextPoint,
) -> Option<TextPoint> {
    let mut point = hover_signature_skip_whitespace(buffer, token_end);
    loop {
        match buffer.text.char_at_point(point)? {
            '(' => {
                let inside_call = buffer.text.point_after(point).unwrap_or(point);
                return Some(hover_signature_skip_whitespace(buffer, inside_call));
            }
            ':' => {
                let next = buffer.text.point_after(point)?;
                if buffer.text.char_at_point(next) != Some(':') {
                    return None;
                }
                point = hover_signature_skip_whitespace(
                    buffer,
                    buffer.text.point_after(next).unwrap_or(next),
                );
            }
            '<' => point = hover_signature_skip_generic_arguments(buffer, point)?,
            _ => return None,
        }
    }
}

fn hover_signature_skip_whitespace(buffer: &ShellBuffer, start: TextPoint) -> TextPoint {
    let mut point = start;
    while buffer
        .text
        .char_at_point(point)
        .is_some_and(char::is_whitespace)
    {
        point = buffer.text.point_after(point).unwrap_or(point);
    }
    point
}

fn hover_signature_skip_generic_arguments(
    buffer: &ShellBuffer,
    start: TextPoint,
) -> Option<TextPoint> {
    let mut point = start;
    let mut depth = 0usize;
    loop {
        match buffer.text.char_at_point(point)? {
            '<' => depth += 1,
            '>' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let after_generics = buffer.text.point_after(point).unwrap_or(point);
                    return Some(hover_signature_skip_whitespace(buffer, after_generics));
                }
            }
            _ => {}
        }
        point = buffer.text.point_after(point)?;
    }
}

fn autocomplete_request_for_buffer(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    buffer: &ShellBuffer,
    root: Option<PathBuf>,
    registry: &AutocompleteRegistry,
    lsp_client: Option<Arc<LspClientManager>>,
    allow_empty_query: bool,
) -> Option<AutocompleteBufferRequest> {
    if registry.providers.is_empty() {
        return None;
    }
    let db_candidates = runtime_db_candidates_for_buffer(runtime, buffer_id, buffer);
    let text = buffer.text.snapshot();
    let query = autocomplete_query(&text, allow_empty_query)?;
    Some(AutocompleteBufferRequest {
        buffer_id,
        buffer_revision: buffer.text.revision(),
        text,
        plugin_kind: match &buffer.kind {
            BufferKind::Plugin(kind) => Some(kind.clone()),
            _ => None,
        },
        db_candidates,
        path: buffer.lsp_path().map(Path::to_path_buf),
        root,
        cursor: buffer.cursor_point(),
        query,
        providers: registry.providers.clone(),
        lsp_client: lsp_client.clone(),
        edits: lsp_client.as_ref().and_then(|client| {
            buffer
                .lsp_path()
                .and_then(|path| lsp_edits_since_last_sync(client, path, &buffer.text))
        }),
        token_edits_from: None,
        token_edits: None,
    })
}

fn attach_token_count_edits(
    request: &mut AutocompleteBufferRequest,
    buffer: &TextBuffer,
    token_map_key: Option<(u64, u64)>,
) {
    match token_map_key {
        Some((cached_id, revision)) if cached_id == request.buffer_id.get() => {
            request.token_edits_from = Some(revision);
            request.token_edits = buffer.edits_since(revision);
        }
        _ => {
            request.token_edits_from = None;
            request.token_edits = None;
        }
    }
}

fn runtime_db_candidates_for_buffer(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    buffer: &ShellBuffer,
) -> Vec<DbAutocompleteCandidate> {
    if !buffer_is_db_query(&buffer.kind) {
        return Vec::new();
    }
    runtime
        .services()
        .get::<DbService>()
        .map(|db| db.autocomplete_candidates_for_buffer(buffer_id.get()))
        .unwrap_or_default()
}
