use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{
        Arc, Condvar, Mutex, MutexGuard, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};

use editor_path::volt_data_dir;
use serde::{Deserialize, Serialize};

const PROJECT_DISCOVERY_BACKGROUND_TICK: Duration = Duration::from_secs(2);
const PROJECTS_FILE_NAME: &str = "projects.json";
const PROJECTS_FILE_VERSION: u32 = 1;

use super::{ProjectCandidate, ProjectKind, ProjectSearchRoot, discover_projects};

/// Freshness window for cached Project Workspace candidates.
pub const PROJECT_DISCOVERY_TTL: Duration = Duration::from_secs(5);

const DEFAULT_WAIT_MESSAGE: &str = "project discovery request timed out";

/// Configured search-root identity used to fingerprint the discovery cache.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiscoveryFingerprint {
    roots: Vec<(PathBuf, usize)>,
}

impl ProjectDiscoveryFingerprint {
    /// Builds a fingerprint from configured search roots (path + max_depth).
    pub fn from_roots(roots: &[ProjectSearchRoot]) -> Self {
        let mut roots = roots
            .iter()
            .map(|root| (root.root().to_path_buf(), root.max_depth()))
            .collect::<Vec<_>>();
        roots.sort();
        Self { roots }
    }

    /// Returns the sorted `(path, max_depth)` pairs.
    pub fn roots(&self) -> &[(PathBuf, usize)] {
        &self.roots
    }

    fn to_search_roots(&self) -> Vec<ProjectSearchRoot> {
        self.roots
            .iter()
            .map(|(path, max_depth)| ProjectSearchRoot::new(path.clone(), *max_depth))
            .collect()
    }
}

/// Cached Project Workspace candidates plus scan bookkeeping.
#[derive(Debug, Clone)]
pub struct ProjectDiscoverySnapshot {
    candidates: Vec<ProjectCandidate>,
    fingerprint: ProjectDiscoveryFingerprint,
    completed_at: Option<Instant>,
    request_id: u64,
    in_progress: bool,
    last_walk_id: u64,
    revision: u64,
}

impl ProjectDiscoverySnapshot {
    /// Returns discovered Project Workspace candidates.
    pub fn candidates(&self) -> &[ProjectCandidate] {
        &self.candidates
    }

    /// Returns the fingerprint this snapshot was produced for.
    pub fn fingerprint(&self) -> &ProjectDiscoveryFingerprint {
        &self.fingerprint
    }

    /// Returns when the last applied walk completed, if any.
    pub fn completed_at(&self) -> Option<Instant> {
        self.completed_at
    }

    /// Returns the latest scan request id (in-flight or last applied).
    pub fn request_id(&self) -> u64 {
        self.request_id
    }

    /// Returns whether a background walk is currently running.
    pub fn in_progress(&self) -> bool {
        self.in_progress
    }

    /// Returns how many walks have been applied to the cache.
    pub fn last_walk_id(&self) -> u64 {
        self.last_walk_id
    }

