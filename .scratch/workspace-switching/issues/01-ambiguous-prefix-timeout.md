# 01 — Ambiguous Prefix Timeout

**What to build:** When a registered chord is both an exact binding and a proper prefix of a longer binding, the short chord waits instead of firing immediately. After `ui.keymap.ambiguous_prefix_timeout_ms` (default 250), the short command runs. Completing the longer chord within the window runs the long command and cancels the short. Incompatible input clears the pending short without firing it. This keeps `<leader> w` as `buffer.save` while allowing longer `<leader> w …` Workspace chords.

**Blocked by:** None — can start immediately.

**Status:** resolved

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [x] Exact chords that are also prefixes of longer registered chords wait the configured timeout before firing
- [x] Completing a longer chord within the window fires the long binding and cancels the pending short
- [x] Incompatible input during the wait clears the pending short without firing it
- [x] Timeout is configurable via `ui.keymap.ambiguous_prefix_timeout_ms` with default 250
- [x] Keymap/sequence seam tests cover timeout fire, longer-chord win, cancel, and configurable timeout without SDL
- [x] Spoken config/UI language matches the PRD (ambiguous prefix timeout, not ad hoc names)

## Answer

Pure keymap/sequence seam lives in `editor-core::key_sequence` (`push_key_sequence` / `tick_key_sequence`) with injectable ms clock. Shell wires wait/execute/cancel plus per-frame ambiguous-prefix fire. Config: `ui.keymap.ambiguous_prefix_timeout_ms` (default 250) via user YAML + `KeymapConfig` ABI (replaced reserved last-prefix slot).
