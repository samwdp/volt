# Workspace Switching and Marks

Status: ready-for-agent

## Problem Statement

Jumping between open Workspaces in Volt today requires the workspace switch picker. There is no fast cycle through open project Workspaces, and no persistent shortlist of favorite project roots with dedicated keys. Users who keep several projects open need next/previous navigation and a small set of always-reachable Marked Workspaces that survive editor restarts and can be reordered by editing a plain file.

## Solution

Extend the workspace package with `workspace.next` / `workspace.previous` to cycle project-backed open Workspaces in open order (wrapping; skipping the Default Workspace). Add a Mark List of project root paths stored in the application state directory as `marked-workspaces.txt`. Users Mark/Unmark the active Workspace, open the Mark List as a normal buffer to reorder by editing and saving, and jump to the first four entries with `Ctrl+n` / `Ctrl+e` / `Ctrl+o` / `Ctrl+i`. Keep `<leader> w` as save by introducing a configurable ambiguous-prefix keymap timeout so short chords that are also prefixes of longer chords wait briefly before firing. Treat Workspace (and overlays) as Minor Modes that override Global keybindings.

Domain language for Workspace / Mark List / Minor Mode lives in root `CONTEXT.md`.

## User Stories

1. As a developer, I want `workspace.next`, so that I can move to the next open project Workspace without opening a picker.
2. As a developer, I want `workspace.previous`, so that I can move to the previous open project Workspace without opening a picker.
3. As a developer, I want next/previous to use open order, so that cycling feels stable and predictable.
4. As a developer, I want next/previous to skip the Default Workspace, so that scratch is not in my project rotation.
5. As a developer, I want next/previous to wrap at the ends, so that I can keep cycling in one direction.
6. As a developer, I want next/previous to silently do nothing when fewer than two project Workspaces are open, so that I am not interrupted with errors.
7. As a developer, I want `<leader> w n` bound to next and `<leader> w p` to previous, so that Workspace navigation lives under a `w` prefix.
8. As a developer, I want `<leader> w` alone to still save the buffer, so that my existing save muscle memory remains.
9. As a developer, I want ambiguous short chords that are prefixes of longer chords to wait a short timeout before firing, so that `w` and `w n` can coexist.
10. As a developer, I want that timeout configurable as `ui.keymap.ambiguous_prefix_timeout_ms` with default 250, so that I can tune for my typing speed.
11. As a developer, I want completing a longer chord within the timeout to run the longer binding (not the short one), so that `w n` never accidentally saves.
12. As a developer, I want a wrong/canceling key during the wait to cancel the pending short command without saving, so that I do not get surprise saves.
13. As a developer, I want `workspace.mark` (`<leader> w +`) to append the active project root to the Mark List if absent, so that I can bookmark quickly.
14. As a developer, I want `workspace.mark` to no-op when the root is already listed, so that duplicates are not created by accident.
15. As a developer, I want `workspace.unmark` (`<leader> w -`) to remove the active project root from the Mark List, so that I have an explicit remove path.
16. As a developer, I want mark/unmark on the Default Workspace to notify that there is no project root, so that failure is explained.
17. As a developer, I want Mark List identity to be the project root path, so that bookmarks survive across sessions and renames of Workspace display names.
18. As a developer, I want the Mark List stored under the application Volt state directory as `marked-workspaces.txt`, so that marks are app-wide, not per-repo.
19. As a developer, I want one path per line in that file, so that I can reorder with ordinary text editing.
20. As a developer, I want blank lines ignored/stripped on save and no comment syntax, so that the format stays strict and simple.
21. As a developer, I want `workspace.marks` (`<leader> w m`) to open the Mark List file in a buffer, so that I can manage membership and order by hand.
22. As a developer, I want saving that buffer to update the live Mark List used by jumps, so that reorder takes effect immediately after save.
23. As a developer, I want to keep more than four paths in the file, so that I can park extras without dedicated keys.
24. As a developer, I want only the first four non-empty lines to get dedicated bindings (`Ctrl+n`, `Ctrl+e`, `Ctrl+o`, `Ctrl+i`), so that hotkeys stay a fixed small set.
25. As a developer, I want an empty slot key to silently no-op, so that unused slots are quiet.
26. As a developer, I want a jump to a Marked Workspace that is already open to switch to it, so that focus moves instantly.
27. As a developer, I want a jump to a Marked Workspace that is closed to open (or create) that project Workspace and switch to it, so that marks work across sessions.
28. As a developer, I want a jump to a path that does not exist on disk to notify me and leave the Mark List unchanged, so that offline or removed disks do not silently delete bookmarks.
29. As a developer, I want Mark jump bindings in the Workspace Minor Mode, so that they apply while editing and yield to overlay Minor Modes.
30. As a developer, I want Global keybindings to be a fallback under active Minor Modes, so that overlays and Workspace can override Global for the same chord.
31. As a developer, I want Popup, autocomplete, and hover treated as Minor Modes that override Workspace while active, so that `Ctrl+n` in a picker still means next item.
32. As a developer, I want autocomplete and hover never considered co-active, so that precedence between those two is undefined and unused.
33. As a developer, I want the unused Vim `Ctrl+e` scroll-line-down binding removed or disregarded so Mark slot 2 can own `Ctrl+e` in Workspace mode.
34. As a developer, I want all of these commands in the existing workspace package, so that Marks are not a separate plugin.
35. As a developer, I want new marks appended at the end of the list, so that existing top-four hotkeys are not reshuffled unless I reorder the file.
36. As an agent / plugin author, I want a pure Workspace navigation/marks domain seam, so that cycle/mark/parse/jump intents are testable without SDL.
37. As an agent / plugin author, I want ambiguous-prefix timeout behavior testable at the keymap/sequence seam, so that short vs long chord resolution is proven without the full shell demo.
38. As a developer, I want notifications only when they explain a failed intentional action (no-root mark, missing mark path), so that successful quiet navigation stays quiet.
39. As a developer, I want `<leader> W` (workspace.save all buffers) to remain unchanged, so that all-buffer save is distinct from buffer save and Workspace cycling.
40. As a developer, I want existing `workspace.switch` / `workspace.new` / `workspace.delete` pickers to keep working, so that Marks and next/previous complement rather than replace discovery.

