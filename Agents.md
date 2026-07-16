# Copilot instructions for `volt`

## Build, test, and lint

- Use `cargo xtask fmt`, `cargo xtask fmt-check`, `cargo xtask check`, `cargo xtask clippy`, `cargo xtask test`, and `cargo xtask ci`. `cargo xtask ci` `cargo clippy --workspace --all-targets -- -D warnings` is the full validation path used by CI.
- Always run `cargo xtask ci` when validating any changes and fix them before you mark anything as complete
- Run a single test with `cargo test -p <package> <test_name>`. Example: `cargo test -p volt-user user_library_exports_themes`.
- For an exact match, use the module-qualified test name: `cargo test -p volt-user tests::user_library_exports_themes -- --exact`.
- Before finishing a task, run the runtime smoke test: `cargo run -p volt -- --shell-hidden` (unless the user asks you to skip runtime checks).
- Useful runtime checks:
  - `cargo run -p volt` launches the SDL shell demo.
  - `cargo run -p volt -- --shell-hidden` runs the one-frame hidden SDL smoke test.
  - `cargo run -p volt -- --bootstrap-demo` exercises the non-UI bootstrap path and prints a subsystem summary.

## Architecture

- This repository is a Cargo workspace with editor/runtime crates under `crates\*`, the compiled user extension library in `user`, and developer automation in `xtask`.
- `crates\volt` is the executable entry point. The default path is thin and launches `editor_sdl::run_demo_shell`; the `--bootstrap-demo` path is the clearest non-UI bootstrap because it builds an `EditorRuntime`, registers core services, commands, hooks, and keybindings, loads user packages, then installs LSP, DAP, syntax, and theme registries.
- `crates\editor-core` owns the central `EditorRuntime`. It bundles the `EditorModel`, service registry, command registry, hook bus, and keymap registry. The model shape is `Window -> Workspace -> Pane/Popup -> Buffer`.
- `user\sdk` is the only stable plugin ABI crate and defines the `abi_stable` types shared across the host/user boundary. `crates\editor-plugin-host` translates `PluginPackage` metadata into runtime commands, hook declarations/subscriptions, and keybindings.
- The `user` crate is the compiled customization layer. It exports packages, syntax languages, themes, language servers, and debug adapters. Both the bootstrap demo and the SDL shell consume those exports.
- `crates\editor-sdl` is not just rendering. It builds its own `EditorRuntime`, registers hook subscribers for cursor movement, Vim editing, pickers, popup control, and workspace actions, then stores shell-specific UI state in runtime services.
- Supporting crates are intentionally separated by domain: `editor-buffer` for rope-backed text editing, `editor-fs` for workspace discovery and directory buffers, `editor-syntax` for tree-sitter registration/install/loading, `editor-theme` for token resolution, `editor-jobs` and `editor-terminal` for external command execution, and `editor-lsp` / `editor-dap` for session planning.

## Repository conventions

