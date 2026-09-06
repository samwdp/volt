#![allow(unused_imports)]
use super::*;

#[test]
fn directory_view_state_uses_user_oil_defaults() {
    let defaults = user::UserLibraryImpl.oil_defaults();
    let state = DirectoryViewState::new(std::path::PathBuf::from("."), Vec::new(), defaults);

    assert_eq!(state.show_hidden, defaults.show_hidden);
    assert_eq!(state.sort_mode, defaults.sort_mode);
    assert_eq!(state.trash_enabled, defaults.trash_enabled);
}

#[test]
fn oil_insert_creates_directory_file_and_nested_paths_on_normal() -> Result<(), String> {
    let root = unique_temp_dir("oil-insert-create");
    std::fs::write(root.join("existing.txt"), "keep\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    oil_type_new_entry_and_leave_insert(&mut state, "Test/")?;
    oil_type_new_entry_and_leave_insert(&mut state, "abc.txt")?;
    oil_type_new_entry_and_leave_insert(&mut state, "nested/dir/file.txt")?;

    assert!(
        root.join("Test").is_dir(),
        "typing Test/ then leaving insert should create directory"
    );
    assert!(
        root.join("abc.txt").is_file(),
        "typing abc.txt then leaving insert should create file"
    );
    assert!(
        root.join("nested").join("dir").join("file.txt").is_file(),
        "typing nested/dir/file.txt then leaving insert should create nested directories and file"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_insert_patches_listing_without_rereading_siblings() -> Result<(), String> {
    let root = unique_temp_dir("oil-insert-patch");
    std::fs::write(root.join("existing.txt"), "keep\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    std::fs::write(root.join("sneaky.txt"), "external\n").map_err(|error| error.to_string())?;
    oil_type_new_entry_and_leave_insert(&mut state, "abc.txt")?;

    assert!(root.join("abc.txt").is_file());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "abc.txt").is_ok(),
        "created file should appear in the patched listing"
    );
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_err(),
        "insert must not reread siblings created on disk after open"
    );

    state
        .runtime
        .execute_command("oil.refresh")
        .map_err(|error| error.to_string())?;
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_ok(),
        "explicit refresh should reread the directory"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_toggle_hidden_filters_cached_entries_without_reread() -> Result<(), String> {
    let root = unique_temp_dir("oil-hidden-cache");
    std::fs::write(root.join(".hidden"), "hidden\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    std::fs::write(root.join("sneaky.txt"), "external\n").map_err(|error| error.to_string())?;
    state
        .runtime
        .execute_command("oil.toggle-hidden")
        .map_err(|error| error.to_string())?;

    assert!(oil_line_index_containing(&state.runtime, buffer_id, ".hidden").is_ok());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_err(),
        "hidden toggle must filter the cached listing"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_cycle_sort_does_not_reread_listing() -> Result<(), String> {
    let root = unique_temp_dir("oil-sort-cache");
    std::fs::write(root.join("alpha.txt"), "alpha\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("zeta.txt"), "zeta\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    std::fs::write(root.join("sneaky.txt"), "external\n").map_err(|error| error.to_string())?;
    state
        .runtime
        .execute_command("oil.cycle-sort")
        .map_err(|error| error.to_string())?;

    assert!(oil_line_index_containing(&state.runtime, buffer_id, "alpha.txt").is_ok());
    assert!(oil_line_index_containing(&state.runtime, buffer_id, "zeta.txt").is_ok());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "sneaky.txt").is_err(),
        "sort must reorder the cached listing without a disk walk"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_rename_moves_cursor_to_new_entry_not_substring_sibling() -> Result<(), String> {
    let root = unique_temp_dir("oil-rename-cursor");
    std::fs::write(root.join("old.txt"), "old\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("a_foo.txt"), "sibling\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    let old_line = oil_line_index_containing(&state.runtime, buffer_id, "old.txt")?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(old_line, 0));

    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("foo.txt")
        .map_err(|error| error.to_string())?;
    state
        .try_runtime_keybinding(Keycode::Escape, Mod::NOMOD)
        .map_err(|error| error.to_string())?;

    assert!(root.join("foo.txt").is_file());
    assert!(!root.join("old.txt").exists());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "a_foo.txt").is_ok(),
        "substring sibling must stay in the patched listing"
    );

    let cursor_line = shell_buffer(&state.runtime, buffer_id)?.cursor_point().line;
    let renamed_line =
        oil_line_index_for_entry_path(&state.runtime, buffer_id, &root.join("foo.txt"))?;
    assert_eq!(
        cursor_line, renamed_line,
        "rename cursor follow must match the new entry path, not a substring sibling"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_root_change_rereads_the_new_directory() -> Result<(), String> {
    let root = unique_temp_dir("oil-root-reread");
    let child = root.join("child");
    std::fs::create_dir(&child).map_err(|error| error.to_string())?;
    std::fs::write(child.join("inside.txt"), "inside\n").map_err(|error| error.to_string())?;
    std::fs::write(root.join("outside.txt"), "outside\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    let child_line = oil_line_index_containing(&state.runtime, buffer_id, "child")?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(child_line, 0));
    state
        .runtime
        .execute_command("oil.set-root")
        .map_err(|error| error.to_string())?;

    assert!(oil_line_index_containing(&state.runtime, buffer_id, "inside.txt").is_ok());
    assert!(oil_line_index_containing(&state.runtime, buffer_id, "outside.txt").is_err());

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_normal_mode_dd_applies_delete_immediately() -> Result<(), String> {
    let root = unique_temp_dir("oil-normal-delete");
    let file_path = root.join("alpha.txt");
    std::fs::write(&file_path, "alpha\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    let file_line = oil_line_index_containing(&state.runtime, buffer_id, "alpha.txt")?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(file_line, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("d")
        .map_err(|error| error.to_string())?;

    assert!(!file_path.exists());
    assert!(
        oil_line_index_containing(&state.runtime, buffer_id, "alpha.txt").is_err(),
        "deleted file should leave the oil listing"
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_normal_mode_yy_p_copies_file_immediately() -> Result<(), String> {
    let root = unique_temp_dir("oil-normal-copy-file");
    let source = root.join("source");
    let dest = root.join("dest");
    std::fs::create_dir_all(&source).map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dest).map_err(|error| error.to_string())?;
    let source_file = source.join("alpha.txt");
    std::fs::write(&source_file, "alpha\n").map_err(|error| error.to_string())?;

    let mut state = state_with_user_library()?;
    open_workspace_from_project(&mut state.runtime, "oil-copy-file", &root)?;
    open_oil_directory(&mut state.runtime, source.clone())?;
    let source_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_buffer_mut(&mut state.runtime, source_buffer_id)?.set_cursor(TextPoint::new(1, 0));
    shell_ui_mut(&mut state.runtime)?.focus_buffer(source_buffer_id);

    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("y")
        .map_err(|error| error.to_string())?;

    open_oil_directory(&mut state.runtime, dest.clone())?;
    let dest_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(dest_buffer_id);
    state
        .handle_text_input("p")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        std::fs::read_to_string(dest.join("alpha.txt")).map_err(|error| error.to_string())?,
        "alpha\n"
    );
    assert!(
        shell_buffer(&state.runtime, dest_buffer_id)?
            .directory_state()
            .ok_or_else(|| "destination directory state missing".to_owned())?
            .entries
            .iter()
            .any(|entry| entry.path() == dest.join("alpha.txt"))
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_open_parent_command_uses_parent_root() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("oil-open-parent");
    let child = root.join("nested");
    std::fs::create_dir_all(&child).map_err(|error| error.to_string())?;

    open_workspace_from_project(&mut state.runtime, "oil-parent", &root)?;
    open_oil_directory(&mut state.runtime, child)?;
    let buffer_id = active_shell_buffer_id(&state.runtime)?;

    state
        .runtime
        .execute_command("oil.open-parent")
        .map_err(|error| error.to_string())?;

    assert_eq!(active_shell_buffer_id(&state.runtime)?, buffer_id);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .directory_state()
            .ok_or_else(|| "directory state missing".to_owned())?
            .root,
        root
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_action_commands_are_registered_and_execute() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let root = unique_temp_dir("oil-command-actions");
    std::fs::write(root.join(".hidden"), "hidden\n").map_err(|error| error.to_string())?;

    open_workspace_from_project(&mut state.runtime, "oil-command-actions", &root)?;
    let buffer_id = open_oil_test_buffer(&mut state, &root)?;
    shell_ui_mut(&mut state.runtime)?.focus_buffer(buffer_id);

    for command_name in [
        "oil.open-entry",
        "oil.open-vertical-split",
        "oil.open-horizontal-split",
        "oil.open-new-pane",
        "oil.preview-entry",
        "oil.refresh",
        "oil.close",
        "oil.open-workspace-root",
        "oil.set-root",
        "oil.show-help",
        "oil.cycle-sort",
        "oil.toggle-hidden",
        "oil.toggle-trash",
        "oil.open-external",
        "oil.set-tab-local-root",
    ] {
        assert!(
            state.runtime.commands().contains(command_name),
            "missing command {command_name}"
        );
    }

    state
        .runtime
        .execute_command("oil.toggle-hidden")
        .map_err(|error| error.to_string())?;

    assert!(
        shell_buffer(&state.runtime, buffer_id)?
            .directory_state()
            .ok_or_else(|| "directory state missing".to_owned())?
            .show_hidden
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn oil_open_directory_is_scoped_per_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let first_root = unique_temp_dir("oil-workspace-first");
    let second_root = unique_temp_dir("oil-workspace-second");

    let first_workspace = open_workspace_from_project(&mut state.runtime, "alpha", &first_root)?;
    open_oil_directory(&mut state.runtime, first_root.clone())?;
    let first_buffer_id = active_shell_buffer_id(&state.runtime)?;

    let second_workspace = open_workspace_from_project(&mut state.runtime, "beta", &second_root)?;
    open_oil_directory(&mut state.runtime, second_root.clone())?;
    let second_buffer_id = active_shell_buffer_id(&state.runtime)?;

    assert_ne!(first_workspace, second_workspace);
    assert_ne!(first_buffer_id, second_buffer_id);
    assert_eq!(
        shell_buffer(&state.runtime, first_buffer_id)?
            .directory_state()
            .ok_or_else(|| "first oil directory state missing".to_owned())?
            .root,
        first_root
    );
    assert_eq!(
        shell_buffer(&state.runtime, second_buffer_id)?
            .directory_state()
            .ok_or_else(|| "second oil directory state missing".to_owned())?
            .root,
        second_root
    );

    switch_runtime_workspace(&mut state.runtime, first_workspace)?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, first_buffer_id);

    switch_runtime_workspace(&mut state.runtime, second_workspace)?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, second_buffer_id);

    std::fs::remove_dir_all(&first_root).map_err(|error| error.to_string())?;
    std::fs::remove_dir_all(&second_root).map_err(|error| error.to_string())?;
    Ok(())
}
