use editor_theme::ThemeRegistry;
use sdl3::video::{Window, WindowFlags};
use std::sync::atomic::{AtomicU8, Ordering};
#[cfg(test)]
use std::sync::{Mutex, MutexGuard};

use crate::ShellError;

pub(crate) const OPTION_WINDOW_OPACITY: &str = "window.opacity";
pub(crate) const OPTION_WINDOW_BLUR: &str = "window.blur";
pub(crate) const OPTION_WINDOW_TRANSPARENCY: &str = "window.transparency";

const DEFAULT_WINDOW_OPACITY: f32 = 1.0;
const DEFAULT_WINDOW_BLUR: f32 = 0.0;
const SDL_VIDEO_DRIVER_X11: &str = "x11";
const SDL_VIDEO_DRIVER_WAYLAND: &str = "wayland";

/// Windows compositor material selected by `window.transparency` in `global.toml`.
///
/// Values (case-insensitive):
/// - `none` — no compositor backdrop effect
/// - `blur` — classic Windows blur (Win10+)
/// - `acrylic` — frosted acrylic (usually more transparent than blur; Win10+)
/// - `mica` — Windows 11 mica
/// - `mica-tabbed` / `tabbed` — Windows 11 tabbed mica
///
/// When the option is omitted, a positive `window.blur` still enables `blur`
/// for backward compatibility. On macOS any non-`none` type maps to vibrancy
/// and `window.blur` is the corner radius. Linux ignores the material type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum WindowTransparency {
    #[default]
    None = 0,
    Blur = 1,
    Acrylic = 2,
    Mica = 3,
    MicaTabbed = 4,
}

impl WindowTransparency {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "none" | "off" | "opaque" => Some(Self::None),
            "blur" => Some(Self::Blur),
            "acrylic" => Some(Self::Acrylic),
            "mica" => Some(Self::Mica),
            "mica-tabbed" | "tabbed" | "mica_tabbed" => Some(Self::MicaTabbed),
            _ => None,
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Blur => "blur",
            Self::Acrylic => "acrylic",
            Self::Mica => "mica",
            Self::MicaTabbed => "mica-tabbed",
        }
    }

    pub(crate) fn is_enabled(self) -> bool {
        self != Self::None
    }
}

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
static REQUESTED_WINDOW_OPACITY_MODE: AtomicU8 = AtomicU8::new(WindowOpacityMode::Surface as u8);
#[cfg(test)]
static WINDOW_EFFECTS_TEST_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct WindowEffects {
    pub(crate) opacity: f32,
    pub(crate) blur: f32,
    pub(crate) transparency: WindowTransparency,
}