- Keep user-facing behavior in `user\*.rs` when possible. Vim bindings, picker commands, statusline segments, theme tokens, language registrations, LSP/DAP defaults, and workspace discovery roots are intended to be edited there and recompiled.
- `user::packages()` is the source of compiled-in packages. Startup behavior depends on each package's `auto_load` flag: auto-loaded packages are registered on boot, while packages with `auto_load = false` (for example `git`) are compiled in but not activated automatically.
- Prefer the package metadata path over ad hoc wiring. Most user packages are intentionally declarative: commands are built from `PluginAction::{LogMessage, OpenBuffer, EmitHook}` plus optional `PluginHookDeclaration` and `PluginHookBinding` entries.
- Hooks matter as much as commands. If a package emits a hook or binds a command to a hook detail, the runtime/UI layer must subscribe to that hook or the feature will register but do nothing. This is especially important for `editor.cursor.*`, `editor.vim.edit`, `ui.picker.*`, and workspace-related flows.
- Keybindings are scoped (`Global`, `Workspace`, `Popup`) and can also be Vim-mode-specific (`Any`, `Normal`, `Insert`, `Visual`). Match existing scope/mode usage before adding new bindings.
- Workspace discovery is user-configured in `user\workspace.rs`. The current search roots are `P:\` and `W:\` with a max depth of 4, so project-picker behavior is not hard-coded in core crates.
- Syntax and themes are coupled by token names. `user\lang\*.rs` maps tree-sitter captures to `syntax.*` tokens, and `user\theme.rs` must define the corresponding theme tokens. Grammar-backed languages install under the per-user Volt grammar directory by default (`%LOCALAPPDATA%\volt\grammars` on Windows, `$XDG_DATA_HOME/volt/grammars` or `~/.local/share/volt/grammars` elsewhere), or `VOLT_GRAMMAR_DIR` when overridden.
- Respect the workspace lint policy from the root `Cargo.toml`: `unsafe_code` is forbidden, `dbg!`, `todo!`, and `unwrap()` are denied, and `cargo xtask clippy` promotes warnings to errors.
- Keep the `editor-sdl` SDL_ttf configuration intact on Windows unless you are intentionally revisiting the build setup: the crate enables `sdl3-ttf-sys` with `no-sdlttf-harfbuzz` to avoid Windows linker problems in the vendored SDL_ttf build.

## caveman
You are in caveman mode. Use the caveman skill

## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- For cross-module "how does X relate to Y" questions, prefer `graphify query "<question>"`, `graphify path "<A>" "<B>"`, or `graphify explain "<concept>"` over grep — these traverse the graph's EXTRACTED + INFERRED edges instead of scanning files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)

## Cursor Cloud specific instructions

These notes are for cloud agents running in the pre-provisioned VM (system libs, the Rust
toolchain, and `cargo`-fetched dependencies are already baked into the snapshot). They capture
non-obvious runtime caveats, not one-off setup steps.

### Toolchain / build
- Volt needs Rust **stable >= 1.91** (edition 2024). The snapshot ships a recent stable
  toolchain; if a fresh VM ever regresses to an older stable, run `rustup default stable`.
- Linux native libraries (GTK3, WebKit2GTK 4.1, SDL/X11/mesa/audio) are required and already
  installed. If `pkg-config` errors reappear, reinstall the exact package set from
  `.github/workflows/ci.yml` (documented in `README.md` under "Linux native dependencies").

### Running the GUI (non-obvious)
- Volt is a native SDL3 desktop app, so it needs a display. Use the VNC desktop on
  `DISPLAY=:1` (this is what the Cursor Desktop pane and screen recording capture), or a headless
  `Xvfb` display. There is **no GPU**, so export `LIBGL_ALWAYS_SOFTWARE=1` and set a writable
  `XDG_RUNTIME_DIR` (e.g. `/tmp/xdg-runtime`, `chmod 700`).
- Startup eagerly constructs the embedded browser host (WebKitGTK via `wry`), so even
  `--shell-hidden` requires a valid display + working GTK; it will not run purely headless.
- Run the editor: `DISPLAY=:1 XDG_RUNTIME_DIR=/tmp/xdg-runtime LIBGL_ALWAYS_SOFTWARE=1 cargo run -p volt`.
- Headless one-frame smoke test (still needs the display/GTK above): `cargo run -p volt -- --shell-hidden`.
- `--bootstrap-demo` tries to highlight Rust and fails with `GrammarNotInstalled` unless the
  tree-sitter Rust grammar has been installed under `~/.local/share/volt/grammars` (network clone
  + C compile). This is not required for the SDL shell.

### Lint / test caveats on Linux
- Full gate is `cargo xtask ci` (fmt-check -> check -> clippy `-D warnings` -> test), per README.
- `cargo xtask ci` passes end-to-end on Linux (fmt-check, check, clippy `-D warnings`, and the
  full test suite). Tests are separator-agnostic, so run them from any host.
- Windows-only code paths (e.g. the MSVC grammar-compile branch in `editor-syntax`, the
  `#[cfg(windows)]` worktree test in `editor-fs`) can't be exercised on Linux. To at least
  compile/clippy-check them, cross-check against the Windows target, which needs mingw for the
  C-backed crates: `rustup target add x86_64-pc-windows-gnu` (+ `gcc-mingw-w64-x86-64`), then
  e.g. `cargo clippy -p editor-syntax --all-targets --target x86_64-pc-windows-gnu -- -D warnings`.

## Agent skills

### Issue tracker

Issues live as local markdown under `.scratch/<feature>/` (PRDs + numbered issue files; triage via `Status:` lines). See `docs/agents/issue-tracker.md`.

### Triage labels

Canonical role names used as-is (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `CONTEXT.md` + `docs/adr/`. See `docs/agents/domain.md`.