    /// Returns a monotonically increasing cache generation.
    pub fn revision(&self) -> u64 {
        self.revision
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedProjectsFile {
    version: u32,
    fingerprint: ProjectDiscoveryFingerprint,
    projects: Vec<PersistedProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedProject {
    name: String,
    root: PathBuf,
    kind: PersistedProjectKind,
    repository_name: String,
    repository_root: PathBuf,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PersistedProjectKind {
    Git,
    GitWorktree,
}

impl From<ProjectKind> for PersistedProjectKind {
    fn from(kind: ProjectKind) -> Self {
        match kind {
            ProjectKind::Git => Self::Git,
            ProjectKind::GitWorktree => Self::GitWorktree,
        }
    }
}

impl From<PersistedProjectKind> for ProjectKind {
    fn from(kind: PersistedProjectKind) -> Self {
        match kind {
            PersistedProjectKind::Git => Self::Git,
            PersistedProjectKind::GitWorktree => Self::GitWorktree,
        }
    }
}

impl From<&ProjectCandidate> for PersistedProject {
    fn from(candidate: &ProjectCandidate) -> Self {
        Self {
            name: candidate.name().to_owned(),
            root: candidate.root().to_path_buf(),
            kind: PersistedProjectKind::from(candidate.kind()),
            repository_name: candidate.repository_name().to_owned(),
            repository_root: candidate.repository_root().to_path_buf(),
        }
    }
}

impl From<PersistedProject> for ProjectCandidate {
    fn from(project: PersistedProject) -> Self {
        ProjectCandidate::from_persisted(
            project.name,
            project.root,
            ProjectKind::from(project.kind),
            project.repository_name,
            project.repository_root,
        )
    }
}

struct ScanJob {
    request_id: u64,
    roots: Vec<ProjectSearchRoot>,
}

struct CacheInner {
    candidates: Vec<ProjectCandidate>,
    fingerprint: Option<ProjectDiscoveryFingerprint>,
    completed_at: Option<Instant>,
    latest_request_id: u64,
    last_finished_request_id: u64,
    in_progress: bool,
    last_walk_id: u64,
    revision: u64,
    ttl: Duration,
}

impl CacheInner {
    fn new() -> Self {
        Self {
            candidates: Vec::new(),
            fingerprint: None,
            completed_at: None,
            latest_request_id: 0,
            last_finished_request_id: 0,
            in_progress: false,
            last_walk_id: 0,
            revision: 0,
            ttl: PROJECT_DISCOVERY_TTL,
        }
    }

    fn fingerprint_or_empty(&self) -> ProjectDiscoveryFingerprint {
        self.fingerprint
            .clone()
            .unwrap_or_else(|| ProjectDiscoveryFingerprint::from_roots(&[]))
    }

    fn to_snapshot(&self) -> ProjectDiscoverySnapshot {
        ProjectDiscoverySnapshot {
            candidates: self.candidates.clone(),
            fingerprint: self.fingerprint_or_empty(),
            completed_at: self.completed_at,
            request_id: self.latest_request_id,
            in_progress: self.in_progress,
            last_walk_id: self.last_walk_id,
            revision: self.revision,
        }
    }

    fn is_stale(&self) -> bool {
        match self.completed_at {
            Some(completed_at) => completed_at.elapsed() > self.ttl,
            None => true,
        }
    }
}

struct ProjectDiscoveryHub {
    state: Mutex<CacheInner>,
    condvar: Condvar,
    scan_tx: Sender<ScanJob>,
    worker_blocked: AtomicBool,
    block_mutex: Mutex<()>,
    block_condvar: Condvar,
}

fn lock_poison<'a, T>(
    result: Result<MutexGuard<'a, T>, std::sync::PoisonError<MutexGuard<'a, T>>>,
) -> MutexGuard<'a, T> {
    result.unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn hub() -> &'static ProjectDiscoveryHub {
    static HUB: OnceLock<Arc<ProjectDiscoveryHub>> = OnceLock::new();
    HUB.get_or_init(|| {
        let (scan_tx, scan_rx) = mpsc::channel();
        let hub = Arc::new(ProjectDiscoveryHub {
            state: Mutex::new(CacheInner::new()),
            condvar: Condvar::new(),
            scan_tx,
            worker_blocked: AtomicBool::new(false),
            block_mutex: Mutex::new(()),
            block_condvar: Condvar::new(),
        });
        let worker_hub = Arc::clone(&hub);
        std::thread::spawn(move || worker_loop(worker_hub, scan_rx));
        hub
    })
}

fn lock_state(hub: &ProjectDiscoveryHub) -> MutexGuard<'_, CacheInner> {
    lock_poison(hub.state.lock())
}

fn persist_path_override() -> &'static Mutex<Option<PathBuf>> {
    static OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

/// Absolute path of the on-disk project candidate list (`…/volt/projects.json`).
pub fn project_discovery_persist_path() -> PathBuf {
    lock_poison(persist_path_override().lock())
        .clone()
        .unwrap_or_else(|| volt_data_dir().join(PROJECTS_FILE_NAME))
}

/// Overrides the projects.json path. Intended for tests.
pub fn set_project_discovery_persist_path_for_test(path: Option<PathBuf>) {
    *lock_poison(persist_path_override().lock()) = path;
}

fn load_persisted_projects(path: &Path) -> Option<PersistedProjectsFile> {
    let contents = fs::read_to_string(path).ok()?;
    let parsed: PersistedProjectsFile = serde_json::from_str(&contents).ok()?;
    if parsed.version != PROJECTS_FILE_VERSION {
        return None;
    }
    Some(parsed)
}

fn persist_projects(fingerprint: &ProjectDiscoveryFingerprint, candidates: &[ProjectCandidate]) {
    let path = project_discovery_persist_path();
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let file = PersistedProjectsFile {
        version: PROJECTS_FILE_VERSION,
        fingerprint: fingerprint.clone(),
        projects: candidates.iter().map(PersistedProject::from).collect(),
    };
    let Ok(body) = serde_json::to_vec_pretty(&file) else {
        return;
    };
    let tmp = path.with_extension("json.tmp");
    let write_tmp = (|| -> io::Result<()> {
        let mut handle = fs::File::create(&tmp)?;
        handle.write_all(&body)?;
        handle.sync_all()?;
        Ok(())
    })();
    if write_tmp.is_err() {
        let _ = fs::remove_file(&tmp);
        return;
    }
    if fs::rename(&tmp, &path).is_err() {
        let _ = fs::remove_file(&tmp);
    }
}

fn maybe_seed_from_disk(inner: &mut CacheInner, requested: &ProjectDiscoveryFingerprint) {
    if inner.fingerprint.is_some() || !inner.candidates.is_empty() {
        return;
    }
    let Some(file) = load_persisted_projects(&project_discovery_persist_path()) else {
        return;
    };
    if &file.fingerprint != requested {
        return;
    }
    inner.candidates = file
        .projects
        .into_iter()
        .map(ProjectCandidate::from)
        .collect();
    inner.fingerprint = Some(file.fingerprint);
    // Leave completed_at unset so the first snapshot treats the seed as stale and
    // kicks a background walk while still serving picker rows immediately.
}

fn candidate_roots_equal(left: &Path, right: &Path) -> bool {
    left == right
}

fn wait_if_worker_blocked(hub: &ProjectDiscoveryHub) {
    if !hub.worker_blocked.load(Ordering::SeqCst) {
        return;
    }
    let mut guard = lock_poison(hub.block_mutex.lock());
    while hub.worker_blocked.load(Ordering::SeqCst) {
        guard = lock_poison(hub.block_condvar.wait(guard));
    }
}

fn worker_loop(hub: Arc<ProjectDiscoveryHub>, rx: Receiver<ScanJob>) {
    while let Ok(mut job) = rx.recv() {
        while let Ok(newer) = rx.try_recv() {
            job = newer;
        }
        wait_if_worker_blocked(&hub);
        let result = discover_projects(&job.roots);
        apply_scan_result(&hub, job.request_id, result.ok());
    }
}

fn apply_scan_result(
    hub: &ProjectDiscoveryHub,
    request_id: u64,
    candidates: Option<Vec<ProjectCandidate>>,
) {
    let mut inner = lock_state(hub);
    inner.last_finished_request_id = inner.last_finished_request_id.max(request_id);
    if request_id == inner.latest_request_id {
        if let Some(candidates) = candidates {
            inner.candidates = candidates;
            inner.last_walk_id = inner.last_walk_id.saturating_add(1);
            inner.completed_at = Some(Instant::now());
            if let Some(fingerprint) = inner.fingerprint.clone() {
                persist_projects(&fingerprint, &inner.candidates);
            }
        }
        inner.in_progress = false;
        inner.revision = inner.revision.saturating_add(1);
    }
    hub.condvar.notify_all();
}

fn kick_scan(
    hub: &ProjectDiscoveryHub,
    inner: &mut CacheInner,
    roots: &[ProjectSearchRoot],
    fingerprint: ProjectDiscoveryFingerprint,
) -> u64 {
    inner.latest_request_id = inner.latest_request_id.saturating_add(1);
    inner.in_progress = true;
    inner.fingerprint = Some(fingerprint);
    inner.revision = inner.revision.saturating_add(1);
    let request_id = inner.latest_request_id;
    if hub
        .scan_tx
        .send(ScanJob {
            request_id,
            roots: roots.to_vec(),
        })
        .is_err()
    {
        inner.in_progress = false;
    }
    hub.condvar.notify_all();
    request_id
}

fn snapshot_for_roots(
    hub: &ProjectDiscoveryHub,
    inner: &mut CacheInner,
    roots: &[ProjectSearchRoot],
    fingerprint: ProjectDiscoveryFingerprint,
    rescan_when_idle: bool,
) {
    maybe_seed_from_disk(inner, &fingerprint);
    let mismatch = inner.fingerprint.as_ref() != Some(&fingerprint);
    if mismatch {
        inner.candidates.clear();
        inner.completed_at = None;
        kick_scan(hub, inner, roots, fingerprint);
        return;
    }
    if !inner.in_progress && (rescan_when_idle || inner.is_stale()) {
        kick_scan(hub, inner, roots, fingerprint);
    }
}

/// Returns the cached snapshot immediately, kicking a background walk when needed.
///
/// Never blocks on the full disk walk. Within [`PROJECT_DISCOVERY_TTL`], unchanged
/// roots reuse the last applied candidates (stale-while-revalidate after TTL).
/// Cold starts seed from `projects.json` when the fingerprint matches.
pub fn project_discovery_snapshot(roots: &[ProjectSearchRoot]) -> ProjectDiscoverySnapshot {
    let fingerprint = ProjectDiscoveryFingerprint::from_roots(roots);
    let hub = hub();
    let mut inner = lock_state(hub);
    snapshot_for_roots(hub, &mut inner, roots, fingerprint, false);
    inner.to_snapshot()
}

/// Returns cached candidates for the project picker and schedules a rescan when idle.
///
/// Unlike [`project_discovery_snapshot`], this always kicks a background walk when no
/// scan is already running so opening the picker sees newly created projects.
pub fn project_discovery_for_picker(roots: &[ProjectSearchRoot]) -> ProjectDiscoverySnapshot {
    let fingerprint = ProjectDiscoveryFingerprint::from_roots(roots);
    let hub = hub();
    let mut inner = lock_state(hub);
    snapshot_for_roots(hub, &mut inner, roots, fingerprint, true);
    inner.to_snapshot()
}

fn background_tick_state() -> &'static Mutex<Option<Instant>> {
    static LAST_TICK: OnceLock<Mutex<Option<Instant>>> = OnceLock::new();
    LAST_TICK.get_or_init(|| Mutex::new(None))
}

