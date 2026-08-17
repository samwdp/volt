use super::*;
use std::cell::RefCell;
use std::ffi::CString;
use std::path::Path;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ClipboardImage {
    pub mime_type: String,
    pub data: Vec<u8>,
}

const CLIPBOARD_IMAGE_MIME_TYPES: &[&str] = &["image/png", "image/jpeg", "image/webp", "image/gif"];

pub(super) fn read_system_clipboard_image() -> Option<ClipboardImage> {
    CLIPBOARD_CONTEXT.with(|context| {
        if context.borrow().is_none() {
            return None;
        }
        for mime_type in CLIPBOARD_IMAGE_MIME_TYPES {
            let mime = CString::new(*mime_type).ok()?;
            let image = unsafe {
                if !sdl3::sys::clipboard::SDL_HasClipboardData(mime.as_ptr()) {
                    continue;
                }
                let mut size = 0usize;
                let ptr = sdl3::sys::clipboard::SDL_GetClipboardData(mime.as_ptr(), &mut size);
                if ptr.is_null() || size == 0 {
                    continue;
                }
                let data = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
                sdl3::sys::stdinc::SDL_free(ptr);
                ClipboardImage {
                    mime_type: (*mime_type).to_owned(),
                    data,
                }
            };
            return Some(image);
        }
        None
    })
}

pub(super) fn clipboard_image_from_path_text(text: &str) -> Option<ClipboardImage> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.contains('\n') {
        return None;
    }
    let path = Path::new(trimmed);
    if !path.is_file() {
        return None;
    }
    let mime_type = match path
        .extension()
        .and_then(|extension| extension.to_str())?
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => return None,
    };
    let data = std::fs::read(path).ok()?;
    if data.is_empty() {
        return None;
    }
    Some(ClipboardImage {
        mime_type: mime_type.to_owned(),
        data,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_image_from_path_text_reads_supported_image_files() {
        let root = std::env::temp_dir().join(format!("volt-clipboard-image-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("temp dir");
        let path = root.join("shot.png");
        std::fs::write(&path, b"\x89PNG\r\n").expect("write png");

        let image = clipboard_image_from_path_text(path.display().to_string().as_str())
            .expect("clipboard image");
        assert_eq!(image.mime_type, "image/png");
        assert_eq!(image.data, b"\x89PNG\r\n");

        let _ = std::fs::remove_dir_all(root);
    }
}
