use editor_theme::ThemeRegistry;
use sdl3::video::{Window, WindowFlags};
use std::sync::atomic::{AtomicU8, Ordering};

#[cfg(target_os = "linux")]
use std::ffi::CStr;

use crate::ShellError;

pub(crate) const OPTION_WINDOW_OPACITY: &str = "window.opacity";
pub(crate) const OPTION_WINDOW_BLUR: &str = "window.blur";

const DEFAULT_WINDOW_OPACITY: f32 = 1.0;
const DEFAULT_WINDOW_BLUR: f32 = 0.0;
const SDL_VIDEO_DRIVER_X11: &str = "x11";
const SDL_VIDEO_DRIVER_WAYLAND: &str = "wayland";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowOpacityMode {
    Surface = 0,
    NativeWindow = 1,
}

impl WindowOpacityMode {
    fn from_stored(value: u8) -> Self {
        match value {
            1 => Self::NativeWindow,
            _ => Self::Surface,
        }
    }
}

static WINDOW_OPACITY_MODE: AtomicU8 = AtomicU8::new(WindowOpacityMode::Surface as u8);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WindowEffects {
    pub(crate) opacity: f32,
    pub(crate) blur: f32,
}

impl Default for WindowEffects {
    fn default() -> Self {
        Self {
            opacity: DEFAULT_WINDOW_OPACITY,
            blur: DEFAULT_WINDOW_BLUR,
        }
    }
}

impl WindowEffects {
    pub(crate) fn resolve(theme_registry: Option<&ThemeRegistry>) -> Self {
        let opacity = theme_registry
            .and_then(|registry| registry.resolve_number(OPTION_WINDOW_OPACITY))
            .map(normalize_window_opacity)
            .unwrap_or(DEFAULT_WINDOW_OPACITY);
        let blur = theme_registry
            .and_then(|registry| registry.resolve_number(OPTION_WINDOW_BLUR))
            .map(normalize_window_blur)
            .unwrap_or(DEFAULT_WINDOW_BLUR);
        Self { opacity, blur }
    }
}

pub(crate) fn window_creation_flags(settings: WindowEffects) -> WindowFlags {
    let _ = settings;
    // CONTEXT: live theme reload can enable opacity or blur after startup, so
    // the SDL window needs a compositor-backed transparent surface from launch.
    WindowFlags::TRANSPARENT
}

trait NativeWindowEffectsTarget {
    fn set_native_window_opacity(&mut self, opacity: f32) -> Result<(), String>;
    fn apply_native_window_blur(&mut self, blur: f32) -> Result<(), String>;
    fn clear_native_window_blur(&mut self) -> Result<(), String>;
}

impl NativeWindowEffectsTarget for Window {
    fn set_native_window_opacity(&mut self, opacity: f32) -> Result<(), String> {
        self.set_opacity(opacity).map_err(|error| error.to_string())
    }

    fn apply_native_window_blur(&mut self, blur: f32) -> Result<(), String> {
        platform::apply_blur(self, blur)
    }

    fn clear_native_window_blur(&mut self) -> Result<(), String> {
        platform::clear_blur(self)
    }
}

pub(crate) fn current_window_effect_settings(
    theme_registry: Option<&ThemeRegistry>,
) -> WindowEffects {
    WindowEffects::resolve(theme_registry)
}

pub(crate) fn normalize_window_opacity(value: f64) -> f32 {
    if !value.is_finite() {
        return DEFAULT_WINDOW_OPACITY;
    }
    value.clamp(0.0, 1.0) as f32
}

pub(crate) fn normalize_window_blur(value: f64) -> f32 {
    if !value.is_finite() {
        return DEFAULT_WINDOW_BLUR;
    }
    value.clamp(0.0, f64::from(f32::MAX)) as f32
}

pub(crate) fn window_surface_opacity(settings: WindowEffects) -> f32 {
    match current_window_opacity_mode() {
        WindowOpacityMode::NativeWindow => DEFAULT_WINDOW_OPACITY,
        WindowOpacityMode::Surface => settings.opacity,
    }
}

pub(crate) fn apply_window_effects(
    window: &mut Window,
    settings: WindowEffects,
) -> Result<(), ShellError> {
    apply_window_effects_to_target(window, settings)
}

