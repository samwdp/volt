// Generic magit-style section-tree PluginBuffer host wiring.

fn plugin_buffer_section_handler(runtime: &EditorRuntime, kind: &str) -> Option<String> {
    shell_user_library(runtime)
        .plugin_buffer(kind)
        .and_then(|buffer| buffer.section_handler().map(str::to_owned))
}

fn buffer_is_section_tree_plugin(runtime: &EditorRuntime, kind: &BufferKind) -> bool {
    let BufferKind::Plugin(plugin_kind) = kind else {
        return false;
    };
    shell_user_library(runtime)
        .plugin_buffer(plugin_kind)
        .is_some_and(|buffer| {
            matches!(
                buffer.content_mode(),
                editor_plugin_api::PluginBufferContentMode::SectionTree
            ) || buffer.section_handler().is_some()
        })
}

fn apply_section_tree(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    tree: editor_core::SectionTree,
) -> Result<(), String> {
    let collapsed = shell_buffer(runtime, buffer_id)?
        .section_state()
        .map(|state| state.collapsed.clone())
        .unwrap_or_default();
    let lines = tree.render_lines(&collapsed);
    let buffer = shell_buffer_mut(runtime, buffer_id)?;
    {
        let state = buffer.ensure_section_state();
        state.collapsed = collapsed;
    }
    buffer.plugin_section_tree = Some(tree);
    buffer.set_section_lines(lines);
    Ok(())
}

fn build_plugin_section_context(
    runtime: &EditorRuntime,
    buffer_id: BufferId,
    action_id: Option<&str>,
    action_detail: Option<&str>,
    chord: Option<&str>,
) -> Result<editor_plugin_api::PluginSectionContext, String> {
    let buffer = shell_buffer(runtime, buffer_id)?;
    let BufferKind::Plugin(kind) = &buffer.kind else {
        return Err("focused buffer is not a plugin buffer".to_owned());
    };
    let cursor_line = buffer.cursor_point().line;
    let line_action = buffer
        .section_line_meta(cursor_line)
        .and_then(|meta| meta.action.clone());
    let mut context =
        editor_plugin_api::PluginSectionContext::new(kind.clone(), buffer.name.clone())
            .with_cursor_line(cursor_line);
    if let Some(root) = active_workspace_root(runtime)?.as_ref() {
        context = context.with_workspace_root(root.to_string_lossy().into_owned());
    }
    if let Some(opaque) = buffer.plugin_section_opaque.as_ref() {
        context = context.with_opaque_state(opaque.clone());
    }
    let resolved_id = action_id.or_else(|| line_action.as_ref().map(SectionAction::id));
    let resolved_detail = action_detail.or_else(|| line_action.as_ref().and_then(SectionAction::detail));
    if let Some(id) = resolved_id {
        context = context.with_action(id, resolved_detail);
    }
    if let Some(chord) = chord {
        context = context.with_chord(chord);
    }
    Ok(context)
}

fn call_plugin_section_handler(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
    action_id: Option<&str>,
    action_detail: Option<&str>,
    chord: Option<&str>,
) -> Result<(), String> {
    ensure_shell_buffer(runtime, buffer_id)?;
    let handler = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let BufferKind::Plugin(kind) = &buffer.kind else {
            return Ok(());
        };
        let Some(handler) = plugin_buffer_section_handler(runtime, kind) else {
            return Ok(());
        };
        handler
    };
    prepare_volt_host(runtime);
    let context =
        build_plugin_section_context(runtime, buffer_id, action_id, action_detail, chord)?;
    let result = shell_user_library(runtime).run_plugin_section_handler(&handler, &context);
    let _ = apply_volt_host_actions(runtime)?;
    let Some(result) = result else {
        return Ok(());
    };
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.plugin_section_opaque = result.opaque_state().map(str::to_owned);
    }
    apply_section_tree(runtime, buffer_id, result.tree())?;
    Ok(())
}

