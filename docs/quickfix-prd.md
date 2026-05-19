# Quickfix List PRD

## 1. Executive Summary

- **Problem Statement**: Volt pickers are optimized for one-shot selection, but they do not support durable result-set workflows. Users need a reusable list of file/location results that can outlive a picker session and support editor-native navigation across many matches.
- **Proposed Solution**: Add a Neovim-style quickfix list to Volt that can be populated from supported pickers with `Ctrl+q`, rendered as a popup buffer, and navigated via Enter and next/previous commands. Picker remains single-select; quickfix becomes the durable multi-result workflow surface.
- **Success Criteria**:
  - Pressing `Ctrl+q` in a supported picker exports all visible/exportable results into a quickfix popup within 150ms for up to 1,000 entries.
  - Pressing `Enter` on a quickfix row opens the exact file and line/column target with 100% correctness in automated tests.
  - After `Enter`, focus moves to the workspace buffer while quickfix list remains available in the popup stack.
  - `quickfix.next` and `quickfix.previous` navigate the active quickfix list with wraparound and no state loss.
  - MVP supports `workspace.search`, `LSP Locations`, and `LSP Diagnostics`.

## 2. User Experience & Functionality

- **User Personas**:
  - Vim-style user expecting quickfix basics for search, diagnostics, and references.
  - Developer triaging many workspace search hits across multiple files.
  - Developer using LSP navigation and wanting to revisit result sets without rerunning commands.

- **User Stories**:
  - As a developer, I want to press `Ctrl+q` in a picker so that I can convert visible results into a reusable quickfix list.
  - As a developer, I want the quickfix list to open in a popup buffer so that I can keep a durable result set available without replacing my workspace panes.
  - As a developer, I want `Enter` on a quickfix entry so that I can jump to the file location and continue editing immediately.
  - As a developer, I want `quickfix.next` and `quickfix.previous` so that I can walk the list from the workspace without reopening the popup each time.
  - As a developer, I want quickfix-only marking so that I can later operate on selected quickfix rows without complicating picker UX.

- **Acceptance Criteria**:
  - Pressing `Ctrl+q` in a supported picker exports all visible/exportable matches into a quickfix popup.
  - Export works from picker state as shown on screen, not from hidden/unmatched items.
  - Quickfix rows display path, line, column, and summary text for each result.
  - Pressing `Enter` on a quickfix row opens the target and moves focus to the workspace buffer.
  - Quickfix popup remains available in the popup stack after opening a target.
  - `quickfix.next` and `quickfix.previous` operate on the active quickfix list even when workspace buffer has focus.
  - Quickfix supports mark/unmark current row in v1.1, and marks persist while the quickfix list remains alive.
  - Picker does not support multi-select, marks, or subset export.
  - Unsupported pickers do not crash; `Ctrl+q` is either inert or emits a clear status message.

- **Non-Goals**:
  - Full Neovim parity in MVP, including `:cdo`, `:cfdo`, editable quickfix buffers, errorformat parsing, or multiple named quickfix stacks.
  - Adding picker-side multi-select.
  - Persisting quickfix lists across app restarts in MVP.
  - Bulk edit/apply workflows over marked quickfix rows in MVP.
  - Implementing location-list/window-local stacks in MVP.

## 3. AI System Requirements (If Applicable)

- **Tool Requirements**:
  - No AI model is required.
  - Required runtime/editor components:
    - picker export metadata path
    - popup buffer rendering
    - quickfix state service
    - file/location open handlers
    - keymap and command registration

- **Evaluation Strategy**:
  - Add SDL shell tests for picker export, quickfix buffer rendering, row activation, and navigation commands.
  - Add integration tests verifying exact file/line/column jumps from:
    - `workspace.search`
    - `LSP Locations`
    - `LSP Diagnostics`
  - Add state tests verifying focus handoff, popup persistence, and quickfix selection retention.
  - Add negative tests ensuring unsupported picker export does not panic or corrupt popup state.

## 4. Technical Specifications

