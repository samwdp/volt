use std::{cell::Cell, rc::Rc};

use gpui::prelude::*;

struct GpuiLaunch {
    summary: Mutex<Option<ShellSummary>>,
    error: Mutex<Option<String>>,
}

struct GpuiSession {
    ttf: &'static sdl3::ttf::Sdl3TtfContext,
    state: ShellState,
    fonts: FontSet<'static>,
    font_path: PathBuf,
    font_family: String,
    font_pixel_size: f32,
    theme_settings: ThemeRuntimeSettings,
    theme_reload: ThemeReloadState,
    user_config_reload: UserConfigReloadState,
    cell_width: i32,
    line_height: i32,
    ascent: i32,
    frames_rendered: u32,
    last_scene: Option<Vec<DrawCommand>>,
    last_visual_key: Option<ShellVisualRefreshKey>,
    fps_overlay: Option<FpsOverlayState>,
    deferred_icon_font_paths: Option<VecDeque<PathBuf>>,
    deferred_icon_fonts_complete: bool,
    deferred_emoji_font_loaded: bool,
    last_user_activity: Instant,
    pending_events: VecDeque<Event>,
    last_mouse: Option<(f32, f32)>,
    content_origin: Rc<Cell<(f32, f32)>>,
    config: ShellConfig,
    launch: Arc<GpuiLaunch>,
    finished: bool,
    quit: bool,
    fonts_dirty: bool,
}

fn shell_flag(state: &mut ShellState, source: &str, result: Result<bool, ShellError>) -> bool {
    match result {
        Ok(changed) => changed,
        Err(error) => {
            state.record_shell_error(source, error);
            false
        }
    }
}

fn runtime_flag(state: &mut ShellState, source: &str, result: Result<bool, String>) -> bool {
    match result {
        Ok(changed) => changed,
        Err(error) => {
            state.record_error(source, error);
            false
        }
    }
}

fn pixels_u32(value: gpui::Pixels) -> u32 {
    let px = value.as_f32();
    if px.is_finite() && px > 1.0 {
        px.round() as u32
    } else {
        1
    }
}

fn lock_mutex<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

impl GpuiSession {
    fn finish(&mut self) {
        if self.finished {
            return;
        }
        self.finished = true;
        if let Some(lsp_client) = self
            .state
            .runtime
            .services()
            .get::<Arc<LspClientManager>>()
            .cloned()
        {
            let _ = lsp_client.prepare_application_quit();
        }
        if let Some(dap_client) = self
            .state
            .runtime
            .services()
            .get::<Arc<DapClientManager>>()
            .cloned()
        {
            let _ = dap_client.stop_all_sessions();
        }
        if let Ok(registry) = process_registry_service(&self.state.runtime)
            && let Ok(mut registry) = registry.lock()
        {
            let _ = registry.application_quit(OWNED_PROCESS_TEARDOWN_GRACE);
        }
        let summary = build_shell_summary(
            &mut self.state,
            self.frames_rendered,
            RenderBackend::Gpui,
            "gpui".to_owned(),
            &self.font_path,
        );
        *lock_mutex(&self.launch.summary) = Some(summary);
    }

    fn push_event(&mut self, event: Event) {
        self.pending_events.push_back(event);
    }

    fn local_point(&self, position: gpui::Point<gpui::Pixels>) -> (f32, f32) {
        let (origin_x, origin_y) = self.content_origin.get();
        (
            position.x.as_f32() - origin_x,
            position.y.as_f32() - origin_y,
        )
    }

    fn push_mouse_button(&mut self, button: gpui::MouseButton, position: gpui::Point<gpui::Pixels>, clicks: usize, down: bool) {
        let (x, y) = self.local_point(position);
        let clicks = u8::try_from(clicks).unwrap_or(u8::MAX);
        let event = if down {
            Event::MouseButtonDown {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: sdl_mouse_button(button),
                clicks,
                x,
                y,
            }
        } else {
            Event::MouseButtonUp {
                timestamp: 0,
                window_id: 0,
                which: 0,
                mouse_btn: sdl_mouse_button(button),
                clicks,
                x,
                y,
            }
        };
        self.push_event(event);
    }

