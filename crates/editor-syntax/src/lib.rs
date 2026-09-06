#![doc = r#"Tree-sitter language registration, installation, parsing, highlighting, and indentation."#]

mod highlight;
mod install;
mod language;
mod query;
mod rainbow_paren;
mod registry;

pub use language::*;
pub use query::*;
pub use rainbow_paren::{
    MAX_DEPTH, TOKEN_MISMATCHED, TOKEN_UNMATCHED, apply_rainbow_delimiter_spans,
    apply_rainbow_delimiter_spans_for_buffer, depth_theme_token,
};
pub use registry::*;

#[cfg(test)]
mod tests;
