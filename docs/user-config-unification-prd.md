# User Configuration Unification PRD

## 1. Executive Summary

- **Problem Statement**: Volt splits user-facing behavior across the compiled `user` crate and host crates such as `editor-plugin-host` and `editor-sdl`. This creates duplicated defaults, stringly-typed feature contracts, and first-party capabilities that internal code can use but external users cannot.
- **Proposed Solution**: Make the `user` crate the single source of truth for all editor behavior and policy that is meant to be customizable, then extend `user/sdk` so first-party features and third-party plugins use the same high-level abstractions.
- **Success Criteria**:
  - 100% of non-SDL user-facing defaults currently duplicated in host crates move behind `user/sdk` abstractions consumed from the compiled `user` crate.
  - `editor-plugin-host::NullUserLibrary` contains no behavior-specific policy defaults beyond an empty bootstrap fallback.
  - At least 5 first-party feature areas currently implemented with host-only command/keybinding tables are rebuilt using public `user/sdk` abstractions.
  - Adding or changing a first-party package command, keybinding, buffer contract, or feature spec requires edits only in `user/*` plus shared SDK types, not bespoke `editor-sdl` tables.
  - `cargo xtask ci` and `cargo run -p volt -- --shell-hidden` pass after migration.

## 2. User Experience & Functionality

- **User Personas**:
  - Volt core maintainer evolving built-in features without duplicating policy between crates.
  - Power user editing `user/*.rs` to customize behavior and ship a compiled user library.
  - Third-party plugin author who needs the same feature-building primitives that first-party packages use.

- **User Stories**:
  - As a Volt maintainer, I want first-party feature defaults defined in `user` so host crates stop carrying competing policy.
  - As a plugin author, I want typed SDK builders for commands, buffers, actions, keybindings, hooks, and feature specs so I can build plugins with the same capabilities as built-in packages.
  - As a user customizing Volt, I want behavior changes to live in one place so modifying `user` is sufficient to change compiled runtime behavior.
  - As a maintainer, I want a migration map of remaining host-owned configuration so work can be sequenced and verified crate by crate.

- **Acceptance Criteria**:
  - Every customizable default duplicated in host fallback code has one canonical owner in `user`.
  - Public `user/sdk` exposes typed abstractions for feature areas now represented by ad hoc strings or host-only tables.
  - First-party packages use those public abstractions instead of private host-only equivalents.
  - Host crates retain engine behavior and SDL-only implementation details, but not end-user policy.
  - Documentation includes a migration table listing source crate, target abstraction, rationale, and rollout priority.

- **Non-Goals**:
  - Exposing SDL shell windowing, renderer, layout padding, or other presentation internals as user configuration.
  - Moving low-level engine constants such as protocol timeouts, cache sizes, or filesystem resolution heuristics into `user` unless they directly define end-user behavior.
  - Preserving ABI compatibility during this refactor.
  - Replacing all internal shell code with plugins in one step.

## 3. AI System Requirements

- **Tool Requirements**: Not applicable. This is a product and architecture refactor for local Rust code and plugin APIs.
- **Evaluation Strategy**: Use repository analysis, targeted migration tests, `cargo xtask ci`, and runtime smoke coverage instead of model-quality evaluation.

## 4. Technical Specifications

- **Architecture Overview**:
  - `user/sdk` becomes the public product surface for configurable behavior and plugin composition.
  - `user` becomes the only default policy implementation. It defines built-in packages, feature specs, commands, keybindings, buffers, feature-specific defaults, and textual/help surfaces.
  - `editor-plugin-host` becomes a thin adapter that loads `UserLibrary`, registers exported specs, and provides only minimal empty fallback behavior when no library is available.
  - `editor-sdl` remains owner of rendering, input dispatch, and host execution, but consumes typed feature specs instead of private command tables and scattered hook strings where behavior is intended to be extensible.

