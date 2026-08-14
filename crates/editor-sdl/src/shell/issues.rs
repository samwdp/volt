use super::*;
use editor_issues::{
    CaptureReport, CodeReference, Issue, IssueId, IssueStatus, JumpDecision, ReferenceMarker,
    RewriteIntent, ScanReport, board_issues, capture_file, confirm_rewrite_applied,
    confirm_rewrite_skipped, create_issue, issue_path, jump_decision, linked_issue_id_on_line,
    list_issues, load_issue, place_code_reference, scan_files, set_status, should_apply_rewrite,
    utc_timestamp_now,
};
use editor_plugin_api::issues_hooks;

const ISSUES_BOARD_KIND: &str = buffer_kinds::ISSUES_BOARD;
const ISSUES_BOARD_BUFFER_NAME: &str = "*issues-board*";
const ISSUES_NOTIFICATION_KEY: &str = "issues.outcome";

#[derive(Debug, Clone)]
enum IssuesWorkerRequest {
    Capture {
        workspace_root: PathBuf,
        relative_path: String,
        absolute_path: PathBuf,
        text: String,
        buffer_id: Option<BufferId>,
        buffer_revision: Option<u64>,
    },
    Scan {
        workspace_root: PathBuf,
    },
}

#[derive(Debug, Clone)]
enum IssuesWorkerResult {
    Capture {
        workspace_root: PathBuf,
        relative_path: String,
        absolute_path: PathBuf,
        buffer_id: Option<BufferId>,
        buffer_revision: Option<u64>,
        report: Result<CaptureReport, String>,
    },
    Scan {
        workspace_root: PathBuf,
        report: Result<ScanReport, String>,
    },
}

pub(super) struct IssuesWorkerState {
    request_tx: Sender<IssuesWorkerRequest>,
    results: Arc<Mutex<Vec<IssuesWorkerResult>>>,
    board_show_closed: bool,
    board_issue_ids: Vec<IssueId>,
    selected_issue_id: Option<IssueId>,
}

impl IssuesWorkerState {
    pub(super) fn new() -> Self {
        let (request_tx, request_rx) = mpsc::channel::<IssuesWorkerRequest>();
        let results = Arc::new(Mutex::new(Vec::new()));
        let worker_results = Arc::clone(&results);
        std::thread::spawn(move || {
            while let Ok(request) = request_rx.recv() {
                let result = match request {
                    IssuesWorkerRequest::Capture {
                        workspace_root,
                        relative_path,
                        absolute_path,
                        text,
                        buffer_id,
                        buffer_revision,
                    } => {
                        let report = capture_file(
                            &workspace_root,
                            &relative_path,
                            &text,
                            &utc_timestamp_now(),
                        )
                        .map_err(|error| error.to_string());
                        IssuesWorkerResult::Capture {
                            workspace_root,
                            relative_path,
                            absolute_path,
                            buffer_id,
                            buffer_revision,
                            report,
                        }
                    }
                    IssuesWorkerRequest::Scan { workspace_root } => {
                        let report = collect_scan_files(&workspace_root).and_then(|files| {
                            scan_files(&workspace_root, &files, &utc_timestamp_now())
                                .map_err(|error| error.to_string())
                        });
                        IssuesWorkerResult::Scan {
                            workspace_root,
                            report,
                        }
                    }
                };
                if let Ok(mut guard) = worker_results.lock() {
                    guard.push(result);
                } else {
                    return;
                }
            }
        });
        Self {
            request_tx,
            results,
            board_show_closed: false,
            board_issue_ids: Vec::new(),
            selected_issue_id: None,
        }
    }

    fn enqueue(&mut self, request: IssuesWorkerRequest) -> Result<(), String> {
        self.request_tx
            .send(request)
            .map_err(|error| format!("issues worker unavailable: {error}"))
    }

    fn take_results(&self) -> Vec<IssuesWorkerResult> {
        self.results
            .lock()
            .map(|mut guard| std::mem::take(&mut *guard))
            .unwrap_or_default()
    }
}