fn activate_plugin_section_line(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    call_plugin_section_handler(runtime, buffer_id, None, None, None)
}

fn activate_plugin_section_with_detail(
    runtime: &mut EditorRuntime,
    detail: Option<&str>,
) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (action_id, action_detail) = match detail {
        Some(detail) if detail.contains('|') => {
            let mut parts = detail.splitn(2, '|');
            (
                parts.next().filter(|part| !part.is_empty()),
                parts.next().filter(|part| !part.is_empty()),
            )
        }
        Some(detail) => (Some(detail), None),
        None => (None, None),
    };
    call_plugin_section_handler(runtime, buffer_id, action_id, action_detail, None)
}

fn toggle_plugin_section(runtime: &mut EditorRuntime) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let (section_id, tree) = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_section_tree_plugin(runtime, &buffer.kind) {
            return Ok(());
        }
        let meta = buffer
            .section_line_meta(buffer.cursor_point().line)
            .cloned();
        let section_id = match meta.as_ref().map(|meta| &meta.kind) {
            Some(SectionRenderLineKind::Header { id, .. }) => id.clone(),
            _ => return Ok(()),
        };
        let tree = buffer.plugin_section_tree.clone();
        (section_id, tree)
    };
    {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        let state = buffer.ensure_section_state();
        state.collapsed.toggle(&section_id);
    }
    if let Some(tree) = tree {
        apply_section_tree(runtime, buffer_id, tree)?;
    } else {
        call_plugin_section_handler(runtime, buffer_id, None, None, None)?;
    }
    Ok(())
}

fn move_plugin_section_item(runtime: &mut EditorRuntime, forward: bool) -> Result<(), String> {
    let buffer_id = active_shell_buffer_id(runtime)?;
    let target_line = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        if !buffer_is_section_tree_plugin(runtime, &buffer.kind) {
            return Ok(());
        }
        let Some(state) = buffer.section_state() else {
            return Ok(());
        };
        let current = buffer.cursor_point().line;
        if forward {
            ((current + 1)..state.lines.len()).find(|&index| {
                matches!(
                    state.lines.get(index).map(|line| &line.kind),
                    Some(SectionRenderLineKind::Item)
                )
            })
        } else {
            (0..current).rev().find(|&index| {
                matches!(
                    state.lines.get(index).map(|line| &line.kind),
                    Some(SectionRenderLineKind::Item)
                )
            })
        }
    };
    if let Some(line) = target_line {
        let buffer = shell_buffer_mut(runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(line, 0));
    }
    Ok(())
}

fn open_plugin_prompt(runtime: &mut EditorRuntime, detail: Option<&str>) -> Result<(), String> {
    let detail = detail.unwrap_or_default();
    let mut parts = detail.splitn(3, '|');
    let id = parts.next().unwrap_or("plugin.prompt").to_owned();
    let title = parts.next().unwrap_or("Input: ");
    let prefill = parts.next().unwrap_or("");
    let label = if title.ends_with(' ') {
        title.to_owned()
    } else {
        format!("{title} ")
    };
    let overlay = InputPromptOverlay::new(id, label, prefill);
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

fn confirm_plugin_prompt(
    runtime: &mut EditorRuntime,
    id: &str,
    text: &str,
) -> Result<(), String> {
    activate_plugin_section_with_detail(runtime, Some(&format!("{id}|{text}")))
}

fn maybe_refresh_section_tree_after_open(
    runtime: &mut EditorRuntime,
    buffer_id: BufferId,
) -> Result<(), String> {
    ensure_shell_buffer(runtime, buffer_id)?;
    let has_handler = {
        let buffer = shell_buffer(runtime, buffer_id)?;
        let BufferKind::Plugin(kind) = &buffer.kind else {
            return Ok(());
        };
        plugin_buffer_section_handler(runtime, kind).is_some()
    };
    if has_handler {
        call_plugin_section_handler(runtime, buffer_id, None, None, None)?;
    }
    Ok(())
}
