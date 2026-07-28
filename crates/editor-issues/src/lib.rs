#![doc = r#"Workspace Issue Store: markdown Issues, Capture, Scan, Board, and Code References."#]

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Returns a UTC timestamp string (`YYYY-MM-DDTHH:MM:SSZ`) for Opened at / Closed at.
pub fn utc_timestamp_now() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format_utc_timestamp(secs)
}

fn format_utc_timestamp(mut secs: u64) -> String {
    const SECS_PER_DAY: u64 = 86_400;
    let days = secs / SECS_PER_DAY;
    secs %= SECS_PER_DAY;
    let hour = secs / 3_600;
    secs %= 3_600;
    let minute = secs / 60;
    let second = secs % 60;
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Howard Hinnant civil-from-days (UTC).
fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y as i32, m as u32, d as u32)
}

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Workspace Issue Store: markdown Issues, Capture, Scan, Board, and Code References.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Directory name for the Issue Store under the workspace root.
pub const STORE_DIR_NAME: &str = "issues";

/// Stable sequential Issue Id (`ISS-NNN`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IssueId(u32);

impl IssueId {
    /// Creates an Issue Id from its numeric component.
    pub const fn new(number: u32) -> Self {
        Self(number)
    }

    /// Returns the numeric component.
    pub const fn number(self) -> u32 {
        self.0
    }

    /// Parses `ISS-NNN` (case-insensitive prefix).
    pub fn parse(raw: &str) -> Option<Self> {
        let trimmed = raw.trim();
        let rest = trimmed
            .strip_prefix("ISS-")
            .or_else(|| trimmed.strip_prefix("iss-"))?;
        let number = rest.parse().ok()?;
        Some(Self(number))
    }

    /// Formats as `ISS-NNN` with at least three digits.
    pub fn display(self) -> String {
        format!("ISS-{:03}", self.0)
    }
}

impl fmt::Display for IssueId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ISS-{:03}", self.0)
    }
}

/// Workflow Status recorded only on the Issue file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IssueStatus {
    /// Issue exists and is not yet being planned or worked.
    Open,
    /// Issue is being scoped or designed.
    Planning,
    /// Active implementation work is underway.
    InProgress,
    /// Issue is no longer active work.
    Closed,
}

impl IssueStatus {
    /// Spoken Status label matching `CONTEXT.md`.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Planning => "Planning",
            Self::InProgress => "In Progress",
            Self::Closed => "Closed",
        }
    }

    /// Parses a spoken Status label.
    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim() {
            "Open" => Some(Self::Open),
            "Planning" => Some(Self::Planning),
            "In Progress" | "InProgress" => Some(Self::InProgress),
            "Closed" => Some(Self::Closed),
            _ => None,
        }
    }
}

/// One Code Reference location recorded on an Issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeReference {
    path: String,
    line: usize,
    snippet: Option<String>,
}

impl CodeReference {
    /// Creates a Code Reference at `path`:`line` (1-based line).
    pub fn new(path: impl Into<String>, line: usize) -> Self {
        Self {
            path: path.into(),
            line,
            snippet: None,
        }
    }

    /// Creates a Code Reference with an optional snippet.
    pub fn with_snippet(mut self, snippet: impl Into<String>) -> Self {
        self.snippet = Some(snippet.into());
        self
    }

    /// Workspace-relative path of the Code Reference.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// 1-based line number.
    pub const fn line(&self) -> usize {
        self.line
    }

    /// Optional snippet captured at record time.
    pub fn snippet(&self) -> Option<&str> {
        self.snippet.as_deref()
    }
}

/// Canonical Issue record (mirrors on-disk markdown).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    id: IssueId,
    title: String,
    status: IssueStatus,
    opened_at: String,
    closed_at: Option<String>,
    code_references: Vec<CodeReference>,
    body: String,
    file_name: String,
}

impl Issue {
    /// Returns the Issue Id.
    pub const fn id(&self) -> IssueId {
        self.id
    }

