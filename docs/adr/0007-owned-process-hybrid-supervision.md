# Owned process hybrid supervision

Volt must not leave OS child processes alive after Workspace Close, Application Quit, or a hard crash of the UI process. We rejected a registry-only or Drop-only approach (orphans survive crashes) and a supervisor-only or Job-Object-only approach (each misses trees or platforms the other covers). Every Owned Process is therefore both placed in a platform kill-tree (Windows Job Object with kill-on-job-close; Unix process group / parent-death where available) and launched under the existing `volt --process-supervisor` watchdog when applicable, with a single Process Launch API registering tree roots in a global Process Registry.

## Spawn-path contract

Product feature code must not create OS children outside Process Launch. Silent Commands use `ProcessRegistry::run_captured` (or `run_captured_in_app_registry` when a crate cannot thread `EditorRuntime`). Interactive PTY shells use `launch_interactive_session`. Expand-phase dual paths (private registries, raw `Command::spawn` next to Launch) are bugs.

### Non-Owned OS children (explicit allowlist)

These spawns are not Owned Processes and do not go through Process Launch:

- **OS handoff openers** (`cmd /C start`, `open`, `xdg-open`) that deliberately outlive Volt.
- **Env probes** (`fnm`/`nvm`/login-shell `env`) that resolve PATH before Launch; they are short sync `.output()` helpers.
- **PDF page render** (`pdftocairo`) sync utility used while painting a buffer.
- **Process Registry / supervisor internals** (`taskkill`, `tasklist`, supervisor child spawn).
- **Build-time hooks** (`volt` `build.rs` / standalone user `git init`).
- **DAP TCP connect-only** adapters with an empty program (no local child to own).

Grammar install/compile must use streamed Command Stream jobs (Process Launch), not `SyntaxRegistry::install_language` on the highlight hot path.
