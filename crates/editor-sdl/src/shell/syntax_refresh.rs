fn refresh_workspace_syntax(runtime: &mut EditorRuntime) -> Result<(), String> {
    if runtime.services().get::<SyntaxRegistry>().is_none() {
        return Ok(());
    }
    let buffer_ids = shell_ui(runtime)?
        .active_workspace_buffer_ids()
        .map(|buffer_ids| buffer_ids.to_vec())
        .unwrap_or_default();
    {
        let ui = shell_ui_mut(runtime)?;
        for buffer_id in buffer_ids {
            if let Some(buffer) = ui.buffer_mut(buffer_id) {
                buffer.force_syntax_refresh();
            }
        }
    }
    refresh_pending_syntax(runtime).map(|_| ())
}

fn queue_buffer_syntax_refresh(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    let ui = shell_ui_mut(runtime)?;
    let Some(buffer) = ui.buffer_mut(buffer_id) else {
        return Ok(());
    };
    buffer.force_syntax_refresh();
    Ok(())
}

fn project_search_roots_from_user_library(
    user_library: &dyn UserLibrary,
) -> Vec<ProjectSearchRoot> {
    user_library
        .workspace_roots()
        .into_iter()
        .map(|root| ProjectSearchRoot::new(root.path, root.max_depth))
        .filter(|root| root.root().exists())
        .collect()
}

fn warm_project_discovery(user_library: &dyn UserLibrary) {
    let roots = project_search_roots_from_user_library(user_library);
    if roots.is_empty() {
        return;
    }
    let _ = project_discovery_snapshot(&roots);
}

fn install_optional_runtime_services(
    runtime: &mut EditorRuntime,
    user_library: &dyn UserLibrary,
) -> Result<(), ShellError> {
    runtime
        .services_mut()
        .insert(DbService::new().map_err(ShellError::Runtime)?);
    acp::init_acp_manager(runtime)?;
    runtime
        .services_mut()
        .insert(AutocompleteRegistry::from_user_config(user_library));
    runtime
        .services_mut()
        .insert(HoverRegistry::from_user_config(user_library));
    let mut lsp_registry = LanguageServerRegistry::new();
    lsp_registry
        .register_all(user_library.language_servers())
        .map_err(|error| ShellError::Runtime(error.to_string()))?;
    runtime
        .services_mut()
        .insert(Arc::new(LspClientManager::new(lsp_registry)));
    let mut dap_registry = DebugAdapterRegistry::new();
    dap_registry
        .register_all(user_library.debug_adapters())
        .map_err(|error| ShellError::Runtime(error.to_string()))?;
    runtime
        .services_mut()
        .insert(Arc::new(DapClientManager::new(dap_registry)));
    let mut syntax_registry = SyntaxRegistry::new();
    syntax_registry
        .register_all(user_library.syntax_languages())
        .map_err(|error| ShellError::Runtime(error.to_string()))?;
    runtime.services_mut().insert(syntax_registry);
    configure_syntax_refresh_worker(runtime).map_err(ShellError::Runtime)?;
    editor_tool_install::ensure_install_layout()
        .map_err(|error| ShellError::Runtime(error.to_string()))?;
    register_lsp_status_hooks(runtime).map_err(ShellError::Runtime)?;
    register_dap_hooks(runtime).map_err(ShellError::Runtime)?;
    Ok(())
}

