use super::*;
use std::{
    io::{BufReader, Read},
    process::Stdio,
    thread,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StreamedCommandExitAction {
    RefreshGitStatusBuffersAndCloseBuffer,
}

#[derive(Debug, Clone)]
pub(super) struct StreamedCommandSpec {
    pub(super) popup_title: String,
    pub(super) buffer_name: String,
    pub(super) command_label: String,
    pub(super) program: String,
    pub(super) args: Vec<String>,
    pub(super) cwd: PathBuf,
    pub(super) on_exit: StreamedCommandExitAction,
}

#[derive(Debug, Clone)]
struct StreamedCommandRequest {
    buffer_id: BufferId,
    popup_title: String,
    command_label: String,
    program: String,
    args: Vec<String>,
    cwd: PathBuf,
    on_exit: StreamedCommandExitAction,
}

#[derive(Debug)]
enum StreamedCommandUpdate {
    Output {
        buffer_id: BufferId,
        lines: Vec<String>,
    },
    Finished {
        buffer_id: BufferId,
        popup_title: String,
        command_label: String,
        success: bool,
        exit_code: Option<i32>,
        error: Option<String>,
        on_exit: StreamedCommandExitAction,
    },
}

#[derive(Debug, Default)]
pub(super) struct StreamedCommandWorkerState {
    active_buffers: BTreeSet<BufferId>,
    updates: Arc<Mutex<Vec<StreamedCommandUpdate>>>,
}

impl StreamedCommandWorkerState {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn contains(&self, buffer_id: BufferId) -> bool {
        self.active_buffers.contains(&buffer_id)
    }

    pub(super) fn remove(&mut self, buffer_id: BufferId) -> bool {
        self.active_buffers.remove(&buffer_id)
    }

    fn take_updates(&self) -> Result<Vec<StreamedCommandUpdate>, String> {
        let mut updates = self
            .updates
            .lock()
            .map_err(|_| "streamed command worker mutex poisoned".to_owned())?;
        Ok(std::mem::take(&mut *updates))
    }

    fn start(&mut self, request: StreamedCommandRequest) -> Result<(), String> {
        let buffer_id = request.buffer_id;
        let updates = Arc::clone(&self.updates);
        self.active_buffers.insert(buffer_id);
        if let Err(error) = thread::Builder::new()
            .name(format!("streamed-command-{buffer_id}"))
            .spawn(move || run_streamed_command(request, updates))
        {
            self.active_buffers.remove(&buffer_id);
            return Err(format!("failed to start streamed command worker: {error}"));
        }
        Ok(())
    }
}

pub(super) fn open_streamed_command_popup(
    runtime: &mut EditorRuntime,
    spec: StreamedCommandSpec,
) -> Result<BufferId, String> {
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_kind = BufferKind::Plugin(INTERACTIVE_READONLY_KIND.to_owned());
    let buffer_id = runtime
        .model_mut()
        .create_popup_buffer(workspace_id, &spec.buffer_name, buffer_kind.clone(), None)
        .map_err(|error| error.to_string())?;
    runtime
        .model_mut()
        .open_popup_buffer(workspace_id, &spec.popup_title, buffer_id)
        .map_err(|error| error.to_string())?;
    let buffer = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?
        .buffer(buffer_id)
        .ok_or_else(|| format!("popup buffer `{buffer_id}` is missing"))?;
    let user_library = shell_user_library(runtime);
    let shell_buffer = ShellBuffer::from_runtime_buffer(
        buffer,
        vec![format!("$ {}", spec.command_label), String::new()],
        &*user_library,
    );
    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    let request = StreamedCommandRequest {
        buffer_id,
        popup_title: spec.popup_title,
        command_label: spec.command_label,
        program: spec.program,
        args: spec.args,
        cwd: spec.cwd,
        on_exit: spec.on_exit,
    };
    if let Err(error) = shell_ui_mut(runtime)?
        .streamed_command_worker
        .start(request)
    {
        close_popup_buffer_and_restore_focus(runtime, buffer_id)?;
        return Err(error);
    }
    Ok(buffer_id)
}

pub(super) fn refresh_pending_streamed_commands(
    runtime: &mut EditorRuntime,
) -> Result<bool, String> {
    let updates = shell_ui_mut(runtime)?
        .streamed_command_worker
        .take_updates()?;
    if updates.is_empty() {
        return Ok(false);
    }

    let mut changed = false;
    let mut buffers_to_close = Vec::new();
    let mut refresh_git_status = false;
    let now = Instant::now();

    for update in updates {
        match update {
            StreamedCommandUpdate::Output { buffer_id, lines } => {
                if !shell_ui(runtime)?
                    .streamed_command_worker
                    .contains(buffer_id)
                {
                    continue;
                }
                if let Ok(buffer) = shell_buffer_mut(runtime, buffer_id) {
                    buffer.append_output_lines(&lines);
                    changed = true;
                }
            }
            StreamedCommandUpdate::Finished {
                buffer_id,
                popup_title,
                command_label,
                success,
                exit_code,
                error,
                on_exit,
            } => {
                if !shell_ui_mut(runtime)?
                    .streamed_command_worker
                    .remove(buffer_id)
                {
                    continue;
                }
                shell_ui_mut(runtime)?.apply_notification(
                    streamed_command_notification(
                        buffer_id,
                        &popup_title,
                        &command_label,
                        success,
                        exit_code,
                        error.as_deref(),
                    ),
                    now,
                );
                match on_exit {
                    StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer => {
                        buffers_to_close.push(buffer_id);
                        refresh_git_status = true;
                    }
                }
                changed = true;
            }
        }
    }

    for buffer_id in buffers_to_close {
        close_popup_buffer_and_restore_focus(runtime, buffer_id)?;
    }
    if refresh_git_status {
        refresh_git_status_buffers(runtime)?;
        changed = true;
    }
    Ok(changed)
}

fn streamed_command_notification(
    buffer_id: BufferId,
    popup_title: &str,
    command_label: &str,
    success: bool,
    exit_code: Option<i32>,
    error: Option<&str>,
) -> NotificationUpdate {
    let mut body_lines = vec![command_label.to_owned()];
    if let Some(error) = error {
        body_lines.push(error.to_owned());
    } else if !success && let Some(exit_code) = exit_code {
        body_lines.push(format!("Exit code: {exit_code}"));
    }
    NotificationUpdate {
        key: format!("streamed-command:{buffer_id}"),
        severity: if success {
            NotificationSeverity::Success
        } else {
            NotificationSeverity::Error
        },
        title: if success {
            format!("{popup_title} succeeded")
        } else {
            format!("{popup_title} failed")
        },
        body_lines,
        progress: None,
        active: false,
        action: None,
    }
}

fn run_streamed_command(
    request: StreamedCommandRequest,
    updates: Arc<Mutex<Vec<StreamedCommandUpdate>>>,
) {
    let StreamedCommandRequest {
        buffer_id,
        popup_title,
        command_label,
        program,
        args,
        cwd,
        on_exit,
    } = request;
    let mut command = Command::new(&program);
    command
        .args(&args)
        .current_dir(&cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    configure_background_command(&mut command);

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            push_streamed_command_update(
                &updates,
                StreamedCommandUpdate::Finished {
                    buffer_id,
                    popup_title,
                    command_label,
                    success: false,
                    exit_code: None,
                    error: Some(format!("Failed to start process: {error}")),
                    on_exit,
                },
            );
            return;
        }
    };

    let stdout_reader = child.stdout.take().map(|stdout| {
        let updates = Arc::clone(&updates);
        thread::spawn(move || stream_command_output(buffer_id, stdout, updates))
    });
    let stderr_reader = child.stderr.take().map(|stderr| {
        let updates = Arc::clone(&updates);
        thread::spawn(move || stream_command_output(buffer_id, stderr, updates))
    });

    let status = child.wait();
    if let Some(reader) = stdout_reader {
        let _ = reader.join();
    }
    if let Some(reader) = stderr_reader {
        let _ = reader.join();
    }

    match status {
        Ok(status) => push_streamed_command_update(
            &updates,
            StreamedCommandUpdate::Finished {
                buffer_id,
                popup_title,
                command_label,
                success: status.success(),
                exit_code: status.code(),
                error: None,
                on_exit,
            },
        ),
        Err(error) => push_streamed_command_update(
            &updates,
            StreamedCommandUpdate::Finished {
                buffer_id,
                popup_title,
                command_label,
                success: false,
                exit_code: None,
                error: Some(format!(
                    "Failed while waiting for process completion: {error}"
                )),
                on_exit,
            },
        ),
    }
}

