use super::{
    tool_install::{ToolInstallState, continue_tool_install},
    treesitter_install::{
        TreeSitterInstallState, TreeSitterRecompileState, continue_tree_sitter_install,
        continue_tree_sitter_recompile,
    },
    *,
};
use editor_jobs::{
    ProcessSupervisionMode, enrich_env_with_node_manager, supervised_command_if_resolved,
};
use std::{
    collections::BTreeMap,
    io::{BufReader, Read},
    process::Stdio,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};

#[derive(Debug)]
pub(super) enum StreamedCommandExitAction {
    RefreshGitStatusBuffersAndCloseBuffer,
    /// Refresh git status, close the stream popup, then open the commit buffer.
    RefreshGitStatusCloseAndOpenCommitBuffer,
    /// Refresh git status, close the stream popup, then open a Project Workspace.
    RefreshGitStatusCloseAndOpenWorkspace {
        name: String,
        path: PathBuf,
    },
    ContinueTreeSitterInstall(Box<TreeSitterInstallState>),
    ContinueTreeSitterRecompile(Box<TreeSitterRecompileState>),
    ContinueToolInstall(Box<ToolInstallState>),
    /// Keep the popup buffer open after the process exits; no git refresh, no close.
    LeaveOpen,
    /// Keep the popup buffer open; if the build succeeded and the command targets
    /// `volt-user`, trigger a user-library hot-reload into the running runtime.
    LeaveOpenAndMaybeReloadUserLibrary {
        command: String,
    },
}

/// How an External Command is invoked.
#[derive(Debug, Clone)]
pub(super) enum ExternalCommandInvocation {
    Argv { program: String, args: Vec<String> },
    Shell(String),
}

/// Spec for the host [`run_command`] entry point (Command Stream or Silent Command).
#[derive(Debug)]
pub(super) struct ExternalCommandSpec {
    pub(super) popup_title: String,
    pub(super) buffer_name: String,
    pub(super) command_label: Option<String>,
    pub(super) invocation: ExternalCommandInvocation,
    pub(super) env: Vec<(String, String)>,
    pub(super) cwd: PathBuf,
    /// Default true: Command Stream into a popup. False: Silent Command (sync wait).
    pub(super) stream: bool,
    pub(super) on_exit: StreamedCommandExitAction,
    pub(super) notify_on_success: bool,
    pub(super) notify_on_failure: bool,
    /// Inject `GIT_EDITOR` / `GIT_SEQUENCE_EDITOR` pointing at the Volt Git Editor stub.
    pub(super) use_git_editor: bool,
}

#[derive(Debug)]
pub(super) enum ExternalCommandResult {
    Streamed,
    Silent,
}

impl ExternalCommandSpec {
    pub(super) fn git_argv(
        popup_title: impl Into<String>,
        args: Vec<String>,
        cwd: PathBuf,
        on_exit: StreamedCommandExitAction,
    ) -> Self {
        let command_label = format!("git {}", args.join(" "));
        let buffer_name = format!("*{command_label}*");
        Self {
            popup_title: popup_title.into(),
            buffer_name,
            command_label: Some(command_label),
            invocation: ExternalCommandInvocation::Argv {
                program: "git".to_owned(),
                args,
            },
            env: Vec::new(),
            cwd,
            stream: true,
            on_exit,
            notify_on_success: true,
            notify_on_failure: true,
            use_git_editor: false,
        }
    }

    pub(super) fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub(super) fn with_notify(mut self, on_success: bool, on_failure: bool) -> Self {
        self.notify_on_success = on_success;
        self.notify_on_failure = on_failure;
        self
    }

    pub(super) fn with_git_editor(mut self, enabled: bool) -> Self {
        self.use_git_editor = enabled;
        self
    }
}

#[derive(Debug, Clone)]
pub(super) struct StreamedCommandOutcome {
    pub(super) success: bool,
    pub(super) exit_code: Option<i32>,
    pub(super) error: Option<String>,
}

