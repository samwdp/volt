use std::{
    collections::{BTreeMap, BTreeSet},
    sync::mpsc::{self, Receiver, Sender},
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
    FocusParentRequested { buffer_id: BufferId },
    OpenDevtoolsRequested { buffer_id: BufferId },
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
fn allow_browser_drag_drop(_event: wry::DragDropEvent) -> bool {
    false
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
struct DesktopBrowserHostService {
    disabled_reason: Option<String>,
    event_tx: Sender<BrowserHostEvent>,
    event_rx: Receiver<BrowserHostEvent>,
    instances: BTreeMap<BufferId, BrowserInstance>,
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
    last_requested_url: Option<String>,
    last_reported_url: Option<String>,
    last_bounds: BrowserViewportRect,
    visible: bool,
}

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
impl BrowserInstance {
    fn new(
        buffer_id: BufferId,
        window: &Window,
        bounds: BrowserViewportRect,
        current_url: Option<&str>,
        event_tx: Sender<BrowserHostEvent>,
    ) -> Result<Self, String> {
        let builder = wry::WebViewBuilder::new()
            .with_visible(true)
            .with_bounds(to_wry_rect(bounds))
            .with_devtools(true)
            .with_drag_drop_handler(allow_browser_drag_drop)
            .with_initialization_script(BROWSER_WEBVIEW_INIT_SCRIPT)
            .with_ipc_handler(move |request| {
                if let Some(event) = browser_host_event_for_ipc(buffer_id, request.body()) {
                    let _ = event_tx.send(event);
                }
            });
        #[cfg(target_os = "windows")]
        let builder = builder.with_browser_accelerator_keys(false);
        let webview = match current_url {
            Some(url) => builder.with_url(url).build_as_child(window),
            None => builder.with_html(BROWSER_HOME_HTML).build_as_child(window),
        }
        .map_err(|error| format!("failed to create embedded browser: {error}"))?;
        Ok(Self {
            webview,
            last_requested_url: current_url.map(str::to_owned),
            last_reported_url: current_url.map(str::to_owned),
            last_bounds: bounds,
            visible: true,
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
                if self.last_requested_url.as_deref() != current_url {
                    match current_url {
                        Some(url) => {
                            self.webview.load_url(url).map_err(|error| {
                                format!("failed to navigate embedded browser to `{url}`: {error}")
                            })?;
                        }
                        None => {
                            self.webview.load_html(BROWSER_HOME_HTML).map_err(|error| {
                                format!("failed to load embedded browser welcome page: {error}")
                            })?;
                        }
                    }
                    self.last_requested_url = current_url.map(str::to_owned);
                }
                if let Ok(url) = self.webview.url()
                    && !url.trim().is_empty()
                    && self.last_reported_url.as_deref() != Some(url.as_str())
                {
                    self.last_reported_url = Some(url.clone());
                    location_updates.push(BrowserLocationUpdate {
                        buffer_id: surface.buffer_id,
                        current_url: url,
                    });
                }
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
}
