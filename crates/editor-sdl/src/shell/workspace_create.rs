fn begin_workspace_create(runtime: &mut EditorRuntime) -> Result<(), String> {
    let start_root = first_project_discovery_root(runtime)?;
    shell_ui_mut(runtime)?.pending_workspace_create = true;
    open_oil_directory(runtime, start_root)
}

fn pending_workspace_create_dir(
    runtime: &EditorRuntime,
    root: &Path,
    actions: &[DirectoryEditAction],
) -> Result<Option<PathBuf>, String> {
    if !shell_ui(runtime)?.pending_workspace_create {
        return Ok(None);
    }
    let mut created_dirs = actions.iter().filter_map(|action| match action {
        DirectoryEditAction::CreateDir(path) => Some(path.clone()),
        _ => None,
    });
    let Some(path) = created_dirs.next() else {
        return Ok(None);
    };
    if created_dirs.next().is_some() || actions.len() != 1 {
        return Ok(None);
    }
    if is_oil_volume_list_root(root) {
        return Err("navigate into a drive before naming the new workspace directory".to_owned());
    }
    Ok(Some(path))
}

fn run_workspace_git_init(runtime: &mut EditorRuntime, path: &Path) -> Result<(), String> {
    if !path.is_dir() {
        return Err(format!(
            "new workspace directory is missing: {}",
            path.display()
        ));
    }
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or("workspace")
        .to_owned();
    run_command(
        runtime,
        ExternalCommandSpec::git_argv(
            "Git Init",
            vec!["init".to_owned()],
            path.to_path_buf(),
            StreamedCommandExitAction::RefreshGitStatusCloseAndOpenWorkspace {
                name,
                path: path.to_path_buf(),
            },
        ),
    )?;
    Ok(())
}
