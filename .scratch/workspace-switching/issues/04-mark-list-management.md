# 04 — Mark List Management

**What to build:** A developer can Mark and Unmark the active Project Workspace root, and open the Mark List as a normal buffer to reorder by editing and saving. The Mark List is app-wide plain text (`marked-workspaces.txt`), one path per line, blanks stripped on save, no comments. Mark appends if absent (else no-op); Unmark removes the current root; no-root Mark/Unmark notifies. Commands/chords: `workspace.mark` (`<leader> w +`), `workspace.unmark` (`<leader> w -`), `workspace.marks` (`<leader> w m`).

**Blocked by:** 01 — Ambiguous Prefix Timeout

**Status:** resolved

## Parent

`.scratch/workspace-switching/PRD.md`

## Acceptance criteria

- [x] Mark appends the active project root to the Mark List if absent; duplicate Mark is a no-op
- [x] Unmark removes the active project root from the Mark List
- [x] Mark/Unmark on Default Workspace (no root) notifies and does not mutate the list
- [x] Mark List lives in the application Volt state directory as `marked-workspaces.txt`, one path per line
- [x] `workspace.marks` opens the real Mark List file; save reloads the live list with blanks stripped
- [x] More than four paths are allowed; order is preserved; new marks append at the end
- [x] Domain seam tests cover parse/serialize, mark, unmark, and duplicate no-op without SDL
- [x] Spoken language matches `CONTEXT.md` (Mark List, Marked Workspace)

## Answer

Pure seam `editor_core::MarkList` (parse/serialize/mark/unmark/slots) plus shell adapter: persist under Volt state dir as `marked-workspaces.txt`, canonicalize existing roots for path identity, open real file via `workspace.marks`, normalize+reload on save. Commands/hooks in workspace package; `<leader> w +` / `w -` / `w m` in vim.