/// Keeps project discovery fresh while the editor runs. Throttled; safe to call each frame.
pub fn project_discovery_background_tick(roots: &[ProjectSearchRoot]) {
    let now = Instant::now();
    let mut last_tick = lock_poison(background_tick_state().lock());
    if last_tick
        .is_some_and(|previous| now.duration_since(previous) < PROJECT_DISCOVERY_BACKGROUND_TICK)
    {
        return;
    }
    *last_tick = Some(now);
    drop(last_tick);
    let _ = project_discovery_snapshot(roots);
}

/// Returns the current cache without kicking a walk.
pub fn current_project_discovery_snapshot() -> ProjectDiscoverySnapshot {
    lock_state(hub()).to_snapshot()
}

/// Starts a new background walk for `roots` and returns the request id.
pub fn project_discovery_request_scan(roots: &[ProjectSearchRoot]) -> u64 {
    let fingerprint = ProjectDiscoveryFingerprint::from_roots(roots);
    let hub = hub();
    let mut inner = lock_state(hub);
    kick_scan(hub, &mut inner, roots, fingerprint)
}

/// Rescans using the fingerprint currently stored in the cache, if any.
pub fn project_discovery_rescan_cached_roots() {
    let hub = hub();
    let mut inner = lock_state(hub);
    let Some(fingerprint) = inner.fingerprint.clone() else {
        return;
    };
    let roots = fingerprint.to_search_roots();
    kick_scan(hub, &mut inner, &roots, fingerprint);
}