    fn push_mouse_move(&mut self, position: gpui::Point<gpui::Pixels>) {
        let (x, y) = self.local_point(position);
        let (xrel, yrel) = self
            .last_mouse
            .map(|(last_x, last_y)| (x - last_x, y - last_y))
            .unwrap_or((0.0, 0.0));
        self.last_mouse = Some((x, y));
        self.push_event(Event::MouseMotion {
            timestamp: 0,
            window_id: 0,
            which: 0,
            mousestate: sdl3::mouse::MouseState::from_sdl_state(0),
            x,
            y,
            xrel,
            yrel,
        });
    }

    fn push_scroll(&mut self, event: &gpui::ScrollWheelEvent) {
        let (mouse_x, mouse_y) = self.local_point(event.position);
        let (x, y) = match event.delta {
            gpui::ScrollDelta::Lines(delta) => (delta.x, delta.y),
            gpui::ScrollDelta::Pixels(delta) => {
                let line = self.line_height.max(1) as f32;
                (delta.x.as_f32() / line, delta.y.as_f32() / line)
            }
        };
        self.push_event(Event::MouseWheel {
            timestamp: 0,
            window_id: 0,
            which: 0,
            x,
            y,
            direction: MouseWheelDirection::Normal,
            mouse_x,
            mouse_y,
        });
    }

    fn push_key_down(&mut self, event: &gpui::KeyDownEvent) {
        self.push_event(key_down_event(event));
        if let Some(text) = text_from_keystroke(&event.keystroke) {
            self.push_event(text_input_event(text));
        }
    }

    fn reload_fonts_for_scale(&mut self, scale: f32) -> bool {
        let updated = theme_runtime_settings(
            self.state.runtime.services().get::<ThemeRegistry>(),
            &self.config,
            scale,
        );
        if updated == self.theme_settings {
            return false;
        }
        let fonts_changed = updated.font_size != self.theme_settings.font_size
            || updated.font_request != self.theme_settings.font_request
            || updated.emoji_font_request != self.theme_settings.emoji_font_request
            || updated.emoji_font_size != self.theme_settings.emoji_font_size
            || updated.display_scale != self.theme_settings.display_scale;
        if fonts_changed {
            let loaded = {
                let user_library = shell_user_library(&self.state.runtime);
                load_font_set_with_mode(
                    self.ttf,
                    &updated,
                    &*user_library,
                    OptionalFontLoadMode::Eager,
                )
            };
            match loaded {
                Ok((fonts, font_path)) => {
                    self.fonts = fonts;
                    self.font_path = font_path;
                    self.remember_font_metrics();
                    self.fonts_dirty = true;
                }
                Err(error) => self.state.record_shell_error("shell.update-theme", error),
            }
        }
        self.theme_settings = updated;
        fonts_changed
    }

    fn remember_font_metrics(&mut self) {
        self.line_height = self.fonts.primary().height().max(1);
        self.ascent = self.fonts.primary().ascent();
        self.cell_width = self.fonts.cell_width();
        self.font_pixel_size = self.fonts.primary_pixel_size;
        self.font_family =
            font_family_name(&self.font_path).unwrap_or_else(|| "Consolas".to_owned());
    }

    fn drain_events(&mut self, width: u32, height: u32) -> bool {
        let mut had_events = false;
        while let Some(event) = self.pending_events.pop_front() {
            had_events = true;
            let profiled_event = TypingEventMetadata::from_event(&event);
            let before = active_buffer_revision_key(&self.state.runtime);
            match self.state.handle_event(
                event,
                width,
                height,
                self.cell_width,
                self.line_height,
            ) {
                Ok(true) => {
                    self.quit = true;
                    break;
                }
                Ok(false) => {}
                Err(error) => self.state.record_shell_error("shell.handle-event", error),
            }
            let after = active_buffer_revision_key(&self.state.runtime);
            let edited = matches!(
                (before, after),
                (Some((before_id, before_revision)), Some((after_id, after_revision)))
                    if before_id == after_id && before_revision != after_revision
            );
            if edited || matches!(profiled_event, TypingEventMetadata::KeyDown) {
                self.state.note_text_edit_activity();
            }
        }
        if had_events {
            self.last_user_activity = Instant::now();
            if let Err(error) = self.state.sync_visible_buffer_layouts(
                width,
                height,
                self.cell_width,
                self.line_height,
            ) {
                self.state
                    .record_shell_error("shell.sync-visible-buffer-layouts", error);
            }
        }
        had_events
    }