#[derive(Debug)]
pub(super) struct StreamedCommandSpec {
    pub(super) popup_title: String,
    pub(super) buffer_name: String,
    pub(super) command_label: String,
    pub(super) program: String,
    pub(super) args: Vec<String>,
    pub(super) env: Vec<(String, String)>,
    pub(super) cwd: PathBuf,
    pub(super) on_exit: StreamedCommandExitAction,
    pub(super) notify_on_success: bool,
    pub(super) notify_on_failure: bool,
}

#[derive(Debug)]
struct StreamedCommandRequest {
    buffer_id: BufferId,
    popup_title: String,
    command_label: String,
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: PathBuf,
    on_exit: StreamedCommandExitAction,
    notify_on_success: bool,
    notify_on_failure: bool,
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
        notify_on_success: bool,
        notify_on_failure: bool,
    },
}

#[derive(Debug, Default)]
pub(super) struct StreamedCommandWorkerState {
    active_buffers: BTreeSet<BufferId>,
    cancel_flags: BTreeMap<BufferId, Arc<AtomicBool>>,
    updates: Arc<Mutex<Vec<StreamedCommandUpdate>>>,
}

impl StreamedCommandWorkerState {
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn contains(&self, buffer_id: BufferId) -> bool {
        self.active_buffers.contains(&buffer_id)
    }

    /// Signal cancel for a running buffer's worker thread and remove it from tracking.
    pub(super) fn cancel_and_remove(&mut self, buffer_id: BufferId) {
        if let Some(flag) = self.cancel_flags.remove(&buffer_id) {
            flag.store(true, Ordering::Relaxed);
        }
        self.active_buffers.remove(&buffer_id);
    }