/// Drops a candidate from memory and `projects.json` without starting a walk.
///
/// Callers that mutate disk (Worktree Remove) should forget first for an immediate
/// picker update, then [`project_discovery_rescan_cached_roots`] after the path is gone.
pub fn project_discovery_forget_candidate(root: &Path) {
    let hub = hub();
    let mut inner = lock_state(hub);
    let before = inner.candidates.len();
    inner
        .candidates
        .retain(|candidate| !candidate_roots_equal(candidate.root(), root));
    if inner.candidates.len() == before {
        return;
    }
    inner.revision = inner.revision.saturating_add(1);
    if let Some(fingerprint) = inner.fingerprint.clone() {
        persist_projects(&fingerprint, &inner.candidates);
    }
    hub.condvar.notify_all();
}

/// Drops cached candidates so the next snapshot request rescans.
pub fn invalidate_project_discovery_cache() {
    let hub = hub();
    let mut inner = lock_state(hub);
    inner.candidates.clear();
    inner.fingerprint = None;
    inner.completed_at = None;
    inner.in_progress = false;
    inner.latest_request_id = inner.latest_request_id.saturating_add(1);
    inner.revision = inner.revision.saturating_add(1);
    hub.condvar.notify_all();
}

/// Ignores the in-flight walk by advancing the latest request id.
pub fn cancel_project_discovery_scan() {
    let hub = hub();
    let mut inner = lock_state(hub);
    inner.latest_request_id = inner.latest_request_id.saturating_add(1);
    inner.in_progress = false;
    inner.revision = inner.revision.saturating_add(1);
    hub.condvar.notify_all();
}

