use super::{
    command_stream::{
        StreamedCommandExitAction, StreamedCommandSpec, continue_streamed_command_popup,
        open_streamed_command_popup,
    },
    *,
};

use editor_syntax::{InstallCommandSpec, LanguageInstallPlan};

const TREE_SITTER_INSTALL_POPUP_TITLE: &str = "Tree-sitter Install";

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
            cwd: clone_command.cwd().to_path_buf(),
            on_exit: StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(
                TreeSitterInstallState::new(install_plan),
            )),
            notify_on_success: false,
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
                &generate_command,
                StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(state)),
                false,
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
            &compile_command,
            StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(state)),
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
            &compile_command,
            StreamedCommandExitAction::ContinueTreeSitterInstall(Box::new(state)),
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
        .invalidate_language(state.plan.language_id())
        .map_err(|error| error.to_string())?;
    refresh_workspace_syntax(runtime)?;
    close_popup_buffer_and_restore_focus(runtime, buffer_id)
}

fn tree_sitter_install_buffer_name(language_id: &str) -> String {
    format!("*treesitter.install {language_id}*")
}

fn streamed_tree_sitter_command_spec(
    command: &InstallCommandSpec,
    on_exit: StreamedCommandExitAction,
    notify_on_success: bool,
) -> StreamedCommandSpec {
    StreamedCommandSpec {
        popup_title: TREE_SITTER_INSTALL_POPUP_TITLE.to_owned(),
        buffer_name: String::new(),
        command_label: command.label().to_owned(),
        program: command.program().to_owned(),
        args: command.args().to_vec(),
        cwd: command.cwd().to_path_buf(),
        on_exit,
        notify_on_success,
    }
}