- **Architecture Overview**:
  - Current picker flow is single-action based:
    - picker providers build `PickerOverlay` and `PickerAction` in [picker.rs](/P:/volt/crates/editor-sdl/src/shell/picker.rs:28) and [mod.rs](/P:/volt/crates/editor-sdl/src/shell/mod.rs:6818)
    - Enter dispatches selected action in [mod.rs](/P:/volt/crates/editor-sdl/src/shell/mod.rs:15585)
  - Existing location-bearing picker entries already exist for:
    - workspace search
    - LSP locations
    - LSP diagnostics
    - file-open-at-location actions in [workspace_search.rs](/P:/volt/crates/editor-sdl/src/shell/workspace_search.rs:313)
  - Popup buffers already support append/reuse semantics in [model.rs](/P:/volt/crates/editor-core/src/model.rs:762)
  - Proposed design:
    - Add a first-class `BufferKind::Quickfix` in [model.rs](/P:/volt/crates/editor-core/src/model.rs:36)
    - Add a `QuickfixEntry` struct in SDL shell with:
      - stable id
      - path or URI-backed target
      - `TextPoint`
      - label/summary
      - detail text
      - source kind metadata
    - Add `QuickfixState` runtime service with:
      - active list entries
      - active row index
      - marked entry ids
      - optional source metadata
      - owning popup buffer id
    - Extend `PickerOverlay` with optional quickfix-export metadata per item id, but no mark state
    - Add a picker `Ctrl+q` command path that:
      - reads current visible matches from picker session
      - resolves exportable quickfix entries
      - stores them in `QuickfixState`
      - opens or refreshes quickfix popup buffer
    - Add a quickfix popup buffer renderer that materializes rows into text lines
    - Add quickfix activation handler:
      - Enter on quickfix row opens target
      - popup focus is dropped
      - workspace buffer becomes focused
    - Add workspace-available quickfix navigation commands:
      - `quickfix.next`
      - `quickfix.previous`
    - Add quickfix-only marking commands in v1.1:
      - `quickfix.toggle-mark`
      - `quickfix.clear-marks`
      - `quickfix.mark-all`

- **Integration Points**:
  - `editor-core`
    - add `BufferKind::Quickfix`
    - reuse popup open/cycle APIs
  - `editor-sdl`
    - extend `PickerOverlay`
    - add `QuickfixState`
    - add quickfix popup buffer creation/render/update path
    - add Enter and next/prev handlers
    - add popup/workspace focus transitions
  - `editor-picker`
    - no required structural changes for MVP
    - keep generic fuzzy picker state unchanged unless later row metadata pressure proves otherwise
  - `workspace_search.rs`
    - attach quickfix-exportable metadata for:
      - workspace search results
      - LSP locations
      - LSP diagnostics
  - `user`
    - add keybindings and command aliases in `user/vim.rs` and related package declarations
    - expose user-facing commands such as `quickfix.open`, `quickfix.next`, `quickfix.previous`

- **Security & Privacy**:
  - No new network or credential handling.
  - Quickfix entries should store only editor-visible metadata needed for navigation.
  - If future persistence is added, paths and labels may be persisted, but file contents must not be serialized into quickfix state snapshots.

## 5. Risks & Roadmap

- **Phased Rollout**:
  - **MVP**
    - add `BufferKind::Quickfix`
    - add `QuickfixEntry` and `QuickfixState`
    - add picker export with `Ctrl+q`
    - support all visible/exportable picker matches
    - open quickfix popup buffer
    - Enter opens target and moves focus to workspace
    - add `quickfix.next` and `quickfix.previous`
    - support providers:
      - `workspace.search`
      - `LSP Locations`
      - `LSP Diagnostics`
  - **v1.1**
    - add quickfix-only marking
    - add mark display in quickfix rows
    - add `quickfix.mark-all`, `quickfix.clear-marks`
    - preserve row selection more aggressively when quickfix list refreshes
    - add more exportable pickers where row semantics map cleanly to file targets
  - **v2.0**
    - add quickfix history stack
    - add location-list/window-local variant
    - add batch actions for marked rows
    - add richer quickfix filtering and sorting commands

