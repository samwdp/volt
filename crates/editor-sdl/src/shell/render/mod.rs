use super::*;

include!("markdown.rs");
include!("frame.rs");
include!("fps.rs");
include!("chrome.rs");
include!("overlays.rs");
include!("layout.rs");
include!("modeline.rs");
include!("buffer_text.rs");
include!("command_line.rs");
include!("plugin.rs");
include!("acp_chat.rs");
include!("text.rs");
include!("textures.rs");
include!("primitives.rs");
include!("canvas.rs");
include!("picker_label.rs");

#[cfg(test)]
mod render_rounded_rect_tests;
