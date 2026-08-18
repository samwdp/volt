use super::*;
use std::{cell::RefCell, ffi::CString, path::Path};

struct ClipboardContext {
    video: sdl3::VideoSubsystem,
}

thread_local! {
    static CLIPBOARD_CONTEXT: RefCell<Option<ClipboardContext>> = const { RefCell::new(None) };
}

const CLIPBOARD_IMAGE_MIME_TYPES: &[&str] = &[
    "image/png",
    "image/jpeg",
    "image/jpg",
    "image/webp",
    "image/gif",
    "image/bmp",
    "image/tiff",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ClipboardImage {
    pub name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ClipboardPaste {
    Empty,
    Text(String),
    Image(ClipboardImage),
}

pub(super) fn register_clipboard_context(video: sdl3::VideoSubsystem) {
    CLIPBOARD_CONTEXT.with(|context| {
        *context.borrow_mut() = Some(ClipboardContext { video });
    });
}

fn with_clipboard_util<T>(f: impl FnOnce(&sdl3::clipboard::ClipboardUtil) -> T) -> Option<T> {
    CLIPBOARD_CONTEXT.with(|context| {
        context.borrow().as_ref().map(|context| {
            let clipboard = context.video.clipboard();
            f(&clipboard)
        })
    })
}

fn clipboard_video_ready() -> bool {
    CLIPBOARD_CONTEXT.with(|context| context.borrow().is_some())
}

pub(super) fn configure_background_command(_command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        _command.creation_flags(CREATE_NO_WINDOW);
    }
}

pub(super) fn write_system_clipboard(text: &str) {
    if text.is_empty() {
        return;
    }
    if let Some(Err(error)) = with_clipboard_util(|clipboard| clipboard.set_clipboard_text(text)) {
        eprintln!("Failed to write clipboard text: {error}.");
    }
}

pub(super) fn read_system_clipboard() -> Option<String> {
    with_clipboard_util(|clipboard| {
        if !clipboard.has_clipboard_text() {
            return None;
        }
        clipboard.clipboard_text().ok()
    })
    .flatten()
    .filter(|text| !text.is_empty())
}

pub(super) fn read_system_clipboard_paste() -> ClipboardPaste {
    if let Some(image) = read_system_clipboard_image() {
        return ClipboardPaste::Image(image);
    }
    match read_system_clipboard() {
        Some(text) => {
            if let Some(image) = clipboard_image_from_path_text(&text) {
                ClipboardPaste::Image(image)
            } else {
                ClipboardPaste::Text(text)
            }
        }
        None => ClipboardPaste::Empty,
    }
}

fn read_system_clipboard_image() -> Option<ClipboardImage> {
    if !clipboard_video_ready() {
        return None;
    }
    for mime in CLIPBOARD_IMAGE_MIME_TYPES {
        if let Some(bytes) = clipboard_data_for_mime(mime)
            && let Some(image) = normalize_clipboard_image(bytes, Some(mime), "Image")
        {
            return Some(image);
        }
    }
    if let Some(uris) = clipboard_text_for_mime("text/uri-list")
        && let Some(image) = clipboard_image_from_uri_list(&uris)
    {
        return Some(image);
    }
    None
}

fn clipboard_data_for_mime(mime: &str) -> Option<Vec<u8>> {
    let c_mime = CString::new(mime).ok()?;
    unsafe {
        if !sdl3::sys::clipboard::SDL_HasClipboardData(c_mime.as_ptr()) {
            return None;
        }
        let mut size = 0usize;
        let ptr = sdl3::sys::clipboard::SDL_GetClipboardData(c_mime.as_ptr(), &mut size);
        if ptr.is_null() || size == 0 {
            return None;
        }
        let bytes = std::slice::from_raw_parts(ptr.cast::<u8>(), size).to_vec();
        sdl3::sys::stdinc::SDL_free(ptr);
        Some(bytes)
    }
}

fn clipboard_text_for_mime(mime: &str) -> Option<String> {
    let bytes = clipboard_data_for_mime(mime)?;
    let text = std::str::from_utf8(&bytes).ok()?.trim_end_matches('\0');
    (!text.is_empty()).then(|| text.to_owned())
}

pub(super) fn sniff_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some("image/png")
    } else if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        Some("image/jpeg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("image/gif")
    } else if bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        Some("image/webp")
    } else if bytes.starts_with(b"BM") {
        Some("image/bmp")
    } else {
        None
    }
}

pub(super) fn is_image_path(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "tif" | "tiff")
    )
}

