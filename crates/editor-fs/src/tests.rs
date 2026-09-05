    use std::{
        fs,
        path::{Path, PathBuf},
        sync::{Mutex, MutexGuard},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use super::{
        DirectoryBuffer, DirectoryEntryKind, ProjectKind, ProjectSearchRoot, compact_project_path,
        discover_projects, project_discovery_for_picker, project_discovery_forget_candidate,
        project_discovery_persist_path, project_discovery_request_scan, project_discovery_snapshot,
        reset_project_discovery_cache, set_project_discovery_persist_path_for_test,
        set_project_discovery_ttl_for_test, set_project_discovery_worker_blocked_for_test,
        wait_for_project_discovery,
    };

    fn discovery_test_lock() -> MutexGuard<'static, ()> {
        static LOCK: Mutex<()> = Mutex::new(());
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct DiscoveryPersistGuard {
        dir: PathBuf,
    }

    impl Drop for DiscoveryPersistGuard {
        fn drop(&mut self) {
            reset_project_discovery_cache();
            set_project_discovery_persist_path_for_test(None);
            let _ = fs::remove_dir_all(&self.dir);
        }
    }

    fn begin_discovery_persist() -> DiscoveryPersistGuard {
        let dir = temp_dir("discovery-persist");
        let path = dir.join("projects.json");
        let _ = fs::create_dir_all(&dir);
        set_project_discovery_persist_path_for_test(Some(path));
        reset_project_discovery_cache();
        DiscoveryPersistGuard { dir }
    }

    fn wait_timeout() -> Duration {
        Duration::from_secs(2)
    }

    fn temp_dir(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-editor-fs-{label}-{unique}"))
    }

    #[test]
    fn directory_buffer_reads_and_renames_entries() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("dirbuf");
        fs::create_dir_all(root.join("subdir"))?;
        fs::write(root.join("alpha.txt"), "alpha")?;

        let mut buffer = DirectoryBuffer::read(&root)?;
        assert_eq!(buffer.entries().len(), 2);
        assert_eq!(buffer.entries()[0].kind(), DirectoryEntryKind::Directory);
        assert_eq!(buffer.entries()[1].name(), "alpha.txt");

        buffer.rename_entry("alpha.txt", "beta.txt")?;
        assert!(
            buffer
                .entries()
                .iter()
                .any(|entry| entry.name() == "beta.txt")
        );
        assert!(
            !buffer
                .entries()
                .iter()
                .any(|entry| entry.name() == "alpha.txt")
        );
        let reread = DirectoryBuffer::read(&root)?;
        assert_eq!(buffer.entries(), reread.entries());

        buffer.create_file("gamma.txt")?;
        let reread = DirectoryBuffer::read(&root)?;
        assert_eq!(buffer.entries(), reread.entries());

        fs::write(root.join("sneaky.txt"), "sneaky")?;
        assert!(
            !buffer
                .entries()
                .iter()
                .any(|entry| entry.name() == "sneaky.txt"),
            "create must patch the listing instead of rereading siblings"
        );

        buffer.delete_entry("gamma.txt")?;
        assert!(
            !buffer
                .entries()
                .iter()
                .any(|entry| entry.name() == "gamma.txt")
        );
        assert!(
            !buffer
                .entries()
                .iter()
                .any(|entry| entry.name() == "sneaky.txt"),
            "delete must not reread siblings created on disk after the last patch"
        );

        let before_failed_rename = buffer.entries().to_vec();
        assert!(buffer.rename_entry("missing.txt", "nope.txt").is_err());
        assert_eq!(buffer.entries(), before_failed_rename);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn discover_projects_finds_git_repositories_and_worktrees()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("discover");
        let repo = root.join("repo");
        let worktree = root.join("trees").join("feature");
        fs::create_dir_all(repo.join(".git"))?;
        fs::create_dir_all(&worktree)?;
        fs::write(worktree.join(".git"), "gitdir: ../.git/worktrees/feature\n")?;

        let projects = discover_projects(&[ProjectSearchRoot::new(&root, 3)])?;
        assert_eq!(projects.len(), 2);
        assert!(
            projects
                .iter()
                .any(|project| project.root() == repo && project.kind() == ProjectKind::Git)
        );
        assert!(projects.iter().any(|project| {
            project.root() == worktree && project.kind() == ProjectKind::GitWorktree
        }));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn discover_projects_resolves_worktree_repository_metadata()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("discover-worktree-repo");
        let repo = root.join("repo-store");
        let gitdir = repo.join(".git").join("worktrees").join("feature");
        let worktree = root.join("project").join("feature");
        fs::create_dir_all(&gitdir)?;
        fs::create_dir_all(&worktree)?;
        fs::write(
            worktree.join(".git"),
            "gitdir: ../../repo-store/.git/worktrees/feature\n",
        )?;
        fs::write(gitdir.join("commondir"), "../../\n")?;

        let projects = discover_projects(&[ProjectSearchRoot::new(&root, 3)])?;
        let worktree_project = projects
            .iter()
            .find(|project| project.root() == worktree)
            .expect("worktree should be discovered");
        assert_eq!(worktree_project.repository_name(), "repo-store");
        assert_eq!(worktree_project.repository_root(), repo);
        assert_eq!(
            worktree_project.worktree_parent_name().as_deref(),
            Some("project")
        );
        assert_eq!(
            worktree_project.repository_display_name(),
            compact_project_path(&repo, 2),
        );
        assert_eq!(worktree_project.display_name(), "project [feature]");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[cfg(windows)]
    #[test]
    fn discover_projects_resolves_git_for_windows_worktree_metadata()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("discover-git-for-windows-worktree-repo");
        let repo = root.join("repo-store");
        let gitdir = repo.join(".git").join("worktrees").join("feature");
        let worktree = root.join("project").join("feature");
        fs::create_dir_all(&gitdir)?;
        fs::create_dir_all(&worktree)?;
        let gitdir_reference = git_for_windows_path(&gitdir)?;
        fs::write(
            worktree.join(".git"),
            format!("gitdir: {gitdir_reference}\n"),
        )?;
        fs::write(gitdir.join("commondir"), "../../\n")?;

        let projects = discover_projects(&[ProjectSearchRoot::new(&root, 3)])?;
        let worktree_project = projects
            .iter()
            .find(|project| project.root() == worktree)
            .expect("worktree should be discovered");
        assert_eq!(worktree_project.repository_name(), "repo-store");
        // The worktree `.git` reference is written with a canonicalized (long-form) path, so
        // the resolved repository root can differ from `repo` purely by 8.3 short-name vs
        // long-name (e.g. `RUNNER~1` vs `runneradmin`) on some Windows hosts. Compare the
        // canonicalized forms so the assertion is stable regardless of short-name expansion.
        assert_eq!(
            worktree_project.repository_root().canonicalize()?,
            repo.canonicalize()?,
        );
        assert_eq!(
            worktree_project.worktree_parent_name().as_deref(),
            Some("project")
        );
        assert_eq!(worktree_project.display_name(), "project [feature]");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn discover_projects_max_depth_zero_considers_only_the_root()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("discover-depth-zero");
        fs::create_dir_all(root.join(".git"))?;
        fs::create_dir_all(root.join("nested").join(".git"))?;

        let as_repo = discover_projects(&[ProjectSearchRoot::new(&root, 0)])?;
        assert_eq!(as_repo.len(), 1);
        assert_eq!(as_repo[0].root(), root.as_path());
        assert_eq!(as_repo[0].kind(), ProjectKind::Git);

        fs::remove_dir_all(&root)?;
        let nested_only = temp_dir("discover-depth-zero-nested");
        fs::create_dir_all(nested_only.join("nested").join(".git"))?;
        let skipped = discover_projects(&[ProjectSearchRoot::new(&nested_only, 0)])?;
        assert!(skipped.is_empty());

        fs::remove_dir_all(nested_only)?;
        Ok(())
    }

    #[test]
    fn discover_projects_skips_missing_roots_without_failing()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("discover-missing-root");
        let repo = root.join("repo");
        fs::create_dir_all(repo.join(".git"))?;
        let missing = root.join("missing-search-root");

        let projects = discover_projects(&[
            ProjectSearchRoot::new(&missing, 2),
            ProjectSearchRoot::new(&root, 2),
        ])?;
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].root(), repo.as_path());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    fn git_and_worktree_tree(
        label: &str,
    ) -> Result<(PathBuf, PathBuf, PathBuf), Box<dyn std::error::Error>> {
        let root = temp_dir(label);
        let repo = root.join("repo");
        let worktree = root.join("trees").join("feature");
        fs::create_dir_all(repo.join(".git"))?;
        fs::create_dir_all(&worktree)?;
        fs::write(worktree.join(".git"), "gitdir: ../.git/worktrees/feature\n")?;
        Ok((root, repo, worktree))
    }

    #[test]
    fn project_discovery_snapshot_returns_git_and_worktree_candidates()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, repo, worktree) = git_and_worktree_tree("discover-cache-candidates")?;
        let missing = root.join("missing-search-root");
        let roots = [
            ProjectSearchRoot::new(&missing, 3),
            ProjectSearchRoot::new(&root, 3),
        ];

        let immediate = project_discovery_snapshot(&roots);
        assert!(immediate.in_progress() || !immediate.candidates().is_empty());
        let snapshot = wait_for_project_discovery(immediate.request_id(), wait_timeout())?;
        assert!(!snapshot.in_progress());
        assert_eq!(snapshot.last_walk_id(), 1);
        assert_eq!(snapshot.candidates().len(), 2);
        assert!(
            snapshot
                .candidates()
                .iter()
                .any(|project| project.root() == repo && project.kind() == ProjectKind::Git)
        );
        assert!(snapshot.candidates().iter().any(|project| {
            project.root() == worktree && project.kind() == ProjectKind::GitWorktree
        }));
        assert_eq!(
            snapshot
                .candidates()
                .iter()
                .find(|project| project.root() == worktree)
                .map(super::ProjectCandidate::display_name)
                .as_deref(),
            Some("feature")
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_cache_preserves_worktree_display_name()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let root = temp_dir("discover-cache-worktree-name");
        let repo = root.join("repo-store");
        let gitdir = repo.join(".git").join("worktrees").join("feature");
        let worktree = root.join("project").join("feature");
        fs::create_dir_all(&gitdir)?;
        fs::create_dir_all(&worktree)?;
        fs::write(
            worktree.join(".git"),
            "gitdir: ../../repo-store/.git/worktrees/feature\n",
        )?;
        fs::write(gitdir.join("commondir"), "../../\n")?;

        let roots = [ProjectSearchRoot::new(&root, 3)];
        let scanning = project_discovery_snapshot(&roots);
        let snapshot = wait_for_project_discovery(scanning.request_id(), wait_timeout())?;
        let worktree_project = snapshot
            .candidates()
            .iter()
            .find(|project| project.root() == worktree)
            .expect("worktree should be discovered");
        assert_eq!(worktree_project.display_name(), "project [feature]");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_second_snapshot_with_same_roots_is_cache_hit()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, _, _) = git_and_worktree_tree("discover-cache-hit")?;
        let roots = [ProjectSearchRoot::new(&root, 3)];

        let first = project_discovery_snapshot(&roots);
        let ready = wait_for_project_discovery(first.request_id(), wait_timeout())?;
        assert_eq!(ready.last_walk_id(), 1);

        let second = project_discovery_snapshot(&roots);
        assert!(!second.in_progress());
        assert_eq!(second.last_walk_id(), 1);
        assert_eq!(second.request_id(), ready.request_id());
        assert_eq!(second.candidates().len(), ready.candidates().len());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_fingerprint_change_invalidates_cache()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, _, _) = git_and_worktree_tree("discover-cache-fingerprint")?;
        let first_roots = [ProjectSearchRoot::new(&root, 3)];
        let first = project_discovery_snapshot(&first_roots);
        wait_for_project_discovery(first.request_id(), wait_timeout())?;

        let second_roots = [ProjectSearchRoot::new(&root, 0)];
        set_project_discovery_worker_blocked_for_test(true);
        let invalidated = project_discovery_snapshot(&second_roots);
        assert!(invalidated.candidates().is_empty());
        assert!(invalidated.in_progress());
        set_project_discovery_worker_blocked_for_test(false);
        let ready = wait_for_project_discovery(invalidated.request_id(), wait_timeout())?;
        assert_eq!(ready.last_walk_id(), 2);
        assert_eq!(
            ready.fingerprint(),
            &super::ProjectDiscoveryFingerprint::from_roots(&second_roots)
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_stale_snapshot_returns_candidates_while_rescan_runs()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, _, _) = git_and_worktree_tree("discover-cache-ttl")?;
        let roots = [ProjectSearchRoot::new(&root, 3)];
        let first = project_discovery_snapshot(&roots);
        wait_for_project_discovery(first.request_id(), wait_timeout())?;
        set_project_discovery_ttl_for_test(Duration::ZERO);
        set_project_discovery_worker_blocked_for_test(true);

        let stale = project_discovery_snapshot(&roots);
        assert!(stale.in_progress());
        assert_eq!(stale.last_walk_id(), 1);
        assert!(!stale.candidates().is_empty());

        set_project_discovery_worker_blocked_for_test(false);
        let refreshed = wait_for_project_discovery(stale.request_id(), wait_timeout())?;
        assert_eq!(refreshed.last_walk_id(), 2);
        assert!(!refreshed.candidates().is_empty());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_for_picker_reschedules_when_cache_is_fresh()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, _, _) = git_and_worktree_tree("discover-picker-fresh")?;
        let roots = [ProjectSearchRoot::new(&root, 3)];
        let first = project_discovery_snapshot(&roots);
        wait_for_project_discovery(first.request_id(), wait_timeout())?;
        set_project_discovery_worker_blocked_for_test(true);

        let picker = project_discovery_for_picker(&roots);
        assert!(picker.in_progress());
        assert_eq!(picker.last_walk_id(), 1);
        assert_eq!(picker.candidates().len(), 2);

        set_project_discovery_worker_blocked_for_test(false);
        let refreshed = wait_for_project_discovery(picker.request_id(), wait_timeout())?;
        assert_eq!(refreshed.last_walk_id(), 2);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_superseded_scan_is_dropped() -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, _, _) = git_and_worktree_tree("discover-cache-cancel")?;
        let roots = [ProjectSearchRoot::new(&root, 3)];
        set_project_discovery_worker_blocked_for_test(true);
        let first_id = project_discovery_request_scan(&roots);
        let second_id = project_discovery_request_scan(&roots);
        assert_ne!(first_id, second_id);
        set_project_discovery_worker_blocked_for_test(false);
        let ready = wait_for_project_discovery(second_id, wait_timeout())?;
        assert_eq!(ready.request_id(), second_id);
        assert_eq!(ready.last_walk_id(), 1);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_blocked_snapshot_is_empty_until_scan_completes()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, _, _) = git_and_worktree_tree("discover-cache-blocked")?;
        let roots = [ProjectSearchRoot::new(&root, 3)];
        set_project_discovery_worker_blocked_for_test(true);
        let scanning = project_discovery_snapshot(&roots);
        assert!(scanning.candidates().is_empty());
        assert!(scanning.in_progress());
        set_project_discovery_worker_blocked_for_test(false);
        let ready = wait_for_project_discovery(scanning.request_id(), wait_timeout())?;
        assert_eq!(ready.last_walk_id(), 1);
        assert_eq!(ready.candidates().len(), 2);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_persists_and_reseeds_cache_from_disk()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, repo, _) = git_and_worktree_tree("discover-persist-reseed")?;
        let roots = [ProjectSearchRoot::new(&root, 3)];
        let scanning = project_discovery_snapshot(&roots);
        let ready = wait_for_project_discovery(scanning.request_id(), wait_timeout())?;
        assert_eq!(ready.candidates().len(), 2);
        assert!(project_discovery_persist_path().is_file());

        reset_project_discovery_cache();
        set_project_discovery_worker_blocked_for_test(true);
        let seeded = project_discovery_snapshot(&roots);
        assert!(seeded.in_progress());
        assert_eq!(seeded.candidates().len(), 2);
        assert!(
            seeded
                .candidates()
                .iter()
                .any(|project| project.root() == repo)
        );

        set_project_discovery_worker_blocked_for_test(false);
        wait_for_project_discovery(seeded.request_id(), wait_timeout())?;
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn project_discovery_forget_candidate_updates_memory_and_disk()
    -> Result<(), Box<dyn std::error::Error>> {
        let _lock = discovery_test_lock();
        let _persist = begin_discovery_persist();
        let (root, _, worktree) = git_and_worktree_tree("discover-forget")?;
        let roots = [ProjectSearchRoot::new(&root, 3)];
        let scanning = project_discovery_snapshot(&roots);
        wait_for_project_discovery(scanning.request_id(), wait_timeout())?;

        set_project_discovery_worker_blocked_for_test(true);
        project_discovery_forget_candidate(&worktree);
        let after_forget = super::current_project_discovery_snapshot();
        assert!(
            after_forget
                .candidates()
                .iter()
                .all(|project| project.root() != worktree)
        );
        assert!(!after_forget.in_progress());
        let body = fs::read_to_string(project_discovery_persist_path())?;
        let parsed: serde_json::Value = serde_json::from_str(&body)?;
        let projects = parsed["projects"].as_array().expect("projects array");
        assert!(projects.iter().all(|project| {
            project["root"]
                .as_str()
                .is_none_or(|root_path| Path::new(root_path) != worktree.as_path())
        }));

        set_project_discovery_worker_blocked_for_test(false);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[cfg(windows)]
    fn git_for_windows_path(path: &std::path::Path) -> Result<String, Box<dyn std::error::Error>> {
        let path = path.canonicalize()?;
        let rendered = path
            .display()
            .to_string()
            .replace('\\', "/")
            .trim_start_matches("//?/")
            .to_owned();
        let mut chars = rendered.chars();
        let drive = chars.next().ok_or("missing drive letter")?;
        if !drive.is_ascii_alphabetic() || chars.next() != Some(':') || chars.next() != Some('/') {
            return Err(format!("unexpected Windows path format: {rendered}").into());
        }
        Ok(format!(
            "/{}/{}",
            drive.to_ascii_lowercase(),
            chars.as_str()
        ))
    }
