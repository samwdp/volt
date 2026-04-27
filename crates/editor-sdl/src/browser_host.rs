use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

use editor_core::BufferId;
use sdl3::video::Window;
#[cfg(target_os = "windows")]
use wry::WebViewBuilderExtWindows;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BrowserBufferPlan {
    pub(crate) buffer_id: BufferId,
    pub(crate) current_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BrowserViewportRect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BrowserSurfacePlan {
    pub(crate) buffer_id: BufferId,
    pub(crate) rect: BrowserViewportRect,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct BrowserSyncPlan {
    pub(crate) buffers: Vec<BrowserBufferPlan>,
    pub(crate) visible_surfaces: Vec<BrowserSurfacePlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BrowserLocationUpdate {
    pub(crate) buffer_id: BufferId,
    pub(crate) current_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum BrowserHostEvent {
    FocusParentRequested {
        buffer_id: BufferId,
    },
    OpenDevtoolsRequested {
        buffer_id: BufferId,
    },
    DocumentTitleChanged {
        buffer_id: BufferId,
        title: Option<String>,
    },
    PageLoadStateChanged {
        buffer_id: BufferId,
        current_url: String,
        is_loading: bool,
    },
    NewWindowRequested {
        buffer_id: BufferId,
        url: String,
    },
}

pub(crate) struct BrowserHostService {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    inner: DesktopBrowserHostService,
}

impl BrowserHostService {
    pub(crate) fn new() -> Self {
        Self {
            #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
            inner: DesktopBrowserHostService::new(),
        }
    }

    pub(crate) fn sync_window(
        &mut self,
        window: &Window,
        plan: &BrowserSyncPlan,
    ) -> Result<Vec<BrowserLocationUpdate>, String> {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            self.inner.sync_window(window, plan)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            let _ = (window, plan);
            Ok(Vec::new())
        }
    }

    pub(crate) fn focus_buffer(&mut self, buffer_id: BufferId) -> Result<(), String> {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            self.inner.focus_buffer(buffer_id)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            let _ = buffer_id;
            Ok(())
        }
    }

    pub(crate) fn focus_parent(&mut self) -> Result<(), String> {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            self.inner.focus_parent()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Ok(())
        }
    }

    pub(crate) fn open_devtools(&mut self, buffer_id: BufferId) -> Result<(), String> {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            self.inner.open_devtools(buffer_id)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            let _ = buffer_id;
            Ok(())
        }
    }

    pub(crate) fn drain_events(&mut self) -> Result<Vec<BrowserHostEvent>, String> {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            self.inner.drain_events()
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Ok(Vec::new())
        }
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
const BROWSER_HOME_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="color-scheme" content="dark light">
  <title>Volt Browser</title>
  <style>
    :root {
      color-scheme: dark light;
      font-family: Segoe UI, system-ui, sans-serif;
    }

    body {
      margin: 0;
      min-height: 100vh;
      display: flex;
      align-items: center;
      justify-content: center;
      background: #0f1014;
      color: #d7dde8;
    }

    main {
      max-width: 36rem;
      padding: 2rem;
      text-align: center;
      line-height: 1.5;
    }

    h1 {
      margin: 0 0 1rem;
      font-size: 1.5rem;
    }

    p {
      margin: 0.75rem 0;
      color: #aab4c7;
    }

    code {
      color: #8dc0ff;
    }
  </style>
</head>
<body>
  <main>
    <h1>Volt embedded browser</h1>
    <p>Enter a URL in the prompt below and press <code>Ctrl+Enter</code> to navigate.</p>
    <p>The browser surface is attached directly to this buffer, so links and page interaction stay inside the editor window.</p>
    <p>Once a page loads, click inside it to interact normally.</p>
  </main>
</body>
</html>
"#;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
const BROWSER_WEBVIEW_FOCUS_PARENT_IPC: &str = "__volt.focus_parent__";

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
const BROWSER_WEBVIEW_OPEN_DEVTOOLS_IPC: &str = "__volt.open_devtools__";

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
const BROWSER_WEBVIEW_INIT_SCRIPT: &str = r#"
window.addEventListener('keydown', (event) => {
  if (
    !event.defaultPrevented &&
    !event.altKey &&
    !event.metaKey &&
    (
      event.key === 'F12' ||
      (event.ctrlKey && event.shiftKey && (event.key === 'I' || event.key === 'i'))
    )
  ) {
    window.ipc.postMessage('__volt.open_devtools__');
    event.preventDefault();
    event.stopPropagation();
    return;
  }

  if (
    event.key === 'Escape' &&
    !event.defaultPrevented &&
    !event.altKey &&
    !event.ctrlKey &&
    !event.metaKey &&
    !event.shiftKey
  ) {
    window.ipc.postMessage('__volt.focus_parent__');
    event.preventDefault();
    event.stopPropagation();
  }
}, true);
"#;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
const BROWSER_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Code/1.112.0 Chrome/142.0.7444.265 Electron/39.8.0 Safari/537.36";

#[cfg(target_os = "windows")]
const BROWSER_DEFAULT_WEBVIEW2_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required";

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn allow_browser_drag_drop(_event: wry::DragDropEvent) -> bool {
    false
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn optional_non_empty_text(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

#[cfg(target_os = "windows")]
fn browser_additional_args_from_env(
    disable_web_security: Option<&str>,
    extra_args: Option<&str>,
) -> Option<String> {
    let disable_web_security = disable_web_security.is_some_and(|value| {
        let value = value.trim();
        value.eq_ignore_ascii_case("1")
            || value.eq_ignore_ascii_case("true")
            || value.eq_ignore_ascii_case("yes")
            || value.eq_ignore_ascii_case("on")
    });
    let extra_args = extra_args.map(str::trim).filter(|value| !value.is_empty());
    if !disable_web_security && extra_args.is_none() {
        return None;
    }
    let mut args = BROWSER_DEFAULT_WEBVIEW2_ARGS.to_owned();
    if disable_web_security {
        args.push_str(" --disable-web-security --allow-running-insecure-content");
    }
    if let Some(extra_args) = extra_args {
        args.push(' ');
        args.push_str(extra_args);
    }
    Some(args)
}

#[cfg(target_os = "windows")]
fn browser_additional_args() -> Option<String> {
    browser_additional_args_from_env(
        env::var("VOLT_BROWSER_DISABLE_WEB_SECURITY")
            .ok()
            .as_deref(),
        env::var("VOLT_BROWSER_ADDITIONAL_ARGS").ok().as_deref(),
    )
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn browser_host_event_for_ipc(buffer_id: BufferId, body: &str) -> Option<BrowserHostEvent> {
    match body {
        BROWSER_WEBVIEW_FOCUS_PARENT_IPC => {
            Some(BrowserHostEvent::FocusParentRequested { buffer_id })
        }
        BROWSER_WEBVIEW_OPEN_DEVTOOLS_IPC => {
            Some(BrowserHostEvent::OpenDevtoolsRequested { buffer_id })
        }
        _ => None,
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
const BROWSER_NAVIGATION_RETRY_INTERVAL: Duration = Duration::from_millis(500);

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
struct DesktopBrowserHostService {
    disabled_reason: Option<String>,
    event_tx: Sender<BrowserHostEvent>,
    event_rx: Receiver<BrowserHostEvent>,
    instances: BTreeMap<BufferId, BrowserInstance>,
    web_context: wry::WebContext,
    #[cfg(target_os = "linux")]
    gtk_initialized: bool,
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
impl DesktopBrowserHostService {
    fn new() -> Self {
        let (event_tx, event_rx) = mpsc::channel();
        Self {
            disabled_reason: None,
            event_tx,
            event_rx,
            instances: BTreeMap::new(),
            web_context: wry::WebContext::new(None),
            #[cfg(target_os = "linux")]
            gtk_initialized: false,
        }
    }

    #[cfg(target_os = "linux")]
    fn ensure_platform_ready(&mut self) -> Result<(), String> {
        if self.gtk_initialized {
            return Ok(());
        }
        gtk::init()
            .map_err(|error| format!("failed to initialize GTK for embedded browser: {error}"))?;
        self.gtk_initialized = true;
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn ensure_platform_ready(&mut self) -> Result<(), String> {
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn pump_platform_events(&self) {
        if !self.gtk_initialized {
            return;
        }
        while gtk::events_pending() {
            gtk::main_iteration_do(false);
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn pump_platform_events(&self) {}

    fn sync_window(
        &mut self,
        window: &Window,
        plan: &BrowserSyncPlan,
    ) -> Result<Vec<BrowserLocationUpdate>, String> {
        if self.disabled_reason.is_some() {
            return Ok(Vec::new());
        }

        let known_buffers = plan
            .buffers
            .iter()
            .map(|buffer| (buffer.buffer_id, buffer.current_url.as_deref()))
            .collect::<BTreeMap<_, _>>();
        let visible_surfaces = plan
            .visible_surfaces
            .iter()
            .copied()
            .map(|surface| (surface.buffer_id, surface))
            .collect::<BTreeMap<_, _>>();

        if (!visible_surfaces.is_empty() || !self.instances.is_empty())
            && let Err(error) = self.ensure_platform_ready()
        {
            return Err(self.disable(error));
        }

        let known_ids = known_buffers.keys().copied().collect::<BTreeSet<_>>();
        self.instances.retain(|buffer_id, instance| {
            let keep = known_ids.contains(buffer_id);
            if !keep {
                let _ = instance.webview.focus_parent();
            }
            keep
        });

        for (buffer_id, surface) in &visible_surfaces {
            if self.instances.contains_key(buffer_id) {
                continue;
            }
            let current_url = known_buffers.get(buffer_id).copied().flatten();
            let instance = match BrowserInstance::new(
                *buffer_id,
                window,
                surface.rect,
                current_url,
                self.event_tx.clone(),
                &mut self.web_context,
            ) {
                Ok(instance) => instance,
                Err(error) => return Err(self.disable(error)),
            };
            self.instances.insert(*buffer_id, instance);
        }

        let mut location_updates = Vec::new();
        let mut sync_error = None;
        for (buffer_id, instance) in &mut self.instances {
            let current_url = known_buffers.get(buffer_id).copied().flatten();
            let visible_surface = visible_surfaces.get(buffer_id).copied();
            if let Err(error) = instance.sync(visible_surface, current_url, &mut location_updates) {
                sync_error = Some(error);
                break;
            }
        }
        if let Some(error) = sync_error {
            return Err(self.disable(error));
        }

        self.pump_platform_events();
        Ok(location_updates)
    }

    fn disable(&mut self, error: String) -> String {
        if self.disabled_reason.is_none() {
            self.disabled_reason = Some(error.clone());
        }
        error
    }

    fn focus_buffer(&mut self, buffer_id: BufferId) -> Result<(), String> {
        let Some(instance) = self.instances.get(&buffer_id) else {
            return Ok(());
        };
        instance
            .webview
            .focus()
            .map_err(|error| format!("failed to focus embedded browser: {error}"))
    }

    fn focus_parent(&mut self) -> Result<(), String> {
        for instance in self.instances.values() {
            instance
                .webview
                .focus_parent()
                .map_err(|error| format!("failed to focus browser parent window: {error}"))?;
        }
        Ok(())
    }

    fn open_devtools(&mut self, buffer_id: BufferId) -> Result<(), String> {
        let Some(instance) = self.instances.get(&buffer_id) else {
            return Ok(());
        };
        instance.open_devtools();
        Ok(())
    }

    fn drain_events(&mut self) -> Result<Vec<BrowserHostEvent>, String> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        Ok(events)
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
struct BrowserInstance {
    webview: wry::WebView,
    desired_url: Option<String>,
    last_reported_url: Option<String>,
    last_navigation_attempt_url: Option<String>,
    last_navigation_attempt_at: Option<Instant>,
    last_bounds: BrowserViewportRect,
    visible: bool,
    showing_home: bool,
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
impl BrowserInstance {
    fn new(
        buffer_id: BufferId,
        window: &Window,
        bounds: BrowserViewportRect,
        current_url: Option<&str>,
        event_tx: Sender<BrowserHostEvent>,
        web_context: &mut wry::WebContext,
    ) -> Result<Self, String> {
        let ipc_event_tx = event_tx.clone();
        let title_event_tx = event_tx.clone();
        let page_load_event_tx = event_tx.clone();
        let new_window_event_tx = event_tx;
        let builder = wry::WebViewBuilder::new_with_web_context(web_context)
            .with_visible(true)
            .with_bounds(to_wry_rect(bounds))
            .with_devtools(true)
            .with_hotkeys_zoom(true)
            .with_back_forward_navigation_gestures(true)
            .with_autoplay(true)
            .with_clipboard(true)
            .with_user_agent(BROWSER_USER_AGENT)
            .with_navigation_handler(|_| true)
            .with_drag_drop_handler(allow_browser_drag_drop)
            .with_initialization_script(BROWSER_WEBVIEW_INIT_SCRIPT)
            .with_document_title_changed_handler(move |title| {
                let _ = title_event_tx.send(BrowserHostEvent::DocumentTitleChanged {
                    buffer_id,
                    title: optional_non_empty_text(&title),
                });
            })
            .with_on_page_load_handler(move |event, url| {
                if let Some(current_url) = optional_non_empty_text(&url) {
                    let _ = page_load_event_tx.send(BrowserHostEvent::PageLoadStateChanged {
                        buffer_id,
                        current_url,
                        is_loading: matches!(event, wry::PageLoadEvent::Started),
                    });
                }
            })
            .with_new_window_req_handler(move |url, _features| {
                if let Some(url) = optional_non_empty_text(&url) {
                    let _ = new_window_event_tx
                        .send(BrowserHostEvent::NewWindowRequested { buffer_id, url });
                }
                wry::NewWindowResponse::Deny
            })
            .with_ipc_handler(move |request| {
                if let Some(event) = browser_host_event_for_ipc(buffer_id, request.body()) {
                    let _ = ipc_event_tx.send(event);
                }
            });
        #[cfg(target_os = "windows")]
        let builder = {
            let builder = builder.with_browser_accelerator_keys(false);
            if let Some(additional_args) = browser_additional_args() {
                builder.with_additional_browser_args(additional_args)
            } else {
                builder
            }
        };
        #[cfg(target_os = "macos")]
        let builder = builder.with_accept_first_mouse(true);
        let webview = match current_url {
            Some(url) => builder.with_url(url).build_as_child(window),
            None => builder.with_html(BROWSER_HOME_HTML).build_as_child(window),
        }
        .map_err(|error| format!("failed to create embedded browser: {error}"))?;
        webview.focus_parent().map_err(|error| {
            format!("failed to restore parent focus after browser creation: {error}")
        })?;
        Ok(Self {
            webview,
            desired_url: current_url.map(str::to_owned),
            last_reported_url: None,
            last_navigation_attempt_url: current_url.map(str::to_owned),
            last_navigation_attempt_at: current_url.map(|_| Instant::now()),
            last_bounds: bounds,
            visible: true,
            showing_home: current_url.is_none(),
        })
    }

    fn open_devtools(&self) {
        self.webview.open_devtools();
    }

    fn sync(
        &mut self,
        visible_surface: Option<BrowserSurfacePlan>,
        current_url: Option<&str>,
        location_updates: &mut Vec<BrowserLocationUpdate>,
    ) -> Result<(), String> {
        match visible_surface {
            Some(surface) => {
                if self.last_bounds != surface.rect {
                    self.webview
                        .set_bounds(to_wry_rect(surface.rect))
                        .map_err(|error| format!("failed to resize embedded browser: {error}"))?;
                    self.last_bounds = surface.rect;
                }
                if !self.visible {
                    self.webview
                        .set_visible(true)
                        .map_err(|error| format!("failed to show embedded browser: {error}"))?;
                    self.visible = true;
                }
                self.sync_navigation(surface.buffer_id, current_url, location_updates)?;
            }
            None => {
                if self.visible {
                    self.webview
                        .set_visible(false)
                        .map_err(|error| format!("failed to hide embedded browser: {error}"))?;
                    let _ = self.webview.focus_parent();
                    self.visible = false;
                }
            }
        }
        Ok(())
    }

    fn sync_navigation(
        &mut self,
        buffer_id: BufferId,
        desired_url: Option<&str>,
        location_updates: &mut Vec<BrowserLocationUpdate>,
    ) -> Result<(), String> {
        if self.desired_url.as_deref() != desired_url {
            self.desired_url = desired_url.map(str::to_owned);
            self.last_navigation_attempt_url = None;
            self.last_navigation_attempt_at = None;
        }
        match desired_url {
            Some(url) => {
                if self.last_reported_url.as_deref() != Some(url)
                    && browser_navigation_retry_required(
                        Some(url),
                        self.last_reported_url.as_deref(),
                        self.last_navigation_attempt_url.as_deref(),
                        self.last_navigation_attempt_at
                            .map(|attempt| attempt.elapsed()),
                    )
                {
                    self.webview.load_url(url).map_err(|error| {
                        format!("failed to navigate embedded browser to `{url}`: {error}")
                    })?;
                    self.last_navigation_attempt_url = Some(url.to_owned());
                    self.last_navigation_attempt_at = Some(Instant::now());
                }
                self.showing_home = false;
            }
            None => {
                if !self.showing_home {
                    self.webview.load_html(BROWSER_HOME_HTML).map_err(|error| {
                        format!("failed to load embedded browser welcome page: {error}")
                    })?;
                    self.last_navigation_attempt_url = None;
                    self.last_navigation_attempt_at = None;
                    self.last_reported_url = None;
                    self.showing_home = true;
                }
            }
        }
        if desired_url.is_some()
            && let Ok(url) = self.webview.url()
            && let Some(url) = optional_non_empty_text(&url)
            && self.last_reported_url.as_deref() != Some(url.as_str())
        {
            self.last_reported_url = Some(url.clone());
            location_updates.push(BrowserLocationUpdate {
                buffer_id,
                current_url: url,
            });
        }
        Ok(())
    }
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn browser_navigation_retry_required(
    desired_url: Option<&str>,
    reported_url: Option<&str>,
    last_attempt_url: Option<&str>,
    last_attempt_age: Option<Duration>,
) -> bool {
    let Some(desired_url) = desired_url else {
        return false;
    };
    if reported_url == Some(desired_url) {
        return false;
    }
    if last_attempt_url != Some(desired_url) {
        return true;
    }
    last_attempt_age.is_none_or(|age| age >= BROWSER_NAVIGATION_RETRY_INTERVAL)
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
fn to_wry_rect(rect: BrowserViewportRect) -> wry::Rect {
    // Wry child-webview bounds are expressed in physical window pixels, which matches the SDL
    // render coordinates we compute for pane and popup layouts.
    wry::Rect {
        position: wry::dpi::PhysicalPosition::new(rect.x, rect.y).into(),
        size: wry::dpi::PhysicalSize::new(rect.width, rect.height).into(),
    }
}

#[cfg(all(
    test,
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
mod tests {
    use super::*;

    fn browser_test_buffer_id() -> BufferId {
        let mut runtime = editor_core::EditorRuntime::new();
        let window_id = runtime.model_mut().create_window("browser-host-test");
        let workspace_id = runtime
            .model_mut()
            .open_workspace(window_id, "browser-host", None)
            .expect("workspace should be created for browser host tests");
        runtime
            .model_mut()
            .create_buffer(
                workspace_id,
                "*browser*",
                editor_core::BufferKind::Scratch,
                None,
            )
            .expect("buffer id should be created for browser host tests")
    }

    #[test]
    fn browser_drag_drop_handler_preserves_default_webview_behavior() {
        assert!(!allow_browser_drag_drop(wry::DragDropEvent::Leave));
    }

    #[test]
    fn browser_host_ipc_event_routes_focus_parent_requests() {
        let buffer_id = browser_test_buffer_id();
        assert_eq!(
            browser_host_event_for_ipc(buffer_id, BROWSER_WEBVIEW_FOCUS_PARENT_IPC),
            Some(BrowserHostEvent::FocusParentRequested { buffer_id })
        );
    }

    #[test]
    fn browser_host_ipc_event_routes_open_devtools_requests() {
        let buffer_id = browser_test_buffer_id();
        assert_eq!(
            browser_host_event_for_ipc(buffer_id, BROWSER_WEBVIEW_OPEN_DEVTOOLS_IPC),
            Some(BrowserHostEvent::OpenDevtoolsRequested { buffer_id })
        );
    }

    #[test]
    fn browser_host_ipc_event_ignores_unknown_messages() {
        let buffer_id = browser_test_buffer_id();
        assert_eq!(
            browser_host_event_for_ipc(buffer_id, "__volt.unknown__"),
            None
        );
    }

    #[test]
    fn wry_rect_preserves_physical_pixel_bounds() {
        let rect = to_wry_rect(BrowserViewportRect {
            x: 48,
            y: 96,
            width: 640,
            height: 360,
        });

        let position = rect.position.to_physical::<i32>(1.0);
        let size = rect.size.to_physical::<u32>(1.0);

        assert_eq!((position.x, position.y), (48, 96));
        assert_eq!((size.width, size.height), (640, 360));
    }

    #[test]
    fn browser_user_agent_header_is_valid() {
        let user_agent = wry::http::HeaderValue::from_str(BROWSER_USER_AGENT)
            .expect("browser user agent should be valid");
        assert_eq!(user_agent.to_str().ok(), Some(BROWSER_USER_AGENT));
    }

    #[test]
    fn optional_non_empty_text_trims_blank_values() {
        assert_eq!(optional_non_empty_text("  Volt  "), Some("Volt".to_owned()));
        assert_eq!(optional_non_empty_text("   "), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn browser_additional_args_from_env_is_empty_without_flags() {
        assert_eq!(browser_additional_args_from_env(None, None), None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn browser_additional_args_from_env_appends_web_security_bypass() {
        let args = browser_additional_args_from_env(Some("1"), None)
            .expect("web security bypass args should be built");
        assert!(args.contains("--disable-web-security"));
        assert!(args.contains("--allow-running-insecure-content"));
        assert!(args.contains("--autoplay-policy=no-user-gesture-required"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn browser_additional_args_from_env_appends_custom_args() {
        let args =
            browser_additional_args_from_env(Some("false"), Some("--remote-debugging-port=0"))
                .expect("custom browser args should be built");
        assert!(args.contains(BROWSER_DEFAULT_WEBVIEW2_ARGS));
        assert!(args.contains("--remote-debugging-port=0"));
        assert!(!args.contains("--disable-web-security"));
    }

    #[test]
    fn browser_navigation_retry_is_required_for_new_targets() {
        assert!(browser_navigation_retry_required(
            Some("https://example.com"),
            None,
            None,
            None
        ));
    }

    #[test]
    fn browser_navigation_retry_waits_for_retry_interval() {
        assert!(!browser_navigation_retry_required(
            Some("https://example.com"),
            None,
            Some("https://example.com"),
            Some(Duration::from_millis(200)),
        ));
        assert!(browser_navigation_retry_required(
            Some("https://example.com"),
            None,
            Some("https://example.com"),
            Some(Duration::from_millis(600)),
        ));
    }

    #[test]
    fn browser_navigation_retry_stops_after_target_url_is_reported() {
        assert!(!browser_navigation_retry_required(
            Some("https://example.com"),
            Some("https://example.com"),
            Some("https://example.com"),
            Some(Duration::from_millis(600)),
        ));
    }
}
