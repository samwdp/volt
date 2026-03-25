#![doc = r#"SDL3 windowing, input, and demo shell rendering for the native editor."#]

mod browser_host;
mod config;
mod shell;
mod state;

#[cfg(test)]
mod tests;

pub use config::{ShellConfig, ShellError, ShellSummary};
pub use shell::run_demo_shell;
