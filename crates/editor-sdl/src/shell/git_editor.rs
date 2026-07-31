//! Git Editor Buffer: stands in for `GIT_EDITOR` / `GIT_SEQUENCE_EDITOR`.

use super::*;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) const GIT_EDITOR_KIND: &str = "git-editor";
pub(super) const VOLT_GIT_EDITOR_DIR_ENV: &str = "VOLT_GIT_EDITOR_DIR";

#[derive(Debug)]
struct GitEditorSession {
    request_id: String,
    file_path: PathBuf,
}

#[derive(Debug, Default)]
pub(super) struct GitEditorState {
    dir: Option<PathBuf>,
    stub_path: Option<PathBuf>,
    sessions: BTreeMap<BufferId, GitEditorSession>,
    /// Request ids already opened (or failed), so we do not reopen them.
    seen_requests: BTreeSet<String>,
}

impl GitEditorState {
    pub(super) fn new() -> Self {
        Self::default()
    }

    fn ensure_dir_and_stub(&mut self) -> Result<(PathBuf, PathBuf), String> {
        if let (Some(dir), Some(stub)) = (self.dir.clone(), self.stub_path.clone()) {
            return Ok((dir, stub));
        }
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0))
            .as_nanos();
        let dir = env::temp_dir().join(format!("volt-git-editor-{}-{unique}", std::process::id()));
        fs::create_dir_all(&dir).map_err(|error| {
            format!(
                "failed to create git editor dir `{}`: {error}",
                dir.display()
            )
        })?;
        let stub_path = write_git_editor_stub(&dir)?;
        self.dir = Some(dir.clone());
        self.stub_path = Some(stub_path.clone());
        Ok((dir, stub_path))
    }

    /// If a Git Editor Buffer is closed without confirm, signal abort to the stub.
    pub(super) fn abort_if_session(&mut self, buffer_id: BufferId) {
        let Some(session) = self.sessions.remove(&buffer_id) else {
            return;
        };
        let Some(dir) = self.dir.as_ref() else {
            return;
        };
        let result_path = dir.join(format!("result-{}", session.request_id));
        let _ = fs::write(&result_path, "1\n");
    }
}

pub(super) fn inject_git_editor_env(
    runtime: &mut EditorRuntime,
    env: &mut Vec<(String, String)>,
) -> Result<(), String> {
    let (dir, stub) = {
        let ui = shell_ui_mut(runtime)?;
        ui.git_editor.ensure_dir_and_stub()?
    };
    let stub_display = stub
        .to_str()
        .ok_or_else(|| format!("non-utf8 git editor stub path `{}`", stub.display()))?
        .to_owned();
    let dir_display = dir
        .to_str()
        .ok_or_else(|| format!("non-utf8 git editor dir `{}`", dir.display()))?
        .to_owned();
    env.retain(|(key, _)| {
        key != "GIT_EDITOR"
            && key != "GIT_SEQUENCE_EDITOR"
            && key != VOLT_GIT_EDITOR_DIR_ENV
            && key != "GIT_TERMINAL_PROMPT"
    });
    env.push(("GIT_EDITOR".to_owned(), stub_display.clone()));
    env.push(("GIT_SEQUENCE_EDITOR".to_owned(), stub_display));
    env.push((VOLT_GIT_EDITOR_DIR_ENV.to_owned(), dir_display));
    env.push(("GIT_TERMINAL_PROMPT".to_owned(), "0".to_owned()));
    Ok(())
}

pub(super) fn refresh_pending_git_editor(runtime: &mut EditorRuntime) -> Result<bool, String> {
    let dir = match shell_ui(runtime)?.git_editor.dir.clone() {
        Some(dir) => dir,
        None => return Ok(false),
    };
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return Ok(false),
    };
    let mut opened = false;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(request_id) = name.strip_prefix("request-") else {
            continue;
        };
        if shell_ui(runtime)?
            .git_editor
            .seen_requests
            .contains(request_id)
        {
            continue;
        }
        let request_path = entry.path();
        let file_path = fs::read_to_string(&request_path)
            .map_err(|error| format!("failed to read git editor request: {error}"))?
            .lines()
            .next()
            .unwrap_or_default()
            .trim()
            .to_owned();
        if file_path.is_empty() {
            shell_ui_mut(runtime)?
                .git_editor
                .seen_requests
                .insert(request_id.to_owned());
            continue;
        }
        let path = PathBuf::from(&file_path);
        open_git_editor_buffer(runtime, request_id, &path)?;
        opened = true;
    }
    Ok(opened)
}