impl Default for WindowEffects {
    fn default() -> Self {
        Self {
            opacity: DEFAULT_WINDOW_OPACITY,
            blur: DEFAULT_WINDOW_BLUR,
            transparency: WindowTransparency::None,
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
        let transparency = resolve_window_transparency(theme_registry, blur);
        Self {
            opacity,
            blur,
            transparency,
        }
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
    fn apply_native_window_transparency(
        &mut self,
        transparency: WindowTransparency,
        blur: f32,
    ) -> Result<(), String>;
    fn clear_native_window_transparency(
        &mut self,
        transparency: WindowTransparency,
    ) -> Result<(), String>;
}

impl NativeWindowEffectsTarget for Window {
    fn set_native_window_opacity(&mut self, opacity: f32) -> Result<(), String> {
        self.set_opacity(opacity).map_err(|error| error.to_string())
    }

    fn apply_native_window_transparency(
        &mut self,
        transparency: WindowTransparency,
        blur: f32,
    ) -> Result<(), String> {
        platform::apply_transparency(self, transparency, blur)
    }

    fn clear_native_window_transparency(
        &mut self,
        transparency: WindowTransparency,
    ) -> Result<(), String> {
        platform::clear_transparency(self, transparency)
    }
}

pub(crate) fn current_window_effect_settings(
    theme_registry: Option<&ThemeRegistry>,
) -> WindowEffects {
    WindowEffects::resolve(theme_registry)
}

pub(crate) fn configure_window_opacity_driver(driver: Option<&str>) {
    REQUESTED_WINDOW_OPACITY_MODE.store(
        window_opacity_mode_for_driver(driver) as u8,
        Ordering::Relaxed,
    );
}

#[cfg(test)]
pub(crate) struct WindowEffectsTestGuard {
    _lock: MutexGuard<'static, ()>,
    previous_window_opacity_mode: u8,
    previous_requested_window_opacity_mode: u8,
}

#[cfg(test)]
impl Drop for WindowEffectsTestGuard {
    fn drop(&mut self) {
        WINDOW_OPACITY_MODE.store(self.previous_window_opacity_mode, Ordering::Relaxed);
        REQUESTED_WINDOW_OPACITY_MODE.store(
            self.previous_requested_window_opacity_mode,
            Ordering::Relaxed,
        );
    }
}

#[cfg(test)]
pub(crate) fn lock_window_effects_for_tests() -> WindowEffectsTestGuard {
    let lock = match WINDOW_EFFECTS_TEST_LOCK.lock() {
        Ok(lock) => lock,
        Err(poisoned) => poisoned.into_inner(),
    };
    WindowEffectsTestGuard {
        previous_window_opacity_mode: WINDOW_OPACITY_MODE.load(Ordering::Relaxed),
        previous_requested_window_opacity_mode: REQUESTED_WINDOW_OPACITY_MODE
            .load(Ordering::Relaxed),
        _lock: lock,
    }
}

#[cfg(test)]
pub(crate) fn force_surface_window_opacity_for_tests() -> WindowEffectsTestGuard {
    let guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    guard
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

fn resolve_window_transparency(
    theme_registry: Option<&ThemeRegistry>,
    blur: f32,
) -> WindowTransparency {
    match theme_registry.and_then(|registry| registry.resolve_string(OPTION_WINDOW_TRANSPARENCY)) {
        Some(raw) => WindowTransparency::parse(raw).unwrap_or(WindowTransparency::None),
        None if blur > DEFAULT_WINDOW_BLUR => WindowTransparency::Blur,
        None => WindowTransparency::None,
    }
}

pub(crate) fn window_surface_opacity(settings: WindowEffects) -> f32 {
    match current_window_opacity_mode() {
        WindowOpacityMode::NativeWindow => DEFAULT_WINDOW_OPACITY,
        WindowOpacityMode::Surface => settings.opacity,
    }
}

pub(crate) fn overlay_window_surface_opacity(_settings: WindowEffects) -> f32 {
    // CONTEXT: floating overlay chrome (pickers, popups, hover, notifications)
    // stays fully opaque so text stays crisp and cards do not muddy-stack. ACP /
    // plugin / browser buffer sections use window_surface_opacity instead so they
    // share the same darkened-base + transparent treatment as editor panes.
    DEFAULT_WINDOW_OPACITY
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
    apply_window_transparency(window, settings.transparency, settings.blur)
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
    if previous.transparency == next.transparency && previous.blur == next.blur {
        return Ok(());
    }
    if previous.transparency.is_enabled() && previous.transparency != next.transparency {
        clear_window_transparency(window, previous.transparency)?;
    }
    if next.transparency.is_enabled() {
        return apply_window_transparency(window, next.transparency, next.blur);
    }
    Ok(())
}

fn apply_window_transparency(
    window: &mut impl NativeWindowEffectsTarget,
    transparency: WindowTransparency,
    blur: f32,
) -> Result<(), ShellError> {
    if !transparency.is_enabled() {
        return Ok(());
    }

    window
        .apply_native_window_transparency(transparency, blur)
        .map_err(|error| {
            ShellError::Runtime(format!(
                "failed to apply {OPTION_WINDOW_TRANSPARENCY}={}: {error}",
                transparency.as_str()
            ))
        })
}

fn clear_window_transparency(
    window: &mut impl NativeWindowEffectsTarget,
    transparency: WindowTransparency,
) -> Result<(), ShellError> {
    if !transparency.is_enabled() {
        return Ok(());
    }
    window
        .clear_native_window_transparency(transparency)
        .map_err(|error| {
            ShellError::Runtime(format!(
                "failed to clear {OPTION_WINDOW_TRANSPARENCY}={}: {error}",
                transparency.as_str()
            ))
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
    WindowOpacityMode::from_stored(REQUESTED_WINDOW_OPACITY_MODE.load(Ordering::Relaxed))
}

fn window_opacity_mode_for_driver(driver: Option<&str>) -> WindowOpacityMode {
    match driver {
        Some(SDL_VIDEO_DRIVER_X11 | SDL_VIDEO_DRIVER_WAYLAND) => WindowOpacityMode::NativeWindow,
        _ => WindowOpacityMode::Surface,
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use super::WindowTransparency;
    use sdl3::video::Window;

    pub(super) fn apply_transparency(
        window: &Window,
        transparency: WindowTransparency,
        blur: f32,
    ) -> Result<(), String> {
        let _ = blur;
        match transparency {
            WindowTransparency::None => Ok(()),
            WindowTransparency::Blur => {
                window_vibrancy::apply_blur(window, None).map_err(|error| {
                    format!("Windows compositor blur is unavailable for this SDL window: {error}")
                })
            }
            WindowTransparency::Acrylic => {
                window_vibrancy::apply_acrylic(window, None).map_err(|error| {
                    format!("Windows acrylic is unavailable for this SDL window: {error}")
                })
            }
            WindowTransparency::Mica => {
                window_vibrancy::apply_mica(window, None).map_err(|error| {
                    format!("Windows mica is unavailable for this SDL window: {error}")
                })
            }
            WindowTransparency::MicaTabbed => {
                window_vibrancy::apply_tabbed(window, None).map_err(|error| {
                    format!("Windows tabbed mica is unavailable for this SDL window: {error}")
                })
            }
        }
    }

    pub(super) fn clear_transparency(
        window: &Window,
        transparency: WindowTransparency,
    ) -> Result<(), String> {
        match transparency {
            WindowTransparency::None => Ok(()),
            WindowTransparency::Blur => window_vibrancy::clear_blur(window).map_err(|error| {
                format!("Windows compositor blur could not be cleared for this SDL window: {error}")
            }),
            WindowTransparency::Acrylic => {
                window_vibrancy::clear_acrylic(window).map_err(|error| {
                    format!("Windows acrylic could not be cleared for this SDL window: {error}")
                })
            }
            WindowTransparency::Mica => window_vibrancy::clear_mica(window).map_err(|error| {
                format!("Windows mica could not be cleared for this SDL window: {error}")
            }),
            WindowTransparency::MicaTabbed => {
                window_vibrancy::clear_tabbed(window).map_err(|error| {
                    format!("Windows tabbed mica could not be cleared for this SDL window: {error}")
                })
            }
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use super::WindowTransparency;
    use sdl3::video::Window;
    use window_vibrancy::{NSVisualEffectMaterial, apply_vibrancy};

    pub(super) fn apply_transparency(
        window: &Window,
        transparency: WindowTransparency,
        blur: f32,
    ) -> Result<(), String> {
        if !transparency.is_enabled() {
            return Ok(());
        }
        apply_vibrancy(
            window,
            NSVisualEffectMaterial::UnderWindowBackground,
            None,
            Some(f64::from(blur)),
        )
        .map_err(|error| format!("macOS vibrancy is unavailable for this SDL window: {error}"))
    }

    pub(super) fn clear_transparency(
        window: &Window,
        transparency: WindowTransparency,
    ) -> Result<(), String> {
        if !transparency.is_enabled() {
            return Ok(());
        }
        window_vibrancy::clear_vibrancy(window)
            .map(|_| ())
            .map_err(|error| {
                format!("macOS vibrancy could not be cleared for this SDL window: {error}")
            })
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use super::WindowTransparency;
    use sdl3::video::Window;

    pub(super) fn apply_transparency(
        _window: &Window,
        _transparency: WindowTransparency,
        _blur: f32,
    ) -> Result<(), String> {
        // Linux compositor blur remains backend-specific; keep this as an
        // intentional no-op so window.opacity can still be applied.
        Ok(())
    }

    pub(super) fn clear_transparency(
        _window: &Window,
        _transparency: WindowTransparency,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod platform {
    use super::WindowTransparency;
    use sdl3::video::Window;

    pub(super) fn apply_transparency(
        _window: &Window,
        transparency: WindowTransparency,
        blur: f32,
    ) -> Result<(), String> {
        if !transparency.is_enabled() {
            return Ok(());
        }
        Err(format!(
            "window transparency `{}` is not implemented for this target platform (blur={blur})",
            transparency.as_str()
        ))
    }

    pub(super) fn clear_transparency(
        _window: &Window,
        _transparency: WindowTransparency,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
#[path = "window_effects_tests.rs"]
mod tests;