pub(super) fn register_issues_hooks(runtime: &mut EditorRuntime) -> Result<(), String> {
    for (name, description) in [
        (
            issues_hooks::BOARD_OPEN,
            "Opens the Issue Board for the active workspace.",
        ),
        (
            issues_hooks::CREATE,
            "Prompts for an Issue title and Creates an Issue.",
        ),
        (
            issues_hooks::SCAN,
            "Runs Issue Scan over the workspace tree asynchronously.",
        ),
        (
            issues_hooks::CAPTURE_FOCUSED,
            "Captures unlinked TODO/FIXME comments in the focused file.",
        ),
        (
            issues_hooks::ACTIVATE_LINE,
            "Opens the Issue under the Issue Board cursor.",
        ),
        (
            issues_hooks::SET_STATUS,
            "Sets Status for the selected Issue (detail = Status label).",
        ),
        (
            issues_hooks::PLACE,
            "Places a Code Reference for the selected Issue at the cursor.",
        ),
        (
            issues_hooks::OPEN_FROM_REF,
            "Opens the Issue for the Code Reference under the cursor.",
        ),
        (
            issues_hooks::JUMP_REFS,
            "Jumps to Code References for the selected Issue.",
        ),
        (
            issues_hooks::TOGGLE_CLOSED,
            "Toggles Closed Issues on the Issue Board.",
        ),
    ] {
        register_hook(runtime, name, description)?;
    }
    Ok(())
}