- **Technical Risks**:
  - Picker export metadata must coexist with today’s single-action picker model without turning `PickerOverlay` into an overgrown state bag.
  - Popup stack is shared with browser, git, compile, and diagnostics. Quickfix behavior must not regress popup cycling or focus restoration.
  - Some picker providers do not represent file locations. Export contract must be explicit and opt-in.
  - `quickfix.next` and `quickfix.previous` need a stable source of truth when popup is not focused.
  - New `BufferKind::Quickfix` will touch buffer summaries, help text, render behavior, and tests across multiple modules.

## Open Design Decisions

- Whether quickfix rows should support URI-only targets in MVP, or only file-backed targets.
- Whether `quickfix.next` from workspace should silently open the target or also keep popup selection visually in sync in the same frame.
- Whether `quickfix.open` should reopen the last list when popup was closed, or only work when a current quickfix list still exists in memory.

## Parallel Implementation Plan

- **Workstream A: Core Model + Buffer Taxonomy**
  - Add `BufferKind::Quickfix`
  - Update buffer summaries/help text/tests
  - Minimal dependency surface for other streams
  - Owner files:
    - [model.rs](/P:/volt/crates/editor-core/src/model.rs:36)
    - [mod.rs](/P:/volt/crates/editor-sdl/src/shell/mod.rs:28499)

- **Workstream B: Quickfix State + Popup Buffer**
  - Add `QuickfixEntry`
  - Add `QuickfixState`
  - Add popup buffer creation/update/render plumbing
  - Add Enter-to-open behavior and focus handoff
  - Depends on A for final buffer kind, but can start with temporary `Diagnostics` if needed
  - Owner files:
    - [mod.rs](/P:/volt/crates/editor-sdl/src/shell/mod.rs:6818)
    - [mod.rs](/P:/volt/crates/editor-sdl/src/shell/mod.rs:16532)

- **Workstream C: Picker Export Contract**
  - Extend `PickerOverlay` with quickfix-export metadata
  - Add `Ctrl+q` picker command path
  - Export all visible/exportable matches
  - No picker-side marking
  - Can proceed in parallel with B once `QuickfixEntry` shape is agreed
  - Owner files:
    - [mod.rs](/P:/volt/crates/editor-sdl/src/shell/mod.rs:6934)
    - [picker.rs](/P:/volt/crates/editor-sdl/src/shell/picker.rs:28)

- **Workstream D: Provider Adapters**
  - Mark exportable picker rows for:
    - workspace search
    - LSP locations
    - LSP diagnostics
  - Convert existing location actions into reusable quickfix metadata
  - Depends on C’s export contract, not on B’s rendering details
  - Owner files:
    - [workspace_search.rs](/P:/volt/crates/editor-sdl/src/shell/workspace_search.rs:313)

- **Workstream E: Commands + Keymaps**
  - Add:
    - `quickfix.open`
    - `quickfix.next`
    - `quickfix.previous`
    - later `quickfix.toggle-mark`
  - Add default keybindings in workspace/popup scopes
  - Depends on B for command targets
  - Owner files:
    - [user/vim.rs](/P:/volt/user/vim.rs:869)
    - command registration sites in SDL shell

- **Workstream F: Test Matrix**
  - Export from picker
  - Popup render/update
  - Enter opens target and focuses workspace
  - next/prev wraparound
  - unsupported picker no-op
  - quickfix state survives popup focus loss
  - This can start once interfaces from B/C stabilize
  - Owner files:
    - `crates/editor-sdl/src/shell/tests.rs`
    - provider-specific tests in [workspace_search.rs](/P:/volt/crates/editor-sdl/src/shell/workspace_search.rs:762)

## Recommended Implementation Order

1. Lock `QuickfixEntry` shape and `BufferKind::Quickfix`
2. Build `QuickfixState` and popup buffer rendering
3. Add picker export path via `Ctrl+q`
4. Adapt `workspace.search`, `LSP Locations`, and `LSP Diagnostics`
5. Add next/prev commands and keymaps
6. Add v1.1 quickfix-only marking
