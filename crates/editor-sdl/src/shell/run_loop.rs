fn active_lsp_workspace_loaded(runtime: &EditorRuntime, ui: &ShellUiState) -> bool {
    let Some(path) = ui
        .active_buffer_id()
        .and_then(|buffer_id| ui.buffer(buffer_id))
        .and_then(ShellBuffer::path)
    else {
        return false;
    };
    runtime
        .services()
        .get::<Arc<LspClientManager>>()
        .map(|manager| manager.has_live_sessions_for_path(path))
        .unwrap_or(false)
}

fn frame_pacing_remaining(frame_started: Instant, now: Instant) -> Duration {
    let elapsed = now
        .checked_duration_since(frame_started)
        .unwrap_or_else(|| Duration::from_secs(0));
    FRAME_PACING_TARGET_120FPS.saturating_sub(elapsed)
}

fn pace_frame_to_120fps(frame_started: Instant) -> Duration {
    let mut now = Instant::now();
    let mut remaining = frame_pacing_remaining(frame_started, now);
    if remaining.is_zero() {
        return Duration::from_secs(0);
    }
    let sleep_started = now;
    while !remaining.is_zero() {
        if remaining > FRAME_PACING_YIELD_THRESHOLD {
            std::thread::sleep(remaining.saturating_sub(FRAME_PACING_YIELD_THRESHOLD));
        } else {
            std::thread::yield_now();
        }
        now = Instant::now();
        remaining = frame_pacing_remaining(frame_started, now);
    }
    sleep_started.elapsed()
}

fn git_refresh_deferred_for_typing(last_text_input_at: Option<Instant>, now: Instant) -> bool {
    last_text_input_at
        .map(|last| {
            now.checked_duration_since(last)
                .unwrap_or_else(|| Duration::from_secs(0))
                < GIT_REFRESH_TYPING_IDLE_THRESHOLD
        })
        .unwrap_or(false)
}

fn secondary_refresh_deferred_for_typing(
    last_text_input_at: Option<Instant>,
    now: Instant,
) -> bool {
    git_refresh_deferred_for_typing(last_text_input_at, now)
}

fn frame_pacing_deferred_for_typing(last_text_input_at: Option<Instant>, now: Instant) -> bool {
    last_text_input_at
        .map(|last| {
            now.checked_duration_since(last)
                .unwrap_or_else(|| Duration::from_secs(0))
                < FRAME_PACING_TYPING_IDLE_THRESHOLD
        })
        .unwrap_or(false)
}

fn should_yield_after_typing_batch(
    text_input_events: usize,
    events_processed: usize,
    batch_started: Instant,
) -> bool {
    text_input_events > 0
        && (events_processed >= TYPING_EVENT_BATCH_LIMIT
            || batch_started.elapsed() >= TYPING_EVENT_BATCH_TIME_BUDGET)
}

