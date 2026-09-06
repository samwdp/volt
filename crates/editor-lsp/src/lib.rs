#![doc = r#"Language Server Protocol registry, session plans, diagnostics, launch metadata, and client runtime management."#]

mod client;
mod registry;
mod workspace_roots;

pub use client::*;
pub use editor_tool_install::InstallRecipe;
pub use registry::*;

#[cfg(test)]
#[path = "lib_tests.rs"]
mod tests;
