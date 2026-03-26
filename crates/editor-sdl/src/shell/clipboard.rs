use super::*;
use std::cell::RefCell;

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

pub(super) fn configure_background_command(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;

        command.creation_flags(CREATE_NO_WINDOW);
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
    }
}

pub(super) fn yank_from_clipboard_text(text: &str) -> Option<YankRegister> {
    if text.ends_with('\n') {
        Some(YankRegister::Line(text.to_owned()))
    } else {
        Some(YankRegister::Character(text.to_owned()))
    }
}