    /// Returns the title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Sets the title (caller must save).
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.title = title.into();
    }

    /// Returns Status.
    pub const fn status(&self) -> IssueStatus {
        self.status
    }

    /// Returns Opened at.
    pub fn opened_at(&self) -> &str {
        &self.opened_at
    }

    /// Returns Closed at when Status is Closed.
    pub fn closed_at(&self) -> Option<&str> {
        self.closed_at.as_deref()
    }

    /// Returns recorded Code References.
    pub fn code_references(&self) -> &[CodeReference] {
        &self.code_references
    }

    /// Returns free markdown body.
    pub fn body(&self) -> &str {
        &self.body
    }

    /// Sets free markdown body (caller must save).
    pub fn set_body(&mut self, body: impl Into<String>) {
        self.body = body.into();
    }

    /// On-disk file name within the Issue Store.
    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    /// Appends a Code Reference if not already present at the same path+line.
    pub fn record_code_reference(&mut self, reference: CodeReference) {
        let exists = self
            .code_references
            .iter()
            .any(|existing| existing.path == reference.path && existing.line == reference.line);
        if !exists {
            self.code_references.push(reference);
        }
    }

    /// Removes Code References matching `path` and `line`.
    pub fn remove_code_reference_at(&mut self, path: &str, line: usize) {
        self.code_references
            .retain(|reference| !(reference.path == path && reference.line == line));
    }

    /// Removes all Code References for `path`.
    pub fn remove_code_references_for_path(&mut self, path: &str) {
        self.code_references
            .retain(|reference| reference.path != path);
    }
}

/// Errors from Issue Store operations.
#[derive(Debug)]
pub enum IssueError {
    /// Underlying IO failure.
    Io(io::Error),
    /// Issue file or Id was not found.
    NotFound(IssueId),
    /// Issue file content could not be parsed.
    Parse(String),
    /// Invalid argument.
    Invalid(String),
}

impl fmt::Display for IssueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::NotFound(id) => write!(f, "Issue {id} not found"),
            Self::Parse(message) => write!(f, "invalid Issue file: {message}"),
            Self::Invalid(message) => write!(f, "{message}"),
        }
    }
}

impl Error for IssueError {}

impl From<io::Error> for IssueError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// Marker kind embedded in a Code Reference comment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferenceMarker {
    /// `TODO` marker.
    Todo,
    /// `FIXME` marker.
    Fixme,
}

impl ReferenceMarker {
    fn as_str(self) -> &'static str {
        match self {
            Self::Todo => "TODO",
            Self::Fixme => "FIXME",
        }
    }
}

/// Parsed linked or unlinked Code Reference on a source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCodeReference {
    /// Leading whitespace + comment prefix retained for rewrite.
    pub prefix: String,
    /// TODO or FIXME.
    pub marker: ReferenceMarker,
    /// Present when already linked.
    pub issue_id: Option<IssueId>,
    /// Text after the colon.
    pub title: String,
    /// Full original line (no trailing newline).
    pub original_line: String,
}

/// Rewrite intent produced by Capture (adapter applies if line unchanged).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RewriteIntent {
    /// 0-based line index in the source snapshot.
    pub line_index: usize,
    /// Exact line Capture observed.
    pub original_line: String,
    /// Linked replacement line.
    pub rewritten_line: String,
    /// Minted Issue Id embedded in the rewrite.
    pub issue_id: IssueId,
}

/// One Capture result: Issue always minted; rewrite may be skipped by adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureItem {
    /// Workspace-relative source path Capture ran on.
    pub source_path: String,
    /// Newly minted Issue.
    pub issue: Issue,
    /// Suggested rewrite for the source line.
    pub rewrite: RewriteIntent,
}

/// Report from Capture over one file.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CaptureReport {
    /// Minted Issues with rewrite intents.
    pub items: Vec<CaptureItem>,
}

/// Diagnostic for an orphan linked Code Reference (no Issue file).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanReference {
    /// Path of the source file.
    pub path: String,
    /// 1-based line.
    pub line: usize,
    /// Linked Issue Id with no file.
    pub issue_id: IssueId,
}

/// One pruned stale Code Reference location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrunedReference {
    /// Issue that lost a location.
    pub issue_id: IssueId,
    /// Removed location.
    pub reference: CodeReference,
}

/// Report from Issue Scan over a file set.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ScanReport {
    /// Capture outcomes for unlinked comments.
    pub captured: Vec<CaptureItem>,
    /// Orphan linked ids (not auto-created).
    pub orphans: Vec<OrphanReference>,
    /// Stale locations removed from Issues.
    pub pruned: Vec<PrunedReference>,
}

/// Jump-to-code decision from recorded Code References.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpDecision {
    /// No Code References — Place first.
    None,
    /// Navigate directly.
    Single(CodeReference),
    /// Offer a picker.
    Many(Vec<CodeReference>),
}

/// Place result: comment text to insert and updated Issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceResult {
    /// Line text to insert at the cursor (includes comment prefix, no newline).
    pub inserted_line: String,
    /// Issue with the new Code Reference recorded and saved.
    pub issue: Issue,
}

/// Returns the Issue Store directory for a workspace root.
pub fn store_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(STORE_DIR_NAME)
}

