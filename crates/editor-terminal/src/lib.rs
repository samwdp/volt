#![doc = r#"Terminal transcript sessions and editor-facing command execution surfaces."#]

mod render;
mod session;

pub use render::*;
pub use session::*;

#[cfg(test)]
#[path = "render_tests.rs"]
mod tests;