fn refresh_buffer_syntax(runtime: &mut EditorRuntime, buffer_id: BufferId) -> Result<(), String> {
    let default_rainbow_parens_enabled =
        shell_user_library(runtime).rainbow_parens_config().enabled;
    let (path, text, buffer_language_id, syntax_window, rainbow_parens_enabled) = {
        let Some(buffer) = shell_ui(runtime)?.buffer(buffer_id) else {
            return Ok(());
        };
        (
            buffer.path().map(|path| path.to_path_buf()),
            buffer.text.clone(),
            buffer
                .language_id()
                .map(|language_id| language_id.to_owned()),
            buffer.desired_syntax_window(),
            buffer.rainbow_parens_enabled(default_rainbow_parens_enabled),
        )
    };

    let mut parse_session = None;
    let (language_id, syntax_result) = compute_buffer_syntax(
        syntax_registry_mut(runtime)?,
        path.as_deref(),
        &text,
        buffer_language_id.as_deref(),
        syntax_window,
        &mut parse_session,
    );

    let ui = shell_ui_mut(runtime)?;
    if let Some(buffer) = ui.buffer_mut(buffer_id) {
        match syntax_result {
            Some(Ok(snapshot)) => {
                buffer.set_language_id(language_id.clone());
                buffer.set_indexed_syntax_lines(
                    Some(index_syntax_lines_with_rainbow_parens(
                        snapshot,
                        &text,
                        rainbow_parens_enabled,
                    )),
                    syntax_window,
                );
                buffer.set_syntax_error(None);
            }
            Some(Err(error)) => {
                let error_label = path
                    .as_ref()
                    .map(|path| path.display().to_string())
                    .or(language_id.clone())
                    .unwrap_or_else(|| "buffer".to_owned());
                eprintln!("tree-sitter syntax refresh failed for `{error_label}`: {error}");
                buffer.set_language_id(language_id.clone());
                buffer.set_syntax_snapshot(None);
                buffer.set_syntax_error(Some(error.to_string()));
            }
            None => {
                buffer.set_syntax_snapshot(None);
                buffer.set_syntax_error(None);
                buffer.set_language_id(None);
            }
        }
    }

    Ok(())
}

fn index_syntax_lines_with_rainbow_parens(
    snapshot: SyntaxSnapshot,
    text: &TextBuffer,
    rainbow_parens_enabled: bool,
) -> IndexedSyntaxLines {
    let mut snapshot = snapshot;
    apply_rainbow_delimiter_spans_for_buffer(&mut snapshot, text, rainbow_parens_enabled);
    index_syntax_lines(snapshot, text)
}

fn compute_buffer_syntax(
    registry: &mut SyntaxRegistry,
    path: Option<&Path>,
    text: &TextBuffer,
    buffer_language_id: Option<&str>,
    syntax_window: Option<SyntaxLineWindow>,
    parse_session: &mut Option<SyntaxParseSession>,
) -> (Option<String>, Option<Result<SyntaxSnapshot, SyntaxError>>) {
    let highlight_window = syntax_window.map(SyntaxLineWindow::to_highlight_window);
    let language_id = path
        .and_then(|path| {
            registry
                .language_for_path(path)
                .map(|language| language.id().to_owned())
        })
        .or_else(|| buffer_language_id.map(str::to_owned));
    let Some(language_id) = language_id else {
        *parse_session = None;
        return (None, None);
    };
    let syntax_result = match match highlight_window {
        Some(window) => registry.highlight_buffer_for_language_window_with_session(
            &language_id,
            text,
            window,
            parse_session,
        ),
        None => {
            registry.highlight_buffer_for_language_with_session(&language_id, text, parse_session)
        }
    } {
        Ok(snapshot) => Ok(snapshot),
        Err(SyntaxError::GrammarNotInstalled {
            language_id: missing_language_id,
            ..
        }) => {
            if let Err(error) = registry.install_language(&missing_language_id) {
                Err(error)
            } else {
                match highlight_window {
                    Some(window) => registry.highlight_buffer_for_language_window_with_session(
                        &language_id,
                        text,
                        window,
                        parse_session,
                    ),
                    None => registry.highlight_buffer_for_language_with_session(
                        &language_id,
                        text,
                        parse_session,
                    ),
                }
            }
        }
        Err(error) => Err(error),
    };
    (Some(language_id), Some(syntax_result))
}

fn configure_syntax_refresh_worker(runtime: &mut EditorRuntime) -> Result<(), String> {
    let (install_root, query_asset_root, configs) = {
        let registry = runtime
            .services()
            .get::<SyntaxRegistry>()
            .ok_or_else(|| "syntax registry service missing".to_owned())?;
        (
            registry.install_root().to_path_buf(),
            registry.query_asset_root().map(Path::to_path_buf),
            registry.languages().cloned().collect::<Vec<_>>(),
        )
    };
    shell_ui_mut(runtime)?.configure_syntax_refresh_worker(configs, install_root, query_asset_root);
    Ok(())
}

fn install_tree_sitter_language(
    runtime: &mut EditorRuntime,
    language_id: &str,
) -> Result<(), String> {
    treesitter_install::install_tree_sitter_language(runtime, language_id)
}

fn recompile_installed_tree_sitter_languages(runtime: &mut EditorRuntime) -> Result<(), String> {
    treesitter_install::recompile_installed_tree_sitter_languages(runtime)
}