/// Runs the SDL3 + SDL_ttf demo shell.
pub fn run_demo_shell(config: ShellConfig) -> Result<ShellSummary, ShellError> {
    let mut startup_trace = StartupTrace::new();
    let log_file_path = default_error_log_path();
    install_panic_hook(log_file_path.clone());

    let sdl_context = sdl3::init().map_err(|error| ShellError::Sdl(error.to_string()))?;
    let video = sdl_context
        .video()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    configure_window_opacity_driver(Some(video.current_video_driver()));
    register_clipboard_context(video.clone());
    let ttf = sdl3::ttf::init().map_err(|error| ShellError::Sdl(error.to_string()))?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.sdl-init");
    }

    let user_library: Arc<dyn UserLibrary> = config
        .user_library
        .clone()
        .unwrap_or_else(|| Arc::new(NullUserLibrary));
    let mut state = ShellState::new_with_user_library_fast_start(
        log_file_path,
        config.profile_input_latency,
        Arc::clone(&user_library),
    )?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.state-fast-start");
    }
    let theme_registry = state.runtime.services().get::<ThemeRegistry>();
    let window_effect_settings = current_window_effect_settings(theme_registry);
    let mut theme_reload_state = ThemeReloadState::new();
    let mut user_config_reload_state = UserConfigReloadState::new();
    let mut window_builder = video.window(&config.title, config.width, config.height);
    window_builder
        .position_centered()
        .resizable()
        .high_pixel_density();
    if config.hidden {
        window_builder.hidden();
    }
    window_builder
        .set_flags(window_builder.flags() | window_creation_flags(window_effect_settings));
    let mut window = window_builder
        .build()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    apply_window_effects(&mut window, window_effect_settings)?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.window-create");
    }
    let mut theme_settings =
        theme_runtime_settings(theme_registry, &config, window.display_scale());
    let (mut fonts, mut font_path) = load_font_set_with_mode(
        &ttf,
        &theme_settings,
        &*user_library,
        OptionalFontLoadMode::StartupPrimaryOnly,
    )?;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.load-fonts");
    }
    let icon = load_window_icon()?;
    if !window.set_icon(icon) {
        return Err(ShellError::Sdl(sdl3::get_error().to_string()));
    }
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.window-icon");
    }
    video.text_input().start(&window);
    let mut line_height = fonts.primary().height().max(1) as usize;
    let mut ascent = fonts.primary().ascent();
    let mut cell_width = fonts.cell_width();

    let mut canvas = window.into_canvas();
    let texture_creator = canvas.texture_creator();
    let mut text_texture_cache = TextTextureCache::new();
    let renderer_name = canvas.renderer_name.clone();
    let mut event_pump = sdl_context
        .event_pump()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    let event_subsystem = sdl_context
        .event()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    attach_shell_wakeup(&event_subsystem)?;
    let _event_subsystem = event_subsystem;
    if let Some(trace) = startup_trace.as_mut() {
        trace.mark("shell.renderer-ready");
    }
    let mut frames_rendered = 0;
    let mut last_scene: Option<Vec<DrawCommand>> = None;
    let mut last_visual_key: Option<ShellVisualRefreshKey> = None;
    let mut fps_overlay_state = config.show_fps_overlay.then(FpsOverlayState::default);

    enum FrameOutcome {
        Continue,
        Quit,
    }

    let mut frame_pacing_sleep = Duration::from_secs(0);
    let mut deferred_icon_font_paths: Option<VecDeque<PathBuf>> = None;
    let mut deferred_icon_fonts_complete = true;
    let mut deferred_emoji_font_loaded = false;
    let mut last_user_activity = Instant::now();
    let mut queued_sdl_event: Option<sdl3::event::Event> = None;
    loop {
        let frame_started = Instant::now();
        let frame_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> FrameOutcome {
                let mut typing_frame =
                    state.begin_typing_frame(frames_rendered, frame_pacing_sleep);
                let theme_reload_changed = match refresh_theme_registry_if_needed(
                    &mut state.runtime,
                    &mut theme_reload_state,
                    Instant::now(),
                ) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.theme-reload", ShellError::Runtime(error));
                        false
                    }
                };
                let user_config_reload_changed =
                    refresh_user_config_if_needed(&mut user_config_reload_state, Instant::now());
                let previous_window_effects = theme_settings.window_effects;
                let fonts_changed = match update_theme_runtime(
                    &ttf,
                    &state,
                    &config,
                    canvas.window().display_scale(),
                    ThemeRuntimeSlots {
                        theme_settings: &mut theme_settings,
                        fonts: &mut fonts,
                        font_path: &mut font_path,
                        text_texture_cache: &mut text_texture_cache,
                        line_height: &mut line_height,
                        ascent: &mut ascent,
                        cell_width: &mut cell_width,
                    },
                ) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.update-theme", error);
                        false
                    }
                };
                if fonts_changed {
                    deferred_icon_font_paths = None;
                    deferred_icon_fonts_complete = true;
                    deferred_emoji_font_loaded = true;
                }
                if theme_settings.window_effects != previous_window_effects
                    && let Err(error) = update_window_effects(
                        canvas.window_mut(),
                        previous_window_effects,
                        theme_settings.window_effects,
                    )
                {
                    state.record_shell_error("shell.update-window-effects", error);
                }

                let (render_width, render_height) = match canvas.output_size() {
                    Ok(size) => size,
                    Err(error) => {
                        state.record_shell_error(
                            "shell.output-size",
                            ShellError::Sdl(error.to_string()),
                        );
                        return FrameOutcome::Continue;
                    }
                };
                let layout_sync_started = Instant::now();
                if let Err(error) = state.sync_visible_buffer_layouts(
                    render_width,
                    render_height,
                    cell_width,
                    line_height as i32,
                ) {
                    state.record_shell_error("shell.sync-visible-buffer-layouts", error);
                }
                if let Some(frame) = typing_frame.as_mut() {
                    frame.layout_sync += layout_sync_started.elapsed();
                }
                let mut had_events = false;
                let event_batch_started = Instant::now();
                let mut frame_polled_events = 0usize;
                let mut frame_text_input_events = 0usize;
                let mut pending_event = queued_sdl_event.take();

                loop {
                    let event = match pending_event.take() {
                        Some(event) => event,
                        None => match event_pump.poll_event() {
                            Some(event) => event,
                            None => break,
                        },
                    };
                    if take_shell_wakeup_event(&event) {
                        continue;
                    }
                    had_events = true;
                    frame_polled_events = frame_polled_events.saturating_add(1);
                    let profiled_event = typing_frame
                        .as_ref()
                        .map(|_| TypingEventMetadata::from_event(&event));
                    let active_buffer_revision_before = active_buffer_revision_key(&state.runtime);
                    let event_started = typing_frame.as_ref().map(|_| Instant::now());
                    match state.handle_event(
                        event,
                        render_width,
                        render_height,
                        cell_width,
                        line_height as i32,
                    ) {
                        Ok(true) => return FrameOutcome::Quit,
                        Ok(false) => {}
                        Err(error) => state.record_shell_error("shell.handle-event", error),
                    }
                    let active_buffer_revision_after = active_buffer_revision_key(&state.runtime);
                    let buffer_text_edited = matches!(
                        (active_buffer_revision_before, active_buffer_revision_after),
                        (Some((before_id, before_revision)), Some((after_id, after_revision)))
                            if before_id == after_id && before_revision != after_revision
                    );
                    if buffer_text_edited {
                        state.note_text_edit_activity();
                    }
                    if buffer_text_edited
                        || matches!(profiled_event, Some(TypingEventMetadata::TextInput { .. }))
                    {
                        frame_text_input_events = frame_text_input_events.saturating_add(1);
                    }
                    if let Some(frame) = typing_frame.as_mut()
                        && let Some(profiled_event) = profiled_event.as_ref()
                        && let Some(event_started) = event_started
                    {
                        frame.record_event(
                            profiled_event,
                            event_started.elapsed(),
                            state.take_last_text_input_profile(),
                        );
                    }
                    if should_yield_after_typing_batch(
                        frame_text_input_events,
                        frame_polled_events,
                        event_batch_started,
                    ) {
                        break;
                    }
                }
                if had_events {
                    last_user_activity = Instant::now();
                    let layout_sync_started = Instant::now();
                    if let Err(error) = state.sync_visible_buffer_layouts(
                        render_width,
                        render_height,
                        cell_width,
                        line_height as i32,
                    ) {
                        state.record_shell_error("shell.sync-visible-buffer-layouts", error);
                    }
                    if let Some(frame) = typing_frame.as_mut() {
                        frame.layout_sync += layout_sync_started.elapsed();
                    }
                }
                if let Err(error) = state.fire_pending_ambiguous_prefix_timeout() {
                    state.record_shell_error("shell.ambiguous-prefix-timeout", error);
                }
                if frames_rendered > 0
                    && !had_events
                    && frame_started.duration_since(last_user_activity)
                        >= DEFERRED_ICON_FONT_IDLE_DELAY
                {
                    match load_next_deferred_icon_font(
                        &ttf,
                        &theme_settings,
                        &*user_library,
                        &mut fonts,
                        &mut deferred_icon_font_paths,
                        &mut deferred_icon_fonts_complete,
                    ) {
                        Ok(true) => text_texture_cache.clear(),
                        Ok(false) => {}
                        Err(error) => {
                            deferred_icon_fonts_complete = true;
                            state.record_shell_error("shell.deferred-icon-fonts", error);
                        }
                    }
                }
                if frames_rendered > 0
                    && !had_events
                    && deferred_icon_fonts_complete
                    && frame_started.duration_since(last_user_activity)
                        >= DEFERRED_EMOJI_FONT_IDLE_DELAY
                    && load_deferred_emoji_font(
                        &ttf,
                        &theme_settings,
                        &mut fonts,
                        &mut deferred_emoji_font_loaded,
                    )
                {
                    text_texture_cache.clear();
                }
                if frames_rendered > 0
                    && !had_events
                    && state.deferred_startup_pending()
                    && let Err(error) = state.finish_deferred_startup()
                {
                    state.record_shell_error("shell.startup-bootstrap", error);
                }

                let refresh_now = Instant::now();
                let secondary_refresh_deferred =
                    state.secondary_refresh_deferred_for_typing(refresh_now);
                if !secondary_refresh_deferred {
                    if let Err(error) = refresh_pending_syntax_prewarm(&mut state.runtime) {
                        state
                            .record_shell_error("shell.syntax-prewarm", ShellError::Runtime(error));
                    }
                    if let Err(error) = refresh_pending_workspace_readme_opens(&mut state.runtime) {
                        state.record_shell_error(
                            "shell.workspace-readme-open",
                            ShellError::Runtime(error),
                        );
                    }
                    state.tick_project_discovery_background();
                }
                let typing_refresh_budget_active = state.typing_refresh_budget_active(refresh_now);
                let text_texture_cache_mode = if secondary_refresh_deferred {
                    TextTextureCacheMode::ReuseOnly
                } else {
                    TextTextureCacheMode::ReadWrite
                };
                let file_reload_changed = match state.refresh_pending_file_reloads(refresh_now) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.file-reload-refresh", error);
                        false
                    }
                };
                let issues_changed = match refresh_pending_issues(&mut state.runtime) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state
                            .record_shell_error("shell.issues-refresh", ShellError::Runtime(error));
                        false
                    }
                };
                let picker_refresh_started = Instant::now();
                let picker_changed = match state.refresh_pending_picker_searches() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.picker-search-refresh", error);
                        false
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.picker_search_refresh = picker_refresh_started.elapsed();
                }
                let lsp_refresh_started = Instant::now();
                let lsp_changed = match state.refresh_pending_lsp(typing_refresh_budget_active) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.lsp-refresh", error);
                        false
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.lsp_refresh = lsp_refresh_started.elapsed();
                }
                let dap_changed = match state.refresh_pending_dap() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.dap-refresh", error);
                        false
                    }
                };
                let notification_refresh_started = Instant::now();
                let notification_changed =
                    match state.refresh_notifications(refresh_now, typing_refresh_budget_active) {
                        Ok(changed) => changed,
                        Err(error) => {
                            state.record_shell_error("shell.notification-refresh", error);
                            false
                        }
                    };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.notification_refresh = notification_refresh_started.elapsed();
                }
                let autocomplete_refresh_started = Instant::now();
                let autocomplete_changed = match state.refresh_pending_autocomplete() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.autocomplete-refresh", error);
                        false
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.autocomplete_refresh = autocomplete_refresh_started.elapsed();
                }
                let inline_completion_changed = match state.refresh_pending_inline_completion() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.inline-completion-refresh", error);
                        false
                    }
                };
                let hover_refresh_started = Instant::now();
                let hover_changed = match state.refresh_hover_state() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.hover-refresh", error);
                        false
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.hover_refresh = hover_refresh_started.elapsed();
                }
                let command_stream_changed = match state.refresh_pending_streamed_commands() {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.command-stream-refresh", error);
                        false
                    }
                };
                let git_editor_changed = match refresh_pending_git_editor(&mut state.runtime) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error(
                            "shell.git-editor-refresh",
                            ShellError::Runtime(error),
                        );
                        false
                    }
                };
                let terminal_refresh_started = Instant::now();
                let terminal_changed =
                    if typing_refresh_budget_active && !state.active_buffer_is_terminal() {
                        false
                    } else {
                        match state.refresh_pending_terminal(
                            render_width,
                            render_height,
                            cell_width,
                            line_height as i32,
                        ) {
                            Ok(changed) => changed,
                            Err(error) => {
                                state.record_shell_error("shell.terminal-refresh", error);
                                false
                            }
                        }
                    };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.terminal_refresh = terminal_refresh_started.elapsed();
                }
                let syntax_refresh_started = Instant::now();
                let syntax_stats = match state.refresh_pending_syntax(typing_refresh_budget_active)
                {
                    Ok(stats) => stats,
                    Err(error) => {
                        state.record_shell_error("shell.syntax-refresh", error);
                        SyntaxRefreshStats::default()
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.syntax_refresh = syntax_refresh_started.elapsed();
                    frame.syntax_worker_compute = syntax_stats.worker_compute;
                    frame.syntax_result_count = syntax_stats.result_count;
                    frame.syntax_highlight_spans = syntax_stats.highlight_spans;
                }
                let syntax_changed = syntax_stats.changed;
                let git_refresh_started = Instant::now();
                if let Err(error) =
                    state.refresh_pending_git(refresh_now, secondary_refresh_deferred)
                {
                    state.record_shell_error("shell.git-refresh", error);
                }
                if let Some(frame) = typing_frame.as_mut() {
                    frame.git_refresh = git_refresh_started.elapsed();
                }
                let acp_refresh_started = Instant::now();
                let acp_changed = match state.refresh_pending_acp(
                    render_width,
                    render_height,
                    line_height as i32,
                    cell_width,
                ) {
                    Ok(changed) => changed,
                    Err(error) => {
                        state.record_shell_error("shell.acp-refresh", error);
                        false
                    }
                };
                if let Some(frame) = typing_frame.as_mut() {
                    frame.acp_refresh = acp_refresh_started.elapsed();
                }

                let visual_key = match state.visual_refresh_key(
                    render_width,
                    render_height,
                    &theme_settings,
                    Instant::now(),
                ) {
                    Ok(key) => key,
                    Err(error) => {
                        state.record_shell_error("shell.visual-refresh-key", error);
                        return FrameOutcome::Continue;
                    }
                };
                // Child webviews composite above the SDL canvas, so hide them before we paint
                // overlays such as the picker or a non-browser popup.
                if let Err(error) = state.sync_browser_hosts(
                    canvas.window(),
                    render_width,
                    render_height,
                    cell_width,
                    line_height as i32,
                ) {
                    state.record_shell_error("shell.browser-host", error);
                }
                let should_render = last_scene.is_none()
                    || had_events
                    || theme_reload_changed
                    || user_config_reload_changed
                    || fonts_changed
                    || file_reload_changed
                    || issues_changed
                    || picker_changed
                    || lsp_changed
                    || dap_changed
                    || notification_changed
                    || autocomplete_changed
                    || inline_completion_changed
                    || hover_changed
                    || command_stream_changed
                    || git_editor_changed
                    || terminal_changed
                    || syntax_changed
                    || acp_changed
                    || config.show_fps_overlay
                    || last_visual_key.as_ref() != Some(&visual_key);
                let presented_at = if should_render {
                    let mut scene = Vec::new();
                    let render_started = Instant::now();
                    let fps_overlay = fps_overlay_state
                        .as_ref()
                        .and_then(FpsOverlayState::snapshot);
                    if let Err(error) = state.render(
                        &mut DrawTarget::Scene(&mut scene),
                        &fonts,
                        WindowSize {
                            width: render_width,
                            height: render_height,
                        },
                        TextMetrics {
                            cell_width,
                            line_height: line_height as i32,
                            ascent,
                        },
                        fps_overlay.as_ref(),
                    ) {
                        state.record_shell_error("shell.render", error);
                        return FrameOutcome::Continue;
                    }
                    if let Some(frame) = typing_frame.as_mut() {
                        frame.render = render_started.elapsed();
                    }
                    if fonts_changed || last_scene.as_ref() != Some(&scene) {
                        let present_started = Instant::now();
                        if let Err(error) = present_scene_to_canvas(
                            &mut canvas,
                            &texture_creator,
                            &mut text_texture_cache,
                            text_texture_cache_mode,
                            &fonts,
                            &scene,
                        ) {
                            state.record_shell_error("shell.present", error);
                        } else if let Some(frame) = typing_frame.as_mut() {
                            frame.present = present_started.elapsed();
                        }
                        last_scene = Some(scene);
                    }
                    last_visual_key = Some(visual_key);
                    Instant::now()
                } else {
                    Instant::now()
                };
                if let Some(frame) = typing_frame.take() {
                    state.record_typing_frame(frame.finish(frame_started.elapsed(), presented_at));
                }
                if let Some(overlay) = fps_overlay_state.as_mut() {
                    overlay.record_frame(frame_started.elapsed());
                }

                FrameOutcome::Continue
            }));

        match frame_result {
            Ok(FrameOutcome::Quit) => {
                return Ok(build_shell_summary(
                    &mut state,
                    frames_rendered,
                    renderer_name.clone(),
                    &font_path,
                ));
            }
            Ok(FrameOutcome::Continue) => {
                frames_rendered += 1;
                if frames_rendered == 1
                    && let Some(trace) = startup_trace.as_mut()
                {
                    trace.mark("shell.first-frame");
                }
                if let Some(frame_limit) = config.frame_limit
                    && frames_rendered >= frame_limit
                {
                    break;
                }
            }
            Err(payload) => {
                state.record_error("panic", panic_payload_message(payload));
            }
        }

        let now = Instant::now();
        let mut extras = vec![
            theme_reload_state.last_checked_at + THEME_SOURCE_POLL_INTERVAL,
            user_config_reload_state.last_checked_at + THEME_SOURCE_POLL_INTERVAL,
        ];
        if !deferred_icon_fonts_complete {
            extras.push(last_user_activity + DEFERRED_ICON_FONT_IDLE_DELAY);
        }
        if !deferred_emoji_font_loaded {
            extras.push(last_user_activity + DEFERRED_EMOJI_FONT_IDLE_DELAY);
        }
        let deadlines = state.idle_wait_deadlines(now, extras);
        let keys_held = event_pump
            .keyboard_state()
            .pressed_scancodes()
            .next()
            .is_some();
        let interaction_active = keys_held || state.frame_pacing_deferred_for_typing(now);
        frame_pacing_sleep = if let Some(timeout) =
            idle_wait_timeout(now, &deadlines, interaction_active, config.show_fps_overlay)
        {
            queued_sdl_event = event_pump.wait_event_timeout(idle_wait_timeout_ms(timeout));
            Duration::from_secs(0)
        } else if state.frame_pacing_deferred_for_typing(now) {
            Duration::from_secs(0)
        } else {
            pace_frame_to_120fps(frame_started)
        };
    }

    Ok(build_shell_summary(
        &mut state,
        frames_rendered,
        renderer_name,
        &font_path,
    ))
}