pub(super) fn normalize_clipboard_image(
    bytes: Vec<u8>,
    claimed_mime: Option<&str>,
    name: impl Into<String>,
) -> Option<ClipboardImage> {
    if bytes.is_empty() {
        return None;
    }
    let mime_type = sniff_image_mime(&bytes)
        .or_else(|| claimed_mime.filter(|mime| mime.starts_with("image/") && *mime != "image/jpg"))
        .map(str::to_owned)
        .or_else(|| {
            claimed_mime
                .filter(|mime| *mime == "image/jpg")
                .map(|_| "image/jpeg".to_owned())
        });
    let mime_type = match mime_type {
        Some(mime) => mime,
        None => {
            image::load_from_memory(&bytes).ok()?;
            "image/png".to_owned()
        }
    };
    Some(ClipboardImage {
        name: sanitize_image_name(name),
        mime_type,
        bytes,
    })
}

pub(super) fn clipboard_image_from_path(path: &Path) -> Option<ClipboardImage> {
    if !is_image_path(path) {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Image");
    normalize_clipboard_image(bytes, None, name)
}

fn clipboard_image_from_path_text(text: &str) -> Option<ClipboardImage> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.contains('\n') {
        return None;
    }
    let unquoted = trimmed
        .trim_matches('"')
        .trim_matches('\'')
        .trim()
        .to_owned();
    if let Some(path) = path_from_file_uri(&unquoted) {
        return clipboard_image_from_path(&path);
    }
    let path = PathBuf::from(&unquoted);
    if path.exists() {
        clipboard_image_from_path(&path)
    } else {
        None
    }
}

fn clipboard_image_from_uri_list(uris: &str) -> Option<ClipboardImage> {
    for line in uris.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let path = path_from_file_uri(line).unwrap_or_else(|| PathBuf::from(line));
        if let Some(image) = clipboard_image_from_path(&path) {
            return Some(image);
        }
    }
    None
}

fn path_from_file_uri(value: &str) -> Option<PathBuf> {
    let url = url::Url::parse(value).ok()?;
    if url.scheme() != "file" {
        return None;
    }
    url.to_file_path().ok()
}

fn sanitize_image_name(name: impl Into<String>) -> String {
    let name = name.into();
    let sanitized = name.replace(['[', ']', '\n', '\r'], "");
    if sanitized.trim().is_empty() {
        "Image".to_owned()
    } else {
        sanitized
    }
}

pub(super) fn yank_to_clipboard_text(yank: &YankRegister) -> Cow<'_, str> {
    match yank {
        YankRegister::Character(text) => Cow::Borrowed(text),
        YankRegister::Line(text) => {
            if text.ends_with('\n') {
                Cow::Borrowed(text)
            } else {
                Cow::Owned(format!("{text}\n"))
            }
        }
        YankRegister::Block(lines) => Cow::Owned(lines.join("\n")),
        YankRegister::Directory(entries) => Cow::Owned(
            entries
                .iter()
                .map(|entry| entry.label.as_str())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n",
        ),
    }
}

pub(super) fn yank_from_clipboard_text(text: &str) -> Option<YankRegister> {
    if text.ends_with('\n') {
        Some(YankRegister::Line(text.to_owned()))
    } else {
        Some(YankRegister::Character(text.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY_PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn sniff_image_mime_recognizes_png_magic() {
        assert_eq!(sniff_image_mime(TINY_PNG), Some("image/png"));
        assert_eq!(sniff_image_mime(b"not-an-image"), None);
    }

    #[test]
    fn normalize_clipboard_image_prefers_sniffed_png_mime() {
        let image = normalize_clipboard_image(TINY_PNG.to_vec(), Some("image/jpeg"), "shot")
            .expect("png should normalize");
        assert_eq!(image.mime_type, "image/png");
        assert_eq!(image.name, "shot");
        assert_eq!(image.bytes, TINY_PNG);
    }

    #[test]
    fn clipboard_image_from_path_loads_named_png() {
        let dir = std::env::temp_dir().join(format!(
            "volt-clipboard-image-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos()
        ));
        fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("screenshot.png");
        fs::write(&path, TINY_PNG).expect("write png");
        let image = clipboard_image_from_path(&path).expect("load png");
        assert_eq!(image.name, "screenshot.png");
        assert_eq!(image.mime_type, "image/png");
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn clipboard_image_from_path_text_ignores_plain_text() {
        assert!(clipboard_image_from_path_text("hello world").is_none());
        assert!(clipboard_image_from_path_text("not/a/real/image.png").is_none());
    }
}
