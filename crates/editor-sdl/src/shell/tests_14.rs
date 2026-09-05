#[test]
fn vim_g_prefix_executes_workspace_keybinding() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.g-prefix-exact",
            "Test exact g-prefix binding",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push("exact".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .register_key_binding_for_mode(
            "g z",
            "tests.g-prefix-exact",
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    // `g` is an exact binding and a prefix of longer sequences, so it waits in
    // the key-sequence resolver without starting the Vim g-prefix yet.
    assert_eq!(
        state.ui().map_err(|error| error.to_string())?.vim().pending,
        None
    );

    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec!["exact".to_owned()]
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);
    Ok(())
}

#[test]
fn vim_g_prefix_preserves_longer_workspace_sequence() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    state.runtime.services_mut().insert(CommandLog::default());
    state
        .runtime
        .register_command(
            "tests.g-prefix-sequence",
            "Test longer g-prefix binding",
            CommandSource::Core,
            |runtime| {
                let log = runtime
                    .services_mut()
                    .get_mut::<CommandLog>()
                    .ok_or_else(|| "command log missing".to_owned())?;
                log.0.push("sequence".to_owned());
                Ok(())
            },
        )
        .map_err(|error| error.to_string())?;
    state
        .runtime
        .register_key_binding_for_mode(
            "g z z",
            "tests.g-prefix-sequence",
            KeymapScope::Workspace,
            KeymapVimMode::Normal,
            CommandSource::Core,
        )
        .map_err(|error| error.to_string())?;

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        Vec::<String>::new()
    );
    // `g z` is a proper prefix of `g z z`, so the resolver keeps waiting
    // without firing anything or starting the Vim g-prefix.
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);

    state
        .handle_text_input("z")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        state
            .runtime
            .services()
            .get::<CommandLog>()
            .ok_or_else(|| "command log missing".to_owned())?
            .0,
        vec!["sequence".to_owned()]
    );
    let ui = state.ui().map_err(|error| error.to_string())?;
    assert_eq!(ui.vim().pending, None);
    assert_eq!(ui.vim().pending_change_prefix, None);
    Ok(())
}

#[test]
fn vim_command_line_completion_includes_user_aliases() -> Result<(), String> {
    let state = state_with_user_library()?;

    let write_matches = vim_command_line_completion_matches(&state.runtime, "wr");
    assert!(write_matches.contains(&"write".to_owned()));

    let buffer_matches = vim_command_line_completion_matches(&state.runtime, "bd");
    assert!(buffer_matches.contains(&"bd".to_owned()));
    assert!(buffer_matches.contains(&"bdelete".to_owned()));
    Ok(())
}

#[test]
fn execute_vim_command_line_split_alias_splits_workspace() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 1);
    execute_vim_command_line(&mut state.runtime, "split")?;
    assert_eq!(shell_ui(&state.runtime)?.pane_count(), 2);
    Ok(())
}

#[test]
fn execute_vim_command_line_commands_alias_opens_picker() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    execute_vim_command_line(&mut state.runtime, "commands")?;
    assert!(shell_ui(&state.runtime)?.picker().is_some());
    Ok(())
}