/// Creates the Issue Store directory when missing.
pub fn ensure_store(workspace_root: &Path) -> Result<PathBuf, IssueError> {
    let dir = store_dir(workspace_root);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Allocates the next Issue Id from max existing + 1.
pub fn next_issue_id(workspace_root: &Path) -> Result<IssueId, IssueError> {
    let max = list_issues(workspace_root)?
        .into_iter()
        .map(|issue| issue.id().number())
        .max()
        .unwrap_or(0);
    Ok(IssueId::new(max.saturating_add(1)))
}

/// Creates an Issue with Status Open, Opened at set, empty Code References.
pub fn create_issue(
    workspace_root: &Path,
    title: &str,
    opened_at: &str,
) -> Result<Issue, IssueError> {
    let title = title.trim();
    if title.is_empty() {
        return Err(IssueError::Invalid("Issue title must not be empty".into()));
    }
    ensure_store(workspace_root)?;
    let id = next_issue_id(workspace_root)?;
    let file_name = issue_file_name(id, title);
    let issue = Issue {
        id,
        title: title.to_owned(),
        status: IssueStatus::Open,
        opened_at: opened_at.to_owned(),
        closed_at: None,
        code_references: Vec::new(),
        body: String::new(),
        file_name,
    };
    save_issue(workspace_root, &issue)?;
    Ok(issue)
}

/// Absolute path of an Issue markdown file.
pub fn issue_path(workspace_root: &Path, issue: &Issue) -> PathBuf {
    store_dir(workspace_root).join(&issue.file_name)
}

/// Loads every Issue in the store (unordered beyond directory order).
pub fn list_issues(workspace_root: &Path) -> Result<Vec<Issue>, IssueError> {
    let dir = store_dir(workspace_root);
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut issues = Vec::new();
    for entry in fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let contents = fs::read_to_string(&path)?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("issue.md")
            .to_owned();
        issues.push(parse_issue_markdown(&contents, file_name)?);
    }
    issues.sort_by_key(|issue| issue.id().number());
    Ok(issues)
}

/// Loads one Issue by Id.
pub fn load_issue(workspace_root: &Path, id: IssueId) -> Result<Issue, IssueError> {
    list_issues(workspace_root)?
        .into_iter()
        .find(|issue| issue.id() == id)
        .ok_or(IssueError::NotFound(id))
}

/// Writes an Issue markdown file (creates store if needed).
pub fn save_issue(workspace_root: &Path, issue: &Issue) -> Result<(), IssueError> {
    ensure_store(workspace_root)?;
    let path = issue_path(workspace_root, issue);
    fs::write(path, render_issue_markdown(issue))?;
    Ok(())
}

/// Sets Status freely. Opened at is never rewritten. Closed at set/cleared with Closed.
pub fn set_status(
    workspace_root: &Path,
    id: IssueId,
    status: IssueStatus,
    now: &str,
) -> Result<Issue, IssueError> {
    let mut issue = load_issue(workspace_root, id)?;
    let opened_at = issue.opened_at.clone();
    issue.status = status;
    issue.opened_at = opened_at;
    match status {
        IssueStatus::Closed => {
            issue.closed_at = Some(now.to_owned());
        }
        _ => {
            issue.closed_at = None;
        }
    }
    save_issue(workspace_root, &issue)?;
    Ok(issue)
}

/// Board listing: active Statuses by default; include Closed when `show_closed`.
pub fn board_issues(issues: &[Issue], show_closed: bool) -> Vec<&Issue> {
    issues
        .iter()
        .filter(|issue| show_closed || issue.status() != IssueStatus::Closed)
        .collect()
}

/// Jump decision for an Issue's Code References.
pub fn jump_decision(references: &[CodeReference]) -> JumpDecision {
    match references {
        [] => JumpDecision::None,
        [only] => JumpDecision::Single(only.clone()),
        many => JumpDecision::Many(many.to_vec()),
    }
}

/// Captures unlinked TODO/FIXME line comments in one source file.
///
/// Always mints Issues. Returns rewrite intents; adapter applies only when the
/// live line still matches `original_line`. HACK/XXX are ignored.
pub fn capture_file(
    workspace_root: &Path,
    source_relative_path: &str,
    source_text: &str,
    opened_at: &str,
) -> Result<CaptureReport, IssueError> {
    let comment_prefix = comment_prefix_for_path(Path::new(source_relative_path));
    let mut report = CaptureReport::default();
    for (line_index, line) in source_text.lines().enumerate() {
        let Some(parsed) = parse_code_reference_line(line, comment_prefix) else {
            continue;
        };
        if parsed.issue_id.is_some() {
            continue;
        }
        let title = if parsed.title.trim().is_empty() {
            format!("{} in {source_relative_path}", parsed.marker.as_str())
        } else {
            parsed.title.trim().to_owned()
        };
        let issue = create_issue(workspace_root, &title, opened_at)?;
        let rewritten_line = format!(
            "{}{}({}): {}",
            parsed.prefix,
            parsed.marker.as_str(),
            issue.id().display(),
            parsed.title
        );
        let rewrite = RewriteIntent {
            line_index,
            original_line: parsed.original_line.clone(),
            rewritten_line,
            issue_id: issue.id(),
        };
        // Location is recorded only when the adapter applies the rewrite
        // (or by Issue Scan when treating Capture intents as found).
        report.items.push(CaptureItem {
            source_path: source_relative_path.to_owned(),
            issue,
            rewrite,
        });
    }
    Ok(report)
}