pub(super) fn subscribe_issues_hooks(runtime: &mut EditorRuntime) -> Result<(), String> {
    runtime
        .subscribe_hook(
            issues_hooks::BOARD_OPEN,
            "shell.issues-board-open",
            |_, runtime| open_issues_board(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(issues_hooks::CREATE, "shell.issues-create", |_, runtime| {
            begin_issues_create(runtime)
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(issues_hooks::SCAN, "shell.issues-scan", |_, runtime| {
            enqueue_issues_scan(runtime)
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            issues_hooks::CAPTURE_FOCUSED,
            "shell.issues-capture-focused",
            |_, runtime| enqueue_capture_focused(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            issues_hooks::ACTIVATE_LINE,
            "shell.issues-activate-line",
            |_, runtime| activate_issues_board_line(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            issues_hooks::SET_STATUS,
            "shell.issues-set-status",
            |event, runtime| {
                let detail = event
                    .detail
                    .as_deref()
                    .ok_or_else(|| "ui.issues.set-status missing Status detail".to_owned())?;
                set_selected_issue_status(runtime, detail)
            },
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(issues_hooks::PLACE, "shell.issues-place", |_, runtime| {
            place_selected_issue(runtime)
        })
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            issues_hooks::OPEN_FROM_REF,
            "shell.issues-open-from-ref",
            |_, runtime| open_issue_from_code_reference(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            issues_hooks::JUMP_REFS,
            "shell.issues-jump-refs",
            |_, runtime| jump_selected_issue_refs(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            issues_hooks::TOGGLE_CLOSED,
            "shell.issues-board-toggle-closed",
            |_, runtime| toggle_issues_board_closed(runtime),
        )
        .map_err(|error| error.to_string())?;
    runtime
        .subscribe_hook(
            builtins::AFTER_SAVE,
            "shell.issues-capture-after-save",
            |event, runtime| {
                let buffer_id = event
                    .buffer_id
                    .ok_or_else(|| "buffer.after-save missing buffer".to_owned())?;
                enqueue_capture_after_save(runtime, buffer_id)
            },
        )
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn issues_workspace_root(runtime: &EditorRuntime) -> Result<PathBuf, String> {
    active_workspace_root(runtime)?
        .ok_or_else(|| "Issues require an active workspace root".to_owned())
}

fn begin_issues_create(runtime: &mut EditorRuntime) -> Result<(), String> {
    let _ = issues_workspace_root(runtime)?;
    shell_ui_mut(runtime)?.set_command_line(CommandLineOverlay::for_issues_create());
    Ok(())
}

pub(super) fn submit_issues_create(runtime: &mut EditorRuntime, title: &str) -> Result<(), String> {
    let title = title.trim();
    if title.is_empty() {
        return Ok(());
    }
    let root = issues_workspace_root(runtime)?;
    let issue =
        create_issue(&root, title, &utc_timestamp_now()).map_err(|error| error.to_string())?;
    open_issue_file(runtime, &root, &issue)?;
    shell_ui_mut(runtime)?.issues_worker.selected_issue_id = Some(issue.id());
    let _ = refresh_issues_board_if_open(runtime);
    notify_issues(
        runtime,
        NotificationSeverity::Success,
        "Issue Created",
        vec![format!("{} — {}", issue.id().display(), issue.title())],
    )?;
    Ok(())
}

fn open_issues_board(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = issues_workspace_root(runtime)?;
    let buffer_id = open_or_focus_workspace_plugin_buffer(
        runtime,
        ISSUES_BOARD_BUFFER_NAME,
        ISSUES_BOARD_KIND,
    )?;
    render_issues_board(runtime, &root, buffer_id)?;
    Ok(())
}

fn toggle_issues_board_closed(runtime: &mut EditorRuntime) -> Result<(), String> {
    {
        let ui = shell_ui_mut(runtime)?;
        ui.issues_worker.board_show_closed = !ui.issues_worker.board_show_closed;
    }
    refresh_issues_board_if_open(runtime)
}

fn refresh_issues_board_if_open(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = match active_workspace_root(runtime)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let Some(buffer_id) = find_open_issues_board(runtime)? else {
        return Ok(());
    };
    render_issues_board(runtime, &root, buffer_id)
}

fn find_open_issues_board(runtime: &EditorRuntime) -> Result<Option<BufferId>, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    find_workspace_named_buffer(
        runtime,
        workspace_id,
        ISSUES_BOARD_BUFFER_NAME,
        &BufferKind::Plugin(ISSUES_BOARD_KIND.to_owned()),
    )
}

fn render_issues_board(
    runtime: &mut EditorRuntime,
    root: &Path,
    buffer_id: BufferId,
) -> Result<(), String> {
    let show_closed = shell_ui(runtime)?.issues_worker.board_show_closed;
    let issues = list_issues(root).map_err(|error| error.to_string())?;
    let rows = board_issues(&issues, show_closed);
    let mut lines = Vec::new();
    let mut ids = Vec::new();
    lines.push(format!(
        "Issue Board  (Closed {})",
        if show_closed { "shown" } else { "hidden" }
    ));
    lines.push(String::new());
    if rows.is_empty() {
        lines.push("No active Issues. Use issues.create or Capture a TODO.".to_owned());
    } else {
        for issue in rows {
            lines.push(format!(
                "{}  [{:12}]  {}",
                issue.id().display(),
                issue.status().label(),
                issue.title()
            ));
            ids.push(issue.id());
        }
    }
    shell_ui_mut(runtime)?.issues_worker.board_issue_ids = ids;
    shell_buffer_mut(runtime, buffer_id)?.replace_with_lines_preserve_view(lines);
    Ok(())
}

fn activate_issues_board_line(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
    if !matches!(kind, BufferKind::Plugin(ref plugin) if plugin == ISSUES_BOARD_KIND) {
        return Ok(());
    }
    let row = shell_buffer(runtime, buffer_id)?.cursor_row();
    let issue_id = board_issue_id_at_row(runtime, row)?;
    let root = issues_workspace_root(runtime)?;
    let issue = load_issue(&root, issue_id).map_err(|error| error.to_string())?;
    shell_ui_mut(runtime)?.issues_worker.selected_issue_id = Some(issue_id);
    open_issue_file(runtime, &root, &issue)
}

fn board_issue_id_at_row(runtime: &EditorRuntime, row: usize) -> Result<IssueId, String> {
    // Header + blank line occupy rows 0 and 1.
    let index = row.saturating_sub(2);
    shell_ui(runtime)?
        .issues_worker
        .board_issue_ids
        .get(index)
        .copied()
        .ok_or_else(|| "no Issue on the current Issue Board row".to_owned())
}

fn resolve_selected_issue_id(runtime: &mut EditorRuntime) -> Result<IssueId, String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let kind = shell_buffer(runtime, buffer_id)?.kind.clone();
    if matches!(kind, BufferKind::Plugin(ref plugin) if plugin == ISSUES_BOARD_KIND) {
        let row = shell_buffer(runtime, buffer_id)?.cursor_row();
        let id = board_issue_id_at_row(runtime, row)?;
        shell_ui_mut(runtime)?.issues_worker.selected_issue_id = Some(id);
        return Ok(id);
    }
    if let Some(path) = shell_buffer(runtime, buffer_id)?
        .path()
        .map(Path::to_path_buf)
        && let Some(id) = issue_id_from_store_path(runtime, &path)?
    {
        shell_ui_mut(runtime)?.issues_worker.selected_issue_id = Some(id);
        return Ok(id);
    }
    shell_ui(runtime)?
        .issues_worker
        .selected_issue_id
        .ok_or_else(|| "no Issue selected — open the Issue Board or an Issue file first".to_owned())
}

fn issue_id_from_store_path(
    runtime: &EditorRuntime,
    path: &Path,
) -> Result<Option<IssueId>, String> {
    let Some(root) = active_workspace_root(runtime)? else {
        return Ok(None);
    };
    let store = editor_issues::store_dir(&root);
    let Ok(relative) = path.strip_prefix(&store) else {
        return Ok(None);
    };
    let name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let stem = name.strip_suffix(".md").unwrap_or(name);
    let rest = stem
        .strip_prefix("ISS-")
        .or_else(|| stem.strip_prefix("iss-"));
    let Some(rest) = rest else {
        return Ok(None);
    };
    let number = rest.split('-').next().unwrap_or(rest);
    Ok(IssueId::parse(&format!("ISS-{number}")))
}

fn open_issue_file(runtime: &mut EditorRuntime, root: &Path, issue: &Issue) -> Result<(), String> {
    let path = issue_path(root, issue);
    open_workspace_file(runtime, &path)?;
    sync_active_buffer(runtime)?;
    Ok(())
}

fn set_selected_issue_status(runtime: &mut EditorRuntime, detail: &str) -> Result<(), String> {
    let status =
        IssueStatus::parse(detail).ok_or_else(|| format!("unknown Issue Status `{detail}`"))?;
    let id = resolve_selected_issue_id(runtime)?;
    let root = issues_workspace_root(runtime)?;
    let issue =
        set_status(&root, id, status, &utc_timestamp_now()).map_err(|error| error.to_string())?;
    // Reload open Issue buffer if present.
    if let Ok(Some(buffer_id)) = find_workspace_file_buffer(
        runtime,
        runtime
            .model()
            .active_workspace_id()
            .map_err(|error| error.to_string())?,
        &issue_path(&root, &issue),
    ) {
        let text = TextBuffer::load_from_path(issue_path(&root, &issue))
            .map_err(|error| error.to_string())?;
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.replace_with_lines_preserve_view(
            text.text().lines().map(str::to_owned).collect::<Vec<_>>(),
        );
        buffer.text.mark_clean();
    }
    let _ = refresh_issues_board_if_open(runtime);
    notify_issues(
        runtime,
        NotificationSeverity::Info,
        "Issue Status updated",
        vec![format!(
            "{} → {}",
            issue.id().display(),
            issue.status().label()
        )],
    )?;
    Ok(())
}

fn place_selected_issue(runtime: &mut EditorRuntime) -> Result<(), String> {
    let issue_id = resolve_selected_issue_id(runtime)?;
    let root = issues_workspace_root(runtime)?;
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (path, cursor) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let path = buffer
            .path()
            .map(Path::to_path_buf)
            .ok_or_else(|| "Place requires a focused file buffer".to_owned())?;
        (path, buffer.cursor_point())
    };
    if path.starts_with(editor_issues::store_dir(&root)) {
        return Err("Place into a code buffer, not an Issue file".to_owned());
    }
    let relative = workspace_relative_path(Some(&root), &path);
    let line_1based = cursor.line.saturating_add(1);
    let placed = place_code_reference(
        &root,
        issue_id,
        &relative,
        line_1based,
        ReferenceMarker::Todo,
    )
    .map_err(|error| error.to_string())?;
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(cursor.line, 0));
        buffer.insert_text(&format!("{}\n", placed.inserted_line));
    }
    shell_ui_mut(runtime)?.issues_worker.selected_issue_id = Some(issue_id);
    let _ = refresh_issues_board_if_open(runtime);
    notify_issues(
        runtime,
        NotificationSeverity::Success,
        "Code Reference Placed",
        vec![format!(
            "{} at {relative}:{}",
            placed.issue.id().display(),
            line_1based
        )],
    )?;
    Ok(())
}

fn open_issue_from_code_reference(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (path, line) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let path = buffer
            .path()
            .map(Path::to_path_buf)
            .ok_or_else(|| "open-from-ref requires a file buffer".to_owned())?;
        let line = line_text_without_newline(buffer, buffer.cursor_point().line)
            .ok_or_else(|| "cursor line missing".to_owned())?;
        (path, line)
    };
    let Some(issue_id) = linked_issue_id_on_line(&line, &path) else {
        return Err(
            "cursor is not on a linked Code Reference (TODO(ISS-NNN): / FIXME(ISS-NNN):)"
                .to_owned(),
        );
    };
    let root = issues_workspace_root(runtime)?;
    match load_issue(&root, issue_id) {
        Ok(issue) => {
            shell_ui_mut(runtime)?.issues_worker.selected_issue_id = Some(issue_id);
            open_issue_file(runtime, &root, &issue)
        }
        Err(_) => {
            notify_issues(
                runtime,
                NotificationSeverity::Warning,
                "Orphan Code Reference",
                vec![format!(
                    "{} has no Issue file — not auto-created",
                    issue_id.display()
                )],
            )?;
            Ok(())
        }
    }
}

fn jump_selected_issue_refs(runtime: &mut EditorRuntime) -> Result<(), String> {
    let issue_id = resolve_selected_issue_id(runtime)?;
    let root = issues_workspace_root(runtime)?;
    let issue = load_issue(&root, issue_id).map_err(|error| error.to_string())?;
    match jump_decision(issue.code_references()) {
        JumpDecision::None => {
            notify_issues(
                runtime,
                NotificationSeverity::Info,
                "No Code References",
                vec![format!(
                    "{} has no Code References — Place first",
                    issue.id().display()
                )],
            )?;
            Ok(())
        }
        JumpDecision::Single(reference) => jump_to_code_reference(runtime, &root, &reference),
        JumpDecision::Many(references) => {
            let entries = references
                .into_iter()
                .enumerate()
                .map(|(index, reference)| {
                    let path = root.join(reference.path());
                    let target = TextPoint::new(reference.line().saturating_sub(1), 0);
                    PickerEntry {
                        item: editor_picker::PickerItem::new(
                            format!("ref-{index}"),
                            format!("{}:{}", reference.path(), reference.line()),
                            reference.snippet().unwrap_or("").to_owned(),
                            None::<String>,
                        ),
                        action: PickerAction::OpenFileLocation { path, target },
                        quickfix: None,
                    }
                })
                .collect();
            shell_ui_mut(runtime)?.set_picker(PickerOverlay::from_entries(
                format!("Code References — {}", issue.id().display()),
                entries,
            ));
            Ok(())
        }
    }
}

fn jump_to_code_reference(
    runtime: &mut EditorRuntime,
    root: &Path,
    reference: &CodeReference,
) -> Result<(), String> {
    let path = root.join(reference.path());
    let target = TextPoint::new(reference.line().saturating_sub(1), 0);
    open_workspace_file_at(runtime, &path, target)?;
    sync_active_buffer(runtime)?;
    Ok(())
}

fn enqueue_capture_focused(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    enqueue_capture_for_buffer(runtime, buffer_id)
}

fn enqueue_capture_after_save(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    enqueue_capture_for_buffer(runtime, buffer_id)
}

fn enqueue_capture_for_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = match active_workspace_root(runtime)? {
        Some(root) => root,
        None => return Ok(()),
    };
    let (absolute_path, text, revision) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !matches!(buffer.kind, BufferKind::File) {
            return Ok(());
        }
        let path = match buffer.path() {
            Some(path) => path.to_path_buf(),
            None => return Ok(()),
        };
        if path.starts_with(editor_issues::store_dir(&root)) {
            return Ok(());
        }
        (path, buffer.text.text(), buffer.text.revision())
    };
    let relative = workspace_relative_path(Some(&root), &absolute_path);
    shell_ui_mut(runtime)?
        .issues_worker
        .enqueue(IssuesWorkerRequest::Capture {
            workspace_root: root,
            relative_path: relative,
            absolute_path,
            text,
            buffer_id: Some(buffer_id),
            buffer_revision: Some(revision),
        })?;
    Ok(())
}

fn enqueue_issues_scan(runtime: &mut EditorRuntime) -> Result<(), String> {
    let root = issues_workspace_root(runtime)?;
    notify_issues(
        runtime,
        NotificationSeverity::Info,
        "Issue Scan started",
        vec!["Scanning workspace tree in background…".to_owned()],
    )?;
    shell_ui_mut(runtime)?
        .issues_worker
        .enqueue(IssuesWorkerRequest::Scan {
            workspace_root: root,
        })?;
    Ok(())
}

fn collect_scan_files(root: &Path) -> Result<Vec<(String, String)>, String> {
    // Prefer a full tree walk so untracked sources are included; git listing is a fallback
    // only when the walk fails.
    let relative_paths = walk_source_files(root)
        .or_else(|_| editor_git::list_repository_files(root).map_err(|error| error.to_string()))?;
    let mut files = Vec::new();
    for relative in relative_paths {
        let relative_str = relative.to_string_lossy().replace('\\', "/");
        if relative_str.starts_with("issues/") || relative_str.contains("/issues/") {
            continue;
        }
        let absolute = root.join(&relative);
        let Ok(text) = fs::read_to_string(&absolute) else {
            continue;
        };
        if text.contains('\0') {
            continue;
        }
        files.push((relative_str, text));
    }
    Ok(files)
}

fn walk_source_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = fs::read_dir(&dir).map_err(|error| error.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == ".git"
                || name == "target"
                || name == "node_modules"
                || name == "issues"
                || name == ".scratch"
            {
                continue;
            }
            let file_type = entry.file_type().map_err(|error| error.to_string())?;
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file()
                && let Ok(relative) = path.strip_prefix(root)
            {
                out.push(relative.to_path_buf());
            }
        }
    }
    Ok(out)
}

