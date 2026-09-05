#[test]
fn find_workspace_by_root_matches_normalized_path_identity() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let project_root = unique_temp_dir("workspace-root-identity");
    let workspace_id =
        open_workspace_from_project(&mut state.runtime, "identity-project", &project_root)?;
    let verbatim = PathBuf::from(format!(r"\\?\{}", project_root.display()));

    assert_eq!(
        find_workspace_by_root(&state.runtime, &verbatim)?,
        Some(workspace_id)
    );

    std::fs::remove_dir_all(&project_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn mark_list_load_canonicalizes_existing_roots_and_keeps_missing_as_written() -> Result<(), String>
{
    let state_dir = unique_temp_dir("workspace-mark-load-normalize");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    let existing = unique_temp_dir("workspace-mark-load-existing");
    let missing = state_dir.join("missing-on-disk");
    let verbatim_existing = PathBuf::from(format!(r"\\?\{}", existing.display()));
    std::fs::write(
        &mark_list_path,
        format!("{}\n{}\n", verbatim_existing.display(), missing.display()),
    )
    .map_err(|error| error.to_string())?;

    let loaded = MarkListState::load(mark_list_path)?;
    assert_eq!(
        loaded.list.roots(),
        &[canonicalize_project_root_path(&existing), missing.clone(),]
    );
    assert!(
        !loaded.list.roots()[0].to_string_lossy().contains(r"\\?\"),
        "existing Mark List roots must strip verbatim prefixes"
    );

    let missing_verbatim =
        PathBuf::from(format!(r"\\?\{}", state_dir.join("also-missing").display()));
    let with_missing_verbatim =
        mark_list_from_persisted_text(&format!("{}\n", missing_verbatim.display()));
    assert_eq!(
        with_missing_verbatim.roots(),
        &[missing_verbatim],
        "missing paths must stay as-written when canonicalize cannot run"
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&existing).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_mark_refreshes_clean_open_mark_list_buffer() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-mark-open-list");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;
    let project_root = unique_temp_dir("workspace-mark-open-project");
    let canonical_root = canonicalize_project_root_path(&project_root);
    open_workspace_from_project(&mut state.runtime, "marked-project", &project_root)?;
    state
        .runtime
        .execute_command("workspace.marks")
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.text.text(),
        format!("{}\n", canonical_root.display())
    );
    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?,
        format!("{}\n", canonical_root.display())
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&project_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_mark_on_default_workspace_notifies_without_mutating_list() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-mark-default");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    state
        .runtime
        .execute_command("workspace.mark")
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .execute_command("workspace.unmark")
        .map_err(|error| error.to_string())?;

    assert!(!mark_list_path.exists());
    assert!(mark_list_state(&state.runtime)?.list.roots().is_empty());
    let notifications = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
    assert!(
        notifications
            .iter()
            .any(|notification| notification.title == "Default Workspace has no project root")
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_marks_opens_real_file_and_save_reloads_normalized_list() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-marks-open");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    std::fs::write(&mark_list_path, "P:\\alpha\n").map_err(|error| error.to_string())?;
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    state
        .runtime
        .execute_command("workspace.marks")
        .map_err(|error| error.to_string())?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(mark_list_path.as_path())
    );
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let end = buffer.text.point_from_char_index(buffer.text.char_count());
        buffer.replace_range(
            TextRange::new(TextPoint::default(), end),
            "P:\\beta\n\n  \nP:\\gamma\n",
        );
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&mark_list_path).map_err(|error| error.to_string())?,
        "P:\\beta\nP:\\gamma\n"
    );
    assert_eq!(
        mark_list_state(&state.runtime)?.list.roots(),
        &[PathBuf::from(r"P:\beta"), PathBuf::from(r"P:\gamma")]
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_marked_slot_jump_switches_open_opens_closed_and_handles_empty_missing()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let state_dir = unique_temp_dir("workspace-marked-jump-state");
    let mark_list_path = state_dir.join(MARK_LIST_FILE_NAME);
    install_mark_list_state_for_test(&mut state.runtime, mark_list_path.clone())?;

    let open_root = unique_temp_dir("workspace-marked-jump-open");
    let closed_root = unique_temp_dir("workspace-marked-jump-closed");
    let missing_root = state_dir.join("missing-marked-workspace");
    open_workspace_from_project(&mut state.runtime, "open-project", &open_root)?;
    let open_workspace_id = shell_ui(&state.runtime)?.active_workspace();

    {
        let list = &mut mark_list_state_mut(&mut state.runtime)?.list;
        assert!(list.mark(&open_root));
        assert!(list.mark(&closed_root));
        assert!(list.mark(&missing_root));
    }
    persist_mark_list(mark_list_state(&state.runtime)?)?;

    let default_workspace = shell_ui(&state.runtime)?.default_workspace();
    switch_runtime_workspace(&mut state.runtime, default_workspace)?;
    assert_ne!(
        shell_ui(&state.runtime)?.active_workspace(),
        open_workspace_id
    );

    state
        .runtime
        .execute_command("workspace.marked-1")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        open_workspace_id
    );

    state
        .runtime
        .execute_command("workspace.marked-2")
        .map_err(|error| error.to_string())?;
    let closed_workspace_id = shell_ui(&state.runtime)?.active_workspace();
    assert_ne!(closed_workspace_id, open_workspace_id);
    assert_ne!(closed_workspace_id, default_workspace);
    assert_eq!(
        state
            .runtime
            .model()
            .workspace(closed_workspace_id)
            .map_err(|error| error.to_string())?
            .root(),
        Some(closed_root.as_path())
    );

    let before_missing = mark_list_state(&state.runtime)?.list.roots().to_vec();
    state
        .runtime
        .execute_command("workspace.marked-3")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        mark_list_state(&state.runtime)?.list.roots(),
        before_missing.as_slice()
    );
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        closed_workspace_id
    );
    let notifications = shell_ui(&state.runtime)?.visible_notifications(Instant::now());
    assert!(
        notifications
            .iter()
            .any(|notification| notification.title == "Marked Workspace path missing")
    );

    switch_runtime_workspace(&mut state.runtime, open_workspace_id)?;
    state
        .runtime
        .execute_command("workspace.marked-4")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        open_workspace_id
    );

    std::fs::remove_dir_all(&state_dir).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&open_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&closed_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_save_command_writes_all_dirty_workspace_files() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("workspace-save-command");
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    std::fs::write(&first, "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "beta\n").map_err(|error| error.to_string())?;

    let first_buffer_id = open_workspace_file(&mut state.runtime, &first)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
        assert!(buffer.is_dirty());
    }

    let second_buffer_id = open_workspace_file(&mut state.runtime, &second)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, second_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("two\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("workspace.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );
    assert_eq!(
        std::fs::read_to_string(&second).map_err(|error| error.to_string())?,
        "two\nbeta\n"
    );
    assert!(!shell_buffer(&state.runtime, first_buffer_id)?.is_dirty());
    assert!(!shell_buffer(&state.runtime, second_buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale() -> Result<(), String>
{
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("buffer-save-stale-focus");
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    std::fs::write(&first, "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "beta\n").map_err(|error| error.to_string())?;

    let first_buffer_id = open_workspace_file(&mut state.runtime, &first)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_buffer_id = open_workspace_file(&mut state.runtime, &second)?;
    assert_ne!(first_buffer_id, second_buffer_id);

    shell_ui_mut(&mut state.runtime)?.focus_buffer(first_buffer_id);
    assert_eq!(
        shell_ui(&state.runtime)?.active_buffer_id(),
        Some(first_buffer_id)
    );

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_hook_prefers_explicit_event_buffer_over_shell_focus() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("buffer-save-explicit-buffer");
    let first = root.join("first.txt");
    let second = root.join("second.txt");
    std::fs::write(&first, "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(&second, "beta\n").map_err(|error| error.to_string())?;

    let first_buffer_id = open_workspace_file(&mut state.runtime, &first)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_buffer_id = open_workspace_file(&mut state.runtime, &second)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, second_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("two\n");
    }

    shell_ui_mut(&mut state.runtime)?.focus_buffer(second_buffer_id);
    let workspace_id = shell_ui(&state.runtime)?.active_workspace();

    state
        .runtime
        .emit_hook(
            HOOK_BUFFER_SAVE,
            HookEvent::new()
                .with_workspace(workspace_id)
                .with_buffer(first_buffer_id),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );
    assert_eq!(
        std::fs::read_to_string(&second).map_err(|error| error.to_string())?,
        "beta\n"
    );
    assert!(!shell_buffer(&state.runtime, first_buffer_id)?.is_dirty());
    assert!(shell_buffer(&state.runtime, second_buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_save_command_uses_shell_active_workspace_when_runtime_workspace_is_stale()
-> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-save-stale-a");
    let second_root = unique_temp_dir("workspace-save-stale-b");
    let first_workspace = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let first_path = first_root.join("alpha.txt");
    std::fs::write(&first_path, "alpha\n").map_err(|error| error.to_string())?;
    let first_buffer_id = open_workspace_file(&mut state.runtime, &first_path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_workspace = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    assert_ne!(first_workspace, second_workspace);
    shell_ui_mut(&mut state.runtime)?.switch_workspace(first_workspace);
    assert_eq!(
        shell_ui(&state.runtime)?.active_workspace(),
        first_workspace
    );

    state
        .runtime
        .execute_command("workspace.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first_path).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );

    std::fs::remove_dir_all(&first_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&second_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn workspace_save_hook_prefers_explicit_event_workspace_over_shell_focus() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("workspace-save-explicit-a");
    let second_root = unique_temp_dir("workspace-save-explicit-b");

    let first_workspace = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    let first_path = first_root.join("alpha.txt");
    std::fs::write(&first_path, "alpha\n").map_err(|error| error.to_string())?;
    let first_buffer_id = open_workspace_file(&mut state.runtime, &first_path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, first_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("one\n");
    }

    let second_workspace = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    let second_path = second_root.join("beta.txt");
    std::fs::write(&second_path, "beta\n").map_err(|error| error.to_string())?;
    let second_buffer_id = open_workspace_file(&mut state.runtime, &second_path)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, second_buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("two\n");
    }

    shell_ui_mut(&mut state.runtime)?.switch_workspace(second_workspace);

    state
        .runtime
        .emit_hook(
            HOOK_WORKSPACE_SAVE,
            HookEvent::new().with_workspace(first_workspace),
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&first_path).map_err(|error| error.to_string())?,
        "one\nalpha\n"
    );
    assert_eq!(
        std::fs::read_to_string(&second_path).map_err(|error| error.to_string())?,
        "beta\n"
    );
    assert!(!shell_buffer(&state.runtime, first_buffer_id)?.is_dirty());
    assert!(shell_buffer(&state.runtime, second_buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&first_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&second_root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn picker_open_file_save_clears_dirty_state_and_closes_cleanly() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("picker-open-file-save");
    let path = root.join("sample.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "picker-save", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Files",
        vec![PickerEntry {
            item: PickerItem::new(
                path.display().to_string(),
                "sample.rs",
                "workspace root",
                Some(path.display().to_string()),
            ),
            action: PickerAction::OpenFile(path.clone()),
            quickfix: None,
        }],
    ));

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(path.as_path())
    );

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    close_buffer_with_prompt(&mut state.runtime, buffer_id)?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(shell_ui(&state.runtime)?.buffer(buffer_id).is_none());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn picker_open_file_location_save_clears_dirty_state_and_closes_cleanly() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("picker-open-location-save");
    let path = root.join("mod.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "picker-location-save", &root)?;

    shell_ui_mut(&mut state.runtime)?.set_picker(PickerOverlay::from_entries(
        "Workspace Search",
        vec![PickerEntry {
            item: PickerItem::new(
                format!("{}:1:1", path.display()),
                "fn main() {}",
                "mod.rs | Ln 1, Col 1",
                Some(path.display().to_string()),
            ),
            action: PickerAction::OpenFileLocation {
                path: path.clone(),
                target: TextPoint::new(0, 0),
            },
            quickfix: None,
        }],
    ));

    state
        .runtime
        .execute_command("picker.submit")
        .map_err(|error| error.to_string())?;

    let buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.path(),
        Some(path.as_path())
    );

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    close_buffer_with_prompt(&mut state.runtime, buffer_id)?;
    assert!(shell_ui(&state.runtime)?.picker().is_none());
    assert!(shell_ui(&state.runtime)?.buffer(buffer_id).is_none());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn buffer_save_still_writes_when_format_on_save_fails() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("save-format-failure");
    let path = root.join("mod.rs");
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;
    open_workspace_from_project(&mut state.runtime, "format-failure", &root)?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    state
        .runtime
        .services_mut()
        .insert(Arc::new(LspClientManager::new(
            LanguageServerRegistry::new(),
        )));
    state
        .runtime
        .services_mut()
        .insert(FormatterRegistry::default());
    formatter_registry_mut(&mut state.runtime)?.register(FormatterSpec {
        language_id: "rust".to_owned(),
        program: "definitely-missing-formatter".to_owned(),
        args: Vec::new(),
    })?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
        assert!(buffer.is_dirty());
    }

    state
        .runtime
        .execute_command("buffer.save")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(&path).map_err(|error| error.to_string())?,
        "// local\nfn main() {}\n"
    );
    assert!(!shell_buffer(&state.runtime, buffer_id)?.is_dirty());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn pdf_buffers_reload_when_backing_file_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("reload-pdf");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["before reload", "second page"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    write_test_pdf(&path, &["after reload"])?;

    let reloaded = {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.mark_backing_file_reload_pending();
        buffer.reload_from_disk_if_changed(true)?
    };
    assert!(reloaded);

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    let pdf_state = buffer
        .pdf_state()
        .ok_or_else(|| "pdf state missing".to_owned())?;
    assert_eq!(pdf_state.page_count(), 1);
    assert!(!buffer.has_pdf_preview_surface());
    assert!(buffer.text.text().contains("after reload"));

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn svg_image_buffers_toggle_between_rendered_and_source_modes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("open-image-svg");
    let path = root.join("sample.svg");
    write_test_svg(&path)?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let image_state = buffer
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?;
        assert_eq!(buffer.kind, BufferKind::Image);
        assert_eq!(image_state.format, ImageBufferFormat::Svg);
        assert_eq!(image_state.mode, ImageBufferMode::Rendered);
        assert!(buffer.is_read_only());
    }

    toggle_active_image_buffer_mode(&mut state.runtime)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        assert!(buffer.is_svg_source_mode());
        assert!(buffer.supports_text_file_actions());
        assert!(!buffer.is_read_only());
        assert!(buffer.text.text().contains("<svg"));
    }

    toggle_active_image_buffer_mode(&mut state.runtime)?;
    {
        let buffer = shell_buffer(&state.runtime, buffer_id)?;
        let image_state = buffer
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?;
        assert_eq!(image_state.mode, ImageBufferMode::Rendered);
        assert!(buffer.is_read_only());
    }

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn image_zoom_controls_adjust_zoom_multiplier() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("image-zoom");
    let path = root.join("sample.png");
    write_test_png(&path)?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        1.0
    );

    zoom_active_image_buffer_in(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        IMAGE_ZOOM_STEP
    );

    zoom_active_image_buffer_out(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        1.0
    );

    zoom_active_image_buffer_in(&mut state.runtime)?;
    reset_active_image_buffer_zoom(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .image_state()
            .ok_or_else(|| "image state missing".to_owned())?
            .zoom,
        1.0
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_target_only_matching_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-targeted");
    let active_path = root.join("src").join("main.rs");
    let hidden_path = root.join("src").join("lib.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&active_path, "fn main() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&hidden_path, "pub fn hidden() {}\n").map_err(|error| error.to_string())?;

    let (active_buffer_id, hidden_buffer_id) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, active_buffer_id, &active_path)?;
    configure_file_buffer(&mut state, hidden_buffer_id, &hidden_path)?;

    std::fs::write(
        &hidden_path,
        "pub fn hidden() {\n    println!(\"disk\");\n}\n",
    )
    .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &hidden_path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false
    )?);
    wait_for_file_reload_worker(&mut state, &[hidden_buffer_id])?;
    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, active_buffer_id)?.text.line(1),
        None
    );
    assert_eq!(
        shell_buffer(&state.runtime, hidden_buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"disk\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_reload_hidden_buffers_without_focus_changes() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-hidden");
    let active_path = root.join("src").join("main.rs");
    let hidden_path = root.join("src").join("lib.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&active_path, "fn main() {}\n").map_err(|error| error.to_string())?;
    std::fs::write(&hidden_path, "pub fn hidden() {}\n").map_err(|error| error.to_string())?;

    let (active_buffer_id, hidden_buffer_id) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, active_buffer_id, &active_path)?;
    configure_file_buffer(&mut state, hidden_buffer_id, &hidden_path)?;

    std::fs::write(
        &hidden_path,
        "pub fn hidden() {\n    println!(\"background\");\n}\n",
    )
    .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &hidden_path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false,
    )?);
    wait_for_file_reload_worker(&mut state, &[hidden_buffer_id])?;
    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, hidden_buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"background\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn file_reload_notifications_wait_for_dirty_buffers_to_become_clean() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("file-reload-dirty");
    let path = root.join("src").join("main.rs");
    std::fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
    std::fs::write(&path, "fn main() {}\n").map_err(|error| error.to_string())?;

    let (buffer_id, _) = active_and_secondary_buffer_ids(&state.runtime)?;
    configure_file_buffer(&mut state, buffer_id, &path)?;

    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        buffer.text.set_cursor(TextPoint::new(0, 0));
        buffer.text.insert_text("// local\n");
    }
    std::fs::write(&path, "fn main() {\n    println!(\"disk\");\n}\n")
        .map_err(|error| error.to_string())?;
    record_file_reload_event(&state, &path)?;

    assert!(!refresh_pending_file_reloads(
        &mut state.runtime,
        Instant::now(),
        false,
    )?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(0)
            .as_deref(),
        Some("// local")
    );

    {
        let buffer = shell_ui_mut(&mut state.runtime)?
            .buffer_mut(buffer_id)
            .ok_or_else(|| format!("buffer `{buffer_id}` is missing"))?;
        assert!(buffer.text.undo());
        assert!(!buffer.text.is_dirty());
    }

    assert!(wait_for_file_reload_change(&mut state)?);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    println!(\"disk\");")
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn lsp_log_buffer_name_includes_server_name() {
    assert_eq!(lsp_log_buffer_name("csharp-ls"), "*lsp-log csharp-ls*");
}

