# User-Owned Extension Surfaces Migration PRD

## 1. Executive Summary

- **Problem Statement**: Volt still has several user-facing behaviors whose entries, actions, or picker rows can only be changed by editing shell or source modules outside `user/`. These seams block first-party features from becoming reusable extension surfaces and force customization through host-side string dispatch.
- **Proposed Solution**: Move remaining user-defined entry shaping and action declaration into typed `user/sdk` contracts consumed from `user/*`, while keeping execution engines, live host services, and SDL rendering in host crates.
- **Success Criteria**:
  - New user-defined entries or actions require edits only in `user/` unless the feature needs a new host primitive.
  - No migrated feature uses shell-side string `match detail` dispatch for user-defined behavior.
  - Current command names, hook names, buffer kinds, and visible behavior stay compatible during migration.
  - Each migrated surface has proof that user-only customization changes runtime behavior.
  - Each migrated surface has shell action conversion coverage and compatibility coverage for existing command and hook names.
  - `cargo xtask ci`, `cargo run -p volt -- --shell-hidden`, and `graphify update .` remain required validation for implementation changes.

## 2. User Experience & Functionality

- **User Personas**:
  - Volt maintainer removing extension blockers without rewriting editor internals.
  - Power user editing `user/*.rs` to customize picker entries, action labels, commands, and feature defaults.
  - Plugin author who needs typed public contracts instead of private shell-only tables and hook details.

- **User Stories**:
  - As a Volt maintainer, I want picker rows and action specs for first-party features declared in `user` so feature policy has one owner.
  - As a power user, I want to add or change a Git, DB, ACP, Oil, Vim, or picker action without editing `editor-sdl`.
  - As a plugin author, I want typed action specs and contexts so I can compose with host engines without relying on undocumented strings.
  - As a maintainer, I want current command names, hook names, and buffer kinds preserved so existing packages and tests keep working.

- **Acceptance Criteria**:
  - Remaining user-owned picker entry shaping moves behind `user/sdk` contexts and specs where the entry is configurable.
  - Host code converts typed user action specs into existing engine calls without exposing direct `EditorRuntime` mutation.
  - String details remain only as compatibility input/output where required by existing commands or hooks, not as the primary customization model.
  - Live host-owned services, such as workspace search, Git execution, DB sessions, ACP clients, Vim editing mechanics, and SDL rendering, remain in host crates.
  - Each migration lands focused tests for user-only customization, shell action conversion, and existing command/hook compatibility.

- **Non-Goals**:
  - No direct user access to mutate `EditorRuntime`.
  - No moving Git, DB, or ACP execution engines into `user/`.
  - No broad rewrite of SDL rendering, input dispatch, pane management, text editing, or core editor mechanics.
  - No attempt to migrate every hard-coded shell string; scope is extension blockers only.
  - No breaking rename of existing command names, hook names, or buffer kinds.

## 3. AI System Requirements

- **Tool Requirements**: Not applicable. This is a local Rust architecture and product-surface migration.
- **Evaluation Strategy**: Use repository analysis, targeted Rust tests, hidden SDL smoke coverage, and graph maintenance. Model-quality evaluation is not part of this PRD.

## 4. Technical Specifications

- **Architecture Overview**:
  - `user/sdk` defines typed contexts and action specs for user-owned extension surfaces.
  - `user/*` defines first-party entries, actions, labels, keybindings, and package metadata through those public SDK types.
  - `editor-plugin-host` adapts exported package metadata and feature specs into runtime registration without owning feature policy.
  - `editor-sdl` keeps rendering, input dispatch, and execution engines, but converts typed user specs into host actions.
  - Existing command, hook, and buffer identifiers remain stable compatibility contracts.

- **Integration Points**:
  - `user/sdk`:
    - Add small typed specs per feature instead of one generic catch-all abstraction.
    - Use context structs for host-provided data that `user` may shape into entries.
    - Keep specs declarative: labels, ids, command ids, hook ids, action variants, and metadata.
  - `user`:
    - Own default first-party definitions in focused modules such as `user/git.rs`, `user/oil.rs`, `user/acp.rs`, `user/db.rs`, and Vim/picker-related modules.
    - Provide customization proof tests by changing only user-owned definitions in test libraries or fixtures.
  - `editor-sdl`:
    - Replace shell-side action table and `match detail` customization with conversion from typed specs.
    - Preserve engine ownership and validate filesystem, process, DB, Git, and ACP effects in host code.
  - `editor-plugin-host`:
    - Register new typed specs with existing command, hook, and keymap registries.
    - Keep fallback behavior minimal and compatibility-focused.