pub(super) fn refresh_pending_issues(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let results = shell_ui(runtime)?.issues_worker.take_results();
    if results.is_empty() {
        return Ok(false);
    }
    let mut changed = false;
    for result in results {
        match result {
            IssuesWorkerResult::Capture {
                workspace_root,
                relative_path,
                absolute_path,
                buffer_id,
                buffer_revision,
                report,
                ..
            } => {
                changed = true;
                match report {
                    Ok(report) => {
                        apply_capture_report(
                            runtime,
                            &workspace_root,
                            &relative_path,
                            &absolute_path,
                            buffer_id,
                            buffer_revision,
                            report,
                        )?;
                    }
                    Err(error) => {
                        notify_issues(
                            runtime,
                            NotificationSeverity::Error,
                            "Capture failed",
                            vec![error],
                        )?;
                    }
                }
            }
            IssuesWorkerResult::Scan {
                workspace_root,
                report,
                ..
            } => {
                changed = true;
                match report {
                    Ok(report) => {
                        apply_scan_report(runtime, &workspace_root, report)?;
                    }
                    Err(error) => {
                        notify_issues(
                            runtime,
                            NotificationSeverity::Error,
                            "Issue Scan failed",
                            vec![error],
                        )?;
                    }
                }
            }
        }
    }
    Ok(changed)
}