    fn load_deferred_fonts(&mut self) {
        let idle = self.frames_rendered > 0
            && self
                .last_user_activity
                .elapsed()
                >= DEFERRED_ICON_FONT_IDLE_DELAY;
        if !idle {
            return;
        }
        let loaded = {
            let user_library = shell_user_library(&self.state.runtime);
            load_next_deferred_icon_font(
                self.ttf,
                &self.theme_settings,
                &*user_library,
                &mut self.fonts,
                &mut self.deferred_icon_font_paths,
                &mut self.deferred_icon_fonts_complete,
            )
        };
        match loaded {
            Ok(true) => self.fonts_dirty = true,
            Ok(false) => {}
            Err(error) => {
                self.deferred_icon_fonts_complete = true;
                self.state
                    .record_shell_error("shell.deferred-icon-fonts", error);
            }
        }
        if self.deferred_icon_fonts_complete
            && self.last_user_activity.elapsed() >= DEFERRED_EMOJI_FONT_IDLE_DELAY
            && load_deferred_emoji_font(
                self.ttf,
                &self.theme_settings,
                &mut self.fonts,
                &mut self.deferred_emoji_font_loaded,
            )
        {
            self.fonts_dirty = true;
        }
    }

    fn note_shell(&mut self, source: &str, result: Result<bool, ShellError>) -> bool {
        shell_flag(&mut self.state, source, result)
    }

    fn note_runtime(&mut self, source: &str, result: Result<bool, String>) -> bool {
        runtime_flag(&mut self.state, source, result)
    }

    fn refresh_side_effects(&mut self, width: u32, height: u32) -> bool {
        let now = Instant::now();
        let secondary_deferred = self.state.secondary_refresh_deferred_for_typing(now);
        let typing_budget = self.state.typing_refresh_budget_active(now);
        let mut changed = false;
        let readme = refresh_pending_workspace_readme_opens(&mut self.state.runtime);
        changed |= self.note_runtime("shell.workspace-readme-open", readme);
        let prewarm = refresh_pending_syntax_prewarm(&mut self.state.runtime);
        changed |= self.note_runtime("shell.syntax-prewarm", prewarm);
        if !secondary_deferred {
            self.state.tick_project_discovery_background();
        }
        let file_reload = self.state.refresh_pending_file_reloads(now);
        changed |= self.note_shell("shell.file-reload-refresh", file_reload);
        let issues = refresh_pending_issues(&mut self.state.runtime);
        changed |= self.note_runtime("shell.issues-refresh", issues);
        let picker = self.state.refresh_pending_picker_searches();
        changed |= self.note_shell("shell.picker-search-refresh", picker);
        let lsp = self.state.refresh_pending_lsp(typing_budget);
        changed |= self.note_shell("shell.lsp-refresh", lsp);
        let dap = self.state.refresh_pending_dap();
        changed |= self.note_shell("shell.dap-refresh", dap);
        let notifications = self.state.refresh_notifications(now, typing_budget);
        changed |= self.note_shell("shell.notification-refresh", notifications);
        let autocomplete = self.state.refresh_pending_autocomplete();
        changed |= self.note_shell("shell.autocomplete-refresh", autocomplete);
        let completion_resolve = self.state.refresh_pending_completion_resolve();
        changed |= self.note_shell("shell.completion-resolve-refresh", completion_resolve);
        let inline_completion = self.state.refresh_pending_inline_completion();
        changed |= self.note_shell("shell.inline-completion-refresh", inline_completion);
        let hover = self.state.refresh_hover_state();
        changed |= self.note_shell("shell.hover-refresh", hover);
        let lsp_ui = apply_lsp_ui_worker_results(&mut self.state.runtime);
        changed |= self.note_runtime("shell.lsp-ui-worker", lsp_ui);
        let volt_host = apply_volt_host_actions(&mut self.state.runtime);
        changed |= self.note_runtime("shell.volt-host", volt_host);
        let command_stream = self.state.refresh_pending_streamed_commands();
        changed |= self.note_shell("shell.command-stream-refresh", command_stream);
        let git_editor = refresh_pending_git_editor(&mut self.state.runtime);
        changed |= self.note_runtime("shell.git-editor-refresh", git_editor);
        let terminal = if typing_budget && !self.state.active_buffer_is_terminal() {
            Ok(false)
        } else {
            self.state.refresh_pending_terminal(
                width,
                height,
                self.cell_width,
                self.line_height,
            )
        };
        changed |= self.note_shell("shell.terminal-refresh", terminal);
        let syntax = self.state.refresh_pending_syntax(typing_budget);
        changed |= match syntax {
            Ok(stats) => stats.changed,
            Err(error) => {
                self.state.record_shell_error("shell.syntax-refresh", error);
                false
            }
        };
        let git = self.state.refresh_pending_git(now, secondary_deferred);
        changed |= self.note_shell("shell.git-refresh", git);
        let acp = self.state
            .refresh_pending_acp(width, height, self.line_height, self.cell_width);
        changed |= self.note_shell("shell.acp-refresh", acp);
        changed
    }