- **Integration Points**:
  - `user/sdk::UserLibrary`:
    - Expand from provider/default getters into typed feature registries and builders.
    - Promote first-party reusable types out of `user/*` into `user/sdk` when those types describe plugin-visible behavior.
  - `user`:
    - Continue exporting packages, themes, syntax, LSP, DAP, and behavior defaults.
    - Add first-party feature specs for areas still split between `user` metadata and host-side command execution tables.
  - `editor-plugin-host`:
    - Remove duplicated product defaults from `NullUserLibrary`.
    - Resolve feature specs generically rather than hard-coding fallback command identities and UI strings.
  - `editor-sdl`:
    - Replace host-only command/keybinding tables with SDK-backed specs where behavior is meant to be user-extensible.
    - Keep SDL-only mechanics private.

- **Security & Privacy**:
  - No new network or credential surface should be introduced by this refactor.
  - Existing sensitive flows such as DB connection storage and external command execution must continue using current host enforcement paths.
  - SDK abstractions must describe intent, not allow bypass of host validation for filesystem, process, or network actions.

### Current State Findings

| Area                             | Current state                                                                                                                   | Evidence                                                                      | Why this is a problem                                                                                           |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Fallback user policy             | `editor-plugin-host::NullUserLibrary` duplicates terminal, hover, oil, statusline, diagnostics, gitfringe, and browser defaults | `crates/editor-plugin-host/src/lib.rs`                                        | Split ownership creates drift between built-in user library and host fallback behavior                          |
| Oil help and contextual UX       | User owns oil defaults and keybindings, but `editor-sdl` still owns contextual help keybinding descriptions                     | `user/oil.rs`, `crates/editor-sdl/src/shell/picker.rs`                        | Same feature described in two places; external plugins cannot reuse help rendering model                        |
| Git status interaction model     | Host owns `GIT_STATUS_COMMANDS` command table and contextual keybinding help                                                    | `crates/editor-sdl/src/shell/git.rs`, `crates/editor-sdl/src/shell/picker.rs` | First-party git feature uses privileged shell-only abstractions unavailable to plugin authors                   |
| Hook and feature contracts       | Many feature contracts are raw strings spread through `editor-sdl` and `user`                                                   | `crates/editor-sdl/src/shell/mod.rs`, `user/browser.rs`, `user/db.rs`         | String coupling makes extension fragile and obscures which hooks are public                                     |
| Browser and DB package contracts | User defines package metadata and buffer kinds, but host still contains paired internal logic and command routing               | `user/browser.rs`, `user/db.rs`, `crates/editor-sdl/src/shell/mod.rs`         | External users can trigger shells of features, but not compose all underlying behavior with shared abstractions |

### Migration Table

| Priority | Source crate/file                                             | Current hard-coded or split behavior                                                                      | Target owner/abstraction                                                                 | Notes                                                                               |
| -------- | ------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| P0       | `crates/editor-plugin-host/src/lib.rs`                        | `NullUserLibrary` behavior defaults for terminal, hover, oil, browser, statusline, diagnostics, gitfringe | `user` as canonical default policy; `NullUserLibrary` reduced to empty/minimal bootstrap | Highest drift risk; first place to remove duplicate policy                          |
| P0       | `crates/editor-sdl/src/shell/git.rs`                          | `GIT_STATUS_COMMANDS` host-only command table                                                             | `user/sdk::GitFeatureSpec` plus `user/git.rs` default instance                           | Lets first-party and third-party git UIs share same abstraction                     |
| P0       | `crates/editor-sdl/src/shell/picker.rs`                       | Contextual help tables for git/oil views                                                                  | `user/sdk::ContextHelpSpec` or per-buffer help metadata                                  | Removes duplicate command descriptions and keybinding docs                          |
| P0       | `crates/editor-sdl/src/shell/mod.rs`                          | Public-ish hook names, buffer kinds, feature IDs spread as raw constants                                  | `user/sdk` typed modules/builders for public feature contracts                           | Public contracts must stop being shell-local string literals                        |
| P1       | `user/oil.rs` plus `editor-sdl`                               | Oil actions split between user keybinding config and host action execution model                          | `user/sdk::OilFeatureSpec` or generic directory feature spec                             | Keep UI engine in host, move behavior contract fully public                         |
| P1       | `user/browser.rs` plus `editor-sdl`                           | Browser commands/buffers exposed, but navigation/focus contract still stringly and partial                | `user/sdk::BrowserFeatureSpec`                                                           | Maintain hidden SDL implementation while making behavior contract public            |
| P1       | `user/db.rs` plus `editor-sdl`                                | DB hooks and buffer kinds declared in user, execution lifecycle owned only by host                        | `user/sdk::DbFeatureSpec`                                                                | Needed so third-party packages can define richer DB workflows using same primitives |
| P1       | `user/terminal.rs` plus `editor-plugin-host`                  | Terminal defaults owned by user but fallback shell policy duplicated in host                              | `user/sdk::TerminalFeatureSpec` or keep current config plus remove fallback duplication  | Small move, high clarity                                                            |
| P2       | `user/autocomplete.rs`, `user/hover.rs`, `editor-plugin-host` | Provider concepts public, but host still seeds built-in provider defaults                                 | Public provider registries with explicit empty-state handling                            | Makes no-library bootstrap deterministic and less magical                           |
| P2       | `user/lib.rs` package export pattern                          | First-party features assembled as module-local functions without consistent shared builders               | SDK builder APIs used by both first-party and third-party packages                       | Improves symmetry and docs                                                          |

