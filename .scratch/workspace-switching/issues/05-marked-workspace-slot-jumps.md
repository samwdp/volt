# 05 — Marked Workspace Slot Jumps

**What to build:** The first four Mark List entries map to `Ctrl+n`, `Ctrl+e`, `Ctrl+o`, and `Ctrl+i` in the Workspace Minor Mode. A filled slot switches to the matching open Workspace or opens then switches; an empty slot is a silent no-op; a missing on-disk path notifies and leaves the Mark List unchanged. Overlay Minor Modes still win for those chords while active. The unused Vim `Ctrl+e` scroll-line-down binding is dropped so slot 2 can own `Ctrl+e`.

**Blocked by:** 02 — Minor Mode Keybinding Precedence; 04 — Mark List Management

**Status:** ready-for-agent

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [ ] First four Mark List entries bind to `Ctrl+n`, `Ctrl+e`, `Ctrl+o`, `Ctrl+i` in Workspace Minor Mode
- [ ] Jump to an open Marked Workspace switches to it
- [ ] Jump to a closed Marked Workspace opens/creates then switches
- [ ] Empty slot → silent no-op; missing path → notify, Mark List unchanged
- [ ] Popup/autocomplete/hover still override those chords while active
- [ ] Vim `Ctrl+e` scroll-line-down no longer claims Workspace `Ctrl+e`
- [ ] Domain seam tests cover slot empty/filled/missing and open-or-switch intents without SDL
- [ ] Spoken language matches `CONTEXT.md` (Marked Workspace, Mark List, Minor Mode)