## Implementation Decisions

- **Package:** extend the existing auto-loaded `workspace` package with commands, hooks, and keybindings for next/previous, mark/unmark, open Mark List, and marked-slot jumps. Do not create a separate marks package.
- **Primary domain seam:** a Workspace navigation/marks domain module (pure inputs/outputs): ordered open project roots + active root → next/previous target or none; Mark List text ↔ ordered paths; mark/unmark of a root; slot 1–4 resolution (empty / path); open-or-switch intent for a root. No SDL types.
- **Shell adapter:** subscribe to hooks / run commands that call the domain seam, then use existing create-workspace-by-root and switch-workspace host actions. Open Mark List by opening the real state file as a normal buffer; on save, re-parse into the live Mark List.
- **Cycle rules:** open order among Workspaces that have a project root; skip Default Workspace; wrap; fewer than two project Workspaces → silent no-op.
- **Mark List store:** application Volt state directory (same family as existing theme/error state), file name `marked-workspaces.txt`. One absolute/canonical project root path per non-empty line. Persist app-wide.
- **Mark semantics:** mark appends if absent, else no-op; unmark removes current root; no root → notify; new marks append at end; first four lines map to `Ctrl+n` / `Ctrl+e` / `Ctrl+o` / `Ctrl+i`.
- **Jump semantics:** if open (match by root path) → switch; else create/open then switch; missing on disk → notify, do not mutate Mark List; empty slot → silent no-op.
- **Leader chords:** `<leader> w n` next; `w p` previous; `w +` mark; `w -` unmark; `w m` open Mark List; lone `<leader> w` remains `buffer.save` via ambiguous-prefix timeout.
- **Ambiguous-prefix keymap (second seam):** when a registered chord is an exact binding and also a proper prefix of a longer registered chord, do not fire immediately; wait `ui.keymap.ambiguous_prefix_timeout_ms` (default 250). On timeout, fire the short binding. On completing a longer binding within the window, fire the long one and cancel the short. On incompatible input, clear pending short without firing it. Applies globally to all such ambiguities, not only `w`.
- **Config:** add under UI config section `keymap.ambiguous_prefix_timeout_ms` (user YAML), default 250.
- **Keymap precedence:** Global is fallback. Active Minor Modes override Global. Workspace is a Minor Mode. Popup, autocomplete, and hover are Minor Modes while active. Autocomplete and hover are never co-active. Mark slot bindings register in Workspace Minor Mode.
- **Conflict:** remove or stop registering Vim `Ctrl+e` → scroll-line-down so Mark slot 2 can bind `Ctrl+e` in Workspace mode. Do not rebind popup/autocomplete/hover `Ctrl+n`; those Minor Modes continue to override while open.
- **Commands (names):** `workspace.next`, `workspace.previous`, `workspace.mark`, `workspace.unmark`, `workspace.marks`, plus four jump commands (e.g. `workspace.marked-1` … `workspace.marked-4` or equivalent stable ids).
- **Glossary:** use `CONTEXT.md` terms (Workspace, Default Workspace, Marked Workspace, Mark List, Minor Mode). UI strings must not invent parallel names (favorite, bookmark tab, pin) unless added to the glossary.