### Proposed Public Abstractions

- `FeatureSpec` pattern in `user/sdk` for first-party-capable surfaces:
  - `GitFeatureSpec`
  - `OilFeatureSpec`
  - `BrowserFeatureSpec`
  - `DbFeatureSpec`
  - `TerminalFeatureSpec`
- Typed public hook catalog:
  - Public hooks moved from shell-local constants into SDK modules when users are expected to emit or subscribe to them.
  - Internal-only SDL hooks remain private.
- Shared help and interaction metadata:
  - Per-buffer or per-feature contextual help entries.
  - Typed action ids instead of free-form command text where practical.
- Builder-style package composition:
  - Internal `user` modules should use the same SDK builders exposed to third-party plugin authors.

### Requirements

1. `user` must be canonical owner of all configurable defaults that affect first-party editor behavior outside SDL-only presentation concerns.
2. `editor-plugin-host` must not invent alternate defaults for behavior that already exists in `user`.
3. Every hook, action id, buffer kind, and command contract intended for user/plugin authors must be exported through `user/sdk` as a documented public abstraction.
4. First-party features implemented in host crates must consume those public abstractions rather than bespoke internal tables when the behavior is extensible.
5. SDK naming must distinguish public extensibility contracts from private shell implementation details.
6. Migration must preserve ability to compile built-in `user` library and load it through current application bootstrap.

## 5. Risks & Roadmap

- **Phased Rollout**:
  - **MVP**:
    - Remove duplicated policy from `NullUserLibrary`.
    - Publish typed public hook modules and feature-spec scaffolding in `user/sdk`.
    - Migrate one feature slice end to end: recommended order is git or oil.
  - **v1.1**:
    - Migrate browser, DB, terminal, and contextual help metadata.
    - Convert first-party `user/*` modules to SDK builders so examples reflect intended public usage.
  - **v2.0**:
    - Finish remaining host-owned feature contracts.
    - Document plugin-author workflow for composing full-feature plugins with same primitives used internally.

- **Technical Risks**:
  - Over-exposing internal shell details could freeze poor abstractions into the public SDK.
  - Large ABI and trait changes may cause broad compile churn across `user`, `volt`, and test helpers.
  - Feature-spec design could become too generic and recreate stringly coupling under a different name.
  - Partial migration could leave three sources of truth instead of two if fallback behavior is not removed early.

- **Mitigations**:
  - Keep SDL mechanics private and expose only product-level behavior contracts.
  - Migrate one feature family at a time with tests that prove first-party code uses public SDK only.
  - Add compile-time and test assertions that built-in packages round-trip through the same public abstractions available to plugin authors.
  - Treat `editor-plugin-host` fallback behavior as temporary bootstrap only, with explicit tests guarding minimal scope.