    fn advance(&mut self, window: &mut gpui::Window) -> Vec<DrawCommand> {
        let frame_started = Instant::now();
        let typing_frame = self
            .state
            .begin_typing_frame(self.frames_rendered, Duration::ZERO);
        let viewport = window.viewport_size();
        let width = pixels_u32(viewport.width);
        let height = pixels_u32(viewport.height);
        let theme_changed = match refresh_theme_registry_if_needed(
            &mut self.state.runtime,
            &mut self.theme_reload,
            Instant::now(),
        ) {
            Ok(changed) => changed,
            Err(error) => {
                self.state
                    .record_error("shell.theme-reload", error);
                false
            }
        };
        let user_config_changed =
            refresh_user_config_if_needed(&mut self.user_config_reload, Instant::now());
        let fonts_changed = self.reload_fonts_for_scale(window.scale_factor());
        if let Err(error) = self.state.sync_visible_buffer_layouts(
            width,
            height,
            self.cell_width,
            self.line_height,
        ) {
            self.state
                .record_shell_error("shell.sync-visible-buffer-layouts", error);
        }
        let had_events = self.drain_events(width, height);
        if let Err(error) = self.state.fire_pending_ambiguous_prefix_timeout() {
            self.state
                .record_shell_error("shell.ambiguous-prefix-timeout", error);
        }
        if self.state.deferred_startup_pending()
            && self.frames_rendered > 0
            && let Err(error) = self.state.finish_deferred_startup()
        {
            self.state
                .record_shell_error("shell.startup-bootstrap", error);
        }
        if !had_events {
            self.load_deferred_fonts();
        }
        let side_effects_changed = self.refresh_side_effects(width, height);
        let visual_key = match self.state.visual_refresh_key(
            width,
            height,
            &self.theme_settings,
            Instant::now(),
        ) {
            Ok(key) => Some(key),
            Err(error) => {
                self.state
                    .record_shell_error("shell.visual-refresh-key", error);
                None
            }
        };
        let visual_changed = visual_key
            .as_ref()
            .is_some_and(|key| self.last_visual_key.as_ref() != Some(key));
        let should_render = self.last_scene.is_none()
            || had_events
            || theme_changed
            || user_config_changed
            || fonts_changed
            || side_effects_changed
            || self.config.show_fps_overlay
            || visual_changed;
        if should_render {
            let mut scene = Vec::new();
            let fps_snapshot = self
                .fps_overlay
                .as_ref()
                .and_then(FpsOverlayState::snapshot);
            if let Err(error) = self.state.render(
                &mut DrawTarget::Scene(&mut scene),
                &self.fonts,
                WindowSize { width, height },
                TextMetrics {
                    cell_width: self.cell_width,
                    line_height: self.line_height,
                    ascent: self.ascent,
                },
                fps_snapshot.as_ref(),
            ) {
                self.state.record_shell_error("shell.render", error);
            } else {
                self.last_scene = Some(scene);
                if let Some(key) = visual_key {
                    self.last_visual_key = Some(key);
                }
            }
        }
        if let Some(frame) = typing_frame {
            self.state
                .record_typing_frame(frame.finish(frame_started.elapsed(), Instant::now()));
        }
        if let Some(overlay) = self.fps_overlay.as_mut() {
            overlay.record_frame(frame_started.elapsed());
        }
        self.frames_rendered = self.frames_rendered.saturating_add(1);
        if self
            .config
            .frame_limit
            .is_some_and(|limit| self.frames_rendered >= limit)
        {
            self.quit = true;
        }
        self.last_scene.clone().unwrap_or_default()
    }
}

impl Drop for GpuiSession {
    fn drop(&mut self) {
        self.finish();
    }
}