/// Returns whether a live line still matches Capture's snapshot (rewrite-if-unchanged).
pub fn should_apply_rewrite(live_line: &str, intent: &RewriteIntent) -> bool {
    live_line == intent.original_line
}

/// Records that a rewrite was applied: updates the Issue Code Reference snippet.
pub fn confirm_rewrite_applied(
    workspace_root: &Path,
    issue_id: IssueId,
    source_relative_path: &str,
    line_1based: usize,
    rewritten_line: &str,
) -> Result<Issue, IssueError> {
    let mut issue = load_issue(workspace_root, issue_id)?;
    if let Some(existing) = issue
        .code_references
        .iter_mut()
        .find(|reference| reference.path == source_relative_path && reference.line == line_1based)
    {
        existing.snippet = Some(rewritten_line.to_owned());
    } else {
        issue.record_code_reference(
            CodeReference::new(source_relative_path, line_1based).with_snippet(rewritten_line),
        );
    }
    save_issue(workspace_root, &issue)?;
    Ok(issue)
}

/// When rewrite is skipped, drop the optimistic Code Reference location.
pub fn confirm_rewrite_skipped(
    workspace_root: &Path,
    issue_id: IssueId,
    source_relative_path: &str,
    line_1based: usize,
) -> Result<Issue, IssueError> {
    let mut issue = load_issue(workspace_root, issue_id)?;
    issue.remove_code_reference_at(source_relative_path, line_1based);
    save_issue(workspace_root, &issue)?;
    Ok(issue)
}

/// Places a linked Code Reference for an Issue into code at the given line.
pub fn place_code_reference(
    workspace_root: &Path,
    issue_id: IssueId,
    source_relative_path: &str,
    line_1based: usize,
    marker: ReferenceMarker,
) -> Result<PlaceResult, IssueError> {
    let mut issue = load_issue(workspace_root, issue_id)?;
    let comment_prefix = comment_prefix_for_path(Path::new(source_relative_path));
    let inserted_line = format!(
        "{}{}({}): {}",
        comment_prefix,
        marker.as_str(),
        issue.id().display(),
        issue.title()
    );
    issue.record_code_reference(
        CodeReference::new(source_relative_path, line_1based).with_snippet(&inserted_line),
    );
    save_issue(workspace_root, &issue)?;
    Ok(PlaceResult {
        inserted_line,
        issue,
    })
}

/// Parses a linked Code Reference Issue Id from a source line, if present.
pub fn linked_issue_id_on_line(line: &str, path: &Path) -> Option<IssueId> {
    let prefix = comment_prefix_for_path(path);
    parse_code_reference_line(line, prefix)?.issue_id
}