    pub(super) fn remove(&mut self, buffer_id: BufferId) -> bool {
        self.cancel_flags.remove(&buffer_id);
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
        let cancel_flag = Arc::new(AtomicBool::new(false));
        let updates = Arc::clone(&self.updates);
        let cancel = Arc::clone(&cancel_flag);
        self.active_buffers.insert(buffer_id);
        self.cancel_flags.insert(buffer_id, cancel_flag);
        if let Err(error) = thread::Builder::new()
            .name(format!("streamed-command-{buffer_id}"))
            .spawn(move || run_streamed_command(request, updates, cancel))
        {
            self.active_buffers.remove(&buffer_id);
            self.cancel_flags.remove(&buffer_id);
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
    let shell_buffer = ShellBuffer::from_runtime_buffer(buffer, Vec::new(), &*user_library);
    {
        let ui = shell_ui_mut(runtime)?;
        ui.insert_buffer(shell_buffer);
        ui.set_popup_buffer(buffer_id);
        ui.set_popup_focus(true);
        ui.enter_normal_mode();
    }

    if let Err(error) = continue_streamed_command_popup(runtime, buffer_id, spec) {
        close_popup_buffer_and_restore_focus(runtime, buffer_id)?;
        return Err(error);
    }
    Ok(buffer_id)
}

/// Run an External Command: Command Stream by default, Silent Command when `stream` is false.
pub(super) fn run_command(
    runtime: &mut EditorRuntime,
    mut spec: ExternalCommandSpec,
) -> Result<ExternalCommandResult, String> {
    if spec.use_git_editor {
        inject_git_editor_env(runtime, &mut spec.env)?;
    }
    let (program, args, resolved_label) = resolve_external_invocation(runtime, &spec.invocation)?;
    let command_label = spec.command_label.clone().unwrap_or(resolved_label);

    if spec.stream {
        let streamed = StreamedCommandSpec {
            popup_title: spec.popup_title,
            buffer_name: spec.buffer_name,
            command_label,
            program,
            args,
            env: spec.env,
            cwd: spec.cwd,
            on_exit: spec.on_exit,
            notify_on_success: spec.notify_on_success,
            notify_on_failure: spec.notify_on_failure,
        };
        open_streamed_command_popup(runtime, streamed)?;
        return Ok(ExternalCommandResult::Streamed);
    }

    run_silent_external_command(runtime, &spec, &program, &args, &command_label)
}

pub(super) fn resolve_external_invocation(
    runtime: &EditorRuntime,
    invocation: &ExternalCommandInvocation,
) -> Result<(String, Vec<String>, String), String> {
    match invocation {
        ExternalCommandInvocation::Argv { program, args } => {
            let label = if args.is_empty() {
                program.clone()
            } else {
                format!("{program} {}", args.join(" "))
            };
            Ok((program.clone(), args.clone(), label))
        }
        ExternalCommandInvocation::Shell(command) => {
            let terminal_config = shell_user_library(runtime).terminal_config();
            let shell_program = terminal_config.program.clone();
            let mut args = terminal_config.args.clone();
            args.extend(
                shell_command_eval_args(&shell_program)
                    .into_iter()
                    .map(str::to_owned),
            );
            args.push(command.clone());
            Ok((shell_program, args, command.clone()))
        }
    }
}

fn run_silent_external_command(
    runtime: &mut EditorRuntime,
    spec: &ExternalCommandSpec,
    program: &str,
    args: &[String],
    command_label: &str,
) -> Result<ExternalCommandResult, String> {
    let env = enrich_env_with_node_manager(Some(&spec.cwd), spec.env.clone());
    let (program, args) = supervised_command_if_resolved(
        program,
        args,
        &env,
        None,
        ProcessSupervisionMode::Background,
    );
    let mut command = Command::new(&program);
    command
        .args(&args)
        .envs(env)
        .current_dir(&spec.cwd)
        .stdin(Stdio::null());
    configure_background_command(&mut command);
    let output = command
        .output()
        .map_err(|error| format!("failed to start `{command_label}`: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let success = output.status.success();
    let exit_code = output.status.code();
    if (success && spec.notify_on_success) || (!success && spec.notify_on_failure) {
        let error = (!success).then(|| {
            let transcript = if stderr.is_empty() {
                stdout.clone()
            } else if stdout.is_empty() {
                stderr.clone()
            } else {
                format!("{stdout}{stderr}")
            };
            if transcript.trim().is_empty() {
                format!("Exit code: {}", exit_code.unwrap_or(-1))
            } else {
                transcript
            }
        });
        let mut body_lines = vec![command_label.to_owned()];
        if let Some(error) = error.as_deref() {
            body_lines.push(error.to_owned());
        } else if !success && let Some(exit_code) = exit_code {
            body_lines.push(format!("Exit code: {exit_code}"));
        }
        shell_ui_mut(runtime)?.apply_notification(
            NotificationUpdate {
                key: format!("silent-command:{command_label}"),
                severity: if success {
                    NotificationSeverity::Success
                } else {
                    NotificationSeverity::Error
                },
                title: if success {
                    format!("{} succeeded", spec.popup_title)
                } else {
                    format!("{} failed", spec.popup_title)
                },
                body_lines,
                progress: None,
                active: false,
                action: None,
                workspace_id: None,
            },
            Instant::now(),
        );
    }
    if !success {
        let transcript = if stderr.is_empty() {
            stdout.clone()
        } else if stdout.is_empty() {
            stderr.clone()
        } else {
            format!("{stdout}{stderr}")
        };
        let detail = if transcript.trim().is_empty() {
            format!("exit code {}", exit_code.unwrap_or(-1))
        } else {
            transcript
        };
        return Err(format!("{command_label} failed: {detail}"));
    }
    Ok(ExternalCommandResult::Silent)
}

pub(super) fn continue_streamed_command_popup(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    spec: StreamedCommandSpec,
) -> Result<(), String> {
    append_streamed_command_header(runtime, buffer_id, &spec.command_label)?;
    let request = StreamedCommandRequest {
        buffer_id,
        popup_title: spec.popup_title,
        command_label: spec.command_label,
        program: spec.program,
        args: spec.args,
        env: spec.env,
        cwd: spec.cwd,
        on_exit: spec.on_exit,
        notify_on_success: spec.notify_on_success,
        notify_on_failure: spec.notify_on_failure,
    };
    shell_ui_mut(runtime)?
        .streamed_command_worker
        .start(request)
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
    let mut open_commit_after_close = false;
    let mut open_workspace_after_close: Option<(String, PathBuf)> = None;
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
                notify_on_success,
                notify_on_failure,
            } => {
                if !shell_ui_mut(runtime)?
                    .streamed_command_worker
                    .remove(buffer_id)
                {
                    continue;
                }
                if (success && notify_on_success) || (!success && notify_on_failure) {
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
                }
                let outcome = StreamedCommandOutcome {
                    success,
                    exit_code,
                    error,
                };
                match on_exit {
                    StreamedCommandExitAction::RefreshGitStatusBuffersAndCloseBuffer => {
                        if outcome.success {
                            buffers_to_close.push(buffer_id);
                            refresh_git_status = true;
                        }
                    }
                    StreamedCommandExitAction::RefreshGitStatusCloseAndOpenCommitBuffer => {
                        if outcome.success {
                            buffers_to_close.push(buffer_id);
                            refresh_git_status = true;
                            open_commit_after_close = true;
                        }
                    }
                    StreamedCommandExitAction::RefreshGitStatusCloseAndOpenWorkspace {
                        name,
                        path,
                    } => {
                        if outcome.success {
                            buffers_to_close.push(buffer_id);
                            refresh_git_status = true;
                            open_workspace_after_close = Some((name, path));
                        }
                    }
                    StreamedCommandExitAction::ContinueTreeSitterInstall(state) => {
                        if outcome.success
                            && let Err(error) =
                                continue_tree_sitter_install(runtime, buffer_id, *state)
                        {
                            append_streamed_command_error(runtime, buffer_id, &error)?;
                            shell_ui_mut(runtime)?.apply_notification(
                                streamed_command_notification(
                                    buffer_id,
                                    &popup_title,
                                    &command_label,
                                    false,
                                    None,
                                    Some(&error),
                                ),
                                now,
                            );
                        }
                    }
                    StreamedCommandExitAction::ContinueTreeSitterRecompile(state) => {
                        if let Err(error) =
                            continue_tree_sitter_recompile(runtime, buffer_id, *state, outcome)
                        {
                            append_streamed_command_error(runtime, buffer_id, &error)?;
                            shell_ui_mut(runtime)?.apply_notification(
                                streamed_command_notification(
                                    buffer_id,
                                    &popup_title,
                                    &command_label,
                                    false,
                                    None,
                                    Some(&error),
                                ),
                                now,
                            );
                        }
                    }
                    StreamedCommandExitAction::ContinueToolInstall(state) => {
                        if let Err(error) =
                            continue_tool_install(runtime, buffer_id, *state, outcome.success)
                        {
                            append_streamed_command_error(runtime, buffer_id, &error)?;
                            shell_ui_mut(runtime)?.apply_notification(
                                streamed_command_notification(
                                    buffer_id,
                                    &popup_title,
                                    &command_label,
                                    false,
                                    None,
                                    Some(&error),
                                ),
                                now,
                            );
                        }
                    }
                    StreamedCommandExitAction::LeaveOpen => {}
                    StreamedCommandExitAction::LeaveOpenAndMaybeReloadUserLibrary { command } => {
                        if outcome.success && command_builds_user_library(&command) {
                            let reload_lines =
                                match built_user_library_path_for_command(runtime, &command) {
                                    Some(built_path) => {
                                        match stage_user_library_for_reload(&built_path) {
                                            Ok(staged_path) => {
                                                DynamicUserLibrary::load_from_file(&staged_path)
                                                    .and_then(|library| {
                                                        validate_runtime_user_library(
                                                            library.as_ref(),
                                                        )?;
                                                        Ok(library)
                                                    })
                                                    .and_then(|library| {
                                                        if let Some(state) = runtime
                                                            .services_mut()
                                                            .get_mut::<UserLibraryReloadState>(
                                                        ) {
                                                            state.last_staged_path =
                                                                Some(staged_path.clone());
                                                        }
                                                        let mut lines =
                                                            replace_runtime_user_library(
                                                                runtime, library,
                                                            )?;
                                                        lines.push(format!(
                                                        "Loaded runtime library from staged copy \
                                                         `{}`.",
                                                        staged_path.display()
                                                    ));
                                                        Ok(lines)
                                                    })
                                                    .unwrap_or_else(|error| {
                                                        vec![format!(
                                                            "── ✗ User library reload failed: \
                                                             {error}"
                                                        )]
                                                    })
                                            }
                                            Err(error) => vec![format!(
                                                "── ✗ User library staging failed: {error}"
                                            )],
                                        }
                                    }
                                    None => vec![
                                        "── ✗ User library reload failed: could not resolve \
                                         build output path"
                                            .to_owned(),
                                    ],
                                };
                            if let Ok(buf) = shell_buffer_mut(runtime, buffer_id) {
                                buf.append_output_lines(&reload_lines);
                            }
                        }
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
    if open_commit_after_close {
        open_git_commit_buffer(runtime)?;
        changed = true;
    }
    if let Some((name, path)) = open_workspace_after_close {
        open_workspace_from_project(runtime, &name, &path)?;
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
        workspace_id: None,
    }
}

pub(super) fn append_streamed_command_lines(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    lines: &[String],
) -> Result<(), String> {
    shell_buffer_mut(runtime, buffer_id)?.append_output_lines(lines);
    Ok(())
}

fn append_streamed_command_header(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    command_label: &str,
) -> Result<(), String> {
    let has_output = shell_buffer(runtime, buffer_id)
        .map(|buffer| buffer.line_count() > 0)
        .unwrap_or(false);
    let mut lines = Vec::with_capacity(if has_output { 3 } else { 2 });
    if has_output {
        lines.push(String::new());
    }
    lines.push(format!("$ {command_label}"));
    lines.push(String::new());
    shell_buffer_mut(runtime, buffer_id)?.append_output_lines(&lines);
    Ok(())
}

fn append_streamed_command_error(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    error: &str,
) -> Result<(), String> {
    shell_buffer_mut(runtime, buffer_id)?
        .append_output_lines(&[String::new(), format!("error: {error}")]);
    Ok(())
}

fn run_streamed_command(
    request: StreamedCommandRequest,
    updates: Arc<Mutex<Vec<StreamedCommandUpdate>>>,
    cancel: Arc<AtomicBool>,
) {
    let StreamedCommandRequest {
        buffer_id,
        popup_title,
        command_label,
        program,
        args,
        env,
        cwd,
        on_exit,
        notify_on_success,
        notify_on_failure,
    } = request;
    let env = enrich_env_with_node_manager(Some(&cwd), env);
    let (program, args) = supervised_command_if_resolved(
        &program,
        &args,
        &env,
        None,
        ProcessSupervisionMode::Background,
    );
    let mut command = Command::new(&program);
    command
        .args(&args)
        .envs(env)
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
                    notify_on_success,
                    notify_on_failure,
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

    let status = loop {
        if cancel.load(Ordering::Relaxed) {
            let _ = child.kill();
            let _ = child.wait();
            if let Some(reader) = stdout_reader {
                let _ = reader.join();
            }
            if let Some(reader) = stderr_reader {
                let _ = reader.join();
            }
            return;
        }
        match child.try_wait() {
            Ok(Some(status)) => break Ok(status),
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(50)),
            Err(error) => break Err(error),
        }
    };
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
                notify_on_success,
                notify_on_failure,
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
                notify_on_success,
                notify_on_failure,
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

/// Detect a build command from marker files at the top level of `dir`.
///
/// Priority order: `Cargo.toml` → `cargo build`, `*.sln`/`*.csproj` →
/// `dotnet build`, `package.json` → `npm run build`, `Makefile` → `make`.
/// Returns an empty string if no marker is found. Detection is shallow (no
/// recursion into sub-directories).
pub(super) fn detect_build_command(dir: &std::path::Path) -> String {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return String::new(),
    };
    let mut has_dotnet = false;
    let mut has_npm = false;
    let mut has_makefile = false;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name == "Cargo.toml" {
            return "cargo build".to_owned();
        }
        if name.ends_with(".sln") || name.ends_with(".csproj") {
            has_dotnet = true;
        } else if name == "package.json" {
            has_npm = true;
        } else if name == "Makefile" {
            has_makefile = true;
        }
    }
    if has_dotnet {
        "dotnet build".to_owned()
    } else if has_npm {
        "npm run build".to_owned()
    } else if has_makefile {
        "make".to_owned()
    } else {
        String::new()
    }
}

fn push_streamed_command_update(
    updates: &Arc<Mutex<Vec<StreamedCommandUpdate>>>,
    update: StreamedCommandUpdate,
) {
    if let Ok(mut updates) = updates.lock() {
        updates.push(update);
        ping_shell_wakeup();
    }
}

#[cfg(test)]
mod tests {
    use super::super::shell_command_eval_args;
    use super::detect_build_command;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn nushell_command_eval_uses_login_flag() {
        assert_eq!(shell_command_eval_args("nu"), vec!["-l", "-c"]);
        assert_eq!(
            shell_command_eval_args(r"C:\Program Files\nu\bin\nu.exe"),
            vec!["-l", "-c"]
        );
    }

    #[test]
    fn windows_shell_command_eval_flags_unchanged() {
        if !cfg!(windows) {
            return;
        }
        assert_eq!(shell_command_eval_args("cmd"), vec!["/C"]);
        assert_eq!(shell_command_eval_args("cmd.exe"), vec!["/C"]);
        assert_eq!(shell_command_eval_args("powershell"), vec!["-Command"]);
        assert_eq!(shell_command_eval_args("pwsh.exe"), vec!["-Command"]);
    }

    struct TempDir {
        path: std::path::PathBuf,
    }

    impl TempDir {
        fn new(tag: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("volt-detect-build-{tag}-{unique}"));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn detects_cargo_toml() {
        let dir = TempDir::new("cargo");
        fs::write(dir.path.join("Cargo.toml"), "").expect("write");
        assert_eq!(detect_build_command(&dir.path), "cargo build");
    }

    #[test]
    fn detects_sln() {
        let dir = TempDir::new("sln");
        fs::write(dir.path.join("MyApp.sln"), "").expect("write");
        assert_eq!(detect_build_command(&dir.path), "dotnet build");
    }

    #[test]
    fn detects_csproj() {
        let dir = TempDir::new("csproj");
        fs::write(dir.path.join("MyApp.csproj"), "").expect("write");
        assert_eq!(detect_build_command(&dir.path), "dotnet build");
    }

    #[test]
    fn detects_package_json() {
        let dir = TempDir::new("npm");
        fs::write(dir.path.join("package.json"), "{}").expect("write");
        assert_eq!(detect_build_command(&dir.path), "npm run build");
    }

    #[test]
    fn detects_makefile() {
        let dir = TempDir::new("make");
        fs::write(dir.path.join("Makefile"), "").expect("write");
        assert_eq!(detect_build_command(&dir.path), "make");
    }

    #[test]
    fn empty_dir_returns_empty_string() {
        let dir = TempDir::new("empty");
        assert_eq!(detect_build_command(&dir.path), "");
    }

    #[test]
    fn cargo_toml_wins_over_other_markers() {
        let dir = TempDir::new("priority");
        fs::write(dir.path.join("Cargo.toml"), "").expect("write");
        fs::write(dir.path.join("package.json"), "{}").expect("write");
        fs::write(dir.path.join("Makefile"), "").expect("write");
        assert_eq!(detect_build_command(&dir.path), "cargo build");
    }

    #[test]
    fn missing_dir_returns_empty_string() {
        let path = std::path::Path::new("/nonexistent/volt/test/dir");
        assert_eq!(detect_build_command(path), "");
    }
}