fn apply_capture_report(
    runtime: &mut EditorRuntime,
    workspace_root: &Path,
    relative_path: &str,
    absolute_path: &Path,
    buffer_id: Option<BufferId>,
    buffer_revision: Option<u64>,
    report: CaptureReport,
) -> Result<(), String> {
    if report.items.is_empty() {
        return Ok(());
    }
    let mut minted = 0usize;
    let mut rewritten = 0usize;
    let mut skipped = 0usize;
    for item in report.items {
        minted = minted.saturating_add(1);
        let applied = apply_rewrite_intent(
            runtime,
            absolute_path,
            buffer_id,
            buffer_revision,
            &item.rewrite,
        )?;
        if applied {
            rewritten = rewritten.saturating_add(1);
            let _ = confirm_rewrite_applied(
                workspace_root,
                item.rewrite.issue_id,
                relative_path,
                item.rewrite.line_index.saturating_add(1),
                &item.rewrite.rewritten_line,
            );
        } else {
            skipped = skipped.saturating_add(1);
            let _ = confirm_rewrite_skipped(
                workspace_root,
                item.rewrite.issue_id,
                relative_path,
                item.rewrite.line_index.saturating_add(1),
            );
        }
    }
    if rewritten > 0
        && let Some(buffer_id) = buffer_id
    {
        let workspace_id = runtime
            .model()
            .active_workspace_id()
            .map_err(|error| error.to_string())?;
        let _ = save_buffer(runtime, workspace_id, buffer_id);
    }
    let _ = refresh_issues_board_if_open(runtime);
    let mut body = vec![format!(
        "{relative_path}: minted {minted}, rewritten {rewritten}, skipped {skipped}"
    )];
    if skipped > 0 {
        body.push("Rewrite skipped — Issue kept; link manually or Place later.".to_owned());
    }
    notify_issues(
        runtime,
        if skipped > 0 {
            NotificationSeverity::Warning
        } else {
            NotificationSeverity::Success
        },
        "Capture complete",
        body,
    )?;
    Ok(())
}