pub(crate) fn update_window_effects(
    window: &mut Window,
    previous: WindowEffects,
    next: WindowEffects,
) -> Result<(), ShellError> {
    update_window_effects_to_target(window, previous, next)
}

fn apply_window_effects_to_target(
    window: &mut impl NativeWindowEffectsTarget,
    settings: WindowEffects,
) -> Result<(), ShellError> {
    // CONTEXT: most platforms keep window.opacity on renderer-owned background
    // surfaces so text stays fully opaque. Linux X11/Wayland compositors are
    // more reliable with native SDL window opacity, so that path can override
    // renderer-side alpha when it succeeds.
    set_window_opacity_mode(sync_window_opacity(
        window,
        settings.opacity,
        requested_window_opacity_mode(),
    ));
    apply_window_blur(window, settings.blur)
}

fn update_window_effects_to_target(
    window: &mut impl NativeWindowEffectsTarget,
    previous: WindowEffects,
    next: WindowEffects,
) -> Result<(), ShellError> {
    if previous.opacity != next.opacity {
        set_window_opacity_mode(sync_window_opacity(
            window,
            next.opacity,
            requested_window_opacity_mode(),
        ));
    }
    if next.blur > DEFAULT_WINDOW_BLUR {
        return apply_window_blur(window, next.blur);
    }
    if previous.blur > DEFAULT_WINDOW_BLUR {
        return clear_window_blur(window);
    }
    Ok(())
}

fn apply_window_blur(
    window: &mut impl NativeWindowEffectsTarget,
    blur: f32,
) -> Result<(), ShellError> {
    if blur <= DEFAULT_WINDOW_BLUR {
        return Ok(());
    }

    window.apply_native_window_blur(blur).map_err(|error| {
        ShellError::Runtime(format!(
            "failed to apply {OPTION_WINDOW_BLUR}={blur}: {error}"
        ))
    })
}

fn clear_window_blur(window: &mut impl NativeWindowEffectsTarget) -> Result<(), ShellError> {
    window.clear_native_window_blur().map_err(|error| {
        ShellError::Runtime(format!("failed to clear {OPTION_WINDOW_BLUR}: {error}"))
    })
}

fn current_window_opacity_mode() -> WindowOpacityMode {
    WindowOpacityMode::from_stored(WINDOW_OPACITY_MODE.load(Ordering::Relaxed))
}

fn set_window_opacity_mode(mode: WindowOpacityMode) {
    WINDOW_OPACITY_MODE.store(mode as u8, Ordering::Relaxed);
}

fn sync_window_opacity(
    window: &mut impl NativeWindowEffectsTarget,
    opacity: f32,
    requested_mode: WindowOpacityMode,
) -> WindowOpacityMode {
    if requested_mode != WindowOpacityMode::NativeWindow {
        return WindowOpacityMode::Surface;
    }
    match window.set_native_window_opacity(opacity) {
        Ok(()) => WindowOpacityMode::NativeWindow,
        Err(_) => WindowOpacityMode::Surface,
    }
}

fn requested_window_opacity_mode() -> WindowOpacityMode {
    #[cfg(target_os = "linux")]
    {
        return window_opacity_mode_for_driver(current_video_driver_name().as_deref());
    }

    #[cfg(not(target_os = "linux"))]
    {
        WindowOpacityMode::Surface
    }
}

#[cfg(any(test, target_os = "linux"))]
fn window_opacity_mode_for_driver(driver: Option<&str>) -> WindowOpacityMode {
    match driver {
        Some(SDL_VIDEO_DRIVER_X11 | SDL_VIDEO_DRIVER_WAYLAND) => WindowOpacityMode::NativeWindow,
        _ => WindowOpacityMode::Surface,
    }
}

