use super::*;
use std::cell::RefCell;
use std::io::Cursor;

struct ClipboardContext {
    video: sdl3::VideoSubsystem,
}

thread_local! {
    static CLIPBOARD_CONTEXT: RefCell<Option<ClipboardContext>> = const { RefCell::new(None) };
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

/// Image bytes taken from the OS clipboard for ACP prompt context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ClipboardImage {
    pub bytes: Vec<u8>,
    pub mime_type: String,
    pub label: String,
}

/// Read an image from the system clipboard when present.
///
/// Matches Zed / ACP clients: clipboard images become `ContentBlock::Image`
/// (base64 + mime) on prompt submit when the agent advertises `prompt.image`.
/// Prefer image payloads when available; callers should fall back to text paste
/// when this returns `None`.
pub(super) fn read_system_clipboard_image() -> Option<ClipboardImage> {
    let mut clipboard = arboard::Clipboard::new().ok()?;
    // Prefer encoded image entries when the platform exposes them via arboard.
    let image = clipboard.get_image().ok()?;
    encode_rgba_image_png(&image.bytes, image.width, image.height, "Image")
}

pub(super) fn encode_rgba_image_png(
    rgba: &[u8],
    width: usize,
    height: usize,
    label: impl Into<String>,
) -> Option<ClipboardImage> {
    if width == 0 || height == 0 {
        return None;
    }
    let expected = width.checked_mul(height)?.checked_mul(4)?;
    if rgba.len() < expected {
        return None;
    }
    let width_u32 = u32::try_from(width).ok()?;
    let height_u32 = u32::try_from(height).ok()?;
    let buffer = image::RgbaImage::from_raw(width_u32, height_u32, rgba[..expected].to_vec())?;
    let mut encoded = Vec::new();
    buffer
        .write_to(&mut Cursor::new(&mut encoded), image::ImageFormat::Png)
        .ok()?;
    Some(ClipboardImage {
        bytes: encoded,
        mime_type: "image/png".to_owned(),
        label: label.into(),
    })
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