fn apply_rewrite_intent(
    runtime: &mut EditorRuntime,
    absolute_path: &Path,
    buffer_id: Option<BufferId>,
    _buffer_revision: Option<u64>,
    intent: &RewriteIntent,
) -> Result<bool, String> {
    if let Some(buffer_id) = buffer_id
        && let Some(buffer) = shell_ui_mut(runtime)?.buffer_mut(buffer_id)
    {
        let Some(live) = line_text_without_newline(buffer, intent.line_index) else {
            return Ok(false);
        };
        if !should_apply_rewrite(&live, intent) {
            return Ok(false);
        }
        let line_len = buffer.line_len_chars(intent.line_index);
        buffer.replace_range(
            TextRange::new(
                TextPoint::new(intent.line_index, 0),
                TextPoint::new(intent.line_index, line_len),
            ),
            &intent.rewritten_line,
        );
        return Ok(true);
    }

    let Ok(contents) = fs::read_to_string(absolute_path) else {
        return Ok(false);
    };
    let mut lines: Vec<String> = contents.lines().map(str::to_owned).collect();
    let Some(live) = lines.get(intent.line_index) else {
        return Ok(false);
    };
    if !should_apply_rewrite(live, intent) {
        return Ok(false);
    }
    lines[intent.line_index] = intent.rewritten_line.clone();
    let mut out = lines.join("\n");
    if contents.ends_with('\n') {
        out.push('\n');
    }
    fs::write(absolute_path, out).map_err(|error| error.to_string())?;
    Ok(true)
}

