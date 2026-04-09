use editor_theme::ThemeRegistry;
use sdl3::video::{Window, WindowFlags};

use crate::ShellError;

pub(crate) const OPTION_WINDOW_OPACITY: &str = "window.opacity";
pub(crate) const OPTION_WINDOW_BLUR: &str = "window.blur";

const DEFAULT_WINDOW_OPACITY: f32 = 1.0;
const DEFAULT_WINDOW_BLUR: f32 = 0.0;

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

trait NativeWindowBlurTarget {
    fn apply_native_window_blur(&mut self, blur: f32) -> Result<(), String>;
    fn clear_native_window_blur(&mut self) -> Result<(), String>;
}

impl NativeWindowBlurTarget for Window {
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
    window: &mut impl NativeWindowBlurTarget,
    settings: WindowEffects,
) -> Result<(), ShellError> {
    // CONTEXT: window.opacity is already applied to renderer-owned background
    // surfaces so text stays fully opaque. Native window opacity would fade the
    // entire OS window and double-apply the effect.
    let _ = settings.opacity;
    apply_window_blur(window, settings.blur)
}

fn update_window_effects_to_target(
    window: &mut impl NativeWindowBlurTarget,
    previous: WindowEffects,
    next: WindowEffects,
) -> Result<(), ShellError> {
    let _ = next.opacity;
    if next.blur > DEFAULT_WINDOW_BLUR {
        return apply_window_blur(window, next.blur);
    }
    if previous.blur > DEFAULT_WINDOW_BLUR {
        return clear_window_blur(window);
    }
    Ok(())
}

fn apply_window_blur(
    window: &mut impl NativeWindowBlurTarget,
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

fn clear_window_blur(window: &mut impl NativeWindowBlurTarget) -> Result<(), ShellError> {
    window.clear_native_window_blur().map_err(|error| {
        ShellError::Runtime(format!("failed to clear {OPTION_WINDOW_BLUR}: {error}"))
    })
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

    pub(super) fn apply_blur(_window: &Window, blur: f32) -> Result<(), String> {
        Err(format!(
            "Linux SDL windows do not expose a reliable blur backend; requested {blur}, but compositor blur remains platform-specific"
        ))
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
        NativeWindowBlurTarget, OPTION_WINDOW_BLUR, OPTION_WINDOW_OPACITY, WindowEffects,
        apply_window_effects_to_target, normalize_window_blur, normalize_window_opacity,
        update_window_effects_to_target, window_creation_flags,
    };
    use editor_theme::{Theme, ThemeRegistry};
    use sdl3::video::WindowFlags;

    #[derive(Default)]
    struct RecordingWindow {
        blur_calls: Vec<f32>,
        clear_calls: usize,
        blur_error: Option<String>,
        clear_error: Option<String>,
    }

    impl NativeWindowBlurTarget for RecordingWindow {
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
        assert_eq!(WindowEffects::resolve(None), WindowEffects::default());
    }

    #[test]
    fn window_effects_resolve_normalized_theme_values() {
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
        assert_eq!(normalize_window_opacity(f64::NAN), 1.0);
        assert_eq!(normalize_window_blur(f64::NEG_INFINITY), 0.0);
        assert_eq!(normalize_window_blur(f64::INFINITY), 0.0);
        assert_eq!(normalize_window_blur(f64::from(f32::MAX) * 2.0), f32::MAX);
    }

    #[test]
    fn window_creation_flags_always_request_transparent_surface_for_live_updates() {
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
        let mut window = RecordingWindow::default();

        must(apply_window_effects_to_target(
            &mut window,
            WindowEffects {
                opacity: 0.5,
                blur: 0.0,
            },
        ));

        assert!(window.blur_calls.is_empty());
        assert_eq!(window.clear_calls, 0);
    }

    #[test]
    fn apply_window_effects_still_calls_native_blur_backend_when_requested() {
        let mut window = RecordingWindow::default();

        must(apply_window_effects_to_target(
            &mut window,
            WindowEffects {
                opacity: 0.5,
                blur: 18.0,
            },
        ));

        assert_eq!(window.blur_calls, vec![18.0]);
        assert_eq!(window.clear_calls, 0);
    }

    #[test]
    fn update_window_effects_clears_native_blur_when_disabled() {
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

        assert!(window.blur_calls.is_empty());
        assert_eq!(window.clear_calls, 1);
    }
}
