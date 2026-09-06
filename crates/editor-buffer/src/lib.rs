#![doc = r#"Rope-backed text storage, editing, cursor movement, and line-oriented access."#]

mod buffer;
mod geometry;
mod motion;
mod objects;

pub use buffer::TextBuffer;
pub use geometry::*;

#[cfg(test)]
mod tests;