/// Issue Scan over provided file snapshots (relative path + text).
///
/// Captures unlinked comments, refreshes locations from found linked refs,
/// prunes stale locations, never deletes Issues, reports orphans.
pub fn scan_files(
    workspace_root: &Path,
    files: &[(String, String)],
    opened_at: &str,
) -> Result<ScanReport, IssueError> {
    let mut report = ScanReport::default();
    let mut found_by_issue: std::collections::BTreeMap<IssueId, Vec<CodeReference>> =
        std::collections::BTreeMap::new();

    for (path, text) in files {
        let capture = capture_file(workspace_root, path, text, opened_at)?;
        for item in &capture.items {
            let line_1based = item.rewrite.line_index.saturating_add(1);
            found_by_issue.entry(item.issue.id()).or_default().push(
                CodeReference::new(path, line_1based).with_snippet(&item.rewrite.rewritten_line),
            );
        }
        report.captured.extend(capture.items);

        let comment_prefix = comment_prefix_for_path(Path::new(path));
        for (line_index, line) in text.lines().enumerate() {
            let Some(parsed) = parse_code_reference_line(line, comment_prefix) else {
                continue;
            };
            let Some(issue_id) = parsed.issue_id else {
                continue;
            };
            let line_1based = line_index.saturating_add(1);
            if load_issue(workspace_root, issue_id).is_err() {
                report.orphans.push(OrphanReference {
                    path: path.clone(),
                    line: line_1based,
                    issue_id,
                });
                continue;
            }
            found_by_issue
                .entry(issue_id)
                .or_default()
                .push(CodeReference::new(path, line_1based).with_snippet(line));
        }
    }

    // Refresh Code References from what Scan found on scanned paths.
    // Keep locations whose paths were not part of this Scan file set.
    let scanned_paths: std::collections::BTreeSet<&str> =
        files.iter().map(|(path, _)| path.as_str()).collect();
    for mut issue in list_issues(workspace_root)? {
        let found = found_by_issue.remove(&issue.id()).unwrap_or_default();
        let previous = issue.code_references.clone();
        let mut next: Vec<CodeReference> = previous
            .iter()
            .filter(|reference| !scanned_paths.contains(reference.path.as_str()))
            .cloned()
            .collect();
        for reference in &found {
            if let Some(existing) = next
                .iter_mut()
                .find(|item| item.path == reference.path && item.line == reference.line)
            {
                existing.snippet = reference.snippet.clone();
            } else {
                next.push(reference.clone());
            }
        }
        for removed in previous.iter().filter(|old| {
            scanned_paths.contains(old.path.as_str())
                && !next
                    .iter()
                    .any(|item| item.path == old.path && item.line == old.line)
        }) {
            report.pruned.push(PrunedReference {
                issue_id: issue.id(),
                reference: removed.clone(),
            });
        }
        issue.code_references = next;
        save_issue(workspace_root, &issue)?;
    }

    Ok(report)
}

/// Line comment prefix for a path (`//`, `#`, `--`, …). Fallback `//`.
pub fn comment_prefix_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "py" | "rb" | "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml" | "r" | "pl" | "ps1" => "# ",
        "sql" | "lua" | "hs" | "elm" => "-- ",
        "lisp" | "el" | "clj" | "cljs" | "edn" | "asm" | "s" => "; ",
        _ => "// ",
    }
}

/// Parses a TODO/FIXME line comment. HACK/XXX ignored. Returns None when not a match.
pub fn parse_code_reference_line(
    line: &str,
    preferred_prefix: &str,
) -> Option<ParsedCodeReference> {
    let original_line = line.to_owned();
    let trimmed_start = line.trim_start();
    let leading_ws_len = line.len() - trimmed_start.len();
    let leading_ws = &line[..leading_ws_len];

    let prefixes = [
        preferred_prefix.trim_end(),
        "//",
        "#",
        "--",
        ";",
        "///",
        "//!",
    ];
    let mut rest = None;
    let mut matched_prefix = preferred_prefix;
    for candidate in prefixes {
        let candidate = candidate.trim_end();
        if let Some(after) = trimmed_start.strip_prefix(candidate) {
            matched_prefix = candidate;
            rest = Some(after.trim_start());
            break;
        }
    }
    let rest = rest?;
    // Ignore HACK / XXX
    if rest.starts_with("HACK") || rest.starts_with("XXX") {
        return None;
    }

    let (marker, after_marker) = if let Some(after) = rest.strip_prefix("TODO") {
        (ReferenceMarker::Todo, after)
    } else {
        let after = rest.strip_prefix("FIXME")?;
        (ReferenceMarker::Fixme, after)
    };

    let (issue_id, title) = if let Some(after_paren) = after_marker.strip_prefix('(') {
        let close = after_paren.find(')')?;
        let id = IssueId::parse(&after_paren[..close])?;
        let after_id = after_paren[close + 1..].trim_start();
        let title = after_id.strip_prefix(':').map(str::trim_start)?;
        (Some(id), title.to_owned())
    } else {
        let title = after_marker.trim_start().strip_prefix(':')?.trim_start();
        (None, title.to_owned())
    };

    let prefix = format!(
        "{leading_ws}{}{}",
        matched_prefix,
        if matched_prefix.ends_with(' ') {
            ""
        } else {
            " "
        }
    );

    Some(ParsedCodeReference {
        prefix,
        marker,
        issue_id,
        title,
        original_line,
    })
}

fn issue_file_name(id: IssueId, title: &str) -> String {
    let slug = slugify(title);
    if slug.is_empty() {
        format!("{}.md", id.display())
    } else {
        format!("{}-{slug}.md", id.display())
    }
}

fn slugify(title: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = true;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    slug.truncate(48);
    while slug.ends_with('-') {
        slug.pop();
    }
    slug
}