fn apply_scan_report(
    runtime: &mut EditorRuntime,
    workspace_root: &Path,
    report: ScanReport,
) -> Result<(), String> {
    let mut rewritten = 0usize;
    let mut skipped = 0usize;
    for item in &report.captured {
        let absolute = workspace_root.join(&item.source_path);
        let applied = apply_rewrite_intent(runtime, &absolute, None, None, &item.rewrite)?;
        if applied {
            rewritten = rewritten.saturating_add(1);
            let _ = confirm_rewrite_applied(
                workspace_root,
                item.rewrite.issue_id,
                &item.source_path,
                item.rewrite.line_index.saturating_add(1),
                &item.rewrite.rewritten_line,
            );
        } else {
            skipped = skipped.saturating_add(1);
            let _ = confirm_rewrite_skipped(
                workspace_root,
                item.rewrite.issue_id,
                &item.source_path,
                item.rewrite.line_index.saturating_add(1),
            );
        }
    }
    let _ = refresh_issues_board_if_open(runtime);
    let mut body = vec![
        format!("Captured {}", report.captured.len()),
        format!("Rewritten {rewritten}, skipped {skipped}"),
        format!("Pruned {}", report.pruned.len()),
        format!("Orphans {}", report.orphans.len()),
    ];
    for orphan in report.orphans.iter().take(5) {
        body.push(format!(
            "orphan {} at {}:{}",
            orphan.issue_id.display(),
            orphan.path,
            orphan.line
        ));
    }
    notify_issues(
        runtime,
        if report.orphans.is_empty() && skipped == 0 {
            NotificationSeverity::Success
        } else {
            NotificationSeverity::Warning
        },
        "Issue Scan complete",
        body,
    )?;
    Ok(())
}

fn notify_issues(
    runtime: &mut EditorRuntime,
    severity: NotificationSeverity,
    title: &str,
    body_lines: Vec<String>,
) -> Result<(), String> {
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key: ISSUES_NOTIFICATION_KEY.to_owned(),
            severity,
            title: title.to_owned(),
            body_lines,
            progress: None,
            active: false,
            action: None,
            workspace_id: None,
        },
        Instant::now(),
    );
    Ok(())
}

/// Used by buffer_interaction match.
pub(super) fn is_issues_board_kind(plugin_kind: &str) -> bool {
    plugin_kind == ISSUES_BOARD_KIND
}
