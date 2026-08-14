use super::{
    command_stream::{
        StreamedCommandExitAction, StreamedCommandOutcome, StreamedCommandSpec,
        append_streamed_command_lines, continue_streamed_command_popup,
        open_streamed_command_popup,
    },
    *,
};

use editor_syntax::{InstallCommandSpec, LanguageInstallPlan};
use std::collections::VecDeque;

const TREE_SITTER_INSTALL_POPUP_TITLE: &str = "Tree-sitter Install";
const TREE_SITTER_RECOMPILE_POPUP_TITLE: &str = "Tree-sitter Recompile";
const TREE_SITTER_RECOMPILE_NOTIFICATION_KEY: &str = "treesitter.recompile-installed";

#[derive(Debug)]
pub(super) enum TreeSitterInstallPhase {
    Clone,
    Generate,
    Compile,
}

#[derive(Debug)]
pub(super) struct TreeSitterInstallState {
    phase: TreeSitterInstallPhase,
    plan: LanguageInstallPlan,
}

impl TreeSitterInstallState {
    fn new(plan: LanguageInstallPlan) -> Self {
        Self {
            phase: TreeSitterInstallPhase::Clone,
            plan,
        }
    }
}

#[derive(Debug)]
pub(super) struct TreeSitterRecompileState {
    phase: TreeSitterInstallPhase,
    plan: LanguageInstallPlan,
    pending_language_ids: VecDeque<String>,
    recompiled: Vec<String>,
    failed: Vec<(String, String)>,
}

impl TreeSitterRecompileState {
    fn new(plan: LanguageInstallPlan, pending_language_ids: VecDeque<String>) -> Self {
        Self {
            phase: TreeSitterInstallPhase::Clone,
            plan,
            pending_language_ids,
            recompiled: Vec::new(),
            failed: Vec::new(),
        }
    }

    fn record_failure(&mut self, message: String) {
        self.failed
            .push((self.plan.language_id().to_owned(), message));
    }
}

pub(super) fn install_tree_sitter_language(
    runtime: &mut EditorRuntime,
    language_id: &str,
) -> Result<(), String> {
    let install_plan = syntax_registry_mut(runtime)?
        .prepare_language_install(language_id)
        .map_err(|error| error.to_string())?;
    let Some(install_plan) = install_plan else {
        return refresh_workspace_syntax(runtime);
    };
    install_plan
        .prepare_clone_root()
        .map_err(|error| error.to_string())?;
    let clone_command = install_plan.clone_command();
    open_streamed_command_popup(
        runtime,
        StreamedCommandSpec {
            popup_title: TREE_SITTER_INSTALL_POPUP_TITLE.to_owned(),
            buffer_name: tree_sitter_install_buffer_name(language_id),
            command_label: clone_command.label().to_owned(),
            program: clone_command.program().to_owned(),
            args: clone_command.args().to_vec(),
            env: clone_command.env().to_vec(),
            cwd: clone_command.cwd().to_path_buf(),
            on_exit: StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(
                TreeSitterInstallState::new(install_plan),
            )),
            notify_on_success: false,
            notify_on_failure: true,
        },
    )?;
    Ok(())
}

pub(super) fn recompile_installed_tree_sitter_languages(
    runtime: &mut EditorRuntime,
) -> Result<(), String> {
    let mut pending_language_ids = {
        let registry = syntax_registry_mut(runtime)?;
        registry
            .installed_grammar_language_ids()
            .into_iter()
            .collect::<VecDeque<_>>()
    };
    if pending_language_ids.is_empty() {
        apply_tree_sitter_recompile_notification(runtime, 0, &[])?;
        return Ok(());
    }
    let language_id = pending_language_ids
        .pop_front()
        .ok_or_else(|| "installed Tree-sitter language list was unexpectedly empty".to_owned())?;
    let plan = syntax_registry_mut(runtime)?
        .prepare_language_install(&language_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("language `{language_id}` does not have installable grammar"))?;
    plan.prepare_clone_root()
        .map_err(|error| error.to_string())?;
    let clone_command = plan.clone_command();
    open_streamed_command_popup(
        runtime,
        StreamedCommandSpec {
            popup_title: TREE_SITTER_RECOMPILE_POPUP_TITLE.to_owned(),
            buffer_name: tree_sitter_recompile_buffer_name(),
            command_label: clone_command.label().to_owned(),
            program: clone_command.program().to_owned(),
            args: clone_command.args().to_vec(),
            env: clone_command.env().to_vec(),
            cwd: clone_command.cwd().to_path_buf(),
            on_exit: StreamedCommandExitAction::ContinueTreeSitterRecompile(Box::new(
                TreeSitterRecompileState::new(plan, pending_language_ids),
            )),
            notify_on_success: false,
            notify_on_failure: false,
        },
    )?;
    Ok(())
}