/// Clears cache bookkeeping. Intended for tests so cases do not leak.
pub fn reset_project_discovery_cache() {
    set_project_discovery_worker_blocked_for_test(false);
    *lock_poison(background_tick_state().lock()) = None;
    let hub = hub();
    let mut inner = lock_state(hub);
    let latest_request_id = inner.latest_request_id.saturating_add(1);
    *inner = CacheInner::new();
    inner.latest_request_id = latest_request_id;
    inner.last_finished_request_id = latest_request_id;
    hub.condvar.notify_all();
}

/// Blocks the discovery worker before each walk. Intended for tests.
pub fn set_project_discovery_worker_blocked_for_test(blocked: bool) {
    let hub = hub();
    hub.worker_blocked.store(blocked, Ordering::SeqCst);
    if !blocked {
        hub.block_condvar.notify_all();
    }
}

/// Overrides the cache TTL. Intended for tests.
pub fn set_project_discovery_ttl_for_test(ttl: Duration) {
    lock_state(hub()).ttl = ttl;
}

/// Waits until scan `request_id` has finished (applied or dropped). No blind sleep.
pub fn wait_for_project_discovery(
    request_id: u64,
    timeout: Duration,
) -> io::Result<ProjectDiscoverySnapshot> {
    let hub = hub();
    let deadline = Instant::now() + timeout;
    let mut guard = lock_state(hub);
    loop {
        if guard.last_finished_request_id >= request_id {
            return Ok(guard.to_snapshot());
        }
        let now = Instant::now();
        if now >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("{DEFAULT_WAIT_MESSAGE} ({request_id})"),
            ));
        }
        let remaining = deadline.saturating_duration_since(now);
        let (next_guard, wait) = hub
            .condvar
            .wait_timeout(guard, remaining)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard = next_guard;
        if wait.timed_out() && guard.last_finished_request_id < request_id {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("{DEFAULT_WAIT_MESSAGE} ({request_id})"),
            ));
        }
    }
}
