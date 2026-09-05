use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use super::{
    REPOSITORY_FILE_PREVIEW_MAX_LINES, invalidate_repository_file_list_cache,
    invalidate_repository_file_list_cache_for, list_repository_files, parse_log_oneline,
    parse_stash_list, parse_status, repository_file_list_generation, repository_file_preview,
};
use std::sync::{Mutex, MutexGuard};

fn repository_file_list_test_lock() -> MutexGuard<'static, ()> {
    static LOCK: Mutex<()> = Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn configure_git_identity(root: &Path) -> Result<(), Box<dyn std::error::Error>> {
    run_git(root, &["config", "user.email", "volt@example.com"])?;
    run_git(root, &["config", "user.name", "volt"])?;
    Ok(())
}

fn temp_repo_root(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("volt-editor-git-{name}-{unique}"))
}

fn git_available() -> bool {
    Command::new("git").arg("--version").output().is_ok()
}

fn run_git(root: &Path, args: &[&str]) -> Result<(), Box<dyn std::error::Error>> {
    let status = Command::new("git").args(args).current_dir(root).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("git {:?} failed with status {status}", args).into())
    }
}

fn git_stdout(root: &Path, args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = Command::new("git").args(args).current_dir(root).output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(format!("git {:?} failed with status {}", args, output.status).into())
    }
}

#[test]
fn parser_extracts_branch_and_sections() {
    let status = parse_status(
        "## main...origin/main [ahead 2, behind 1]\nM  src/main.rs\n M README.md\n?? notes.txt\n",
    )
    .expect("status");

    assert_eq!(status.branch(), Some("main"));
    assert_eq!(status.ahead(), 2);
    assert_eq!(status.behind(), 1);
    assert_eq!(status.staged().len(), 1);
    assert_eq!(status.unstaged().len(), 1);
    assert_eq!(status.untracked(), ["notes.txt"]);
}

#[test]
fn parser_extracts_unborn_branch_name() {
    let status =
        parse_status("## No commits yet on master\n?? notes.txt\n").expect("unborn status");

    assert_eq!(status.branch(), Some("master"));
    assert_eq!(status.untracked(), ["notes.txt"]);
    assert!(status.staged().is_empty());
    assert!(status.unstaged().is_empty());
}

#[test]
fn repository_file_listing_excludes_gitignored_paths() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = repository_file_list_test_lock();
    if !git_available() {
        return Ok(());
    }

    let root = temp_repo_root("files");
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join(".gitignore"), "ignored.txt\n")?;
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n")?;
    fs::write(root.join("notes.txt"), "notes\n")?;
    fs::write(root.join("ignored.txt"), "ignored\n")?;

    run_git(&root, &["init", "-q"])?;
    run_git(&root, &["add", ".gitignore", "src/main.rs"])?;

    let files = list_repository_files(&root)?;

    assert!(files.contains(&PathBuf::from(".gitignore")));
    assert!(files.contains(&PathBuf::from("src/main.rs")));
    assert!(files.contains(&PathBuf::from("notes.txt")));
    assert!(!files.contains(&PathBuf::from("ignored.txt")));

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn cached_repository_file_listing_reuses_paths_until_identity_changes()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = repository_file_list_test_lock();
    if !git_available() {
        return Ok(());
    }

    invalidate_repository_file_list_cache();
    let root = temp_repo_root("cache-hit");
    fs::create_dir_all(root.join("src"))?;
    fs::write(root.join("src").join("main.rs"), "fn main() {}\n")?;
    run_git(&root, &["init", "-q"])?;
    run_git(&root, &["add", "src/main.rs"])?;

    let generation_before = repository_file_list_generation();
    let first = list_repository_files(&root)?;
    let generation_after_miss = repository_file_list_generation();
    let second = list_repository_files(&root)?;
    let generation_after_hit = repository_file_list_generation();

    assert!(first.contains(&PathBuf::from("src/main.rs")));
    assert_eq!(first, second);
    assert!(generation_after_miss > generation_before);
    assert_eq!(generation_after_hit, generation_after_miss);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn cached_repository_file_listing_is_keyed_by_workspace_root()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = repository_file_list_test_lock();
    if !git_available() {
        return Ok(());
    }

    invalidate_repository_file_list_cache();
    let first_root = temp_repo_root("cache-root-a");
    let second_root = temp_repo_root("cache-root-b");
    fs::create_dir_all(&first_root)?;
    fs::create_dir_all(&second_root)?;
    fs::write(first_root.join("alpha.txt"), "a\n")?;
    fs::write(second_root.join("beta.txt"), "b\n")?;
    for root in [&first_root, &second_root] {
        run_git(root, &["init", "-q"])?;
        run_git(root, &["add", "."])?;
    }

    let first = list_repository_files(&first_root)?;
    let generation_after_first = repository_file_list_generation();
    let second = list_repository_files(&second_root)?;
    let generation_after_second = repository_file_list_generation();
    let first_again = list_repository_files(&first_root)?;

    assert!(first.contains(&PathBuf::from("alpha.txt")));
    assert!(!first.contains(&PathBuf::from("beta.txt")));
    assert!(second.contains(&PathBuf::from("beta.txt")));
    assert!(!second.contains(&PathBuf::from("alpha.txt")));
    assert_eq!(first, first_again);
    assert!(generation_after_second > generation_after_first);
    assert_eq!(
        repository_file_list_generation(),
        generation_after_second,
        "listing the first root again should reuse its cache entry"
    );

    fs::remove_dir_all(first_root)?;
    fs::remove_dir_all(second_root)?;
    Ok(())
}