struct VoltGpuiRoot {
    focus_handle: gpui::FocusHandle,
    focused: bool,
    backdrop: Option<WindowEffects>,
    session: GpuiSession,
}

fn gpui_background_appearance(effects: WindowEffects) -> gpui::WindowBackgroundAppearance {
    match effects.transparency {
        WindowTransparency::None if effects.opacity < 1.0 => {
            gpui::WindowBackgroundAppearance::Transparent
        }
        WindowTransparency::None => gpui::WindowBackgroundAppearance::Opaque,
        WindowTransparency::Blur | WindowTransparency::Acrylic => {
            gpui::WindowBackgroundAppearance::Blurred
        }
        WindowTransparency::Mica => gpui::WindowBackgroundAppearance::MicaBackdrop,
        WindowTransparency::MicaTabbed => gpui::WindowBackgroundAppearance::MicaAltBackdrop,
    }
}

impl VoltGpuiRoot {
    fn text_style(&self) -> GpuiTextPaint {
        GpuiTextPaint {
            family: self.session.font_family.clone().into(),
            font_size: gpui::px(self.session.font_pixel_size),
            line_height: gpui::px(self.session.line_height as f32),
        }
    }

    fn register_fonts(&mut self, cx: &mut gpui::App) {
        if !self.session.fonts_dirty {
            return;
        }
        let icon_paths = resolve_icon_font_paths().unwrap_or_default();
        let fonts = gpui_font_bytes(&self.session.font_path, &icon_paths);
        if fonts.is_empty() {
            self.session.fonts_dirty = false;
            return;
        }
        if let Err(error) = cx.text_system().add_fonts(fonts) {
            self.session
                .state
                .record_error("shell.gpui-fonts", error.to_string());
        }
        self.session.fonts_dirty = false;
    }

    fn sync_backdrop(&mut self, window: &mut gpui::Window) {
        let effects = current_window_effect_settings(
            self.session
                .state
                .runtime
                .services()
                .get::<ThemeRegistry>(),
        );
        if self.backdrop == Some(effects) {
            return;
        }
        window.set_background_appearance(gpui_background_appearance(effects));
        self.backdrop = Some(effects);
    }
}

impl Render for VoltGpuiRoot {
    fn render(&mut self, window: &mut gpui::Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.focused {
            self.focused = true;
            window.focus(&self.focus_handle, cx);
        }
        self.sync_backdrop(window);
        self.register_fonts(cx);
        let scene = self.session.advance(window);
        let style = self.text_style();
        let origin = Rc::clone(&self.session.content_origin);
        if self.session.quit {
            self.session.finish();
            cx.quit();
        }
        let clear_width = pixels_u32(window.viewport_size().width);
        let clear_height = pixels_u32(window.viewport_size().height);
        gpui::div()
            .id("volt-gpui")
            .size_full()
            .occlude()
            .bg(gpui::transparent_black())
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                this.session.push_key_down(event);
                cx.notify();
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                    this.session
                        .push_mouse_button(event.button, event.position, event.click_count, true);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Right,
                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                    this.session
                        .push_mouse_button(event.button, event.position, event.click_count, true);
                    cx.notify();
                }),
            )
            .on_mouse_down(
                gpui::MouseButton::Middle,
                cx.listener(|this, event: &gpui::MouseDownEvent, _, cx| {
                    this.session
                        .push_mouse_button(event.button, event.position, event.click_count, true);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.session
                        .push_mouse_button(event.button, event.position, event.click_count, false);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Right,
                cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.session
                        .push_mouse_button(event.button, event.position, event.click_count, false);
                    cx.notify();
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Middle,
                cx.listener(|this, event: &gpui::MouseUpEvent, _, cx| {
                    this.session
                        .push_mouse_button(event.button, event.position, event.click_count, false);
                    cx.notify();
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                this.session.push_mouse_move(event.position);
                cx.notify();
            }))
            .on_scroll_wheel(cx.listener(
                |this, event: &gpui::ScrollWheelEvent, _, cx| {
                    this.session.push_scroll(event);
                    cx.notify();
                },
            ))
            .child(gpui::canvas(
                move |bounds, _, _| {
                    origin.set((bounds.origin.x.as_f32(), bounds.origin.y.as_f32()));
                },
                move |bounds, _, window, cx| {
                    window.request_animation_frame();
                    paint_draw_scene(
                        window,
                        cx,
                        &ScenePaint {
                            origin_x: bounds.origin.x.as_f32(),
                            origin_y: bounds.origin.y.as_f32(),
                            style: &style,
                        },
                        clear_width,
                        clear_height,
                        &scene,
                    );
                },
            ))
    }
}

