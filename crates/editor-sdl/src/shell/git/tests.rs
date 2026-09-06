use super::super::*;
use super::*;
use editor_git::{GitStatusSnapshot, invalidate_git_probe_cache};
use std::fs;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn summary(branch: &str, head: &str, added: usize, removed: usize) -> GitSummarySnapshot {
    GitSummarySnapshot {
        branch: Some(branch.to_owned()),
        head: Some(head.to_owned()),
        added,
        removed,
    }
}

#[test]
fn git_summary_changed_tracks_head_updates() {
    let state = GitSummaryState::new();
    assert!(!state.take_changed());

    let first = Some(summary("main", "abc123", 1, 0));
    state.set_snapshot(first.clone());
    assert!(state.take_changed());
    assert!(!state.take_changed());

    state.set_snapshot(first);
    assert!(!state.take_changed());

    state.set_snapshot(Some(summary("main", "def456", 0, 0)));
    assert!(state.take_changed());
}

#[test]
fn git_summary_refresh_due_until_interval_or_stale() {
    let mut state = GitSummaryState::new();
    let now = Instant::now();
    assert!(state.refresh_due(now));
    state.mark_refreshed(now);
    assert!(!state.refresh_due(now));
    assert!(state.refresh_due(now + GIT_SUMMARY_REFRESH_INTERVAL));
    state.mark_stale();
    assert!(state.refresh_due(now));
}

#[test]
fn parse_git_numstat_sums_changed_lines() {
    assert_eq!(
        editor_git::parse_git_numstat("10\t2\ta.rs\n1\t0\tb.rs\n"),
        (11, 2)
    );
    assert_eq!(editor_git::parse_git_numstat("-\t-\tbin\n"), (0, 0));
}

#[test]
fn git_summary_snapshot_branch_matches_rev_parse_without_three_spawns_on_retry() {
    invalidate_git_probe_cache();
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temp dir");
    git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
    git_read_command_output(
        &root,
        "config user.email",
        &["config", "user.email", "volt@example.com"],
    )
    .expect("user email");
    git_read_command_output(
        &root,
        "config user.name",
        &["config", "user.name", "Volt Tests"],
    )
    .expect("user name");
    fs::write(root.join("main.txt"), "hello\n").expect("write");
    git_read_command_output(&root, "add", &["add", "main.txt"]).expect("add");
    git_read_command_output(&root, "commit", &["commit", "-qm", "init"]).expect("commit");

    let expected_branch = git_read_command_output(
        &root,
        "rev-parse --abbrev-ref HEAD",
        &["rev-parse", "--abbrev-ref", "HEAD"],
    )
    .expect("branch")
    .trim()
    .to_owned();
    let expected_head = git_read_command_output(
        &root,
        "rev-parse --verify HEAD",
        &["rev-parse", "--verify", "HEAD"],
    )
    .expect("head")
    .trim()
    .to_owned();

    let first = build_git_summary_snapshot(&root).expect("first snapshot");
    let second = build_git_summary_snapshot(&root).expect("second snapshot");

    assert_eq!(first.branch.as_deref(), Some(expected_branch.as_str()));
    assert_eq!(first.head.as_deref(), Some(expected_head.as_str()));
    assert_eq!(second.branch, first.branch);
    assert_eq!(second.head, first.head);
    assert_eq!(second.added, first.added);
    assert_eq!(second.removed, first.removed);
    assert!(git_repository_present(&root));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn git_summary_snapshot_non_git_root_is_none() {
    invalidate_git_probe_cache();
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temp dir");
    assert!(build_git_summary_snapshot(&root).is_none());
    assert!(!git_repository_present(&root));
    let _ = fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn git_worktree_list_parser_normalizes_windows_drive_paths() {
    let entries = parse_git_worktree_list(
        "worktree w:/w/ftc-ui-web\nHEAD abc123\nbranch refs/heads/main\n\n",
    )
    .expect("worktree list parses");

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].path, PathBuf::from(r"W:\w\ftc-ui-web"));
}

fn temp_dir() -> std::path::PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("volt-shell-git-{unique}"))
}