#[test]
fn ctrl_enter_variants_match_manual_lsp_code_action_command() -> Result<(), String> {
    let root = unique_temp_dir("lsp-code-action-binding");
    let path = root.join("main.rs");
    fs::write(
        &path,
        "fn main() {\n    let value = 1;\n    let _ = value;\n}\n",
    )
    .map_err(|error| error.to_string())?;

    let manual_title = {
        let mut state = state_with_user_library()?;
        open_workspace_from_project(&mut state.runtime, "lsp-code-actions-manual", &root)?;
        open_workspace_file(&mut state.runtime, &path)?;
        shell_ui_mut(&mut state.runtime)?.enter_normal_mode();
        state
            .runtime
            .execute_command("lsp.code-action")
            .map_err(|error| error.to_string())?;
        shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned())
            .ok_or_else(|| "manual lsp code-action did not open a picker".to_owned())?
    };

    for (name, keycode) in [
        ("return", Keycode::Return),
        ("kp-enter", Keycode::KpEnter),
        ("return2", Keycode::Return2),
    ] {
        let mut state = state_with_user_library()?;
        open_workspace_from_project(
            &mut state.runtime,
            &format!("lsp-code-actions-binding-{name}"),
            &root,
        )?;
        open_workspace_file(&mut state.runtime, &path)?;
        shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

        let binding = state
            .runtime
            .keymaps()
            .get_for_mode(
                &editor_core::KeymapScope::Workspace,
                editor_core::KeymapVimMode::Normal,
                "Ctrl+Enter",
            )
            .ok_or_else(|| "Ctrl+Enter workspace binding is missing".to_owned())?;
        assert_eq!(binding.command_name(), "lsp.code-actions");

        let (render_width, render_height, cell_width, line_height) =
            markdown_table_event_dimensions();
        let handled = state
            .handle_event(
                Event::KeyDown {
                    timestamp: 0,
                    window_id: 0,
                    keycode: Some(keycode),
                    scancode: None,
                    keymod: ctrl_mod(),
                    repeat: false,
                    which: 0,
                    raw: 0,
                },
                render_width,
                render_height,
                cell_width,
                line_height,
            )
            .map_err(|error| error.to_string())?;

        assert!(!handled);
        let binding_title = shell_ui(&state.runtime)?
            .picker()
            .map(|picker| picker.session.title().to_owned())
            .ok_or_else(|| format!("Ctrl+Enter variant `{name}` did not open an LSP picker"))?;
        assert_eq!(binding_title, manual_title);
    }

    fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn f7_keydown_opens_keybinding_picker_from_user_binding() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let binding = state
        .runtime
        .keymaps()
        .get(&editor_core::KeymapScope::Global, "F7")
        .ok_or_else(|| "F7 global binding is missing".to_owned())?;
    assert_eq!(binding.command_name(), "picker.open-keybindings");

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    let handled = state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::F7),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert!(!handled);
    let picker_title = shell_ui(&state.runtime)?
        .picker()
        .map(|picker| picker.session.title().to_owned())
        .ok_or_else(|| "F7 binding did not open the keybinding picker".to_owned())?;
    assert_eq!(picker_title, "Keybindings");
    Ok(())
}

#[test]
fn browser_normal_mode_i_binding_focuses_input_without_inserting_text() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.enter_normal_mode();
        ui.set_active_vim_target(VimTarget::Buffer);
    }

    state
        .handle_text_input("I")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.active_buffer_id(), Some(buffer_id));
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert_eq!(ui.vim().target, VimTarget::Input);
    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .input_field()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .text(),
        ""
    );
    Ok(())
}

#[test]
fn browser_insert_mode_enter_binding_submits_current_url() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .set_text("example.com/docs");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: Mod::NOMOD,
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    Ok(())
}

#[test]
fn browser_insert_mode_ctrl_enter_binding_submits_current_url() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_user_plugin_buffer(&mut state, BROWSER_BUFFER_NAME, BROWSER_KIND)?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let _ = buffer.focus_browser_input();
        buffer
            .input_field_mut()
            .ok_or_else(|| "browser input field missing".to_owned())?
            .set_text("example.com/docs");
    }
    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.set_active_vim_target(VimTarget::Input);
        ui.enter_insert_mode();
    }

    let (render_width, render_height, cell_width, line_height) = markdown_table_event_dimensions();
    state
        .handle_event(
            Event::KeyDown {
                timestamp: 0,
                window_id: 0,
                keycode: Some(Keycode::Return),
                scancode: None,
                keymod: ctrl_mod(),
                repeat: false,
                which: 0,
                raw: 0,
            },
            render_width,
            render_height,
            cell_width,
            line_height,
        )
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .browser_state
            .as_ref()
            .and_then(|state| state.requested_url.as_deref()),
        Some("https://example.com/docs")
    );
    Ok(())
}

