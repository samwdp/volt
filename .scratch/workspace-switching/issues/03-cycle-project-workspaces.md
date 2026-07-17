# 03 — Cycle Project Workspaces

**What to build:** A developer can run `workspace.next` and `workspace.previous` (bound to `<leader> w n` and `<leader> w p`) to move among open Project Workspaces in open order, skipping the Default Workspace, wrapping at the ends, and silently no-oping when fewer than two Project Workspaces are open. Lone `<leader> w` still saves via the ambiguous-prefix timeout from ticket 01.

**Blocked by:** 01 — Ambiguous Prefix Timeout

**Status:** ready-for-agent

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [ ] `workspace.next` / `workspace.previous` switch among open Project Workspaces in open order
- [ ] Default Workspace is skipped in the cycle
- [ ] Cycle wraps at both ends
- [ ] Fewer than two Project Workspaces → silent no-op
- [ ] `<leader> w n` and `<leader> w p` are bound; `<leader> w` alone still saves after timeout
- [ ] Domain seam tests cover next/previous targets (wrap, skip default, <2 → none) without SDL
- [ ] Spoken language matches `CONTEXT.md` (Project Workspace, Default Workspace)
