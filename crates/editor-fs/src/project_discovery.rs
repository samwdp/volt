use std::{
    io,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex, MutexGuard, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};

use super::{ProjectCandidate, ProjectSearchRoot, discover_projects};

/// Freshness window for cached Project Workspace candidates.
pub const PROJECT_DISCOVERY_TTL: Duration = Duration::from_secs(30);

const DEFAULT_WAIT_MESSAGE: &str = "project discovery request timed out";

/// Configured search-root identity used to fingerprint the discovery cache.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Returns the cached snapshot immediately, kicking a background walk when needed.
///
/// Never blocks on the full disk walk. Within [`PROJECT_DISCOVERY_TTL`], unchanged
/// roots reuse the last applied candidates (stale-while-revalidate after TTL).
pub fn project_discovery_snapshot(roots: &[ProjectSearchRoot]) -> ProjectDiscoverySnapshot {
    let fingerprint = ProjectDiscoveryFingerprint::from_roots(roots);
    let hub = hub();
    let mut inner = lock_state(hub);
    let mismatch = inner.fingerprint.as_ref() != Some(&fingerprint);
    if mismatch {
        inner.candidates.clear();
        inner.completed_at = None;
        kick_scan(hub, &mut inner, roots, fingerprint);
        return inner.to_snapshot();
    }
    if !inner.in_progress && inner.is_stale() {
        kick_scan(hub, &mut inner, roots, fingerprint);
    }
    inner.to_snapshot()
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