#[test]
fn lsp_log_buffer_lines_only_include_entries_for_requested_server() {
    let entries = vec![
        LspLogEntry::new(LspLogDirection::Outgoing, "csharp-ls", "{\"id\":1}"),
        LspLogEntry::new(LspLogDirection::Incoming, "rust-analyzer", "{\"id\":2}"),
    ];
    let filtered = lsp_log_entries_for_server(&entries, "csharp-ls");
    let lines = lsp_log_buffer_lines("csharp-ls", &filtered);
    let body = lines.join("\n");

    assert!(body.contains("*lsp-log csharp-ls* captures live JSON-RPC traffic for `csharp-ls`."));
    assert!(body.contains("OUT csharp-ls"));
    assert!(!body.contains("rust-analyzer"));
}

#[test]
fn errors_buffer_updates_stay_in_the_background() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let active_before = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing".to_owned())?;

    assert_ne!(active_before.2, "*errors*");
    record_runtime_error(&mut state.runtime, "test.error", "boom");

    let active_after = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after logging".to_owned())?;
    assert_eq!(active_after.0, active_before.0);
    assert_eq!(active_after.1, active_before.1);
    assert_eq!(active_after.2, active_before.2);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, active_before.1);
    Ok(())
}

#[test]
fn lsp_log_buffers_stay_in_the_background_until_explicitly_focused() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let active_before = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing".to_owned())?;

    let buffer_id = ensure_lsp_log_buffer(&mut state.runtime, workspace_id, "rust-analyzer")?;
    let active_after_creation = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after creating log buffer".to_owned())?;

    assert_eq!(active_after_creation.0, active_before.0);
    assert_eq!(active_after_creation.1, active_before.1);
    assert_eq!(active_after_creation.2, active_before.2);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, active_before.1);

    state
        .runtime
        .model_mut()
        .focus_buffer(workspace_id, buffer_id)
        .map_err(|error| error.to_string())?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);
    sync_active_buffer(&mut state.runtime)?;

    let active_after_focus = active_runtime_buffer(&state.runtime)?
        .ok_or_else(|| "active runtime buffer is missing after focusing log buffer".to_owned())?;
    assert_eq!(active_after_focus.1, buffer_id);
    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);
    Ok(())
}

