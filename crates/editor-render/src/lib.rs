#![doc = r#"Render-adjacent layout helpers and font discovery used by the native editor shell."#]

use std::{
    env, fmt, fs,
    path::{Path, PathBuf},
};

/// Pixel-space rectangle used by the shell renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelRect {
    /// X origin.
    pub x: i32,
    /// Y origin.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl PixelRect {
    /// Creates a new pixel rectangle.
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// Selects the concrete renderer implementation used by the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderBackend {
    /// SDL canvas-backed rendering used by the current shell.
    SdlCanvas,
    /// Vulkan-backed rendering planned for the native shell.
    Vulkan,
}

/// Backend-agnostic RGBA color used by the shell display list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderColor {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel.
    pub a: u8,
}

impl RenderColor {
    /// Creates an opaque RGB color.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    /// Creates an RGBA color.
    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

/// Backend-agnostic draw command emitted by the shell renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommand {
    /// Clears the output with a solid color.
    Clear { color: RenderColor },
    /// Fills a rectangle with a solid color.
    FillRect { rect: PixelRect, color: RenderColor },
    /// Fills a rounded rectangle with a solid color.
    FillRoundedRect {
        rect: PixelRect,
        radius: u32,
        color: RenderColor,
    },
    /// Draws a diagnostic undercurl at the given text baseline.
    Undercurl {
        x: i32,
        y: i32,
        width: u32,
        line_height: u32,
        color: RenderColor,
    },
    /// Draws a text run at the given origin.
    Text {
        x: i32,
        y: i32,
        text: String,
        color: RenderColor,
    },
}

/// Errors raised by the render support crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// No suitable monospace font was found on the current system.
    FontNotFound(Vec<PathBuf>),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FontNotFound(candidates) => write!(
                formatter,
                "no system monospace font found; checked: {}",
                candidates
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl std::error::Error for RenderError {}

/// Returns platform-specific monospace font candidates for the shell renderer.
pub fn default_font_candidates() -> Vec<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        vec![
            PathBuf::from(r"C:\Windows\Fonts\consola.ttf"),
            PathBuf::from(r"C:\Windows\Fonts\cour.ttf"),
            PathBuf::from(r"C:\Windows\Fonts\lucon.ttf"),
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            PathBuf::from("/System/Library/Fonts/Menlo.ttc"),
            PathBuf::from("/System/Library/Fonts/SFNSMono.ttf"),
            PathBuf::from("/Library/Fonts/Courier New.ttf"),
        ]
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        vec![
            PathBuf::from("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"),
            PathBuf::from("/usr/share/fonts/TTF/DejaVuSansMono.ttf"),
            PathBuf::from("/usr/share/fonts/truetype/liberation2/LiberationMono-Regular.ttf"),
        ]
    }
}

fn preferred_font_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        roots.push(PathBuf::from(r"C:\Windows\Fonts"));
        if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
            roots.push(PathBuf::from(local_app_data).join("Microsoft\\Windows\\Fonts"));
        }
    }

    #[cfg(target_os = "macos")]
    {
        roots.push(PathBuf::from("/System/Library/Fonts"));
        roots.push(PathBuf::from("/Library/Fonts"));
        if let Some(home) = env::var_os("HOME") {
            roots.push(PathBuf::from(home).join("Library/Fonts"));
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        roots.push(PathBuf::from("/usr/share/fonts"));
        roots.push(PathBuf::from("/usr/local/share/fonts"));
        if let Some(home) = env::var_os("HOME") {
            roots.push(PathBuf::from(&home).join(".local/share/fonts"));
            roots.push(PathBuf::from(home).join(".fonts"));
        }
    }

    roots
}

fn preferred_berkeley_mono_font() -> Option<PathBuf> {
    let mut stack = preferred_font_search_roots();
    while let Some(path) = stack.pop() {
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };

        if metadata.is_dir() {
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                stack.push(entry.path());
            }
            continue;
        }

        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase());
        if !matches!(extension.as_deref(), Some("ttf" | "otf" | "ttc")) {
            continue;
        }

        let Some(file_name) = path
            .file_name()
            .map(|file_name| file_name.to_string_lossy().to_ascii_lowercase())
        else {
            continue;
        };
        let looks_like_berkeley_mono =
            file_name.contains("berkeleymono") || file_name.contains("berkeley mono");
        if looks_like_berkeley_mono {
            return Some(path);
        }
    }

    None
}