#[test]
fn leader_space_o_b_opens_browser_from_normal_mode() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let original_buffer_id = active_shell_buffer_id(&state.runtime)?;
    shell_ui_mut(&mut state.runtime)?.enter_normal_mode();

    state
        .handle_text_input(" ")
        .map_err(|error| error.to_string())?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);

    state
        .handle_text_input("o")
        .map_err(|error| error.to_string())?;
    assert_eq!(active_shell_buffer_id(&state.runtime)?, original_buffer_id);

    state
        .handle_text_input("b")
        .map_err(|error| error.to_string())?;

    let ui = shell_ui(&state.runtime)?;
    let browser_buffer_id = active_shell_buffer_id(&state.runtime)?;
    assert_ne!(browser_buffer_id, original_buffer_id);
    assert_eq!(ui.pane_count(), 2);
    assert_eq!(ui.input_mode(), InputMode::Insert);
    assert!(matches!(
        shell_buffer(&state.runtime, browser_buffer_id)?.kind,
        BufferKind::Plugin(ref kind) if kind == user::browser::BROWSER_KIND
    ));
    Ok(())
}

#[test]
fn execute_vim_command_line_substitute_defaults_to_current_line() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*substitute-current-line*",
        vec!["alpha one".to_owned(), "alpha two".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));

    execute_vim_command_line(&mut state.runtime, "s/alpha/omega/")?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("omega one"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("alpha two"));
    Ok(())
}

#[test]
fn execute_vim_command_line_substitute_supports_numeric_ranges() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*substitute-range*",
        vec![
            "alpha one".to_owned(),
            "alpha two".to_owned(),
            "alpha three".to_owned(),
        ],
    )?;

    execute_vim_command_line(&mut state.runtime, "2,3s/alpha/beta/")?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha one"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("beta two"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("beta three"));
    Ok(())
}

#[test]
fn gcc_toggles_current_line_comments() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*comment-line*",
        vec![
            "fn main() {".to_owned(),
            "    println!(\"hi\");".to_owned(),
            "}".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_language_id(Some("rust".to_owned()));
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 4));

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    assert_eq!(
        shell_ui(&state.runtime)?.vim().pending,
        Some(VimPending::CommentToggle { count: 1 })
    );
    assert_eq!(
        shell_ui(&state.runtime)?.vim().pending_change_prefix,
        Some(VimRecordedInput::Chord("g c".to_owned()))
    );
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    assert_eq!(
        shell_buffer(&state.runtime, buffer_id)?
            .text
            .line(1)
            .as_deref(),
        Some("    // println!(\"hi\");")
    );

    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(
        buffer.text.line(1).as_deref(),
        Some("    println!(\"hi\");")
    );
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

fn run_gcc_comment_toggle(state: &mut ShellState) -> Result<(), String> {
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())
}

#[test]
fn comment_toggle_styles_cover_all_shipped_syntax_languages() {
    let missing = user::syntax_languages()
        .into_iter()
        .filter_map(|language| {
            comment_style_for_language_path(
                Some(language.id()),
                language.file_extensions().first().map(String::as_str),
                language.file_names().first().map(String::as_str),
            )
            .is_none()
            .then(|| language.id().to_owned())
        })
        .collect::<Vec<_>>();

    assert!(
        missing.is_empty(),
        "missing comment styles for: {}",
        missing.join(", ")
    );
}

#[test]
fn gcc_toggles_prefix_comment_styles() -> Result<(), String> {
    for (language_id, original, commented) in [
        ("clojure", "  (inc value)", "  ; (inc value)"),
        ("latex", "  \\section{Intro}", "  % \\section{Intro}"),
        ("vim", "  set number", "  \" set number"),
    ] {
        let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
        let mut state =
            ShellState::new_with_user_library(default_error_log_path(), false, user_library)
                .map_err(|error| error.to_string())?;
        let buffer_id = install_text_test_buffer(
            &mut state,
            &format!("*{language_id}-comment-line*"),
            vec![original.to_owned()],
        )?;
        shell_buffer_mut(&mut state.runtime, buffer_id)?
            .set_language_id(Some(language_id.to_owned()));
        shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(commented),
            "expected `{language_id}` to use `{commented}`",
        );

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(original),
            "expected `{language_id}` to restore the original line",
        );
    }

    Ok(())
}

