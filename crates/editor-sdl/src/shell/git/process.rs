use super::super::*;

use super::remote::*;
use super::status::*;
use std::process::Stdio;

pub(crate) fn git_args_with_no_pager(command: &str, extra: &[&str]) -> Vec<String> {
    let mut args = Vec::with_capacity(2 + extra.len());
    args.push("--no-pager".to_owned());
    args.push(command.to_owned());
    args.extend(extra.iter().map(|arg| (*arg).to_owned()));
    args
}

pub(crate) fn git_command_output_background(
    root: &Path,
    args: &[&str],
    allowed_exit_codes: &[i32],
) -> Option<String> {
    let output = run_direct_git_command(root, args).ok()?;
    let exit_code = output.status.code()?;
    if exit_code != 0 && !allowed_exit_codes.contains(&exit_code) {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(crate) fn git_repository_present(root: &Path) -> bool {
    git_probe_snapshot(root).present()
}

pub(crate) fn git_command_output(
    runtime: &mut EditorRuntime,
    root: &Path,
    label: &str,
    args: &[&str],
) -> Result<String, String> {
    let spec = JobSpec::command(
        label,
        "git",
        args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>(),
    )
    .with_cwd(root.to_path_buf());
    let manager = runtime
        .services()
        .get::<Mutex<JobManager>>()
        .ok_or_else(|| "job manager service missing".to_owned())?;
    let mut manager = manager
        .lock()
        .map_err(|_| "job manager lock poisoned".to_owned())?;
    let handle = manager.spawn(spec).map_err(|error| error.to_string())?;
    drop(manager);
    let result = handle.wait().map_err(|error| error.to_string())?;
    if !result.succeeded() {
        return Err(format!("git {label} failed: {}", result.transcript()));
    }
    Ok(result.stdout().to_owned())
}

pub(crate) fn git_command_output_owned(
    runtime: &mut EditorRuntime,
    root: &Path,
    label: &str,
    args: &[String],
) -> Result<String, String> {
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    git_command_output(runtime, root, label, &refs)
}

pub(crate) fn git_read_command_output(
    root: &Path,
    label: &str,
    args: &[&str],
) -> Result<String, String> {
    git_read_command_output_allow_exit_codes(root, label, args, &[0])
}

pub(crate) fn git_read_command_output_optional(
    root: &Path,
    label: &str,
    args: &[&str],
) -> Option<String> {
    git_read_command_output(root, label, args).ok()
}

pub(crate) fn git_read_log_oneline_optional(
    root: &Path,
    label: &str,
    revision: &str,
) -> Vec<GitLogEntry> {
    let limit = GIT_LOG_LIMIT.to_string();
    git_read_command_output_optional(root, label, &["log", "-n", &limit, "--oneline", revision])
        .map(|output| parse_log_oneline(&output))
        .unwrap_or_default()
}

pub(crate) fn git_read_command_output_allow_exit_codes(
    root: &Path,
    label: &str,
    args: &[&str],
    allowed_exit_codes: &[i32],
) -> Result<String, String> {
    let output = run_direct_git_command(root, args)?;
    let exit_code = output.status.code().ok_or_else(|| {
        format!(
            "git {label} failed to return an exit code: {}",
            command_output_transcript(&output)
        )
    })?;
    if exit_code != 0 && !allowed_exit_codes.contains(&exit_code) {
        return Err(format!(
            "git {label} failed: {}",
            command_output_transcript(&output)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub(crate) fn run_direct_git_command(
    root: &Path,
    args: &[&str],
) -> Result<std::process::Output, String> {
    let mut command = Command::new("git");
    configure_background_command(&mut command);
    command
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| {
            format!(
                "failed to run git {:?} in {}: {error}",
                args,
                root.display()
            )
        })
}

pub(crate) fn command_output_transcript(output: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.is_empty() {
        stdout.into_owned()
    } else if stdout.is_empty() {
        stderr.into_owned()
    } else {
        format!("{stdout}{stderr}")
    }
}

pub(crate) fn git_dir_path(_runtime: &mut EditorRuntime, root: &Path) -> Option<PathBuf> {
    let probe = git_probe_snapshot(root);
    if let Some(git_dir) = probe.git_dir() {
        return Some(git_dir.to_path_buf());
    }
    if !probe.present() {
        return None;
    }
    let output =
        git_read_command_output_optional(root, "rev-parse --git-dir", &["rev-parse", "--git-dir"])?;
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return None;
    }
    let path = normalize_git_output_path(trimmed);
    if path.is_absolute() {
        Some(path)
    } else {
        Some(root.join(path))
    }
}

pub(crate) fn invalidate_git_identity_for_active_workspace(runtime: &mut EditorRuntime) {
    if let Ok(Some(root)) = active_workspace_root(runtime) {
        invalidate_git_probe_cache_for(&root);
    }
    if let Ok(ui) = shell_ui_mut(runtime) {
        ui.mark_git_summary_stale();
    }
}

pub(crate) fn git_status_snapshot(
    runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<GitStatusSnapshot, String> {
    let status_output = git_read_command_output(
        root,
        "status --short --branch",
        &["status", "--short", "--branch"],
    )?;
    let status = parse_status(&status_output).map_err(|error| error.to_string())?;

    let recent_output = git_read_command_output_optional(
        root,
        "log --oneline",
        &["log", "-n", &GIT_LOG_LIMIT.to_string(), "--oneline"],
    )
    .unwrap_or_default();
    let recent = parse_log_oneline(&recent_output);
    let head = recent.first().cloned();
    let head_exists = head.is_some();

    let upstream = git_read_command_output_optional(
        root,
        "rev-parse --abbrev-ref @{upstream}",
        &["rev-parse", "--abbrev-ref", "@{upstream}"],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
    .or_else(|| status_output_upstream(&status_output));
    let push_remote = git_read_command_output_optional(
        root,
        "rev-parse --abbrev-ref @{push}",
        &["rev-parse", "--abbrev-ref", "@{push}"],
    )
    .map(|value| value.trim().to_owned())
    .filter(|value| !value.is_empty())
    .or_else(|| upstream.clone());
    let tag = git_head_tag(root);

    let stash_output = git_read_command_output_optional(root, "stash list", &["stash", "list"])
        .unwrap_or_default();
    let stashes = parse_stash_list(&stash_output);

    let unpulled = if head_exists && upstream.is_some() {
        git_read_log_oneline_optional(root, "log --oneline ..@{upstream}", "..@{upstream}")
    } else {
        Vec::new()
    };
    let unpushed = if head_exists && upstream.is_some() {
        git_read_log_oneline_optional(root, "log --oneline @{upstream}..", "@{upstream}..")
    } else {
        Vec::new()
    };

    let in_progress = git_dir_path(runtime, root)
        .map(detect_in_progress)
        .unwrap_or_default();

    Ok(GitStatusSnapshot::default()
        .with_status(status)
        .with_head(head)
        .with_upstreams(upstream, push_remote)
        .with_tag(tag)
        .with_stashes(stashes)
        .with_unpulled(unpulled)
        .with_unpushed(unpushed)
        .with_recent(recent)
        .with_in_progress(in_progress))
}

pub(crate) fn git_remote_list(
    _runtime: &mut EditorRuntime,
    root: &Path,
) -> Result<Vec<String>, String> {
    let output = git_read_command_output(root, "remote", &["remote"])?;
    let mut remotes = output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned())
        .collect::<Vec<_>>();
    remotes.sort();
    remotes.dedup();
    Ok(remotes)
}