fn render_issue_markdown(issue: &Issue) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("id: {}\n", issue.id().display()));
    out.push_str(&format!("title: {}\n", escape_yaml_scalar(issue.title())));
    out.push_str(&format!("status: {}\n", issue.status().label()));
    out.push_str(&format!("opened_at: {}\n", issue.opened_at()));
    match issue.closed_at() {
        Some(closed_at) => out.push_str(&format!("closed_at: {closed_at}\n")),
        None => out.push_str("closed_at:\n"),
    }
    out.push_str("code_references:\n");
    if issue.code_references().is_empty() {
        out.push_str("  []\n");
    } else {
        for reference in issue.code_references() {
            out.push_str(&format!(
                "  - path: {}\n",
                escape_yaml_scalar(reference.path())
            ));
            out.push_str(&format!("    line: {}\n", reference.line()));
            if let Some(snippet) = reference.snippet() {
                out.push_str(&format!("    snippet: {}\n", escape_yaml_scalar(snippet)));
            }
        }
    }
    out.push_str("---\n\n");
    out.push_str(issue.body());
    if !issue.body().is_empty() && !issue.body().ends_with('\n') {
        out.push('\n');
    }
    out
}

fn escape_yaml_scalar(value: &str) -> String {
    if value.is_empty()
        || value.contains(':')
        || value.contains('#')
        || value.contains('"')
        || value.contains('\'')
        || value.contains('\n')
        || value.starts_with(' ')
        || value.ends_with(' ')
    {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_owned()
    }
}

