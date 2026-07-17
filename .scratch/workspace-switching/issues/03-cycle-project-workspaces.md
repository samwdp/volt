# 03 — Cycle Project Workspaces

**What to build:** A developer can run `workspace.next` and `workspace.previous` (bound to `<leader> w n` and `<leader> w p`) to move among open Project Workspaces in open order, skipping the Default Workspace, wrapping at the ends, and silently no-oping when fewer than two Project Workspaces are open. Lone `<leader> w` still saves via the ambiguous-prefix timeout from ticket 01.

**Blocked by:** 01 — Ambiguous Prefix Timeout

**Status:** resolved

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [x] `workspace.next` / `workspace.previous` switch among open Project Workspaces in open order
- [x] Default Workspace is skipped in the cycle
- [x] Cycle wraps at both ends
- [x] Fewer than two Project Workspaces → silent no-op
- [x] `<leader> w n` and `<leader> w p` are bound; `<leader> w` alone still saves after timeout
- [x] Domain seam tests cover next/previous targets (wrap, skip default, <2 → none) without SDL
- [x] Spoken language matches `CONTEXT.md` (Project Workspace, Default Workspace)

## Answer

Pure seam `editor_core::cycle_project_workspace` (open-order Project Workspace ids + active + direction → target or none). Shell filters out Default Workspace, then switches. Commands/hooks in workspace package; `<leader> w n` / `w p` in vim (short `w` save kept via ticket 01 ambiguous-prefix timeout).
