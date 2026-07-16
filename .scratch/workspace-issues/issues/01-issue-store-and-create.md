# Issue Store and Create

Status: resolved
Blocked by:

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

A developer can Create an Issue with a title in the open workspace: a markdown file appears under the Issue Store (`issues/`), gets a sequential Issue Id (`ISS-NNN`), Opened at is set, Status is Open, and the Issue file opens as a normal buffer for editing title/body. Domain API owns load/save/mint; package + shell only adapt Create UX.

## Acceptance criteria

- [ ] Issue Store directory is created under the workspace root when missing
- [ ] Create mints the next sequential Issue Id from max existing id + 1
- [ ] Create writes an Issue file with Id, Title, Status Open, Opened at, empty Code References, and editable body
- [ ] Create opens the Issue markdown in a buffer
- [ ] Domain API tests cover mint, load, and save without SDL
- [ ] Spoken UI/domain names match `CONTEXT.md` (Issue, not task/ticket)