fn install_test_lsp_manager(
    runtime: &mut EditorRuntime,
    server_ids: &[&str],
) -> Result<Arc<LspClientManager>, String> {
    let mut registry = LanguageServerRegistry::new();
    for server_id in server_ids {
        registry
            .register(editor_lsp::LanguageServerSpec::new(
                *server_id,
                "rust",
                ["rs"],
                "dummy-lsp",
                std::iter::empty::<&str>(),
            ))
            .map_err(|error| error.to_string())?;
    }
    let manager = Arc::new(LspClientManager::new(registry));
    runtime.services_mut().insert(Arc::clone(&manager));
    Ok(manager)
}

fn install_lsp_enabled_file_buffer(
    state: &mut ShellState,
    name: &str,
    path: &Path,
    lines: Vec<String>,
) -> Result<BufferId, String> {
    let buffer_id = install_text_test_buffer(state, name, lines)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.text.set_path(path.to_path_buf());
        buffer.set_lsp_enabled(true);
        buffer.set_lsp_path(Some(path.to_path_buf()));
    }
    Ok(buffer_id)
}

fn sample_lsp_diagnostic(message: &str) -> Diagnostic {
    Diagnostic::new(
        "rustc",
        message,
        DiagnosticSeverity::Error,
        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 4)),
    )
}