#[test]
fn gcc_toggles_block_comment_styles() -> Result<(), String> {
    for (language_id, original, commented) in [
        ("css", "  color: red;", "  /* color: red; */"),
        ("html", "  <div>volt</div>", "  <!-- <div>volt</div> -->"),
        (
            "json",
            "  \"name\": \"volt\",",
            "  /* \"name\": \"volt\", */",
        ),
        ("markdown", "  - item", "  <!-- - item -->"),
        ("xml", "  <tag/>", "  <!-- <tag/> -->"),
    ] {
        let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
        let mut state =
            ShellState::new_with_user_library(default_error_log_path(), false, user_library)
                .map_err(|error| error.to_string())?;
        let buffer_id = install_text_test_buffer(
            &mut state,
            &format!("*{language_id}-block-comment-line*"),
            vec![original.to_owned()],
        )?;
        shell_buffer_mut(&mut state.runtime, buffer_id)?
            .set_language_id(Some(language_id.to_owned()));
        shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 2));

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(commented),
            "expected `{language_id}` to use `{commented}`",
        );

        run_gcc_comment_toggle(&mut state)?;
        assert_eq!(
            shell_buffer(&state.runtime, buffer_id)?
                .text
                .line(0)
                .as_deref(),
            Some(original),
            "expected `{language_id}` to restore the original line",
        );
    }

    Ok(())
}

#[test]
fn visual_gc_toggles_region_comments() -> Result<(), String> {
    let user_library: Arc<dyn UserLibrary> = Arc::new(user::UserLibraryImpl);
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library)
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*comment-region*",
        vec![
            "let alpha = 1;".to_owned(),
            "let beta = 2;".to_owned(),
            "let gamma = 3;".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_language_id(Some("rust".to_owned()));
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));

    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("// let alpha = 1;"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("// let beta = 2;"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("let gamma = 3;"));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);

    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    state
        .handle_text_input("V")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("j")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("g")
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("c")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("let alpha = 1;"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("let beta = 2;"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("let gamma = 3;"));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_put_replaces_selection_and_updates_yank() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-put*",
        vec!["alpha beta gamma".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 6));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 6), VisualSelectionKind::Character);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 9));
    shell_ui_mut(&mut state.runtime)?.vim_mut().yank =
        Some(YankRegister::Character("delta".to_owned()));

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-put-after"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha delta gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 11));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Normal);
    assert_eq!(
        ui.vim().yank,
        Some(YankRegister::Character("beta".to_owned()))
    );
    Ok(())
}

#[test]
fn visual_indent_shifts_selected_lines_right() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-indent*",
        vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));

    state
        .runtime
        .emit_hook(HOOK_VIM_EDIT, HookEvent::new().with_detail("visual-indent"))
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("    alpha"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    beta"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 4));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_outdent_shifts_selected_lines_left() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-outdent*",
        vec![
            "    alpha".to_owned(),
            "        beta".to_owned(),
            "gamma".to_owned(),
        ],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-outdent"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    beta"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 0));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_join_merges_selected_lines() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-join*",
        vec!["alpha".to_owned(), "  beta".to_owned(), "gamma".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 0));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 0), VisualSelectionKind::Line);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(1, 0));

    state
        .runtime
        .emit_hook(HOOK_VIM_EDIT, HookEvent::new().with_detail("visual-join"))
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.line_count(), 2);
    assert_eq!(buffer.text.line(0).as_deref(), Some("alpha beta"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("gamma"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 5));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn visual_move_down_reorders_selected_lines_and_keeps_visual_selection() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-move-down*",
        vec![
            "fn main() {".to_owned(),
            "    if ready {".to_owned(),
            "        alpha();".to_owned(),
            "    }".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(2, 0));
    }
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(2, 0), VisualSelectionKind::Line);

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-move-down"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("fn main() {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if ready {"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("    }"));
    assert_eq!(buffer.text.line(3).as_deref(), Some("    alpha();"));
    assert_eq!(buffer.text.line(4).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(3, 0));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(3, 0)));
    Ok(())
}

