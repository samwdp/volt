# `user`

This directory is Volt's compiled customization layer. The `volt-user` crate builds the shared library that Volt loads at startup, so changes here control the default packages, project discovery, themes, fonts, keybindings, and other editor behavior.

## What this package contains

- `lib.rs` wires the user library together and exports the compiled packages, themes, language servers, and debug adapters.
- `config.rs` loads runtime YAML configuration from `user\config.yaml` and the referenced child files under `user\config\`.
- `workspace.rs` controls workspace commands and the directories Volt scans when it looks for projects.
- `theme.rs` loads `user\themes\*.toml`, merges in `user\themes\global.toml`, and sets the default theme order.
- `config.yaml` is the master user config file; it points at child config files for workspace, ACP, UI, and oil settings.
- `themes\global.toml` contains shared editor options and language defaults.
- `themes\*.toml` contains the bundled named themes and their palette/token colors.

## Making configuration changes

Most user-facing changes happen in this folder:

- edit a Rust module such as `workspace.rs`, `picker.rs`, `vim.rs`, or `statusline.rs` when you want to change behavior
- edit `config.yaml` or a file under `config\` when you want to change runtime-loaded settings without recompiling
- edit `themes\global.toml` when you want to change shared options such as font, font size, scrolloff, or language defaults
- edit a named theme file under `themes\` when you want to change colors and token mappings
- rebuild `volt-user` so Volt picks up the new shared library

Volt now supports a phase-1 runtime config layer for selected user-library settings. Changes in these YAML files are picked up while Volt is running, so values like project search roots, ACP clients, picker truncation, terminal defaults, ligatures, pane layout, and oil defaults/keybindings no longer require a rebuild.

## Runtime config: `config.yaml`

The master config file lives at `user\config.yaml` and points at child files:

```yaml
workspace: config/workspace.yaml
acp: config/acp.yaml
ui: config/ui.yaml
oil: config/oil.yaml
```

This is a reference file, not a YAML include. Volt reads the master file, then loads each child file at runtime.

The crate name is `volt-user`, but the built library is named `user`:

- Windows: `target\debug\user.dll` or `target\release\user.dll`
- Linux: `target/debug/libuser.so` or `target/release/libuser.so`
- macOS: `target/debug/libuser.dylib` or `target/release/libuser.dylib`

## Project discovery: `workspace.rs`

Volt's project picker now reads its search locations from `user\config\workspace.yaml`:

```yaml
project_search_roots:
  - path: P:\
    max_depth: 4
  - path: W:\
    max_depth: 4
  - path: C:\Users\sam\
    max_depth: 4
```

To add another place for Volt to search, add another `ProjectSearchRoot::new(...)` entry:

```yaml
  - path: D:\code\
    max_depth: 4
```

Each entry takes:

- a root directory
- a max search depth

If the path does not exist on the current machine, it is filtered out automatically.

## Other runtime-backed config files

- `config\acp.yaml`: ACP client list
- `config\ui.yaml`: picker truncation, ligatures, pane layout, terminal defaults
- `config\oil.yaml`: default hidden/sort/trash settings plus oil keybindings

## Changing the theme and font

### Change the default theme

`user\theme.rs` keeps the default theme first by matching `DEFAULT_THEME_ID`:

```rust
const DEFAULT_THEME_ID: &str = "gruvbox-dark";
```

Change that value to the theme id you want to use by default. The bundled ids currently include:

- `gruvbox-dark`
- `gruvbox-light`
- `rosepine-dark`
- `volt-dark`
- `volt-light`
- `vscode-dark`
- `vscode-light`

### Change theme colors

Named theme definitions live in `user\themes\*.toml`. These files now contain the per-theme color data: the palette plus the token mappings that point at that palette.

To customize the appearance of a specific theme:

- change token colors in `[tokens]`
- change palette values in `[palette]` / `[pallet]`

### Change shared UI options, font, and language defaults

Shared editor options now live in `user\themes\global.toml`:

```toml
[options]
"ui.line-number.relative" = true
scrolloff = 5
font = "Liga Berkeley Mono"
font_size = 18
cursor_roundness = 2
corner_radius = 16
```

This is where you change:

- edit `font` to use a specific installed font, or use `"default"` for Volt's fallback font selection
- edit `font_size` to change text size
- edit `scrolloff`, `cursor_roundness`, `corner_radius`, or other shared UI options

Language-specific defaults also live in `global.toml` under sections like:

```toml
[langs.rust]
indent = 4
format_on_save = false
use_tabs = false
```

That lets you keep formatting and indentation defaults in one place instead of repeating them in every theme file.

If you only want to try a different bundled theme without changing the default config, the current default keymap opens the theme picker with `F6`.

## Building the user package

Run these commands from the repository root.

Build just the user library in debug mode:

```bash
cargo build -p volt-user
```

Build the user library in release mode:

```bash
cargo build -p volt-user --release
```

If you want to rebuild the editor and the user library together:

```bash
cargo build -p volt -p volt-user
```

After rebuilding, start Volt normally or run the hidden smoke path:

```bash
cargo run -p volt -- --shell-hidden
```

If you are working with a packaged build, replace the shared library next to `volt.exe` / `volt` with the newly built `user` library so the packaged binary picks up your changes.
