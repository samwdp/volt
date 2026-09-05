
use super::*;
use std::{
    process::Command,
    sync::{Mutex, MutexGuard},
    time::{SystemTime, UNIX_EPOCH},
};

fn probe_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn temp_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("volt-editor-git-probe-{name}-{unique}"))
}

fn run_git_ok(root: &Path, args: &[&str]) -> Result<(), String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn git_stdout(root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn init_committed_repo(name: &str) -> Result<PathBuf, String> {
    let root = temp_root(name);
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;
    run_git_ok(&root, &["init", "-q"])?;
    run_git_ok(&root, &["config", "user.email", "volt@example.com"])?;
    run_git_ok(&root, &["config", "user.name", "volt"])?;
    fs::write(root.join("main.txt"), "hello\n").map_err(|error| error.to_string())?;
    run_git_ok(&root, &["add", "main.txt"])?;
    run_git_ok(&root, &["commit", "-qm", "init"])?;
    Ok(root)
}

#[test]
fn parse_git_numstat_sums_rows_and_ignores_binary_placeholders() {
    assert_eq!(parse_git_numstat("1\t2\tfoo.rs\n3\t4\tbar.rs\n"), (4, 6));
    assert_eq!(parse_git_numstat("-\t-\tblob.bin\n"), (0, 0));
    assert_eq!(parse_git_numstat(""), (0, 0));
}

#[test]
fn git_probe_snapshot_matches_rev_parse_and_reuses_identity() -> Result<(), String> {
    let _lock = probe_test_lock();
    if !git_available() {
        return Ok(());
    }
    invalidate_git_probe_cache();
    let root = init_committed_repo("identity-hit")?;
    let expected_branch = git_stdout(&root, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let expected_head = git_stdout(&root, &["rev-parse", "--verify", "HEAD"])?;

    let generation_before = git_probe_generation();
    let first = git_probe_snapshot(&root);
    let generation_after_first = git_probe_generation();
    let second = git_probe_snapshot(&root);
    let generation_after_second = git_probe_generation();

    assert!(first.present());
    assert_eq!(first.branch(), Some(expected_branch.as_str()));
    assert_eq!(first.head(), Some(expected_head.as_str()));
    assert_eq!(second.revision(), first.revision());
    assert_eq!(
        generation_after_first, generation_before,
        "identity from HEAD/ref files must not spawn git"
    );
    assert_eq!(generation_after_second, generation_after_first);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn git_probe_numstat_spawns_once_until_head_or_index_changes() -> Result<(), String> {
    let _lock = probe_test_lock();
    if !git_available() {
        return Ok(());
    }
    invalidate_git_probe_cache();
    let root = init_committed_repo("numstat-hit")?;

    let generation_before = git_probe_generation();
    let first = git_probe_snapshot_with_numstat(&root);
    let generation_after_first = git_probe_generation();
    let second = git_probe_snapshot_with_numstat(&root);
    let generation_after_second = git_probe_generation();

    assert!(first.present());
    assert_eq!(second.revision(), first.revision());
    assert!(
        generation_after_first > generation_before,
        "first numstat should spawn git diff"
    );
    assert_eq!(
        generation_after_second, generation_after_first,
        "unchanged HEAD/index must reuse numstat"
    );

    fs::write(root.join("extra.txt"), "extra\n").map_err(|error| error.to_string())?;
    run_git_ok(&root, &["add", "extra.txt"])?;
    let after_index = git_probe_snapshot_with_numstat(&root);
    assert!(git_probe_generation() > generation_after_second);
    assert_ne!(after_index.revision(), first.revision());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn git_probe_snapshot_worktree_gitdir_file_is_present() -> Result<(), String> {
    let _lock = probe_test_lock();
    if !git_available() {
        return Ok(());
    }
    invalidate_git_probe_cache();
    let root = init_committed_repo("worktree-src")?;
    let worktree = temp_root("worktree-link");
    run_git_ok(
        &root,
        &[
            "worktree",
            "add",
            "-qb",
            "feature",
            &worktree.to_string_lossy(),
        ],
    )?;

    let snapshot = git_probe_snapshot(&worktree);
    assert!(
        snapshot.present(),
        "worktree .git file must count as repo present"
    );
    assert_eq!(snapshot.branch(), Some("feature"));
    assert_eq!(snapshot.dock_branch(), Some("feature"));

    let _ = fs::remove_dir_all(worktree);
    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn git_probe_snapshot_non_git_root_is_absent_without_spawn() -> Result<(), String> {
    let _lock = probe_test_lock();
    invalidate_git_probe_cache();
    let root = temp_root("non-git");
    fs::create_dir_all(&root).map_err(|error| error.to_string())?;

    let generation_before = git_probe_generation();
    let first = git_probe_snapshot(&root);
    let second = git_probe_snapshot(&root);

    assert!(!first.present());
    assert!(first.branch().is_none());
    assert!(first.head().is_none());
    assert_eq!(second.revision(), first.revision());
    assert_eq!(git_probe_generation(), generation_before);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn git_probe_snapshot_shares_cache_across_canonical_roots() -> Result<(), String> {
    let _lock = probe_test_lock();
    if !git_available() {
        return Ok(());
    }
    invalidate_git_probe_cache();
    let root = init_committed_repo("canonical")?;
    let alias = root.join(".");

    let first = git_probe_snapshot(&root);
    let generation = git_probe_generation();
    let second = git_probe_snapshot(&alias);

    assert_eq!(second.revision(), first.revision());
    assert_eq!(second.branch(), first.branch());
    assert_eq!(git_probe_generation(), generation);

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn git_probe_snapshot_hides_detached_head_from_dock() -> Result<(), String> {
    let _lock = probe_test_lock();
    if !git_available() {
        return Ok(());
    }
    invalidate_git_probe_cache();
    let root = init_committed_repo("detached")?;
    run_git_ok(&root, &["checkout", "--detach", "-q"])?;

    let snapshot = git_probe_snapshot(&root);
    assert_eq!(snapshot.branch(), Some("HEAD"));
    assert!(snapshot.dock_branch().is_none());

    let _ = fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn git_probe_snapshot_two_roots_do_not_share_entries() -> Result<(), String> {
    let _lock = probe_test_lock();
    if !git_available() {
        return Ok(());
    }
    invalidate_git_probe_cache();
    let first_root = init_committed_repo("root-a")?;
    let second_root = init_committed_repo("root-b")?;
    run_git_ok(&second_root, &["checkout", "-qb", "other"])?;

    let first = git_probe_snapshot(&first_root);
    let second = git_probe_snapshot(&second_root);
    assert_ne!(first.branch(), second.branch());
    assert_eq!(git_probe_snapshot(&first_root).revision(), first.revision());

    let _ = fs::remove_dir_all(first_root);
    let _ = fs::remove_dir_all(second_root);
    Ok(())
}
