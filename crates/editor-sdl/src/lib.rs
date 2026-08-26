#![doc = r#"SDL3 windowing, input, and demo shell rendering for the native editor."#]

mod browser_host;
mod config;
mod shell;
mod state;
mod window_effects;

#[cfg(test)]
mod tests;

pub use config::{ShellConfig, ShellError, ShellSummary};
pub use shell::{IDLE_WAIT_CAP, idle_wait_timeout, run_demo_shell};