#[cfg(target_os = "linux")]
fn current_video_driver_name() -> Option<String> {
    unsafe {
        let driver = sdl3::sys::video::SDL_GetCurrentVideoDriver();
        if driver.is_null() {
            return None;
        }
        // SAFETY: SDL owns this NUL-terminated driver name for the lifetime of the
        // initialized video subsystem, and we copy it into an owned String
        // immediately before returning.
        CStr::from_ptr(driver).to_str().ok().map(str::to_owned)
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use sdl3::video::Window;

    pub(super) fn apply_blur(window: &Window, blur: f32) -> Result<(), String> {
        let _ = blur;
        window_vibrancy::apply_blur(window, None).map_err(|error| {
            format!("Windows compositor blur is unavailable for this SDL window: {error}")
        })
    }

    pub(super) fn clear_blur(window: &Window) -> Result<(), String> {
        window_vibrancy::clear_blur(window).map_err(|error| {
            format!("Windows compositor blur could not be cleared for this SDL window: {error}")
        })
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use sdl3::video::Window;
    use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};

    pub(super) fn apply_blur(window: &Window, blur: f32) -> Result<(), String> {
        apply_vibrancy(
            window,
            NSVisualEffectMaterial::UnderWindowBackground,
            None,
            Some(f64::from(blur)),
        )
        .map_err(|error| format!("macOS vibrancy is unavailable for this SDL window: {error}"))
    }

    pub(super) fn clear_blur(window: &Window) -> Result<(), String> {
        window_vibrancy::clear_vibrancy(window)
            .map(|_| ())
            .map_err(|error| {
                format!("macOS vibrancy could not be cleared for this SDL window: {error}")
            })
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use sdl3::video::Window;

    pub(super) fn apply_blur(_window: &Window, _blur: f32) -> Result<(), String> {
        // Linux compositor blur remains backend-specific; keep this as an
        // intentional no-op so window.opacity can still be applied.
        Ok(())
    }

    pub(super) fn clear_blur(_window: &Window) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod platform {
    use sdl3::video::Window;

    pub(super) fn apply_blur(_window: &Window, blur: f32) -> Result<(), String> {
        Err(format!(
            "window blur is not implemented for this target platform (requested {blur})"
        ))
    }

    pub(super) fn clear_blur(_window: &Window) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NativeWindowEffectsTarget, OPTION_WINDOW_BLUR, OPTION_WINDOW_OPACITY, WindowEffects,
        WindowOpacityMode, apply_window_effects_to_target, current_window_opacity_mode,
        normalize_window_blur, normalize_window_opacity, set_window_opacity_mode,
        sync_window_opacity, update_window_effects_to_target, window_creation_flags,
        window_opacity_mode_for_driver,
    };
    use editor_theme::{Theme, ThemeRegistry};
    use sdl3::video::WindowFlags;

    #[derive(Default)]
    struct RecordingWindow {
        opacity_calls: Vec<f32>,
        blur_calls: Vec<f32>,
        clear_calls: usize,
        opacity_error: Option<String>,
        blur_error: Option<String>,
        clear_error: Option<String>,
    }

    impl NativeWindowEffectsTarget for RecordingWindow {
        fn set_native_window_opacity(&mut self, opacity: f32) -> Result<(), String> {
            self.opacity_calls.push(opacity);
            match &self.opacity_error {
                Some(error) => Err(error.clone()),
                None => Ok(()),
            }
        }

        fn apply_native_window_blur(&mut self, blur: f32) -> Result<(), String> {
            self.blur_calls.push(blur);
            match &self.blur_error {
                Some(error) => Err(error.clone()),
                None => Ok(()),
            }
        }

        fn clear_native_window_blur(&mut self) -> Result<(), String> {
            self.clear_calls += 1;
            match &self.clear_error {
                Some(error) => Err(error.clone()),
                None => Ok(()),
            }
        }
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    #[test]
    fn window_effects_default_to_opaque_without_theme_values() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        assert_eq!(WindowEffects::resolve(None), WindowEffects::default());
    }

    #[test]
    fn window_effects_resolve_normalized_theme_values() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        let mut registry = ThemeRegistry::new();
        must(
            registry.register(
                Theme::new("test-theme", "Test Theme")
                    .with_option(OPTION_WINDOW_OPACITY, 1.4)
                    .with_option(OPTION_WINDOW_BLUR, -6.0),
            ),
        );

        assert_eq!(
            WindowEffects::resolve(Some(&registry)),
            WindowEffects {
                opacity: 1.0,
                blur: 0.0,
            }
        );
    }

    #[test]
    fn window_effect_normalizers_handle_non_finite_values() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        assert_eq!(normalize_window_opacity(f64::NAN), 1.0);
        assert_eq!(normalize_window_blur(f64::NEG_INFINITY), 0.0);
        assert_eq!(normalize_window_blur(f64::INFINITY), 0.0);
        assert_eq!(normalize_window_blur(f64::from(f32::MAX) * 2.0), f32::MAX);
    }

    #[test]
    fn window_creation_flags_always_request_transparent_surface_for_live_updates() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        assert!(
            window_creation_flags(WindowEffects {
                opacity: 0.75,
                blur: 0.0,
            })
            .contains(WindowFlags::TRANSPARENT)
        );
        assert!(
            window_creation_flags(WindowEffects {
                opacity: 1.0,
                blur: 12.0,
            })
            .contains(WindowFlags::TRANSPARENT)
        );
        assert!(window_creation_flags(WindowEffects::default()).contains(WindowFlags::TRANSPARENT));
    }

    #[test]
    fn apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        let mut window = RecordingWindow::default();

        must(apply_window_effects_to_target(
            &mut window,
            WindowEffects {
                opacity: 0.5,
                blur: 0.0,
            },
        ));

        assert!(window.opacity_calls.is_empty());
        assert!(window.blur_calls.is_empty());
        assert_eq!(window.clear_calls, 0);
    }

    #[test]
    fn apply_window_effects_still_calls_native_blur_backend_when_requested() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        let mut window = RecordingWindow::default();

        must(apply_window_effects_to_target(
            &mut window,
            WindowEffects {
                opacity: 0.5,
                blur: 18.0,
            },
        ));

        assert!(window.opacity_calls.is_empty());
        assert_eq!(window.blur_calls, vec![18.0]);
        assert_eq!(window.clear_calls, 0);
    }

    #[test]
    fn update_window_effects_clears_native_blur_when_disabled() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        let mut window = RecordingWindow::default();

        must(update_window_effects_to_target(
            &mut window,
            WindowEffects {
                opacity: 1.0,
                blur: 18.0,
            },
            WindowEffects {
                opacity: 1.0,
                blur: 0.0,
            },
        ));

        assert!(window.opacity_calls.is_empty());
        assert!(window.blur_calls.is_empty());
        assert_eq!(window.clear_calls, 1);
    }

    #[test]
    fn linux_native_window_opacity_targets_x11_and_wayland_only() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        assert_eq!(
            window_opacity_mode_for_driver(Some("x11")),
            WindowOpacityMode::NativeWindow
        );
        assert_eq!(
            window_opacity_mode_for_driver(Some("wayland")),
            WindowOpacityMode::NativeWindow
        );
        assert_eq!(
            window_opacity_mode_for_driver(Some("cocoa")),
            WindowOpacityMode::Surface
        );
        assert_eq!(
            window_opacity_mode_for_driver(None),
            WindowOpacityMode::Surface
        );
    }

    #[test]
    fn sync_window_opacity_uses_native_window_when_supported() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        let mut window = RecordingWindow::default();

        let mode = sync_window_opacity(&mut window, 0.4, WindowOpacityMode::NativeWindow);
        set_window_opacity_mode(mode);

        assert_eq!(mode, WindowOpacityMode::NativeWindow);
        assert_eq!(
            current_window_opacity_mode(),
            WindowOpacityMode::NativeWindow
        );
        assert_eq!(window.opacity_calls, vec![0.4]);
    }

    #[test]
    fn sync_window_opacity_falls_back_to_surface_when_native_call_fails() {
        set_window_opacity_mode(WindowOpacityMode::Surface);
        let mut window = RecordingWindow {
            opacity_error: Some("unsupported".to_owned()),
            ..RecordingWindow::default()
        };

        let mode = sync_window_opacity(&mut window, 0.4, WindowOpacityMode::NativeWindow);
        set_window_opacity_mode(mode);

        assert_eq!(mode, WindowOpacityMode::Surface);
        assert_eq!(current_window_opacity_mode(), WindowOpacityMode::Surface);
        assert_eq!(window.opacity_calls, vec![0.4]);
    }
}