fn open_gpui_window(session: GpuiSession, cx: &mut gpui::App) -> Result<(), String> {
    let title = session.config.title.clone();
    let width = session.config.width;
    let height = session.config.height;
    let hidden = session.config.hidden;
    cx.open_window(
        gpui::WindowOptions {
            window_bounds: Some(gpui::WindowBounds::centered(
                gpui::size(gpui::px(width as f32), gpui::px(height as f32)),
                cx,
            )),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some(title.into()),
                ..gpui::TitlebarOptions::default()
            }),
            show: !hidden,
            ..gpui::WindowOptions::default()
        },
        |window, cx| {
            let root = cx.new(|cx| VoltGpuiRoot {
                focus_handle: cx.focus_handle(),
                backdrop: None,
                focused: false,
                session,
            });
            window.on_window_should_close(cx, |_window, cx| {
                cx.quit();
                true
            });
            root
        },
    )
    .map(|_| ())
    .map_err(|error| error.to_string())?;
    cx.activate(true);
    Ok(())
}

fn run_gpui_shell(config: ShellConfig) -> Result<ShellSummary, ShellError> {
    editor_jobs::mark_current_thread_as_ui();
    let log_file_path = default_error_log_path();
    install_panic_hook(log_file_path.clone());
    let sdl_context = sdl3::init().map_err(|error| ShellError::Sdl(error.to_string()))?;
    let video = sdl_context
        .video()
        .map_err(|error| ShellError::Sdl(error.to_string()))?;
    configure_window_opacity_driver(Some(video.current_video_driver()));
    register_clipboard_context(video.clone());
    let _sdl = Box::leak(Box::new(sdl_context));
    let _video = Box::leak(Box::new(video));
    let ttf = Box::leak(Box::new(
        sdl3::ttf::init().map_err(|error| ShellError::Sdl(error.to_string()))?,
    ));
    let user_library = config
        .user_library
        .clone()
        .unwrap_or_else(|| Arc::new(NullUserLibrary));
    let state = ShellState::new_with_user_library_fast_start(
        log_file_path,
        config.profile_input_latency,
        user_library,
    )?;
    let theme_registry = state.runtime.services().get::<ThemeRegistry>();
    let theme_settings = theme_runtime_settings(theme_registry, &config, 1.0);
    let (fonts, font_path) = load_font_set_with_mode(
        ttf,
        &theme_settings,
        &*shell_user_library(&state.runtime),
        OptionalFontLoadMode::StartupPrimaryOnly,
    )?;
    let launch = Arc::new(GpuiLaunch {
        summary: Mutex::new(None),
        error: Mutex::new(None),
    });
    let mut session = GpuiSession {
        ttf,
        state,
        fonts,
        font_path,
        font_family: "Consolas".to_owned(),
        font_pixel_size: config.font_size as f32,
        theme_settings,
        theme_reload: ThemeReloadState::new(),
        user_config_reload: UserConfigReloadState::new(),
        cell_width: 1,
        line_height: 1,
        ascent: 1,
        frames_rendered: 0,
        last_scene: None,
        last_visual_key: None,
        fps_overlay: config.show_fps_overlay.then(FpsOverlayState::default),
        deferred_icon_font_paths: None,
        deferred_icon_fonts_complete: false,
        deferred_emoji_font_loaded: false,
        last_user_activity: Instant::now(),
        pending_events: VecDeque::new(),
        last_mouse: None,
        content_origin: Rc::new(Cell::new((0.0, 0.0))),
        config,
        launch: Arc::clone(&launch),
        finished: false,
        quit: false,
        fonts_dirty: true,
    };
    session.remember_font_metrics();
    let launch_for_run = Arc::clone(&launch);
    gpui_platform::application().run(move |cx| {
        if let Err(error) = open_gpui_window(session, cx) {
            *lock_mutex(&launch_for_run.error) = Some(error);
            cx.quit();
        }
    });
    if let Some(error) = lock_mutex(&launch.error).take() {
        return Err(ShellError::Runtime(error));
    }
    lock_mutex(&launch.summary)
        .take()
        .ok_or_else(|| ShellError::Runtime("gpui shell exited without a summary".to_owned()))
}