/// Finds the first available system monospace font from the known candidates.
pub fn find_system_monospace_font() -> Result<PathBuf, RenderError> {
    if let Some(font_path) = preferred_berkeley_mono_font() {
        return Ok(font_path);
    }

    let candidates = default_font_candidates();
    candidates
        .iter()
        .find(|path| path.exists())
        .cloned()
        .ok_or(RenderError::FontNotFound(candidates))
}

/// Attempts to find a font path by name in platform font directories.
pub fn find_font_by_name(name: &str) -> Option<PathBuf> {
    let normalized = normalize_font_name(name);
    if normalized.is_empty() {
        return None;
    }

    const MAX_FONT_SEARCH_DEPTH: usize = 6;
    let mut stack = preferred_font_search_roots()
        .into_iter()
        .map(|path| (path, 0usize))
        .collect::<Vec<_>>();
    while let Some((path, depth)) = stack.pop() {
        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        if metadata.is_dir() {
            if depth >= MAX_FONT_SEARCH_DEPTH {
                continue;
            }
            let Ok(entries) = fs::read_dir(&path) else {
                continue;
            };
            for entry in entries.flatten() {
                stack.push((entry.path(), depth + 1));
            }
            continue;
        }

        if !is_font_file(&path) {
            continue;
        }

        if font_name_matches(&path, &normalized) {
            return Some(path);
        }
    }

    None
}

fn is_font_file(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase())
            .as_deref(),
        Some("ttf" | "otf" | "ttc")
    )
}

fn font_name_matches(path: &Path, normalized: &str) -> bool {
    let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return false;
    };
    let stem = normalize_font_name(stem);
    stem == normalized
}

fn normalize_font_name(name: &str) -> String {
    name.chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .map(|character| character.to_ascii_lowercase())
        .collect()
}

/// Splits the available content area into one or two stacked pane rectangles.
pub fn horizontal_pane_rects(width: u32, content_height: u32, pane_count: usize) -> Vec<PixelRect> {
    if pane_count <= 1 {
        return vec![PixelRect::new(0, 0, width, content_height)];
    }

    let first_height = content_height / 2;
    vec![
        PixelRect::new(0, 0, width, first_height),
        PixelRect::new(0, first_height as i32, width, content_height - first_height),
    ]
}

/// Splits the available content area into one or two side-by-side pane rectangles.
pub fn vertical_pane_rects(width: u32, content_height: u32, pane_count: usize) -> Vec<PixelRect> {
    if pane_count <= 1 {
        return vec![PixelRect::new(0, 0, width, content_height)];
    }

    let first_width = width / 2;
    vec![
        PixelRect::new(0, 0, first_width, content_height),
        PixelRect::new(first_width as i32, 0, width - first_width, content_height),
    ]
}

/// Returns a centered rectangle of the requested size inside the container.
pub fn centered_rect(
    container_width: u32,
    container_height: u32,
    width: u32,
    height: u32,
) -> PixelRect {
    PixelRect::new(
        ((container_width.saturating_sub(width)) / 2) as i32,
        ((container_height.saturating_sub(height)) / 2) as i32,
        width,
        height,
    )
}

/// Converts a rectangle into a path-independent tuple useful in tests and logs.
pub fn rect_tuple(rect: PixelRect) -> (i32, i32, u32, u32) {
    (rect.x, rect.y, rect.width, rect.height)
}

/// Returns whether the provided path exists.
pub fn path_exists(path: &Path) -> bool {
    path.exists()
}

#[cfg(test)]
mod tests {
    use super::{
        centered_rect, horizontal_pane_rects, preferred_font_search_roots, rect_tuple,
        vertical_pane_rects,
    };

    #[test]
    fn centered_rect_places_content_in_middle() {
        assert_eq!(rect_tuple(centered_rect(100, 80, 40, 20)), (30, 30, 40, 20));
    }

    #[test]
    fn horizontal_split_returns_two_stacked_rects() {
        let rects = horizontal_pane_rects(120, 60, 2);
        assert_eq!(rect_tuple(rects[0]), (0, 0, 120, 30));
        assert_eq!(rect_tuple(rects[1]), (0, 30, 120, 30));
    }

    #[test]
    fn vertical_split_returns_two_side_by_side_rects() {
        let rects = vertical_pane_rects(120, 60, 2);
        assert_eq!(rect_tuple(rects[0]), (0, 0, 60, 60));
        assert_eq!(rect_tuple(rects[1]), (60, 0, 60, 60));
    }

    #[test]
    fn font_search_roots_include_platform_locations() {
        assert!(!preferred_font_search_roots().is_empty());
    }
}
