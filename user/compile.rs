//! Compile / build plugin.
//!
//! Provides `workspace.compile` and `workspace.recompile` commands backed by
//! the host's generic `plugin.run-command` / `plugin.rerun-command` hooks.
//! The user only needs this file — no editor-internal crates are touched.
//!
//! # Workflow
//!
//! 1. `workspace.compile` emits `plugin.run-command` with the active language
//!    as the hook detail.  The host looks up the default command via
//!    `UserLibrary::default_build_command`, opens a `*compile <workspace>*`
//!    popup buffer with an input field pre-filled, and runs the command on
//!    Ctrl+Enter.
//! 2. `workspace.recompile` emits `plugin.rerun-command`.  The host re-runs
//!    the last stored command for the active workspace (or falls back to
//!    `workspace.compile` if none has been run).
//! 3. In the compilation popup, pressing Enter on a line matching the pattern
//!    `path:line` or `path:line:col` navigates to that location.
//!
//! # Adding a new language
//!
//! Add an entry to [`default_build_commands`].  No other files need changing.

use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, plugin_hooks,
};

// ─── Package ─────────────────────────────────────────────────────────────────

/// Returns the plugin package for the compile/build integration.
pub fn package() -> PluginPackage {
    PluginPackage::new("compile", true, "Workspace build/compile commands.")
        .with_commands(vec![
            PluginCommand::new(
                "workspace.compile",
                "Open the compilation buffer and run (or prompt for) the build command.",
                vec![PluginAction::emit_hook(
                    plugin_hooks::RUN_COMMAND,
                    None::<&str>,
                )],
            ),
            PluginCommand::new(
                "workspace.recompile",
                "Re-run the last build command for the active workspace.",
                vec![PluginAction::emit_hook(
                    plugin_hooks::RERUN_COMMAND,
                    None::<&str>,
                )],
            ),
        ])
        .with_key_bindings(vec![
            PluginKeyBinding::new("F5", "workspace.compile", PluginKeymapScope::Global),
            PluginKeyBinding::new("S-F5", "workspace.recompile", PluginKeymapScope::Global),
        ])
}

// ─── Default build commands ───────────────────────────────────────────────────

/// Returns the default build command for a given language name.
/// Add new entries here to support additional languages — no other files need
/// to be changed.
pub fn default_build_command(language: &str) -> Option<&'static str> {
    let commands: &[(&str, &str)] = &[
        ("rust", "cargo build"),
        ("rs", "cargo build"),
        ("javascript", "npm run build"),
        ("js", "npm run build"),
        ("typescript", "npm run build"),
        ("ts", "npm run build"),
        ("tsx", "npm run build"),
        ("jsx", "npm run build"),
        ("csharp", "dotnet build"),
        ("cs", "dotnet build"),
        ("python", "python -m py_compile"),
        ("py", "python -m py_compile"),
        ("go", "go build ./..."),
        ("java", "mvn compile"),
        ("c", "make"),
        ("cpp", "make"),
        ("cc", "make"),
        ("toml", "cargo build"),
        ("latex", "latexmk -pdf"),
        ("tex", "latexmk -pdf"),
    ];
    commands
        .iter()
        .find_map(|(lang, cmd)| (*lang == language).then_some(*cmd))
}

// ─── Error line parsing ───────────────────────────────────────────────────────

