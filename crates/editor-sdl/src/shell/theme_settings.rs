#[derive(Debug, Clone, PartialEq)]
struct ThemeRuntimeSettings {
    font_request: Option<String>,
    emoji_font_request: Option<String>,
    font_size: u32,
    emoji_font_size: u32,
    display_scale: f32,
    window_effects: WindowEffects,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ThemeSourceFingerprint {
    files: Vec<ThemeSourceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UserConfigSourceFingerprint {
    files: Vec<UserConfigSourceFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ThemeSourceFile {
    path: PathBuf,
    size: u64,
    modified_at: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct UserConfigSourceFile {
    path: PathBuf,
    size: u64,
    modified_at: Option<SystemTime>,
}

#[derive(Debug, Clone)]
struct ThemeReloadState {
    last_checked_at: Instant,
    fingerprint: Option<ThemeSourceFingerprint>,
}

impl ThemeReloadState {
    fn new() -> Self {
        Self {
            last_checked_at: Instant::now(),
            fingerprint: current_theme_source_fingerprint(),
        }
    }
}

#[derive(Debug, Clone)]
struct UserConfigReloadState {
    last_checked_at: Instant,
    fingerprint: Option<UserConfigSourceFingerprint>,
}

impl UserConfigReloadState {
    fn new() -> Self {
        Self {
            last_checked_at: Instant::now(),
            fingerprint: current_user_config_source_fingerprint(),
        }
    }
}

fn preferred_primary_font_hinting() -> Option<Hinting> {
    if cfg!(target_os = "windows") {
        // Transparent compositor surfaces do not preserve ClearType-style
        // subpixel assumptions. Keep glyphs unhinted so picker/footer text
        // stays visually closer to the buffer instead of snapping stems harder.
        Some(Hinting::NONE)
    } else {
        None
    }
}

/// Normalizes the SDL window display scale for font rasterization.
///
/// SDL reports integral and fractional HiDPI scale factors; rounding to three
/// decimal places keeps repeated comparisons stable across frames while
/// preserving practical DPI values. Invalid or non-positive values fall back to
/// 1.0 so font loading never requests a zero-sized raster.
fn normalize_display_scale(display_scale: f32) -> f32 {
    if display_scale.is_finite() && display_scale > 0.0 {
        (display_scale * 1000.0).round() / 1000.0
    } else {
        1.0
    }
}

/// Converts the logical theme font size into the window's effective pixel size.
///
/// The logical font size is clamped to at least 1px, then multiplied by the
/// window display scale so text stays physically readable on HiDPI displays.
fn scaled_font_size(font_size: u32, display_scale: f32) -> f32 {
    font_size.max(1) as f32 * normalize_display_scale(display_scale)
}

fn normalized_raster_pixel_size(
    requested_pixel_size: f32,
    target_line_height: i32,
    line_metrics: Option<fontdue::LineMetrics>,
) -> f32 {
    let fallback_pixel_size = requested_pixel_size.max(1.0);
    let target_line_height = target_line_height.max(1) as f32;
    line_metrics
        .map(|metrics| metrics.ascent - metrics.descent)
        .filter(|height| *height > f32::EPSILON)
        .map(|height| fallback_pixel_size * target_line_height / height)
        .filter(|pixel_size| pixel_size.is_finite() && *pixel_size > f32::EPSILON)
        .unwrap_or(fallback_pixel_size)
}