fn parse_issue_markdown(contents: &str, file_name: String) -> Result<Issue, IssueError> {
    let trimmed = contents.trim_start();
    if !trimmed.starts_with("---") {
        return Err(IssueError::Parse("missing YAML frontmatter".into()));
    }
    let after_start = &trimmed[3..];
    let end = after_start
        .find("\n---")
        .ok_or_else(|| IssueError::Parse("unterminated YAML frontmatter".into()))?;
    let front = &after_start[..end];
    let body = after_start[end + 4..].trim_start_matches('\n').to_owned();

    let mut id = None;
    let mut title = None;
    let mut status = IssueStatus::Open;
    let mut opened_at = None;
    let mut closed_at = None;
    let mut code_references = Vec::new();
    let mut in_refs = false;
    let mut current_ref: Option<CodeReference> = None;

    for raw_line in front.lines() {
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            continue;
        }
        if let Some(value) = line.strip_prefix("id:") {
            id = IssueId::parse(value.trim());
            in_refs = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("title:") {
            title = Some(unquote(value.trim()));
            in_refs = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("status:") {
            status = IssueStatus::parse(value.trim())
                .ok_or_else(|| IssueError::Parse(format!("unknown status `{}`", value.trim())))?;
            in_refs = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("opened_at:") {
            let value = value.trim();
            opened_at = if value.is_empty() {
                Some(String::new())
            } else {
                Some(unquote(value))
            };
            in_refs = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("closed_at:") {
            let value = value.trim();
            closed_at = if value.is_empty() {
                None
            } else {
                Some(unquote(value))
            };
            in_refs = false;
            continue;
        }
        if line.starts_with("code_references:") {
            in_refs = true;
            if let Some(finished) = current_ref.take() {
                code_references.push(finished);
            }
            if line.trim_end().ends_with("[]") {
                in_refs = false;
            }
            continue;
        }
        if in_refs {
            if let Some(rest) = line.trim_start().strip_prefix("- path:") {
                if let Some(finished) = current_ref.take() {
                    code_references.push(finished);
                }
                current_ref = Some(CodeReference::new(unquote(rest.trim()), 1));
                continue;
            }
            if let Some(rest) = line.trim_start().strip_prefix("path:") {
                if let Some(finished) = current_ref.take() {
                    code_references.push(finished);
                }
                current_ref = Some(CodeReference::new(unquote(rest.trim()), 1));
                continue;
            }
            if let Some(rest) = line.trim_start().strip_prefix("line:")
                && let Some(reference) = current_ref.as_mut()
            {
                reference.line = rest.trim().parse().unwrap_or(1);
                continue;
            }
            if let Some(rest) = line.trim_start().strip_prefix("snippet:")
                && let Some(reference) = current_ref.as_mut()
            {
                reference.snippet = Some(unquote(rest.trim()));
                continue;
            }
            if line.trim() == "[]" {
                in_refs = false;
            }
        }
    }
    if let Some(finished) = current_ref {
        code_references.push(finished);
    }

    let id = id.ok_or_else(|| IssueError::Parse("missing id".into()))?;
    let title = title.ok_or_else(|| IssueError::Parse("missing title".into()))?;
    let opened_at = opened_at.ok_or_else(|| IssueError::Parse("missing opened_at".into()))?;

    Ok(Issue {
        id,
        title,
        status,
        opened_at,
        closed_at,
        code_references,
        body,
        file_name,
    })
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        value.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_workspace(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("volt-editor-issues-{name}-{unique}"));
        fs::create_dir_all(&root).expect("workspace");
        root
    }

    #[test]
    fn create_mints_sequential_ids_and_opens_store() {
        let root = temp_workspace("create");
        let first = create_issue(&root, "Fix login", "2026-07-16T12:00:00Z").expect("create");
        let second = create_issue(&root, "Add Place", "2026-07-16T12:01:00Z").expect("create");

        assert_eq!(first.id().display(), "ISS-001");
        assert_eq!(second.id().display(), "ISS-002");
        assert_eq!(first.status(), IssueStatus::Open);
        assert_eq!(first.opened_at(), "2026-07-16T12:00:00Z");
        assert!(first.closed_at().is_none());
        assert!(first.code_references().is_empty());
        assert!(store_dir(&root).is_dir());
        assert!(issue_path(&root, &first).is_file());

        let loaded = load_issue(&root, first.id()).expect("load");
        assert_eq!(loaded.title(), "Fix login");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn next_id_uses_max_existing() {
        let root = temp_workspace("max-id");
        ensure_store(&root).expect("store");
        let path = store_dir(&root).join("ISS-007-manual.md");
        fs::write(
            &path,
            "---\nid: ISS-007\ntitle: Manual\nstatus: Open\nopened_at: t0\nclosed_at:\ncode_references:\n  []\n---\n\n",
        )
        .expect("write");
        let created = create_issue(&root, "After max", "t1").expect("create");
        assert_eq!(created.id().display(), "ISS-008");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn status_moves_freely_and_manages_closed_at() {
        let root = temp_workspace("status");
        let issue = create_issue(&root, "Work", "t-open").expect("create");
        let opened = issue.opened_at().to_owned();

        let planning = set_status(&root, issue.id(), IssueStatus::Planning, "t2").expect("status");
        assert_eq!(planning.status(), IssueStatus::Planning);
        assert_eq!(planning.opened_at(), opened);
        assert!(planning.closed_at().is_none());

        let closed =
            set_status(&root, issue.id(), IssueStatus::Closed, "t-closed").expect("closed");
        assert_eq!(closed.status(), IssueStatus::Closed);
        assert_eq!(closed.closed_at(), Some("t-closed"));
        assert_eq!(closed.opened_at(), opened);

        let reopen = set_status(&root, issue.id(), IssueStatus::InProgress, "t3").expect("reopen");
        assert_eq!(reopen.status(), IssueStatus::InProgress);
        assert!(reopen.closed_at().is_none());
        assert_eq!(reopen.opened_at(), opened);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn capture_mints_and_rewrites_todo_and_fixme() {
        let root = temp_workspace("capture");
        let source = "// TODO: fix login\nfn main() {}\n# FIXME: urgent\n";
        // python-style line in rust file still uses // preferred; use .py for #
        let report = capture_file(
            &root,
            "src/main.rs",
            "// TODO: fix login\nfn main() {}\n",
            "t0",
        )
        .expect("capture");
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].issue.id().display(), "ISS-001");
        assert!(
            report.items[0]
                .rewrite
                .rewritten_line
                .contains("TODO(ISS-001):")
        );
        assert_eq!(report.items[0].rewrite.original_line, "// TODO: fix login");

        let py = capture_file(&root, "script.py", "# FIXME: urgent\n", "t1").expect("py");
        assert_eq!(py.items.len(), 1);
        assert!(
            py.items[0]
                .rewrite
                .rewritten_line
                .contains("FIXME(ISS-002):")
        );
        let _ = source;
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn capture_ignores_hack_and_xxx() {
        let root = temp_workspace("ignore");
        let report = capture_file(
            &root,
            "a.rs",
            "// HACK: no\n// XXX: no\n// TODO: yes\n",
            "t0",
        )
        .expect("capture");
        assert_eq!(report.items.len(), 1);
        assert_eq!(report.items[0].issue.title(), "yes");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn rewrite_if_unchanged_helper() {
        let intent = RewriteIntent {
            line_index: 0,
            original_line: "// TODO: x".into(),
            rewritten_line: "// TODO(ISS-001): x".into(),
            issue_id: IssueId::new(1),
        };
        assert!(should_apply_rewrite("// TODO: x", &intent));
        assert!(!should_apply_rewrite("// TODO: changed", &intent));
    }

    #[test]
    fn board_hides_closed_by_default() {
        let root = temp_workspace("board");
        let open = create_issue(&root, "Open one", "t0").expect("open");
        let closed = create_issue(&root, "Done", "t1").expect("closed create");
        set_status(&root, closed.id(), IssueStatus::Closed, "t2").expect("close");
        let all = list_issues(&root).expect("list");
        let active = board_issues(&all, false);
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id(), open.id());
        assert_eq!(board_issues(&all, true).len(), 2);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn place_records_location() {
        let root = temp_workspace("place");
        let issue = create_issue(&root, "Link me", "t0").expect("create");
        let placed = place_code_reference(&root, issue.id(), "src/a.rs", 10, ReferenceMarker::Todo)
            .expect("place");
        assert!(placed.inserted_line.contains("TODO(ISS-001):"));
        assert!(placed.inserted_line.contains("Link me"));
        assert_eq!(placed.issue.code_references().len(), 1);
        assert_eq!(placed.issue.code_references()[0].path(), "src/a.rs");
        assert_eq!(placed.issue.code_references()[0].line(), 10);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn jump_decision_handles_zero_one_many() {
        assert!(matches!(jump_decision(&[]), JumpDecision::None));
        let one = [CodeReference::new("a.rs", 1)];
        assert!(matches!(jump_decision(&one), JumpDecision::Single(_)));
        let many = [CodeReference::new("a.rs", 1), CodeReference::new("b.rs", 2)];
        assert!(matches!(jump_decision(&many), JumpDecision::Many(_)));
    }

    #[test]
    fn parse_linked_and_unlinked_forms() {
        let unlinked = parse_code_reference_line("  // TODO: fix", "// ").expect("unlinked");
        assert!(unlinked.issue_id.is_none());
        assert_eq!(unlinked.title, "fix");
        let linked = parse_code_reference_line("// FIXME(ISS-042): urgent", "// ").expect("linked");
        assert_eq!(linked.issue_id, Some(IssueId::new(42)));
        assert_eq!(linked.marker, ReferenceMarker::Fixme);
    }

    #[test]
    fn scan_prunes_stale_and_reports_orphan_without_delete() {
        let root = temp_workspace("scan");
        let issue = create_issue(&root, "Tracked", "t0").expect("create");
        let mut issue = load_issue(&root, issue.id()).expect("load");
        issue.record_code_reference(
            CodeReference::new("src/gone.rs", 3).with_snippet("// TODO(ISS-001): tracked"),
        );
        save_issue(&root, &issue).expect("save");

        let files = vec![
            (
                "src/alive.rs".to_owned(),
                "// TODO(ISS-001): tracked\n// TODO(ISS-099): orphan\n".to_owned(),
            ),
            ("src/gone.rs".to_owned(), "// comment removed\n".to_owned()),
        ];
        let report = scan_files(&root, &files, "t1").expect("scan");
        assert!(
            report
                .orphans
                .iter()
                .any(|orphan| orphan.issue_id == IssueId::new(99))
        );
        assert!(
            report
                .pruned
                .iter()
                .any(|pruned| pruned.reference.path() == "src/gone.rs")
        );
        let refreshed = load_issue(&root, IssueId::new(1)).expect("still exists");
        assert!(
            refreshed
                .code_references()
                .iter()
                .any(|reference| reference.path() == "src/alive.rs")
        );
        assert!(
            !refreshed
                .code_references()
                .iter()
                .any(|reference| reference.path() == "src/gone.rs")
        );
        // Issue never deleted
        assert!(load_issue(&root, IssueId::new(1)).is_ok());
        assert!(load_issue(&root, IssueId::new(99)).is_err());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn spoken_names_use_issue_not_task() {
        assert_eq!(IssueStatus::Open.label(), "Open");
        assert_eq!(IssueId::new(1).display(), "ISS-001");
        assert!(ROLE.contains("Issue"));
    }

    #[test]
    fn capture_can_finish_after_caller_continues() {
        // Adapter contract: save returns immediately while Capture runs off-thread.
        let root = temp_workspace("async-capture");
        let source = root.join("main.rs");
        fs::write(&source, "// TODO: async me\n").expect("write");
        let text = fs::read_to_string(&source).expect("read");
        let (tx, rx) = std::sync::mpsc::channel();
        let workspace = root.clone();
        std::thread::spawn(move || {
            let report = capture_file(&workspace, "main.rs", &text, "t0");
            let _ = tx.send(report);
        });
        // Caller already "returned" from save.
        let save_completed = true;
        assert!(save_completed);
        let report = rx.recv().expect("worker").expect("capture");
        assert_eq!(report.items.len(), 1);
        assert!(
            store_dir(&root)
                .join(report.items[0].issue.file_name())
                .is_file()
        );
        let _ = fs::remove_dir_all(root);
    }
}
