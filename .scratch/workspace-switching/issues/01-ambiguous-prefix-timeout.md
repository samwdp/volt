# 01 — Ambiguous Prefix Timeout

**What to build:** When a registered chord is both an exact binding and a proper prefix of a longer binding, the short chord waits instead of firing immediately. After `ui.keymap.ambiguous_prefix_timeout_ms` (default 250), the short command runs. Completing the longer chord within the window runs the long command and cancels the short. Incompatible input clears the pending short without firing it. This keeps `<leader> w` as `buffer.save` while allowing longer `<leader> w …` Workspace chords.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [ ] Exact chords that are also prefixes of longer registered chords wait the configured timeout before firing
- [ ] Completing a longer chord within the window fires the long binding and cancels the pending short
- [ ] Incompatible input during the wait clears the pending short without firing it
- [ ] Timeout is configurable via `ui.keymap.ambiguous_prefix_timeout_ms` with default 250
- [ ] Keymap/sequence seam tests cover timeout fire, longer-chord win, cancel, and configurable timeout without SDL
- [ ] Spoken config/UI language matches the PRD (ambiguous prefix timeout, not ad hoc names)