#[test]
fn git_push_remote_name_prefers_branch_push_remote_for_slashy_branch_names() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temp dir");
    git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
    git_read_command_output(
        &root,
        "config user.email",
        &["config", "user.email", "volt-tests@example.com"],
    )
    .expect("user email");
    git_read_command_output(
        &root,
        "config user.name",
        &["config", "user.name", "Volt Tests"],
    )
    .expect("user name");
    git_read_command_output(
        &root,
        "checkout branch",
        &["checkout", "-qb", "feature/TASK-1234-abc"],
    )
    .expect("branch");
    git_read_command_output(
        &root,
        "config pushRemote",
        &[
            "config",
            "branch.feature/TASK-1234-abc.pushRemote",
            "origin",
        ],
    )
    .expect("pushRemote");

    let snapshot = GitStatusSnapshot::default()
        .with_upstreams(
            Some("origin/feature/TASK-1234-abc".to_owned()),
            Some("feature/TASK-1234-abc".to_owned()),
        )
        .with_status(editor_git::RepositoryStatus::new(
            Some("feature/TASK-1234-abc".to_owned()),
            0,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));

    assert_eq!(
        git_push_remote_name(&root, &snapshot),
        Some("origin".to_owned())
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn git_push_remote_name_uses_upstream_branch_remote_for_local_tracking_worktree_branch() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temp dir");
    git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
    git_read_command_output(
        &root,
        "config branch feature remote",
        &["config", "branch.feature/TASK-1234-abc.remote", "origin"],
    )
    .expect("feature remote");
    git_read_command_output(
        &root,
        "config branch feature merge",
        &[
            "config",
            "branch.feature/TASK-1234-abc.merge",
            "refs/heads/feature/TASK-1234-abc",
        ],
    )
    .expect("feature merge");
    git_read_command_output(
        &root,
        "config branch abc remote",
        &["config", "branch.abc.remote", "."],
    )
    .expect("abc remote");
    git_read_command_output(
        &root,
        "config branch abc merge",
        &[
            "config",
            "branch.abc.merge",
            "refs/heads/feature/TASK-1234-abc",
        ],
    )
    .expect("abc merge");

    let snapshot = GitStatusSnapshot::default()
        .with_upstreams(Some("feature/TASK-1234-abc".to_owned()), None)
        .with_status(editor_git::RepositoryStatus::new(
            Some("abc".to_owned()),
            0,
            0,
            Vec::new(),
            Vec::new(),
            Vec::new(),
        ));

    assert_eq!(
        git_push_remote_name(&root, &snapshot),
        Some("origin".to_owned())
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn status_output_upstream_reads_short_branch_tracking_ref() {
    assert_eq!(
        status_output_upstream("## master...origin/master [ahead 2, behind 1]\n M file.rs\n"),
        Some("origin/master".to_owned())
    );
    assert_eq!(status_output_upstream("## master\n"), None);
}

#[test]
fn git_read_log_oneline_optional_returns_empty_for_unknown_revision() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temp dir");
    git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");

    let entries =
        git_read_log_oneline_optional(&root, "log --oneline ..@{upstream}", "..@{upstream}");

    assert!(entries.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn git_fringe_snapshot_is_empty_when_buffer_matches_head() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temp dir");
    git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
    git_read_command_output(
        &root,
        "config user email",
        &["config", "user.email", "volt@example.com"],
    )
    .expect("user email");
    git_read_command_output(
        &root,
        "config user name",
        &["config", "user.name", "Volt Tests"],
    )
    .expect("user name");
    let path = root.join("main.rs");
    fs::write(&path, "fn main() {}").expect("write file");
    git_read_command_output(&root, "add", &["add", "main.rs"]).expect("git add");
    git_read_command_output(&root, "commit", &["commit", "-qm", "init"]).expect("git commit");

    let snapshot = build_git_fringe_snapshot_with_cache(
        &root,
        Path::new("main.rs"),
        "fn main() {}",
        1,
        None,
        None,
    );
    assert!(snapshot.lines.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn git_fringe_snapshot_ignores_crlf_only_difference() {
    let root = temp_dir();
    fs::create_dir_all(&root).expect("temp dir");
    git_read_command_output(&root, "init", &["init", "-q"]).expect("git init");
    git_read_command_output(
        &root,
        "config user email",
        &["config", "user.email", "volt@example.com"],
    )
    .expect("user email");
    git_read_command_output(
        &root,
        "config user name",
        &["config", "user.name", "Volt Tests"],
    )
    .expect("user name");
    let path = root.join("main.rs");
    fs::write(&path, "fn main() {\r\n    println!(\"hi\");\r\n}\r\n").expect("write file");
    git_read_command_output(&root, "add", &["add", "main.rs"]).expect("git add");
    git_read_command_output(&root, "commit", &["commit", "-qm", "init"]).expect("git commit");

    let snapshot = build_git_fringe_snapshot_with_cache(
        &root,
        Path::new("main.rs"),
        "fn main() {\n    println!(\"hi\");\n}\n",
        3,
        None,
        None,
    );
    assert!(snapshot.lines.is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn git_fringe_snapshot_from_texts_is_empty_when_identical() {
    let snapshot = git_fringe_snapshot_from_texts("fn main() {}\n", "fn main() {}\n", 1);
    assert!(snapshot.lines.is_empty());
}

#[test]
fn git_fringe_snapshot_from_texts_ignores_crlf_only_difference() {
    let snapshot = git_fringe_snapshot_from_texts(
        "fn main() {\r\n    println!(\"hi\");\r\n}\r\n",
        "fn main() {\n    println!(\"hi\");\n}\n",
        3,
    );
    assert!(snapshot.lines.is_empty());
}

#[test]
fn git_fringe_snapshot_from_texts_marks_modified_middle_line() {
    let snapshot = git_fringe_snapshot_from_texts("a\nb\nc\n", "a\nx\nc\n", 3);
    assert_eq!(snapshot.line_kind(1), Some(GitFringeKind::Modified));
    assert_eq!(snapshot.lines.len(), 1);
}

#[test]
fn git_fringe_snapshot_from_texts_marks_removed_on_adjacent_line() {
    let snapshot = git_fringe_snapshot_from_texts("a\nb\nc\n", "a\nc\n", 2);
    assert_eq!(snapshot.line_kind(1), Some(GitFringeKind::Removed));
    assert_eq!(snapshot.lines.len(), 1);
}

#[test]
fn git_fringe_snapshot_from_texts_marks_inserted_line_added() {
    let snapshot = git_fringe_snapshot_from_texts("a\nc\n", "a\nx\nc\n", 3);
    assert_eq!(snapshot.line_kind(1), Some(GitFringeKind::Added));
    assert_eq!(snapshot.lines.len(), 1);
}

#[test]
fn git_fringe_snapshot_from_texts_marks_all_lines_added_without_head() {
    let snapshot = git_fringe_snapshot_from_texts("", "a\nb\n", 2);
    assert_eq!(snapshot.line_kind(0), Some(GitFringeKind::Added));
    assert_eq!(snapshot.line_kind(1), Some(GitFringeKind::Added));
    assert_eq!(snapshot.lines.len(), 2);
}

#[test]
fn git_head_blob_cache_reuses_text_for_same_head() {
    let cache = GitHeadBlobCache::new();
    let root = Path::new("P:\\repo");
    cache.insert(root, "main.rs", "abc123", "fn main() {}".to_owned());
    cache.insert(root, "lib.rs", "abc123", "pub fn lib() {}".to_owned());
    assert_eq!(
        cache.get(root, "main.rs", "abc123").as_deref(),
        Some("fn main() {}")
    );
    cache.insert(root, "main.rs", "def456", "fn main() { 1 }".to_owned());
    assert!(cache.get(root, "main.rs", "abc123").is_none());
    assert_eq!(
        cache.get(root, "main.rs", "def456").as_deref(),
        Some("fn main() { 1 }")
    );
}

#[test]
fn parse_git_fringe_diff_marks_modified_hunk() {
    let snapshot = parse_git_fringe_diff("@@ -2,1 +2,1 @@\n", 3);
    assert_eq!(snapshot.line_kind(1), Some(GitFringeKind::Modified));
    assert_eq!(snapshot.lines.len(), 1);
}

#[test]
fn myers_diff_ops_match_lcs_for_middle_replace() {
    let old = ["a", "b", "c"];
    let new = ["a", "x", "c"];
    let myers = hunks_from_ops(&myers_diff_ops(&old, &new));
    let lcs = hunks_from_ops(&lcs_diff_ops(&old, &new));
    assert_eq!(myers, lcs);
    assert_eq!(myers, vec![(1, 2, 1)]);
}
