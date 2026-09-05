use super::*;
use editor_git::{git_probe_snapshot, invalidate_git_probe_cache};
use std::{
    fs,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

fn temp_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("volt-shell-dock-probe-{name}-{unique}"))
}

fn run_git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?} failed");
}

fn init_repo(name: &str, branch: Option<&str>) -> PathBuf {
    let root = temp_root(name);
    fs::create_dir_all(&root).expect("temp dir");
    run_git(&root, &["init", "-q"]);
    run_git(&root, &["config", "user.email", "volt@example.com"]);
    run_git(&root, &["config", "user.name", "volt"]);
    fs::write(root.join("file.txt"), "ok\n").expect("write");
    run_git(&root, &["add", "file.txt"]);
    run_git(&root, &["commit", "-qm", "init"]);
    if let Some(branch) = branch {
        run_git(&root, &["checkout", "-qb", branch]);
    }
    root
}

#[test]
fn workspace_dock_branch_for_two_roots_shares_probe_cache() {
    invalidate_git_probe_cache();
    let first = init_repo("dock-a", Some("alpha"));
    let second = init_repo("dock-b", Some("beta"));
    let cache = WorkspaceDockBranchCache::new();

    assert_eq!(cache.branch_for_root(&first).as_deref(), Some("alpha"));
    assert_eq!(cache.branch_for_root(&second).as_deref(), Some("beta"));
    let first_revision = git_probe_snapshot(&first).revision();
    assert_eq!(
        cache.branch_for_root(&first).as_deref(),
        Some("alpha"),
        "second Dock read must reuse the same snapshot"
    );
    assert_eq!(git_probe_snapshot(&first).revision(), first_revision);

    let _ = fs::remove_dir_all(first);
    let _ = fs::remove_dir_all(second);
}

#[test]
fn workspace_dock_hides_detached_head_label() {
    invalidate_git_probe_cache();
    let root = init_repo("dock-detach", None);
    run_git(&root, &["checkout", "--detach", "-q"]);
    let cache = WorkspaceDockBranchCache::new();
    assert!(cache.branch_for_root(&root).is_none());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn workspace_dock_non_git_root_has_no_branch() {
    invalidate_git_probe_cache();
    let root = temp_root("dock-non-git");
    fs::create_dir_all(&root).expect("temp dir");
    let cache = WorkspaceDockBranchCache::new();
    assert!(cache.branch_for_root(&root).is_none());
    let _ = fs::remove_dir_all(root);
}