#[test]
fn apply_pending_lsp_state_skips_diagnostic_lookups_when_generation_unchanged() -> Result<(), String>
{
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let path = PathBuf::from("src").join("main.rs");
    let buffer_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-skip*",
        &path,
        vec!["fn main() {}".to_owned()],
    )?;
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;

    apply_pending_lsp_state(&mut state.runtime)?;
    let after_publish = shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision();
    assert_eq!(after_publish, 1);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics()[0].message(),
        "cannot find value `missing`"
    );
    let lookups_after_publish = manager.diagnostics_for_path_lookups();
    assert!(
        lookups_after_publish >= 1,
        "first apply should look up diagnostics"
    );

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        after_publish
    );
    assert_eq!(
        manager.diagnostics_for_path_lookups(),
        lookups_after_publish,
        "unchanged diagnostics generation must not clone diagnostics again"
    );

    manager
        .apply_published_diagnostics(&path, vec![sample_lsp_diagnostic("unused variable")])
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        after_publish + 1
    );
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics()[0].message(),
        "unused variable"
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_refreshes_only_paths_whose_diagnostics_changed() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let main_path = PathBuf::from("src").join("main.rs");
    let lib_path = PathBuf::from("src").join("lib.rs");
    let main_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-main*",
        &main_path,
        vec!["fn main() {}".to_owned()],
    )?;
    let lib_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-lib*",
        &lib_path,
        vec!["pub fn lib() {}".to_owned()],
    )?;
    manager
        .attach_memory_session(
            "rust-analyzer",
            &main_path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;
    manager
        .attach_memory_session("rust-analyzer", &lib_path, Vec::new())
        .map_err(|error| error.to_string())?;
    let _ = manager.take_dirty_diagnostic_paths();
    manager
        .apply_published_diagnostics(
            &main_path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(manager.diagnostics_for_path_lookups(), 1);
    assert_eq!(
        shell_buffer(&state.runtime, main_id)?.lsp_diagnostics_revision(),
        1
    );
    assert_eq!(
        shell_buffer(&state.runtime, lib_id)?.lsp_diagnostics_revision(),
        0
    );

    manager
        .apply_published_diagnostics(&lib_path, vec![sample_lsp_diagnostic("unused variable")])
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(manager.diagnostics_for_path_lookups(), 2);
    assert_eq!(
        shell_buffer(&state.runtime, main_id)?.lsp_diagnostics_revision(),
        1
    );
    assert_eq!(
        shell_buffer(&state.runtime, lib_id)?.lsp_diagnostics_revision(),
        1
    );
    assert_eq!(
        shell_buffer(&state.runtime, lib_id)?.lsp_diagnostics()[0].message(),
        "unused variable"
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_clears_diagnostics_after_session_disconnect() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let path = PathBuf::from("src").join("main.rs");
    let buffer_id = install_lsp_enabled_file_buffer(
        &mut state,
        "*lsp-diag-disconnect*",
        &path,
        vec!["fn main() {}".to_owned()],
    )?;
    manager
        .attach_memory_session(
            "rust-analyzer",
            &path,
            vec![sample_lsp_diagnostic("cannot find value `missing`")],
        )
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        1
    );

    manager
        .disconnect_memory_sessions_for_path(&path)
        .map_err(|error| error.to_string())?;
    apply_pending_lsp_state(&mut state.runtime)?;
    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .lsp_diagnostics()
            .is_empty()
    );
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?.lsp_diagnostics_revision(),
        2
    );
    Ok(())
}

#[test]
fn apply_pending_lsp_state_skips_log_snapshot_until_revision_moves() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let manager = install_test_lsp_manager(&mut state.runtime, &["rust-analyzer"])?;
    let workspace_id = state
        .runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let buffer_id = ensure_lsp_log_buffer(&mut state.runtime, workspace_id, "rust-analyzer")?;
    apply_pending_lsp_state(&mut state.runtime)?;
    let before = shell_buffer(&state.runtime, buffer_id)?.text.text();

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.text.text(), before);

    manager.record_transport_log_event("rust-analyzer", "started language server");
    apply_pending_lsp_state(&mut state.runtime)?;
    let after = shell_buffer(&state.runtime, buffer_id)?.text.text();
    assert_ne!(after, before);
    assert!(after.contains("started language server"));

    apply_pending_lsp_state(&mut state.runtime)?;
    assert_eq!(shell_buffer(&state.runtime, buffer_id)?.text.text(), after);
    Ok(())
}
