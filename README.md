> [!WARNING]
> Volt is in early development and issues are to be expected. Please feel free to report bugs and issues in the Issues section.

![volt](./crates/volt/assets/banner.png)

<p align="center">
  <a href="https://github.com/samwdp/volt/releases"><img alt="Latest Release" src="https://img.shields.io/github/v/release/samwdp/volt?style=flat-square&color=blue" /></a>
  <a href="https://github.com/samwdp/volt/stargazers"><img alt="Stars" src="https://img.shields.io/github/stars/samwdp/volt?style=flat-square" /></a>
  <img alt="Platform" src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-brightgreen?style=flat-square" />
  <img alt="License" src="https://img.shields.io/github/license/samwdp/volt?style=flat-square" />
  <a href="https://github.com/samwdp/volt/actions"><img alt="Build" src="https://img.shields.io/github/actions/workflow/status/samwdp/volt/ci.yml?style=flat-square&label=build" /></a>
  <a href="https://github.com/samwdp/volt/actions"><img alt="Build" src="https://img.shields.io/github/actions/workflow/status/samwdp/volt/release.yml?style=flat-square&label=release" /></a>
</p>

---

`volt` is a greenfield native text editor project built in Rust. The long-term direction is an Emacs-inspired, 4coder-style editor with a Rust core, a compiled `user` extension library, and native rendering.

---
## Workspace layout

- `crates/volt` - process entry point and startup bootstrap for the `volt` executable
- `crates/editor-core` - shared runtime and editor domain concepts
- `crates/editor-buffer` - text storage and editing engine
- `crates/editor-render` - rendering abstractions and viewport drawing
- `crates/editor-sdl` - SDL3 platform and windowing integration
- `crates/editor-theme` - theme token registry and palette resolution
- `crates/editor-syntax` - tree-sitter orchestration
- `crates/editor-jobs` - async jobs and compilation runners
- `crates/editor-terminal` - builtin terminal buffers
- `crates/editor-lsp` - language server integration
- `crates/editor-dap` - debug adapter integration
- `crates/editor-git` - magit-style git workflows
- `crates/editor-fs` - workspace file system services
- `crates/editor-picker` - fuzzy picker and list UI abstractions
- `user/sdk` - the only stable ABI crate shared between the host and the compiled user library
- `crates/editor-plugin-host` - plugin hosting and loading services
- `user` - compiled user extension library and packages
- `xtask` - developer automation commands

## Developer commands

- `cargo xtask fmt` - format the workspace
- `cargo xtask fmt-check` - verify formatting in CI
- `cargo xtask check` - run `cargo check --workspace`
- `cargo xtask clippy` - run clippy with warnings denied
- `cargo xtask test` - run workspace tests
- `cargo xtask ci` - run formatting, check, clippy, and tests

## Building locally

### Build the Volt application

To build the editor binary in debug mode:

```bash
cargo build -p volt
```

For a release build:

```bash
cargo build -p volt --release
```

The executable is written to `target/debug/volt` or `target/release/volt`
(`volt.exe` on Windows).

### Build the user shared library

The compiled user customization layer lives in the `volt-user` crate and is built as both
an `rlib` and a shared library.

To build it in debug mode:

```bash
cargo build -p volt-user
```

For a release build:

```bash
cargo build -p volt-user --release
```

The shared library is written next to the `volt` binary:

- Linux: `target/<profile>/libuser.so`
- macOS: `target/<profile>/libuser.dylib`
- Windows: `target/<profile>/user.dll`

### Build both at the same time

```bash
cargo build -p volt -p volt-user
```

For a release build:

```bash
cargo build -p volt -p volt-user --release
```

### Build the packaged local distribution

To build the local bundle layout used by releases, build both crates together:

```bash
cargo build -p volt -p volt-user --release
```

After that, `target/release/` contains:

- `volt` / `volt.exe`
- the compiled user shared library
- `assets/`
- a copied `user/` tree that can be rebuilt standalone