#[test]
fn visual_move_up_reorders_selected_lines_and_reindents() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-move-up*",
        vec![
            "fn main() {".to_owned(),
            "    if ready {".to_owned(),
            "    }".to_owned(),
            "    alpha();".to_owned(),
            "}".to_owned(),
        ],
    )?;
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_language_id(Some("rust".to_owned()));
        buffer.set_cursor(TextPoint::new(3, 0));
    }
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(3, 0), VisualSelectionKind::Line);

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-move-up"),
        )
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("fn main() {"));
    assert_eq!(buffer.text.line(1).as_deref(), Some("    if ready {"));
    assert_eq!(buffer.text.line(2).as_deref(), Some("        alpha();"));
    assert_eq!(buffer.text.line(3).as_deref(), Some("    }"));
    assert_eq!(buffer.text.line(4).as_deref(), Some("}"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(2, 0));
    let ui = shell_ui(&state.runtime)?;
    assert_eq!(ui.input_mode(), InputMode::Visual);
    assert_eq!(ui.vim().visual_kind, VisualSelectionKind::Line);
    assert_eq!(ui.vim().visual_anchor, Some(TextPoint::new(2, 0)));
    Ok(())
}

#[test]
fn visual_replace_char_replaces_selected_text() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*visual-replace-char*",
        vec!["alpha".to_owned()],
    )?;
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 1));
    shell_ui_mut(&mut state.runtime)?
        .enter_visual_mode(TextPoint::new(0, 1), VisualSelectionKind::Character);
    shell_buffer_mut(&mut state.runtime, buffer_id)?.set_cursor(TextPoint::new(0, 3));

    state
        .runtime
        .emit_hook(
            HOOK_VIM_EDIT,
            HookEvent::new().with_detail("visual-replace-char"),
        )
        .map_err(|error| error.to_string())?;
    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    assert_eq!(buffer.text.line(0).as_deref(), Some("axxxa"));
    assert_eq!(buffer.cursor_point(), TextPoint::new(0, 1));
    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    Ok(())
}

#[test]
fn browser_viewport_rect_stays_above_prompt_footer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let buffer = shell_ui(&state.runtime)?
        .buffer(buffer_id)
        .ok_or_else(|| "browser shell buffer missing".to_owned())?;
    let rect = PixelRectToRect::rect(0, 0, 800, 400);
    let layout = buffer_footer_layout_with_command_line(
        buffer,
        rect,
        18,
        8,
        state.user_library.commandline_enabled(),
    );
    let viewport = browser_viewport_rect(
        buffer,
        rect,
        8,
        18,
        state.user_library.commandline_enabled(),
    )
    .ok_or_else(|| "browser viewport missing".to_owned())?;
    let viewport_bottom = viewport.y + viewport.height as i32;

    assert!(viewport.width > 0);
    assert!(viewport.height > 0);
    assert!(viewport.y >= layout.body_y - 2);
    assert!(viewport_bottom <= layout.input_y);
    Ok(())
}

#[test]
fn browser_surface_hit_testing_excludes_prompt_footer() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let buffer_id = install_browser_test_buffer(&mut state)?;
    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 480,
                height: 180,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
    )
    .map_err(|error| error.to_string())?;
    let surface = plan
        .visible_surfaces
        .iter()
        .find(|surface| surface.buffer_id == buffer_id)
        .ok_or_else(|| "browser surface missing".to_owned())?;

    assert_eq!(
        browser_surface_buffer_at_point(&plan, surface.rect.x + 4, surface.rect.y + 4),
        Some(buffer_id)
    );
    assert_eq!(
        browser_surface_buffer_at_point(
            &plan,
            surface.rect.x + 4,
            surface.rect.y + surface.rect.height as i32 + 4
        ),
        None
    );
    Ok(())
}