### Module Plans

| Area | Current blocker | Target contract | Migration plan |
| --- | --- | --- | --- |
| Picker parity | Remaining workspace dashboard item shaping still lives in shell paths. | User-owned picker provider path for dashboard items; host-owned live workspace search stays separate. | Move configurable dashboard row shaping to `user` provider specs. Keep fuzzy matching, live filesystem/workspace search, and picker rendering in host. |
| Vim | `editor.vim.edit` behavior depends on detail strings dispatched in shell code. | `VimActionSpec` and `VimActionContext`. | Introduce typed Vim action registry in `user/sdk`, migrate user-defined Vim edit bindings to typed specs, and make shell convert specs into current edit engine calls. |
| Git | Git status command bindings and Git pickers are built from shell-owned tables/builders. | `GitActionSpec`. | Move Git status action declarations, picker labels, and command binding metadata to `user/git.rs`; keep repository status, staging, diff, and process execution in host. |
| Oil | Oil commands/keybindings duplicate string action mapping between user config and shell execution. | `OilKeyAction` emission and typed Oil action specs. | Make commands and keybindings emit typed Oil actions, remove duplicated shell string mapping, and keep directory buffer mechanics in host. |
| ACP | Mode, model, session, and slash pickers are shaped by ACP shell code. | `AcpPickerContext` and `AcpActionSpec`. | Let `user/acp.rs` shape ACP picker entries from host-provided contexts while host keeps ACP client/session execution. |
| DB | DB browser rows and actions are shaped in host-owned browser logic. | `DbBrowserContext`, `DbBrowserItemSpec`, and `DbActionSpec`. | Move DB browser row/action shaping to `user/db.rs`; keep connection handling, query execution, schema inspection, and credentials in host. |

### Requirements

1. User-owned specs must be typed and feature-specific enough to avoid recreating stringly coupling under a new name.
2. Host action conversion must be explicit, testable, and narrow: each action variant maps to an existing host primitive or a newly added primitive.
3. Compatibility strings for existing command details or hook payloads must be converted at the boundary and documented as legacy-compatible inputs.
4. User contexts must contain only data needed for shaping entries and action specs, not mutable runtime handles.
5. Live searches and execution engines remain host-owned because they depend on filesystem, process, network, credential, or session state.
6. Every migrated feature must keep existing package exports and startup behavior stable.

### Acceptance Checklist

For each migrated area:

- User-only customization proof test: changing a `user/` definition changes the exported entry/action without shell/source edits.
- Shell action conversion test: typed action spec maps to the intended host primitive and rejects unsupported data.
- Compatibility test: existing command names, hook names, buffer kinds, and legacy detail payloads continue to work.
- Runtime validation: `cargo xtask ci` and `cargo run -p volt -- --shell-hidden` pass.
- Graph validation: `graphify update .` runs after code changes so `graphify-out/` stays current.

## 5. Risks & Roadmap

- **Phased Rollout**:
  - **MVP**:
    - Land one small typed action surface end to end, preferably Oil or picker parity.
    - Add compatibility tests for existing command and hook names.
    - Prove customization by editing only `user/` test definitions.
  - **v1.1**:
    - Migrate Vim and Git action specs.
    - Remove shell-side string `match detail` paths for migrated user-defined behavior.
    - Keep legacy detail parsing only at compatibility boundaries.
  - **v1.2**:
    - Migrate ACP and DB picker shaping.
    - Add context structs and item specs that expose host data without runtime mutation access.
  - **v2.0**:
    - Audit remaining extension blockers and decide whether each needs a typed SDK contract or should stay host-private.

- **Technical Risks**:
  - Specs may expose too much shell detail and freeze unstable internals.
  - Specs may be too generic and preserve string coupling under typed wrappers.
  - Partial migration can leave duplicate action declarations in both `user` and `editor-sdl`.
  - Compatibility conversion can hide behavior drift if tests only cover happy paths.

- **Mitigations**:
  - Keep each spec tied to a concrete feature and action family.
  - Move one surface at a time and delete replaced shell-owned builders in the same change.
  - Require one user-only customization test, one shell conversion test, and one compatibility test per migrated feature.
  - Keep host engines responsible for validation, credentials, filesystem/process effects, session state, and rendering.
  - Run `cargo xtask ci`, hidden shell smoke, and `graphify update .` for implementation changes before marking migrations complete.