pub(super) fn continue_tree_sitter_install(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    state: TreeSitterInstallState,
) -> Result<(), String> {
    match state.phase {
        TreeSitterInstallPhase::Clone => {
            continue_tree_sitter_install_after_clone(runtime, buffer_id, state)
        }
        TreeSitterInstallPhase::Generate => {
            continue_tree_sitter_install_after_generate(runtime, buffer_id, state)
        }
        TreeSitterInstallPhase::Compile => finish_tree_sitter_install(runtime, buffer_id, state),
    }
}

pub(super) fn continue_tree_sitter_recompile(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    state: TreeSitterRecompileState,
    outcome: StreamedCommandOutcome,
) -> Result<(), String> {
    let mut state = state;
    if !outcome.success {
        state.record_failure(tree_sitter_recompile_failure_message(&outcome));
        return continue_next_tree_sitter_recompile(runtime, buffer_id, state);
    }
    match state.phase {
        TreeSitterInstallPhase::Clone => {
            continue_tree_sitter_recompile_after_clone(runtime, buffer_id, state)
        }
        TreeSitterInstallPhase::Generate => {
            continue_tree_sitter_recompile_after_generate(runtime, buffer_id, state)
        }
        TreeSitterInstallPhase::Compile => {
            finish_tree_sitter_recompile_language(runtime, buffer_id, state)
        }
    }
}

fn continue_tree_sitter_install_after_clone(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mut state: TreeSitterInstallState,
) -> Result<(), String> {
    state
        .plan
        .prepare_install_root()
        .map_err(|error| error.to_string())?;
    if let Some(generate_command) = state
        .plan
        .generate_command_if_needed()
        .map_err(|error| error.to_string())?
    {
        state.phase = TreeSitterInstallPhase::Generate;
        return continue_streamed_command_popup(
            runtime,
            buffer_id,
            streamed_tree_sitter_command_spec(
                TREE_SITTER_INSTALL_POPUP_TITLE,
                &generate_command,
                StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(state)),
                false,
                true,
            ),
        );
    }

    let compile_command = state
        .plan
        .compile_command()
        .map_err(|error| error.to_string())?;
    state.phase = TreeSitterInstallPhase::Compile;
    continue_streamed_command_popup(
        runtime,
        buffer_id,
        streamed_tree_sitter_command_spec(
            TREE_SITTER_INSTALL_POPUP_TITLE,
            &compile_command,
            StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(state)),
            true,
            true,
        ),
    )
}

fn continue_tree_sitter_install_after_generate(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mut state: TreeSitterInstallState,
) -> Result<(), String> {
    let compile_command = state
        .plan
        .compile_command()
        .map_err(|error| error.to_string())?;
    state.phase = TreeSitterInstallPhase::Compile;
    continue_streamed_command_popup(
        runtime,
        buffer_id,
        streamed_tree_sitter_command_spec(
            TREE_SITTER_INSTALL_POPUP_TITLE,
            &compile_command,
            StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(state)),
            true,
            true,
        ),
    )
}

fn finish_tree_sitter_install(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    state: TreeSitterInstallState,
) -> Result<(), String> {
    syntax_registry_mut(runtime)?
        .finalize_language_install(&state.plan)
        .map_err(|error| error.to_string())?;
    refresh_workspace_syntax(runtime)?;
    close_popup_buffer_and_restore_focus(runtime, buffer_id)
}

