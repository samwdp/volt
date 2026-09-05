
use super::{
    NativeWindowEffectsTarget, OPTION_WINDOW_BLUR, OPTION_WINDOW_OPACITY,
    OPTION_WINDOW_TRANSPARENCY, WindowEffects, WindowOpacityMode, WindowTransparency,
    apply_window_effects_to_target, configure_window_opacity_driver, current_window_opacity_mode,
    lock_window_effects_for_tests, normalize_window_blur, normalize_window_opacity,
    requested_window_opacity_mode, set_window_opacity_mode, sync_window_opacity,
    update_window_effects_to_target, window_creation_flags, window_opacity_mode_for_driver,
};
use editor_theme::{Theme, ThemeRegistry};
use sdl3::video::WindowFlags;

#[derive(Default)]
struct RecordingWindow {
    opacity_calls: Vec<f32>,
    transparency_calls: Vec<(WindowTransparency, f32)>,
    clear_calls: Vec<WindowTransparency>,
    opacity_error: Option<String>,
    transparency_error: Option<String>,
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

    fn apply_native_window_transparency(
        &mut self,
        transparency: WindowTransparency,
        blur: f32,
    ) -> Result<(), String> {
        self.transparency_calls.push((transparency, blur));
        match &self.transparency_error {
            Some(error) => Err(error.clone()),
            None => Ok(()),
        }
    }

    fn clear_native_window_transparency(
        &mut self,
        transparency: WindowTransparency,
    ) -> Result<(), String> {
        self.clear_calls.push(transparency);
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
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    assert_eq!(WindowEffects::resolve(None), WindowEffects::default());
}

#[test]
fn window_effects_resolve_normalized_theme_values() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
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
            transparency: WindowTransparency::None,
        }
    );
}

#[test]
fn window_effects_resolve_transparency_type_from_theme_string() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    let mut registry = ThemeRegistry::new();
    must(
        registry.register(
            Theme::new("test-theme", "Test Theme")
                .with_option(OPTION_WINDOW_OPACITY, 0.4)
                .with_option(OPTION_WINDOW_BLUR, 0.0)
                .with_option(OPTION_WINDOW_TRANSPARENCY, "acrylic"),
        ),
    );

    assert_eq!(
        WindowEffects::resolve(Some(&registry)),
        WindowEffects {
            opacity: 0.4,
            blur: 0.0,
            transparency: WindowTransparency::Acrylic,
        }
    );
}

#[test]
fn window_effects_legacy_blur_option_implies_blur_transparency() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    let mut registry = ThemeRegistry::new();
    must(
        registry
            .register(Theme::new("test-theme", "Test Theme").with_option(OPTION_WINDOW_BLUR, 12.0)),
    );

    assert_eq!(
        WindowEffects::resolve(Some(&registry)).transparency,
        WindowTransparency::Blur
    );
}

#[test]
fn window_transparency_parse_accepts_aliases() {
    assert_eq!(
        WindowTransparency::parse("Mica-Tabbed"),
        Some(WindowTransparency::MicaTabbed)
    );
    assert_eq!(
        WindowTransparency::parse("tabbed"),
        Some(WindowTransparency::MicaTabbed)
    );
    assert_eq!(WindowTransparency::parse("nope"), None);
}

#[test]
fn window_effect_normalizers_handle_non_finite_values() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    assert_eq!(normalize_window_opacity(f64::NAN), 1.0);
    assert_eq!(normalize_window_blur(f64::NEG_INFINITY), 0.0);
    assert_eq!(normalize_window_blur(f64::INFINITY), 0.0);
    assert_eq!(normalize_window_blur(f64::from(f32::MAX) * 2.0), f32::MAX);
}

#[test]
fn window_creation_flags_always_request_transparent_surface_for_live_updates() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    assert!(
        window_creation_flags(WindowEffects {
            opacity: 0.75,
            blur: 0.0,
            transparency: WindowTransparency::None,
        })
        .contains(WindowFlags::TRANSPARENT)
    );
    assert!(
        window_creation_flags(WindowEffects {
            opacity: 1.0,
            blur: 12.0,
            transparency: WindowTransparency::Blur,
        })
        .contains(WindowFlags::TRANSPARENT)
    );
    assert!(window_creation_flags(WindowEffects::default()).contains(WindowFlags::TRANSPARENT));
}

#[test]
fn overlay_window_surface_opacity_stays_opaque_when_window_is_transparent() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    let settings = WindowEffects {
        opacity: 0.25,
        blur: 0.0,
        transparency: WindowTransparency::Acrylic,
    };
    assert_eq!(
        crate::window_effects::overlay_window_surface_opacity(settings),
        1.0
    );
    assert_eq!(
        crate::window_effects::window_surface_opacity(settings),
        0.25
    );
}

#[test]
fn apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    let mut window = RecordingWindow::default();

    must(apply_window_effects_to_target(
        &mut window,
        WindowEffects {
            opacity: 0.5,
            blur: 0.0,
            transparency: WindowTransparency::None,
        },
    ));

    assert!(window.opacity_calls.is_empty());
    assert!(window.transparency_calls.is_empty());
    assert!(window.clear_calls.is_empty());
}

#[test]
fn apply_window_effects_calls_native_transparency_backend_when_requested() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    let mut window = RecordingWindow::default();

    must(apply_window_effects_to_target(
        &mut window,
        WindowEffects {
            opacity: 0.5,
            blur: 18.0,
            transparency: WindowTransparency::Acrylic,
        },
    ));

    assert!(window.opacity_calls.is_empty());
    assert_eq!(
        window.transparency_calls,
        vec![(WindowTransparency::Acrylic, 18.0)]
    );
    assert!(window.clear_calls.is_empty());
}

#[test]
fn update_window_effects_clears_native_transparency_when_disabled() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    let mut window = RecordingWindow::default();

    must(update_window_effects_to_target(
        &mut window,
        WindowEffects {
            opacity: 1.0,
            blur: 18.0,
            transparency: WindowTransparency::Mica,
        },
        WindowEffects {
            opacity: 1.0,
            blur: 0.0,
            transparency: WindowTransparency::None,
        },
    ));

    assert!(window.opacity_calls.is_empty());
    assert!(window.transparency_calls.is_empty());
    assert_eq!(window.clear_calls, vec![WindowTransparency::Mica]);
}

#[test]
fn update_window_effects_switches_transparency_types() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
    set_window_opacity_mode(WindowOpacityMode::Surface);
    let mut window = RecordingWindow::default();

    must(update_window_effects_to_target(
        &mut window,
        WindowEffects {
            opacity: 1.0,
            blur: 0.0,
            transparency: WindowTransparency::Blur,
        },
        WindowEffects {
            opacity: 1.0,
            blur: 0.0,
            transparency: WindowTransparency::Acrylic,
        },
    ));

    assert_eq!(window.clear_calls, vec![WindowTransparency::Blur]);
    assert_eq!(
        window.transparency_calls,
        vec![(WindowTransparency::Acrylic, 0.0)]
    );
}

#[test]
fn linux_native_window_opacity_targets_x11_and_wayland_only() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
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
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
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
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(None);
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

#[test]
fn configure_window_opacity_driver_updates_requested_mode() {
    let _guard = lock_window_effects_for_tests();
    configure_window_opacity_driver(Some("x11"));
    let mut window = RecordingWindow::default();
    let mode = sync_window_opacity(&mut window, 0.4, requested_window_opacity_mode());

    assert_eq!(mode, WindowOpacityMode::NativeWindow);

    configure_window_opacity_driver(None);
}