## Testing Decisions

- **Good tests** assert external behavior of the two seams: next/previous targets, Mark List parse/serialize/mark/unmark, slot resolution, open-or-switch intents, and ambiguous-prefix short vs long resolution under timeout. Do not assert private helpers, SDL frames, or draw calls.
- **Primary tested module:** Workspace navigation/marks domain seam (temp files or in-memory path lists).
- **Secondary tested module:** keymap/sequence ambiguous-prefix behavior (registry + fake clock or injectable timeout), covering: short alone fires after timeout; short+continuation fires long; timeout cancelled by incompatible key; configurable timeout value.
- **Prior art:** Issues PRD domain-seam style; existing `editor-sdl` workspace open/switch/delete helper tests for adapter smoke only when domain cannot express create-then-switch wiring; `editor-core` keymap registry tests for chord registration/prefix detection.
- **Suggested domain cases:** wrap next/prev; skip Default Workspace; &lt;2 project Workspaces → none; mark append / mark duplicate no-op; unmark; parse blanks stripped; slot empty vs filled; missing path intent signals notify without list mutation; open-or-switch when root already open vs closed.
- **CI:** covered by `cargo xtask ci`.

## Out of Scope

- MRU / name-sorted / custom tab-bar order for next/previous.
- Including the Default Workspace in the cycle.
- Per-project Mark Lists under a repo `.volt/` directory.
- Comment lines or rich formats (TOML/YAML) in the Mark List file.
- Auto-removing missing paths from the Mark List.
- Toggle-on-mark (mark removes if present).
- More than four dedicated Mark hotkeys.
- Binding Mark jumps at Global scope instead of Workspace Minor Mode.
- Rebinding popup/autocomplete/hover chords to free `Ctrl+n`.
- Separate marks plugin package.
- Session restore of the full open Workspace set beyond Mark List jumps (opening unmarked Workspaces from last session).
- Configurable Mark hotkey chords in v1 (fixed `Ctrl+n/e/o/i`).
- Changing `<leader> W` workspace.save behavior.

## Further Notes

- Grilling locked shared understanding; glossary updates land in root `CONTEXT.md` with this PRD.
- Recommended follow-up ADR (not yet filed): keymap precedence as Minor Modes over Global, plus ambiguous-prefix timeout — hard to reverse, surprising vs “exact chord fires immediately,” and a real trade-off against moving `buffer.save` off `<leader> w`.
- Existing shell already tracks a single `previous_workspace` for delete fallback; next/previous cycle is a separate ordered walk of project-backed open Workspaces, not that one-slot MRU.
- Existing sequence timeout (~1200ms) currently drops expired pending chords without firing a short binding; ambiguous-prefix behavior is new and must not be confused with that drop-only timeout.
- Hook/command registration without shell subscription will no-op (existing Volt lesson): wire create/switch and Mark List reload on save.