fn continue_tree_sitter_recompile_after_clone(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mut state: TreeSitterRecompileState,
) -> Result<(), String> {
    if let Err(error) = state.plan.prepare_install_root() {
        state.record_failure(error.to_string());
        return continue_next_tree_sitter_recompile(runtime, buffer_id, state);
    }
    let generate_command = match state.plan.generate_command_if_needed() {
        Ok(command) => command,
        Err(error) => {
            state.record_failure(error.to_string());
            return continue_next_tree_sitter_recompile(runtime, buffer_id, state);
        }
    };
    if let Some(generate_command) = generate_command {
        state.phase = TreeSitterInstallPhase::Generate;
        return continue_streamed_command_popup(
            runtime,
            buffer_id,
            streamed_tree_sitter_command_spec(
                TREE_SITTER_RECOMPILE_POPUP_TITLE,
                &generate_command,
                StreamedCommandExitAction::ContinueTreeSitterRecompile(Box::new(state)),
                false,
                false,
            ),
        );
    }

    let compile_command = match state.plan.compile_command() {
        Ok(command) => command,
        Err(error) => {
            state.record_failure(error.to_string());
            return continue_next_tree_sitter_recompile(runtime, buffer_id, state);
        }
    };
    state.phase = TreeSitterInstallPhase::Compile;
    continue_streamed_command_popup(
        runtime,
        buffer_id,
        streamed_tree_sitter_command_spec(
            TREE_SITTER_RECOMPILE_POPUP_TITLE,
            &compile_command,
            StreamedCommandExitAction::ContinueTreeSitterRecompile(Box::new(state)),
            false,
            false,
        ),
    )
}

fn continue_tree_sitter_recompile_after_generate(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mut state: TreeSitterRecompileState,
) -> Result<(), String> {
    let compile_command = match state.plan.compile_command() {
        Ok(command) => command,
        Err(error) => {
            state.record_failure(error.to_string());
            return continue_next_tree_sitter_recompile(runtime, buffer_id, state);
        }
    };
    state.phase = TreeSitterInstallPhase::Compile;
    continue_streamed_command_popup(
        runtime,
        buffer_id,
        streamed_tree_sitter_command_spec(
            TREE_SITTER_RECOMPILE_POPUP_TITLE,
            &compile_command,
            StreamedCommandExitAction::ContinueTreeSitterRecompile(Box::new(state)),
            false,
            false,
        ),
    )
}

fn finish_tree_sitter_recompile_language(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mut state: TreeSitterRecompileState,
) -> Result<(), String> {
    if let Err(error) = syntax_registry_mut(runtime)?
        .finalize_language_install(&state.plan)
        .map_err(|error| error.to_string())
    {
        state.record_failure(error);
        return continue_next_tree_sitter_recompile(runtime, buffer_id, state);
    }
    state.recompiled.push(state.plan.language_id().to_owned());
    continue_next_tree_sitter_recompile(runtime, buffer_id, state)
}

fn continue_next_tree_sitter_recompile(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    mut state: TreeSitterRecompileState,
) -> Result<(), String> {
    let Some(language_id) = state.pending_language_ids.pop_front() else {
        return finish_tree_sitter_recompile(runtime, buffer_id, state);
    };
    let plan = syntax_registry_mut(runtime)?
        .prepare_language_install(&language_id)
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("language `{language_id}` does not have installable grammar"))?;
    if let Err(error) = plan.prepare_clone_root() {
        state.failed.push((language_id, error.to_string()));
        return continue_next_tree_sitter_recompile(runtime, buffer_id, state);
    }
    let clone_command = plan.clone_command();
    let next_state = TreeSitterRecompileState {
        phase: TreeSitterInstallPhase::Clone,
        plan,
        pending_language_ids: state.pending_language_ids,
        recompiled: state.recompiled,
        failed: state.failed,
    };
    continue_streamed_command_popup(
        runtime,
        buffer_id,
        streamed_tree_sitter_command_spec(
            TREE_SITTER_RECOMPILE_POPUP_TITLE,
            &clone_command,
            StreamedCommandExitAction::ContinueTreeSitterRecompile(Box::new(next_state)),
            false,
            false,
        ),
    )
}

