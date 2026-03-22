# Copilot instructions for `volt`

## Build, test, and lint

- Use `cargo xtask fmt`, `cargo xtask fmt-check`, `cargo xtask check`, `cargo xtask clippy`, `cargo xtask test`, and `cargo xtask ci`. `cargo xtask ci` is the full validation path used by CI.
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
- `crates\editor-plugin-api` defines the stable plugin ABI with `abi_stable` types. `crates\editor-plugin-host` translates `PluginPackage` metadata into runtime commands, hook declarations/subscriptions, and keybindings.
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
- Syntax and themes are coupled by token names. `user\lang\*.rs` maps tree-sitter captures to `syntax.*` tokens, and `user\theme.rs` must define the corresponding theme tokens. Grammar-backed languages install under `user\lang\grammars` by default.
- Respect the workspace lint policy from the root `Cargo.toml`: `unsafe_code` is forbidden, `dbg!`, `todo!`, and `unwrap()` are denied, and `cargo xtask clippy` promotes warnings to errors.
- Keep the `editor-sdl` SDL_ttf configuration intact on Windows unless you are intentionally revisiting the build setup: the crate enables `sdl3-ttf-sys` with `no-sdlttf-harfbuzz` to avoid Windows linker problems in the vendored SDL_ttf build.