/// Attempt to parse a compilation error line of the form
/// `path:line` or `path:line:col` (and Rust/cargo variants like
/// `  --> path:line:col`).
/// Returns `(path, line_number, column_number)` on success.
pub fn parse_error_location(line: &str) -> Option<(String, u32, u32)> {
    let line = line.trim();
    // Rust-style:  ` --> src/main.rs:10:5`
    let line = line.strip_prefix("-->").map(str::trim).unwrap_or(line);
    // Split on ':'  —  first segment is the path, rest are line/col.
    // Paths on Windows may start with a drive letter (`C:`), so we need at
    // least 3 segments for `path:line:col` and must handle drive letters.
    let parts: Vec<&str> = line.splitn(4, ':').collect();
    match parts.as_slice() {
        // path:line:col (possibly with trailing message after third colon)
        [path, line_str, col_str, ..] => {
            let line_num = line_str.trim().parse::<u32>().ok()?;
            let col_num = col_str
                .trim()
                .split_once(|c: char| !c.is_ascii_digit())
                .map_or_else(
                    || col_str.trim().parse::<u32>().ok(),
                    |(n, _)| n.parse().ok(),
                )
                .unwrap_or(1);
            if !path.is_empty() && line_num > 0 {
                return Some(((*path).to_owned(), line_num, col_num));
            }
            None
        }
        // path:line
        [path, line_str] => {
            let line_num = line_str.trim().parse::<u32>().ok()?;
            if !path.is_empty() && line_num > 0 {
                return Some(((*path).to_owned(), line_num, 1));
            }
            None
        }
        _ => None,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_package_exports_compile_and_recompile_commands() {
        let pkg = package();
        assert!(
            pkg.commands()
                .iter()
                .any(|c| c.name() == "workspace.compile")
        );
        assert!(
            pkg.commands()
                .iter()
                .any(|c| c.name() == "workspace.recompile")
        );
    }

    #[test]
    fn compile_command_emits_run_command_hook() {
        let pkg = package();
        let cmd = pkg
            .commands()
            .iter()
            .find(|c| c.name() == "workspace.compile")
            .expect("workspace.compile must exist");
        assert!(
            cmd.actions().iter().any(|a| a
                .hook()
                .is_some_and(|h| h.hook_name() == plugin_hooks::RUN_COMMAND)),
            "workspace.compile must emit plugin.run-command"
        );
    }

    #[test]
    fn recompile_command_emits_rerun_command_hook() {
        let pkg = package();
        let cmd = pkg
            .commands()
            .iter()
            .find(|c| c.name() == "workspace.recompile")
            .expect("workspace.recompile must exist");
        assert!(
            cmd.actions().iter().any(|a| a
                .hook()
                .is_some_and(|h| h.hook_name() == plugin_hooks::RERUN_COMMAND)),
            "workspace.recompile must emit plugin.rerun-command"
        );
    }

    #[test]
    fn default_build_command_returns_cargo_for_rust() {
        assert_eq!(default_build_command("rust"), Some("cargo build"));
        assert_eq!(default_build_command("rs"), Some("cargo build"));
    }

    #[test]
    fn default_build_command_returns_latexmk_for_latex() {
        assert_eq!(default_build_command("latex"), Some("latexmk -pdf"));
        assert_eq!(default_build_command("tex"), Some("latexmk -pdf"));
    }

    #[test]
    fn default_build_command_returns_none_for_unknown_language() {
        assert_eq!(default_build_command("brainfuck"), None);
    }

    #[test]
    fn parse_error_location_handles_path_line_col() {
        let (path, line, col) = parse_error_location("src/main.rs:10:5").expect("should parse");
        assert_eq!(path, "src/main.rs");
        assert_eq!(line, 10);
        assert_eq!(col, 5);
    }

    #[test]
    fn parse_error_location_handles_path_line_only() {
        let (path, line, col) = parse_error_location("src/lib.rs:42").expect("should parse");
        assert_eq!(path, "src/lib.rs");
        assert_eq!(line, 42);
        assert_eq!(col, 1);
    }

    #[test]
    fn parse_error_location_handles_rust_arrow_prefix() {
        let (path, line, col) =
            parse_error_location("  --> src/main.rs:10:5").expect("should parse");
        assert_eq!(path, "src/main.rs");
        assert_eq!(line, 10);
        assert_eq!(col, 5);
    }

    #[test]
    fn parse_error_location_returns_none_for_plain_text() {
        assert!(parse_error_location("error: mismatched types").is_none());
        assert!(parse_error_location("   Compiling volt v0.1.0").is_none());
    }

    #[test]
    fn compile_package_binds_f5_keybinding() {
        let pkg = package();
        assert!(pkg.key_bindings().iter().any(|kb| kb.chord() == "F5"));
    }
}
