# 02 — Minor Mode Keybinding Precedence

**What to build:** Global keybindings become the fallback. Active Minor Modes override Global for the same chord. Workspace editing, Popup, autocomplete, and hover are Minor Modes while active. Autocomplete and hover are never co-active, so their mutual precedence is undefined. This lets Workspace claim chords such as `Ctrl+n` without stealing overlay behavior when those overlays are open.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [ ] Active Minor Mode bindings override Global bindings for the same chord
- [ ] Global bindings remain the fallback when no active Minor Mode claims the chord
- [ ] Workspace is treated as a Minor Mode for precedence
- [ ] Popup, autocomplete, and hover override Workspace/Global while active
- [ ] Overlay `Ctrl+n` (picker/autocomplete/hover) still works while those Minor Modes are active
- [ ] Spoken language matches `CONTEXT.md` (Minor Mode, Global fallback)