The `volt` binary now prefers the shared library that lives next to the executable, so the
release-style rebuild workflow is:

1. build `volt` and `volt-user`
2. edit files under `user/`
3. rebuild just the user library with `cargo build -p volt-user --release`
4. replace the shared library next to `volt`

If you want to rebuild the copied standalone user tree that was staged into the release folder,
you can also run:

```bash
cd target/release/user
cargo build --release -p volt-user
```

You can also point the binary at a specific user library with `VOLT_USER_LIBRARY=/path/to/libuser.so`
(or the platform equivalent file name).

### Linux native dependencies

On Linux, building the SDL/browser-enabled application requires the GTK/WebKit development
packages used in CI. If you hit `pkg-config` errors for `glib-2.0`, `gtk`, or `webkit2gtk`,
install the same packages as the release workflow, for example:

```bash
sudo apt-get install -y pkg-config libgtk-3-dev libwebkit2gtk-4.1-dev
```

## Current status

The repository now has a validated multi-crate foundation that covers the major architecture slices requested for the editor:

- a Cargo workspace with `xtask` automation and CI wiring
- an `editor-core` runtime with the `Window -> Workspace -> Pane/Popup -> Buffer` model
- service, command, hook, and keymap registries
- an `abi_stable`-shaped compiled `user` library with auto-loaded packages
- an SDL3 shell demo using SDL_ttf (FreeType-backed) with split panes, auto-loaded `user/*` packages, Vim-style defaults, searchable pickers, user-defined statusline segments, workspace management, and the current SDL canvas renderer
- a rope-backed `editor-buffer` engine with cursor movement, range edits, undo/redo, streaming file reads, and large-buffer coverage
- an `editor-picker` fuzzy list engine used by the command palette flow
- `editor-jobs` and `editor-terminal` foundations for async command execution, compile-style runs, and terminal transcripts
- `editor-lsp` and `editor-dap` registries for Rust server/adapter session plans
- an `editor-syntax` registry with tree-sitter language registration and Rust capture-to-theme-token mappings from `user/lang/rust.rs`
- an `editor-theme` registry with themes loaded from `user/themes/*.toml`
- `editor-fs` and `editor-git` models for oil-style directory buffers and magit-style status parsing
- the SDL shell prefers a system-installed Berkeley Mono Nerd Font when present, with cross-platform monospace fallbacks otherwise, and now always loads the bundled icon fonts from `crates/volt/assets/font`

You can run the current shell and bootstrap demos with:

`cargo run -p volt`

`cargo run -p volt -- --shell-demo`

`cargo run -p volt -- --shell-hidden`

`cargo run -p volt -- --profile-typing`

`cargo run -p volt -- --bootstrap-demo`

The default launch path opens the visible SDL3 shell on the stable SDL canvas path. The hidden smoke-test path prints the selected backend/renderer so you can verify shell startup. `--profile-typing` keeps per-frame input timing samples in memory and writes a typing profile log on exit so you can inspect which stages are slow while typing. The bootstrap demo prints a startup summary showing the currently wired picker, job, terminal, LSP, DAP, theme, directory, git, and syntax subsystems.

Inside the SDL shell, the default user package wiring now gives you:

- Vim-style normal/insert mode controls from `user/vim.rs`
- `:` and `F3` for the command picker
- `F4` for the buffer picker
- `F5` to toggle the docked popup window
- `F6` for a searchable keybinding picker
- `F7` for the theme picker
- `Ctrl-n`, `Ctrl-p`, and `Enter` to navigate and run picker entries
- a per-buffer statusline composed from `user/statusline.rs`
- `workspace.new`, `workspace.switch`, `workspace.delete`, and `workspace.list-files` commands backed by `user/workspace.rs`

Theme files live under `user/themes/*.toml` and support UI options like font, font size, and
cursor/picker roundness. Bundled icon fonts are loaded automatically at startup and are no longer