fn stream_command_output<R: Read>(
    buffer_id: BufferId,
    reader: R,
    updates: Arc<Mutex<Vec<StreamedCommandUpdate>>>,
) {
    let mut reader = BufReader::new(reader);
    let mut pending = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => {
                if !pending.is_empty() {
                    push_streamed_command_update(
                        &updates,
                        StreamedCommandUpdate::Output {
                            buffer_id,
                            lines: vec![String::from_utf8_lossy(&pending).into_owned()],
                        },
                    );
                }
                break;
            }
            Ok(read) => {
                pending.extend_from_slice(&chunk[..read]);
                let lines = drain_completed_output_lines(&mut pending);
                if !lines.is_empty() {
                    push_streamed_command_update(
                        &updates,
                        StreamedCommandUpdate::Output { buffer_id, lines },
                    );
                }
            }
            Err(error) => {
                push_streamed_command_update(
                    &updates,
                    StreamedCommandUpdate::Output {
                        buffer_id,
                        lines: vec![format!("command output stream failed: {error}")],
                    },
                );
                break;
            }
        }
    }
}

fn drain_completed_output_lines(pending: &mut Vec<u8>) -> Vec<String> {
    let mut lines = Vec::new();
    while let Some(index) = pending
        .iter()
        .position(|byte| *byte == b'\n' || *byte == b'\r')
    {
        let line = String::from_utf8_lossy(&pending[..index]).into_owned();
        let delimiter = pending[index];
        pending.drain(..=index);
        if let Some(next) = pending.first().copied()
            && ((delimiter == b'\r' && next == b'\n') || (delimiter == b'\n' && next == b'\r'))
        {
            pending.remove(0);
        }
        lines.push(line);
    }
    lines
}

fn push_streamed_command_update(
    updates: &Arc<Mutex<Vec<StreamedCommandUpdate>>>,
    update: StreamedCommandUpdate,
) {
    if let Ok(mut updates) = updates.lock() {
        updates.push(update);
    }
}
