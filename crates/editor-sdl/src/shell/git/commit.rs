#![allow(unused_imports)]
use super::super::*;

#[allow(unused_imports)]
use super::commands::*;
#[allow(unused_imports)]
use super::diff::*;
#[allow(unused_imports)]
use super::fringe::*;
#[allow(unused_imports)]
use super::log::*;
#[allow(unused_imports)]
use super::merge_rebase::*;
#[allow(unused_imports)]
use super::pickers::*;
#[allow(unused_imports)]
use super::process::*;
#[allow(unused_imports)]
use super::remote::*;
#[allow(unused_imports)]
use super::staging::*;
#[allow(unused_imports)]
use super::stash::*;
#[allow(unused_imports)]
use super::status::*;
#[allow(unused_imports)]
use super::worktree::*;

pub(crate) fn open_git_commit_buffer(runtime: &mut EditorRuntime) -> Result<(), String> {
    let existing = shell_ui(runtime)
        .ok()
        .and_then(|ui| find_shell_buffer_by_kind(ui, GIT_COMMIT_KIND));
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    if let Some(existing) = existing {
        runtime
            .model_mut()
            .focus_buffer(workspace_id, existing)
            .map_err(|error| error.to_string())?;
        let ui = shell_ui_mut(runtime)?;
        ui.focus_buffer_in_active_pane(existing);
        ui.enter_normal_mode();
        return Ok(());
    }
    let buffer_id = {
        runtime
            .model_mut()
            .create_buffer(
                workspace_id,
                "*git-commit*",
                BufferKind::Plugin(GIT_COMMIT_KIND.to_owned()),
                None,
            )
            .map_err(|error| error.to_string())?
    };
    let root = git_root(runtime)?;
    let snapshot = git_status_snapshot(runtime, &root)?;
    let user_library = shell_user_library(runtime);
    let template = user_library.git_commit_template(&snapshot);
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, template, &*user_library);
    shell_buffer.set_language_id(Some("gitcommit".to_owned()));
    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    queue_buffer_syntax_refresh(runtime, buffer_id)
}

pub(crate) fn git_commit_temp_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    env::temp_dir().join(format!(
        "volt-git-commit-{}-{unique}.txt",
        std::process::id()
    ))
}

pub(crate) fn git_commit_message(buffer: &ShellBuffer) -> String {
    let raw = buffer.text.text();
    let mut lines = Vec::new();
    for line in raw.lines() {
        let trimmed = line.trim_end();
        if trimmed.trim_start().starts_with('#') {
            continue;
        }
        lines.push(trimmed);
    }
    lines.join("\n").trim().to_owned()
}

pub(crate) fn commit_git_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let root = git_root(runtime)?;
    let message = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        git_commit_message(buffer)
    };
    if message.trim().is_empty() {
        return Err("commit message is empty".to_owned());
    }
    let temp_path = git_commit_temp_path();
    fs::write(&temp_path, &message)
        .map_err(|error| format!("failed to write commit message: {error}"))?;
    let result = git_command_output(
        runtime,
        &root,
        "commit",
        &["commit", "-F", &temp_path.to_string_lossy()],
    );
    fs::remove_file(&temp_path).ok();
    result?;
    mark_git_fringe_snapshots_stale(runtime)?;
    invalidate_git_identity_for_active_workspace(runtime);
    close_buffer_discard(runtime, buffer_id)?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}

pub(crate) fn cancel_git_commit_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    close_buffer_discard(runtime, buffer_id)?;
    refresh_git_status_if_active(runtime)?;
    Ok(())
}