#[test]
fn cached_repository_file_listing_refreshes_after_index_or_head_change()
-> Result<(), Box<dyn std::error::Error>> {
    let _lock = repository_file_list_test_lock();
    if !git_available() {
        return Ok(());
    }

    invalidate_repository_file_list_cache();
    let root = temp_repo_root("cache-head");
    fs::create_dir_all(&root)?;
    fs::write(root.join("main.txt"), "main\n")?;
    run_git(&root, &["init", "-q"])?;
    configure_git_identity(&root)?;
    run_git(&root, &["add", "main.txt"])?;
    run_git(&root, &["commit", "-qm", "init"])?;
    let default_branch = git_stdout(&root, &["rev-parse", "--abbrev-ref", "HEAD"])?;

    let first = list_repository_files(&root)?;
    let generation_after_first = repository_file_list_generation();
    assert!(first.contains(&PathBuf::from("main.txt")));
    assert!(!first.contains(&PathBuf::from("extra.txt")));

    fs::write(root.join("extra.txt"), "extra\n")?;
    run_git(&root, &["add", "extra.txt"])?;
    let after_index = list_repository_files(&root)?;
    let generation_after_index = repository_file_list_generation();
    assert!(after_index.contains(&PathBuf::from("extra.txt")));
    assert!(generation_after_index > generation_after_first);

    run_git(&root, &["commit", "-qm", "add extra"])?;
    run_git(&root, &["checkout", "-qb", "other"])?;
    fs::write(root.join("other.txt"), "other\n")?;
    run_git(&root, &["add", "other.txt"])?;
    run_git(&root, &["commit", "-qm", "other branch"])?;
    run_git(&root, &["checkout", "-q", &default_branch])?;
    let after_checkout = list_repository_files(&root)?;
    assert!(after_checkout.contains(&PathBuf::from("extra.txt")));
    assert!(!after_checkout.contains(&PathBuf::from("other.txt")));
    assert!(repository_file_list_generation() > generation_after_index);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn invalidate_repository_file_list_cache_forces_rescan() -> Result<(), Box<dyn std::error::Error>> {
    let _lock = repository_file_list_test_lock();
    if !git_available() {
        return Ok(());
    }

    invalidate_repository_file_list_cache();
    let root = temp_repo_root("cache-invalidate");
    fs::create_dir_all(&root)?;
    fs::write(root.join("keep.txt"), "keep\n")?;
    run_git(&root, &["init", "-q"])?;
    run_git(&root, &["add", "keep.txt"])?;

    let _ = list_repository_files(&root)?;
    let generation_after_first = repository_file_list_generation();
    invalidate_repository_file_list_cache_for(&root);
    let _ = list_repository_files(&root)?;
    assert!(repository_file_list_generation() > generation_after_first);

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn repository_file_preview_includes_path_and_caps_lines() -> Result<(), Box<dyn std::error::Error>>
{
    let root = temp_repo_root("preview-caps");
    fs::create_dir_all(&root)?;
    let path = root.join("notes.txt");
    let body = (0..40)
        .map(|index| format!("line-{index}"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(&path, body)?;

    let preview = repository_file_preview(&path);
    let lines = preview.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], path.display().to_string());
    assert_eq!(lines.len(), REPOSITORY_FILE_PREVIEW_MAX_LINES + 1);
    assert_eq!(lines[1], "line-0");
    assert_eq!(
        lines[REPOSITORY_FILE_PREVIEW_MAX_LINES],
        format!("line-{}", REPOSITORY_FILE_PREVIEW_MAX_LINES - 1)
    );

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn repository_file_preview_falls_back_to_path_for_invalid_utf8()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_repo_root("preview-binary");
    fs::create_dir_all(&root)?;
    let path = root.join("blob.bin");
    fs::write(&path, [0xff, 0xfe, 0x00, 0x01])?;

    assert_eq!(repository_file_preview(&path), path.display().to_string());

    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn parses_log_oneline_entries() {
    let entries = parse_log_oneline("abc123 first\nfed456 second commit\n");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].hash(), "abc123");
    assert_eq!(entries[0].summary(), "first");
    assert_eq!(entries[1].hash(), "fed456");
    assert_eq!(entries[1].summary(), "second commit");
}

#[test]
fn parses_stash_list_entries() {
    let entries = parse_stash_list("stash@{0}: WIP on main\nstash@{1}: update\n");
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].name(), "stash@{0}");
    assert_eq!(entries[0].summary(), "WIP on main");
    assert_eq!(entries[1].name(), "stash@{1}");
    assert_eq!(entries[1].summary(), "update");
}
