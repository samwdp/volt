# User Packages

Volt's extension system is a **compiled user library** — every plugin is a Rust
module that lives under the `user/` directory, compiled alongside the editor into
a shared library (`libuser.so` / `libuser.dylib` / `user.dll`).  This guide
explains the package concepts, walks through creating a new plugin from scratch,
and shows how to edit the builtin plugins that ship with Volt.

---

## Table of Contents

- [Architecture Overview](#architecture-overview)
- [Core Concepts](#core-concepts)
  - [Packages](#packages)
  - [Commands](#commands)
  - [Actions](#actions)
  - [Hooks](#hooks)
  - [Keybindings](#keybindings)
  - [Plugin Buffers](#plugin-buffers)
- [Creating a New Plugin](#creating-a-new-plugin)
  - [Step 1 — Create the Module File](#step-1--create-the-module-file)
  - [Step 2 — Define the Package](#step-2--define-the-package)
  - [Step 3 — Register the Module](#step-3--register-the-module)
  - [Step 4 — Build and Test](#step-4--build-and-test)
- [Complete Plugin Examples](#complete-plugin-examples)
  - [Minimal Plugin — Hook-Only Commands](#minimal-plugin--hook-only-commands)
  - [Buffer Plugin — Custom Evaluator](#buffer-plugin--custom-evaluator)
  - [Language Plugin — Tree-Sitter and LSP](#language-plugin--tree-sitter-and-lsp)
- [Editing Builtin Plugins](#editing-builtin-plugins)
  - [Changing Keybindings](#changing-keybindings)
  - [Adding a Command to an Existing Plugin](#adding-a-command-to-an-existing-plugin)
  - [Changing the Terminal Shell](#changing-the-terminal-shell)
  - [Adding a Build Command for a New Language](#adding-a-build-command-for-a-new-language)
  - [Modifying Oil Directory Browser Defaults](#modifying-oil-directory-browser-defaults)
  - [Editing the Statusline](#editing-the-statusline)
- [Autocomplete Providers](#autocomplete-providers)
- [Adding Language Support](#adding-language-support)
- [Building and Testing](#building-and-testing)

---

## Architecture Overview

```
┌────────────────────────────────────┐
│          volt (binary)             │  Editor executable
├────────────────────────────────────┤
│  editor-plugin-host                │  Loads packages, wires hooks & commands
├────────────────────────────────────┤
│  user/sdk  (editor-plugin-api)     │  Stable ABI surface — shared types
├────────────────────────────────────┤
│  user/  (volt-user library)        │  Your compiled plugins live here
│   ├─ lib.rs                        │  Package registry & trait impl
│   ├─ vim.rs                        │  Vim bindings
│   ├─ calculator.rs                 │  Calculator evaluator
│   ├─ compile.rs                    │  Build / compile integration
│   ├─ lsp.rs                        │  LSP lifecycle
│   ├─ lang/                         │  Per-language configs
│   └─ ...                           │  26+ modules
└────────────────────────────────────┘
```

The **plugin host** (`crates/editor-plugin-host`) reads `PluginPackage`
metadata from the user library and registers commands, keybindings, hooks, and
buffers into the editor runtime.  The **user/sdk** crate (`editor-plugin-api`) is
the only stable ABI boundary — both the host and the user library depend on it.

The user library is compiled as both a **cdylib** (shared library for runtime
loading) and an **rlib** (for static linking during development).

---

## Core Concepts

### Packages

A `PluginPackage` is the top-level unit of extension.  It bundles:

| Field              | Description                                              |
|--------------------|----------------------------------------------------------|
| `name`             | Unique identifier (e.g. `"calculator"`, `"lsp"`)         |
| `auto_load`        | `true` to register at startup, `false` for on-demand     |
| `description`      | Human-readable summary                                   |
| `commands`         | Commands the package exports                             |
| `key_bindings`     | Keyboard chords mapped to those commands                 |
| `hook_declarations`| Custom hooks the package introduces                      |
| `hook_bindings`    | Subscriptions — run a command when a hook fires          |
| `buffers`          | Plugin-owned buffer types (e.g. calculator, git-status)  |

```rust
PluginPackage::new("my-plugin", true, "A short description.")
    .with_commands(vec![/* ... */])
    .with_key_bindings(vec![/* ... */])
    .with_hook_declarations(vec![/* ... */])
    .with_hook_bindings(vec![/* ... */])
    .with_buffers(vec![/* ... */])
```

### Commands

A `PluginCommand` has a name, a description, and a list of **actions** to
execute.

```rust
PluginCommand::new(
    "my-plugin.greet",
    "Logs a greeting message.",
    vec![PluginAction::log_message("Hello from my-plugin!")],
)
```

### Actions

Each command carries one or more `PluginAction` values.  There are three kinds:

| Factory method                     | What it does                                   |
|------------------------------------|------------------------------------------------|
| `PluginAction::log_message(msg)`   | Writes a diagnostic message through the host   |
| `PluginAction::open_buffer(name, kind, popup_title)` | Creates or surfaces a buffer    |
| `PluginAction::emit_hook(hook, detail)` | Fires a hook event for other subscribers  |

```rust
// Log a message
PluginAction::log_message("Build started.")

// Open a workspace buffer
PluginAction::open_buffer("*calculator*", "calculator", None::<&str>)

// Open a buffer in a popup window
PluginAction::open_buffer("*terminal-popup*", "terminal", Some("Terminal"))

// Emit a hook (other plugins or the host can react)
PluginAction::emit_hook("lsp.server-start", Some("rust-analyzer"))
```

### Hooks

Hooks are the event bus.  A package can **declare** new hooks and **bind** to
existing ones.

**Declaring a hook** tells the runtime that the hook exists:

```rust
PluginHookDeclaration::new(
    "lang.rust.attached",
    "Runs after the Rust language package attaches to a buffer.",
)
```

**Binding to a hook** subscribes a command so it runs when the hook fires.  An
optional `detail_filter` restricts the subscription to a specific detail value:

```rust
PluginHookBinding::new(
    "buffer.file-open",         // hook to subscribe to
    "lang-rust.auto-attach",    // subscriber identifier
    "lang-rust.attach",         // command to run
    Some(".rs"),                 // only fire for .rs files
)
```

Common host-owned hooks include:

| Hook                    | Detail              | Description                        |
|-------------------------|---------------------|------------------------------------|
| `buffer.file-open`      | file extension      | Fires when a file buffer opens     |
| `buffer.save`           | —                   | Fires to save the active buffer    |
| `buffer.close`          | —                   | Fires to close the active buffer   |
| `plugin.evaluate`       | —                   | Evaluates a plugin buffer          |
| `plugin.switch-pane`    | —                   | Switches panes in a split buffer   |
| `plugin.run-command`    | language name       | Opens a compilation buffer         |
| `plugin.rerun-command`  | —                   | Re-runs the last compilation       |
| `ui.picker.open`        | picker variant      | Opens a picker popup               |
| `ui.popup.toggle`       | —                   | Toggles the popup window           |
| `ui.pane.split-*`       | —                   | Splits the active pane             |
| `workspace.save`        | —                   | Saves all modified buffers         |
| `workspace.format`      | —                   | Formats the active buffer          |

### Keybindings

A `PluginKeyBinding` maps a keyboard chord to a command within a scope.

```rust
PluginKeyBinding::new(
    "F5",                           // chord
    "workspace.compile",            // command
    PluginKeymapScope::Global,      // scope
)
```

Chord strings use `Ctrl+`, `Shift+`, `Alt+` prefixes (e.g. `"Ctrl+Shift+h"`).
The shorthand `C-` is equivalent to `Ctrl+` (so `"C-c C-c"` means press
`Ctrl+c` twice).

**Scopes** control when the binding is active:

| Scope                        | When active                             |
|------------------------------|-----------------------------------------|
| `PluginKeymapScope::Global`  | Always, regardless of focus             |
| `PluginKeymapScope::Workspace` | Only when a workspace pane is focused |
| `PluginKeymapScope::Popup`   | Only inside a popup window              |

Bindings can also be restricted to a **Vim mode**:

```rust
PluginKeyBinding::new("Ctrl+n", "popup.next", PluginKeymapScope::Global)
    .with_vim_mode(PluginVimMode::Normal)
```

| Vim mode              | When active                              |
|-----------------------|------------------------------------------|
| `PluginVimMode::Any`  | Always (default)                        |
| `PluginVimMode::Normal` | Vim normal mode only                  |
| `PluginVimMode::Insert` | Vim insert mode only                  |
| `PluginVimMode::Visual` | Vim visual mode only                  |

### Plugin Buffers

Plugins can declare custom buffer types.  The host manages the buffer lifecycle;
the plugin provides initial content and an optional split-pane layout.

```rust
PluginBuffer::new("calculator", vec!["a = 1", "b = 2", "sqrt(a + b)"])
    .with_sections(PluginBufferSections::new(
        "Input",                                   // input pane title
        "Output",                                  // output pane title
        1,                                         // minimum output rows
        vec!["(press Ctrl+c Ctrl+c to evaluate)"], // initial output lines
    ))
    .with_evaluate_handler("calculator.evaluate-buffer")
```

When the user triggers `plugin.evaluate`, the host calls back into the user
library's evaluator for the buffer kind and replaces the output section with the
returned lines.

---

## Creating a New Plugin

This walkthrough creates a plugin called **"hello"** that logs a greeting when
invoked from the command palette.

### Step 1 — Create the Module File

Create `user/hello.rs`:

```rust
use editor_plugin_api::{
    PluginAction, PluginCommand, PluginKeyBinding, PluginKeymapScope,
    PluginPackage,
};

/// Returns the metadata for the hello package.
pub fn package() -> PluginPackage {
    PluginPackage::new("hello", true, "A simple greeting plugin.")
        .with_commands(vec![
            PluginCommand::new(
                "hello.greet",
                "Logs a friendly greeting to the message log.",
                vec![PluginAction::log_message("Hello from Volt!")],
            ),
        ])
        .with_key_bindings(vec![
            PluginKeyBinding::new(
                "Ctrl+Shift+h",
                "hello.greet",
                PluginKeymapScope::Global,
            ),
        ])
}
```

### Step 2 — Define the Package

There is nothing else to implement in the module — the `package()` function
returns all the metadata the host needs.

### Step 3 — Register the Module

Open `user/lib.rs` and add the new module declaration near the top of the file
with the other `pub mod` statements:

```rust
/// A simple greeting plugin.
pub mod hello;
```

Then add the package to the `packages()` function:

```rust
pub fn packages() -> Vec<PluginPackage> {
    let mut pkgs = vec![
        buffer::package(),
        acp::package(),
        // ... existing packages ...
        hello::package(),      // ← add this line
    ];
    pkgs.extend(lang::packages());
    pkgs
}
```

### Step 4 — Build and Test

```bash
# Build the user library
cargo build -p volt-user

# Run the full test suite
cargo xtask test

# Smoke-test the editor (one-frame headless run)
cargo run -p volt -- --shell-hidden

# Run the bootstrap demo (prints registered packages)
cargo run -p volt -- --bootstrap-demo
```

Your new `hello.greet` command will appear in the command palette (`F3` or `:`),
and `Ctrl+Shift+h` will trigger it from anywhere.

---

## Complete Plugin Examples

### Minimal Plugin — Hook-Only Commands

The simplest plugins emit hooks and let the host handle the behavior.  Here is
`user/pane.rs` (builtin):

```rust
use editor_plugin_api::{PluginAction, PluginCommand, PluginPackage};

pub fn package() -> PluginPackage {
    PluginPackage::new("pane", true, "Pane layout and split commands.")
        .with_commands(vec![
            hook_command(
                "pane.split-horizontal",
                "Splits the active workspace horizontally.",
                "ui.pane.split-horizontal",
            ),
            hook_command(
                "pane.split-vertical",
                "Splits the active workspace vertically.",
                "ui.pane.split-vertical",
            ),
            hook_command(
                "pane.close",
                "Closes the currently focused split.",
                "ui.pane.close",
            ),
        ])
}

fn hook_command(name: &str, description: &str, hook_name: &str) -> PluginCommand {
    PluginCommand::new(
        name,
        description,
        vec![PluginAction::emit_hook(hook_name, None::<&str>)],
    )
}
```

### Buffer Plugin — Custom Evaluator

The **calculator** plugin (`user/calculator.rs`) shows how to create a
split-pane buffer with an evaluate cycle:

```rust
use editor_plugin_api::{
    PluginAction, PluginBuffer, PluginBufferSections, PluginCommand,
    PluginKeyBinding, PluginKeymapScope, PluginPackage,
    buffer_kinds, plugin_hooks,
};

pub const BUFFER_NAME: &str = "*calculator*";
pub const EVALUATE_HANDLER: &str = "calculator.evaluate-buffer";
pub const EVALUATE_CHORD: &str = "C-c C-c";
pub const SWITCH_PANE_CHORD: &str = "Ctrl+Tab";

pub fn package() -> PluginPackage {
    PluginPackage::new("calculator", true, "Expression evaluator buffer.")
        .with_commands(vec![
            PluginCommand::new(
                "calculator.open",
                "Open the calculator buffer in the active pane.",
                vec![PluginAction::open_buffer(
                    BUFFER_NAME,
                    buffer_kinds::CALCULATOR,
                    None::<&str>,
                )],
            ),
            PluginCommand::new(
                "calculator.evaluate",
                "Evaluate the calculator input.",
                vec![PluginAction::emit_hook(
                    plugin_hooks::EVALUATE,
                    None::<&str>,
                )],
            ),
        ])
        .with_buffers(vec![
            PluginBuffer::new(
                buffer_kinds::CALCULATOR,
                initial_buffer_lines(),
            )
            .with_sections(PluginBufferSections::new(
                "Input",
                "Output",
                1,
                vec!["(press Ctrl+c Ctrl+c to evaluate)".to_owned()],
            ))
            .with_evaluate_handler(EVALUATE_HANDLER),
        ])
        .with_key_bindings(vec![
            PluginKeyBinding::new(
                EVALUATE_CHORD,
                "calculator.evaluate",
                PluginKeymapScope::Workspace,
            ),
        ])
}

pub fn initial_buffer_lines() -> Vec<String> {
    vec![
        "# Write expressions below.".to_owned(),
        String::new(),
        "a = 1".to_owned(),
        "b = 2".to_owned(),
        "sqrt(a + b)".to_owned(),
    ]
}

/// Called by the host when plugin.evaluate fires for this buffer kind.
pub fn evaluate(input: &str) -> Vec<String> {
    // Parse and evaluate each line, returning output lines.
    // See user/calculator.rs for the full implementation.
    todo!()
}
```

The evaluate handler must also be wired into `UserLibraryImpl` in `user/lib.rs`
so the host can call it:

```rust
fn run_plugin_buffer_evaluator(&self, handler_id: &str, input: &str) -> Vec<String> {
    match handler_id {
        calculator::EVALUATE_HANDLER => calculator::evaluate(input),
        // Add your handler here:
        // my_plugin::EVALUATE_HANDLER => my_plugin::evaluate(input),
        _ => vec![format!("unknown evaluator: {handler_id}")],
    }
}
```

### Language Plugin — Tree-Sitter and LSP

Language packages combine tree-sitter grammars, theme mappings, and hook bindings
so that opening a file with a matching extension automatically attaches language
features.  Here is a simplified version of the Rust language plugin
(`user/lang/rust.rs`):

```rust
use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding,
    PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-rust",
        true,
        "Rust language defaults, tree-sitter mapping, and startup hooks.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "lang-rust.attach",
            "Attaches Rust language defaults to the active workspace.",
            vec![
                PluginAction::log_message("Rust language package attached."),
                PluginAction::emit_hook(
                    "workspace.formatter.register",
                    Some("rust|rustfmt"),
                ),
            ],
        ),
    ])
    .with_hook_declarations(vec![
        PluginHookDeclaration::new(
            "lang.rust.attached",
            "Runs after the Rust language package attaches to a buffer.",
        ),
    ])
    .with_hook_bindings(vec![
        // When a .rs file opens, automatically run lang-rust.attach
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-rust.auto-attach",
            "lang-rust.attach",
            Some(".rs"),
        ),
    ])
}

pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "rust",
        ["rs"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-rust.git",
            ".",
            "src",
            "tree-sitter-rust",
            "tree_sitter_rust",
        ),
        [
            CaptureThemeMapping::new("comment",  "syntax.comment"),
            CaptureThemeMapping::new("keyword",  "syntax.keyword"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("string",   "syntax.string"),
            CaptureThemeMapping::new("type",     "syntax.type"),
            // ... additional mappings
        ],
    )
}
```

---

## Editing Builtin Plugins

All builtin plugins live as `.rs` files under `user/`.  You can edit them
directly — they are designed to be user-customizable.

### Changing Keybindings

Open the plugin file and find the `.with_key_bindings(vec![...])` call.  Change
the chord string to your preferred binding.

**Example — change the compile keybinding from `F5` to `Ctrl+B`:**

In `user/compile.rs`:

```rust
// Before
PluginKeyBinding::new("F5", "workspace.compile", PluginKeymapScope::Global),

// After
PluginKeyBinding::new("Ctrl+B", "workspace.compile", PluginKeymapScope::Global),
```

**Example — change the Vim leader key:**

In `user/vim.rs`, change the constant at the top of the file:

```rust
// Before
const LEADER_KEY: &str = "Space";

// After
const LEADER_KEY: &str = "Comma";
```

### Adding a Command to an Existing Plugin

Find the `.with_commands(vec![...])` block in the plugin's `package()` function
and add a new `PluginCommand` entry.

**Example — add a "save all" command to the buffer plugin:**

In `user/buffer.rs`:

```rust
pub fn package() -> PluginPackage {
    PluginPackage::new("buffer", true, "Buffer save and management commands.")
        .with_commands(vec![
            // ... existing commands ...
            PluginCommand::new(
                "buffer.save-all",
                "Saves all modified file buffers.",
                vec![PluginAction::emit_hook("workspace.save", None::<&str>)],
            ),
        ])
}
```

### Changing the Terminal Shell

In `user/terminal.rs`, edit the `default_shell_program()` function:

```rust
pub fn default_shell_program() -> String {
    if cfg!(target_os = "windows") {
        // Change to your preferred Windows shell:
        "pwsh".to_owned()
        // "bash".to_owned()
        // "nu".to_owned()
    } else {
        // Change to your preferred Unix shell:
        env::var("SHELL")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| "/bin/sh".to_owned())
    }
}
```

### Adding a Build Command for a New Language

In `user/compile.rs`, add an entry to the `default_build_command()` function:

```rust
pub fn default_build_command(language: &str) -> Option<&'static str> {
    let commands: &[(&str, &str)] = &[
        ("rust", "cargo build"),
        ("typescript", "npm run build"),
        // Add your language here:
        ("zig", "zig build"),
        ("haskell", "cabal build"),
    ];
    commands
        .iter()
        .find_map(|(lang, cmd)| (*lang == language).then_some(*cmd))
}
```

### Modifying Oil Directory Browser Defaults

In `user/oil.rs`, edit the `defaults()` function to change the initial state of
the directory browser:

```rust
pub fn defaults() -> OilDefaults {
    OilDefaults {
        show_hidden: true,       // show dotfiles by default
        sort_mode: OilSortMode::TypeThenName,
        trash_enabled: true,     // use trash instead of permanent delete
    }
}
```

You can also change keybindings in the `keybindings()` function in the same
file.

### Editing the Statusline

The statusline is rendered by `user/statusline.rs`.  You receive a
`StatuslineContext` with fields like `vim_mode`, `buffer_name`, `line`, `column`,
`git_branch`, and `lsp_diagnostics`.  Edit the rendering functions to customize
what appears in the statusline.

---

---

## Autocomplete Providers

Volt's autocomplete system is backed by a list of **providers** registered in
`user/autocomplete.rs` via the `backends()` function.  Providers can be grouped
into an **or-group**: once the highest-priority provider in a group returns
results, all lower-priority providers in the same group are skipped.

### Provider Priority Order

| Priority | Provider     | Or-Group | Notes                                                       |
|----------|--------------|----------|-------------------------------------------------------------|
| 1        | `lsp`        | `source` | Live completions from an attached language server           |
| 2        | `calculator` | `source` | Built-in function/constant names; only active in calculator buffers |
| 3        | `buffer`     | `source` | Words already present in the open buffer                    |

Because all three share the `source` or-group, only the highest-priority
provider with results is shown.  Inside a calculator buffer, if the
`calculator` provider returns matches, the `buffer` provider is silently
skipped.

### How the Source Group Works

```rust
// user/autocomplete.rs — backends()

pub fn backends() -> Vec<AutocompleteProviderConfig> {
    vec![
        // 1. LSP wins if an LSP server is attached and returns completions.
        AutocompleteProviderConfig::new(PROVIDER_LSP, "LSP", /* icon */)
            .with_or_group(PROVIDER_SOURCE_GROUP),

        // 2. Calculator provider is checked next; only active inside
        //    calculator buffers (buffer_kind filter).  If it returns
        //    results, buffer completions are suppressed.
        calculator::autocomplete_provider()
            .with_or_group(PROVIDER_SOURCE_GROUP),

        // 3. Buffer word completions — fallback when LSP and calculator
        //    both return nothing.
        AutocompleteProviderConfig::new(PROVIDER_BUFFER, "Buffer", /* icon */)
            .with_or_group(PROVIDER_SOURCE_GROUP),
    ]
}
```

### Adding Your Own Provider

To add a manual autocomplete provider to a plugin buffer, define an
`autocomplete_provider()` function in your plugin module and add it to the
`backends()` vector in `user/autocomplete.rs`.  Call
`.with_buffer_kind(YOUR_KIND)` to limit it to your buffer type and
`.with_or_group(PROVIDER_SOURCE_GROUP)` to slot it into the shared fallback
chain.

```rust
// user/myplugin.rs
pub fn autocomplete_provider() -> AutocompleteProviderConfig {
    AutocompleteProviderConfig::new(
        "myplugin",
        "My Plugin",
        MY_PROVIDER_ICON,
        MY_ITEM_ICON,
    )
    .with_buffer_kind(MY_BUFFER_KIND)
    .with_or_group(autocomplete::PROVIDER_SOURCE_GROUP)
    .with_items(my_autocomplete_items())
}

// user/autocomplete.rs — add to backends():
myplugin::autocomplete_provider()
```

---

## Adding Language Support

Adding a new language requires creating a module in `user/lang/` and registering
it.

### Step 1 — Create the Language Module

Create `user/lang/python.rs`:

```rust
use editor_plugin_api::{
    PluginAction, PluginCommand, PluginHookBinding,
    PluginHookDeclaration, PluginPackage,
};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};

pub fn package() -> PluginPackage {
    PluginPackage::new(
        "lang-python",
        true,
        "Python language defaults and tree-sitter mapping.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "lang-python.attach",
            "Attaches Python language defaults to the active workspace.",
            vec![
                PluginAction::log_message("Python language package attached."),
            ],
        ),
    ])
    .with_hook_declarations(vec![
        PluginHookDeclaration::new(
            "lang.python.attached",
            "Runs after the Python language package attaches.",
        ),
    ])
    .with_hook_bindings(vec![
        PluginHookBinding::new(
            "buffer.file-open",
            "lang-python.auto-attach",
            "lang-python.attach",
            Some(".py"),
        ),
    ])
}

pub fn syntax_language() -> LanguageConfiguration {
    LanguageConfiguration::from_grammar(
        "python",
        ["py"],
        GrammarSource::new(
            "https://github.com/tree-sitter/tree-sitter-python.git",
            ".",
            "src",
            "tree-sitter-python",
            "tree_sitter_python",
        ),
        [
            CaptureThemeMapping::new("comment",  "syntax.comment"),
            CaptureThemeMapping::new("keyword",  "syntax.keyword"),
            CaptureThemeMapping::new("function", "syntax.function"),
            CaptureThemeMapping::new("string",   "syntax.string"),
            CaptureThemeMapping::new("type",     "syntax.type"),
            CaptureThemeMapping::new("variable", "syntax.variable"),
            CaptureThemeMapping::new("number",   "syntax.constant"),
            CaptureThemeMapping::new("operator", "syntax.operator"),
        ],
    )
}
```

### Step 2 — Register in `user/lang/mod.rs`

```rust
/// Python language support and theme mappings.
pub mod python;

pub fn packages() -> Vec<editor_plugin_api::PluginPackage> {
    vec![
        // ... existing languages ...
        python::package(),
    ]
}

pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    vec![
        // ... existing languages ...
        python::syntax_language(),
    ]
}
```

### Step 3 — Add an LSP Server (Optional)

In `user/lsp.rs`, add a new `LanguageServerSpec` to the `language_servers()`
function for your language's LSP server.

---

## Building and Testing

### Developer Commands

| Command                        | Purpose                                       |
|--------------------------------|-----------------------------------------------|
| `cargo build -p volt-user`     | Build the user library (debug)                |
| `cargo build -p volt`          | Build the editor binary (debug)               |
| `cargo build -p volt -p volt-user --release` | Release build of both         |
| `cargo xtask fmt`              | Format the workspace                          |
| `cargo xtask check`            | Run cargo check                               |
| `cargo xtask clippy`           | Run clippy (warnings → errors)                |
| `cargo xtask test`             | Run all workspace tests                       |
| `cargo xtask ci`               | Full CI validation                            |

### Running and Verifying

```bash
# Launch the SDL shell (interactive)
cargo run -p volt

# One-frame headless smoke test
cargo run -p volt -- --shell-hidden

# Bootstrap demo — prints registered packages and subsystem summary
cargo run -p volt -- --bootstrap-demo
```

### Running a Single Test

```bash
# Run tests in the user crate matching a name pattern
cargo test -p volt-user <test_name>

# Exact match with module path
cargo test -p volt-user tests::user_library_exports_themes -- --exact
```

### Output Artifacts

| Platform | User library                  | Editor binary        |
|----------|-------------------------------|----------------------|
| Linux    | `target/<profile>/libuser.so` | `target/<profile>/volt` |
| macOS    | `target/<profile>/libuser.dylib` | `target/<profile>/volt` |
| Windows  | `target/<profile>/user.dll`   | `target/<profile>/volt.exe` |

### Lint Policy

The workspace enforces these lints (defined in the root `Cargo.toml`):

- `unsafe_code` is **forbidden**
- `dbg!`, `todo!`, and `unwrap()` are **denied**
- `cargo xtask clippy` promotes warnings to errors

Always run `cargo xtask clippy` before submitting changes.