fn open_git_editor_buffer(
    runtime: &mut EditorRuntime,
    request_id: &str,
    path: &Path,
) -> Result<(), String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let contents = fs::read_to_string(path).unwrap_or_default();
    let lines = contents.lines().map(str::to_owned).collect::<Vec<_>>();
    let display_name = format!(
        "*git-editor {}*",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("file")
    );
    let buffer_id = runtime
        .model_mut()
        .create_buffer(
            workspace_id,
            &display_name,
            BufferKind::Plugin(GIT_EDITOR_KIND.to_owned()),
            Some(path.to_path_buf()),
        )
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("git editor buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let mut shell_buffer = ShellBuffer::from_runtime_buffer(buffer, lines, &*user_library);
    shell_buffer.set_language_id(Some("gitcommit".to_owned()));
    {
        let ui = shell_ui_mut(runtime)?;
        ui.git_editor.seen_requests.insert(request_id.to_owned());
        ui.git_editor.sessions.insert(
            buffer_id,
            GitEditorSession {
                request_id: request_id.to_owned(),
                file_path: path.to_path_buf(),
            },
        );
        ui.insert_buffer(shell_buffer);
        ui.focus_buffer_in_active_pane(buffer_id);
        ui.enter_normal_mode();
    }
    runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    Ok(())
}

pub(super) fn confirm_git_editor_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    finish_git_editor_buffer(runtime, buffer_id, 0)
}

pub(super) fn abort_git_editor_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    finish_git_editor_buffer(runtime, buffer_id, 1)
}

fn finish_git_editor_buffer(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    exit_code: i32,
) -> Result<(), String> {
    let session = shell_ui_mut(runtime)?
        .git_editor
        .sessions
        .remove(&buffer_id)
        .ok_or_else(|| "git editor session is missing".to_owned())?;
    if exit_code == 0 {
        let text = shell_buffer(runtime, buffer_id)?.text.text();
        if let Some(parent) = session.file_path.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to create git editor parent `{}`: {error}",
                    parent.display()
                )
            })?;
        }
        fs::write(&session.file_path, text).map_err(|error| {
            format!(
                "failed to write git editor file `{}`: {error}",
                session.file_path.display()
            )
        })?;
    }
    let dir = shell_ui(runtime)?
        .git_editor
        .dir
        .clone()
        .ok_or_else(|| "git editor dir is missing".to_owned())?;
    let result_path = dir.join(format!("result-{}", session.request_id));
    fs::write(&result_path, format!("{exit_code}\n")).map_err(|error| {
        format!(
            "failed to write git editor result `{}`: {error}",
            result_path.display()
        )
    })?;
    close_buffer_immediate(runtime, buffer_id)?;
    Ok(())
}

fn write_git_editor_stub(dir: &Path) -> Result<PathBuf, String> {
    #[cfg(windows)]
    {
        let stub = dir.join("volt-git-editor.cmd");
        let script = format!(
            "@echo off\r\n\
             setlocal EnableDelayedExpansion\r\n\
             set \"FILE=%~1\"\r\n\
             if \"%FILE%\"==\"\" exit /b 1\r\n\
             set \"DIR=%{VOLT_GIT_EDITOR_DIR_ENV}%\"\r\n\
             if \"%DIR%\"==\"\" exit /b 1\r\n\
             set \"ID=%RANDOM%%RANDOM%\"\r\n\
             echo !FILE!>\"%DIR%\\request-!ID!\"\r\n\
             :volt_git_editor_wait\r\n\
             if exist \"%DIR%\\result-!ID!\" (\r\n\
               set /p CODE=<\"%DIR%\\result-!ID!\"\r\n\
               exit /b !CODE!\r\n\
             )\r\n\
             ping -n 1 127.0.0.1 >nul\r\n\
             goto volt_git_editor_wait\r\n"
        );
        fs::write(&stub, script)
            .map_err(|error| format!("failed to write git editor stub: {error}"))?;
        Ok(stub)
    }
    #[cfg(not(windows))]
    {
        let stub = dir.join("volt-git-editor.sh");
        let script = format!(
            "#!/bin/sh\n\
             FILE=\"$1\"\n\
             DIR=\"${{{VOLT_GIT_EDITOR_DIR_ENV}}}\"\n\
             if [ -z \"$FILE\" ] || [ -z \"$DIR\" ]; then exit 1; fi\n\
             ID=\"$$-$(date +%s)-$RANDOM\"\n\
             printf '%s\\n' \"$FILE\" > \"$DIR/request-$ID\"\n\
             while [ ! -f \"$DIR/result-$ID\" ]; do\n\
               sleep 0.1\n\
             done\n\
             CODE=$(cat \"$DIR/result-$ID\")\n\
             exit \"$CODE\"\n"
        );
        fs::write(&stub, script)
            .map_err(|error| format!("failed to write git editor stub: {error}"))?;
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = fs::metadata(&stub)
            .map_err(|error| error.to_string())?
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&stub, permissions).map_err(|error| error.to_string())?;
        Ok(stub)
    }
}