fn finish_tree_sitter_recompile(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    state: TreeSitterRecompileState,
) -> Result<(), String> {
    configure_syntax_refresh_worker(runtime)?;
    refresh_workspace_syntax(runtime)?;

    let mut summary_lines = if state.recompiled.is_empty() && state.failed.is_empty() {
        vec![
            String::new(),
            "No installed Tree-sitter grammars found.".to_owned(),
        ]
    } else {
        vec![
            String::new(),
            format!(
                "Recompiled {} installed Tree-sitter grammar{}.",
                state.recompiled.len(),
                if state.recompiled.len() == 1 { "" } else { "s" }
            ),
        ]
    };
    if !state.failed.is_empty() {
        summary_lines.push(format!(
            "Failed to recompile {} grammar{}.",
            state.failed.len(),
            if state.failed.len() == 1 { "" } else { "s" }
        ));
        for (language_id, message) in state.failed.iter().take(3) {
            summary_lines.push(format!("{language_id}: {message}"));
        }
    }
    append_streamed_command_lines(runtime, buffer_id, &summary_lines)?;
    apply_tree_sitter_recompile_notification(runtime, state.recompiled.len(), &state.failed)?;
    if state.failed.is_empty() {
        close_popup_buffer_and_restore_focus(runtime, buffer_id)?;
    }
    Ok(())
}

fn apply_tree_sitter_recompile_notification(
    runtime: &mut EditorRuntime,
    recompiled_count: usize,
    failed: &[(String, String)],
) -> Result<(), String> {
    let mut body_lines = if recompiled_count == 0 && failed.is_empty() {
        vec!["No installed Tree-sitter grammars found.".to_owned()]
    } else {
        vec![format!(
            "Recompiled {} installed Tree-sitter grammar{}.",
            recompiled_count,
            if recompiled_count == 1 { "" } else { "s" }
        )]
    };
    if !failed.is_empty() {
        body_lines.push(format!(
            "Failed to recompile {} grammar{}.",
            failed.len(),
            if failed.len() == 1 { "" } else { "s" }
        ));
        for (language_id, message) in failed.iter().take(3) {
            body_lines.push(format!("{language_id}: {message}"));
        }
    }
    shell_ui_mut(runtime)?.apply_notification(
        NotificationUpdate {
            key: TREE_SITTER_RECOMPILE_NOTIFICATION_KEY.to_owned(),
            severity: if failed.is_empty() {
                NotificationSeverity::Success
            } else {
                NotificationSeverity::Warning
            },
            title: if failed.is_empty() {
                "Tree-sitter recompile complete".to_owned()
            } else {
                "Tree-sitter recompile partially complete".to_owned()
            },
            body_lines,
            progress: None,
            active: false,
            action: None,
            workspace_id: None,
        },
        Instant::now(),
    );
    Ok(())
}

fn tree_sitter_install_buffer_name(language_id: &str) -> String {
    format!("*treesitter.install {language_id}*")
}

fn tree_sitter_recompile_buffer_name() -> String {
    "*treesitter.recompile-installed*".to_owned()
}

fn tree_sitter_recompile_failure_message(outcome: &StreamedCommandOutcome) -> String {
    if let Some(error) = outcome.error.as_deref() {
        return error.to_owned();
    }
    if let Some(exit_code) = outcome.exit_code {
        return format!("command exited with status {exit_code}");
    }
    "command failed".to_owned()
}

fn streamed_tree_sitter_command_spec(
    popup_title: &str,
    command: &InstallCommandSpec,
    on_exit: StreamedCommandExitAction,
    notify_on_success: bool,
    notify_on_failure: bool,
) -> StreamedCommandSpec {
    StreamedCommandSpec {
        popup_title: popup_title.to_owned(),
        buffer_name: String::new(),
        command_label: command.label().to_owned(),
        program: command.program().to_owned(),
        args: command.args().to_vec(),
        env: command.env().to_vec(),
        cwd: command.cwd().to_path_buf(),
        on_exit,
        notify_on_success,
        notify_on_failure,
    }
}
