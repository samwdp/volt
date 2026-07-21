# 05 — Marked Workspace Slot Jumps

**What to build:** The first four Mark List entries map to `Ctrl+n`, `Ctrl+e`, `Ctrl+o`, and `Ctrl+i` in the Workspace Minor Mode. A filled slot switches to the matching open Workspace or opens then switches; an empty slot is a silent no-op; a missing on-disk path notifies and leaves the Mark List unchanged. Overlay Minor Modes still win for those chords while active. The unused Vim `Ctrl+e` scroll-line-down binding is dropped so slot 2 can own `Ctrl+e`.

**Blocked by:** 02 — Minor Mode Keybinding Precedence; 04 — Mark List Management

**Status:** resolved

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [x] First four Mark List entries bind to `Ctrl+n`, `Ctrl+e`, `Ctrl+o`, `Ctrl+i` in Workspace Minor Mode
- [x] Jump to an open Marked Workspace switches to it
- [x] Jump to a closed Marked Workspace opens/creates then switches
- [x] Empty slot → silent no-op; missing path → notify, Mark List unchanged
- [x] Popup/autocomplete/hover still override those chords while active
- [x] Vim `Ctrl+e` scroll-line-down no longer claims Workspace `Ctrl+e`
- [x] Domain seam tests cover slot empty/filled/missing and open-or-switch intents without SDL
- [x] Spoken language matches `CONTEXT.md` (Marked Workspace, Mark List, Minor Mode)

## Answer

Domain `marked_workspace_jump` + shell `jump_to_marked_workspace_slot` (Switch / OpenThenSwitch / NotifyMissing). Slot chords in Workspace Minor Mode; overlays win while active. Vim `Ctrl+e` scroll-line-down removed. Canonical path identity used for open-root match.
