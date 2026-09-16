use super::*;
use std::{cell::RefCell, io::Cursor, path::Path};

struct ClipboardContext {
    video: sdl3::VideoSubsystem,
}

thread_local! {
    static CLIPBOARD_CONTEXT: RefCell<Option<ClipboardContext>> = const { RefCell::new(None) };
    static CLIPBOARD_TEXT_OVERRIDE_FOR_TEST: RefCell<Option<String>> = const { RefCell::new(None) };
    static CLIPBOARD_IMAGE_OVERRIDE_FOR_TEST: RefCell<Option<ClipboardImage>> =
        const { RefCell::new(None) };
}

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

#[cfg(test)]
pub(super) fn set_clipboard_text_override_for_test(text: Option<&str>) {
    CLIPBOARD_TEXT_OVERRIDE_FOR_TEST.with(|override_text| {
        *override_text.borrow_mut() = text.map(str::to_owned);
    });
}

#[cfg(test)]
pub(super) fn set_clipboard_image_override_for_test(image: Option<ClipboardImage>) {
    CLIPBOARD_IMAGE_OVERRIDE_FOR_TEST.with(|override_image| {
        *override_image.borrow_mut() = image;
    });
}

pub(super) fn read_system_clipboard() -> Option<String> {
    if let Some(text) = CLIPBOARD_TEXT_OVERRIDE_FOR_TEST.with(|override_text| {
        override_text
            .borrow()
            .as_ref()
            .filter(|text| !text.is_empty())
            .cloned()
    }) {
        return Some(text);
    }
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
    #[cfg(test)]
    {
        // Prefer the test override; never read the live OS clipboard in unit tests.
        let _ = std::mem::size_of::<arboard::Clipboard>();
        CLIPBOARD_IMAGE_OVERRIDE_FOR_TEST.with(|override_image| override_image.borrow().clone())
    }
    #[cfg(not(test))]
    {
        let mut clipboard = arboard::Clipboard::new().ok()?;
        let image = clipboard.get_image().ok()?;
        clipboard_image_from_rgba(image.width, image.height, image.bytes.as_ref())
    }
}

fn clipboard_image_from_rgba(width: usize, height: usize, rgba: &[u8]) -> Option<ClipboardImage> {
    let width = u32::try_from(width).ok()?;
    let height = u32::try_from(height).ok()?;
    let buffer = image::ImageBuffer::<image::Rgba<u8>, _>::from_raw(width, height, rgba.to_vec())?;
    let mut png = Vec::new();
    image::DynamicImage::ImageRgba8(buffer)
        .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    normalize_clipboard_image(png, Some("image/png"), "Image")
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
#[path = "clipboard_tests.rs"]
mod tests;