#[test]
fn browser_sync_plan_excludes_pdf_buffers() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let root = unique_temp_dir("pdf-browser-plan");
    let path = root.join("sample.pdf");
    write_test_pdf(&path, &["page one"])?;

    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 400,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
    )
    .map_err(|error| error.to_string())?;

    assert!(
        plan.buffers
            .iter()
            .all(|buffer| buffer.buffer_id != buffer_id)
    );
    assert!(
        plan.visible_surfaces
            .iter()
            .all(|surface| surface.buffer_id != buffer_id)
    );

    std::fs::remove_dir_all(&root).map_err(|error| error.to_string())?;
    Ok(())
}

#[test]
fn browser_sync_plan_hides_surfaces_while_picker_is_visible() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .set_picker(PickerOverlay::from_entries("Buffers", Vec::new()));

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 400,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now: Instant::now(),
        },
    )
    .map_err(|error| error.to_string())?;

    assert_eq!(plan.buffers.len(), 1);
    assert!(plan.visible_surfaces.is_empty());
    Ok(())
}

#[test]
fn browser_sync_plan_avoids_notification_overlays() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    let _buffer_id = install_browser_test_buffer(&mut state)?;
    let now = Instant::now();
    state
        .ui_mut()
        .map_err(|error| error.to_string())?
        .apply_notification(
            test_notification_update(
                "progress",
                NotificationSeverity::Info,
                "LSP · rust-analyzer",
                &[
                    "Indexing workspace",
                    "Scanning project files",
                    "Resolving dependencies",
                    "Refreshing diagnostics",
                    "Updating symbol cache",
                    "Preparing semantic tokens",
                ],
                Some(32),
                true,
            ),
            now,
        );

    let plan = browser_sync_plan(
        state.ui().map_err(|error| error.to_string())?,
        BrowserSyncView {
            runtime_popup: None,
            user_library: &*state.user_library,
            size: WindowSize {
                width: 800,
                height: 260,
            },
            metrics: CellMetrics {
                cell_width: 8,
                line_height: 18,
            },
            now,
        },
    )
    .map_err(|error| error.to_string())?;
    let notifications = state
        .ui()
        .map_err(|error| error.to_string())?
        .visible_notifications(now);
    let notification_rects = notification_overlay_layouts(&notifications, 800, 260, 8, 18)
        .into_iter()
        .map(|layout| layout.rect)
        .collect::<Vec<_>>();

    assert_eq!(plan.buffers.len(), 1);
    assert!(!notification_rects.is_empty());
    assert!(plan.visible_surfaces.iter().all(|surface| {
        notification_rects
            .iter()
            .all(|overlay| !rects_intersect(browser_viewport_rect_rect(surface.rect), *overlay))
    }));
    Ok(())
}

#[test]
fn detect_browser_url_uses_cursor_hit_or_single_line_url() -> Result<(), String> {
    let mut state = ShellState::new().map_err(|error| error.to_string())?;
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .text = TextBuffer::from_text("See https://example.com/docs.");
    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 10));
    let cursor_hit = detect_browser_url(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "browser URL missing under cursor".to_owned())?;
    assert_eq!(cursor_hit, "https://example.com/docs");

    state
        .active_buffer_mut()
        .map_err(|error| error.to_string())?
        .set_cursor(TextPoint::new(0, 0));
    let single_url = detect_browser_url(
        state
            .active_buffer_mut()
            .map_err(|error| error.to_string())?,
    )
    .ok_or_else(|| "browser URL missing from single-url line".to_owned())?;
    assert_eq!(single_url, "https://example.com/docs");
    Ok(())
}
