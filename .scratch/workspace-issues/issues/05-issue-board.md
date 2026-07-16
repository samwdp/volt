# Issue Board

Status: ready-for-agent
Blocked by: 01, 02

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

A developer can open an Issue Board: a generated commands-only buffer listing Issues from the Issue Store. By default Closed Issues are hidden; a command toggles showing Closed. From a Board row the developer can open the Issue file and change Status. The Board refreshes after Create/Capture/Status changes. The Board is not a second store file and is not freeform write-through.

## Acceptance criteria

- [ ] Board lists Open, Planning, and In Progress by default
- [ ] Closed Issues are hidden until a show-Closed command/toggle
- [ ] Opening a row opens the Issue markdown buffer
- [ ] Status commands work for the Issue under the cursor
- [ ] Board content refreshes after Create, Capture, or Status changes
- [ ] Editing Board text does not write through to Issue files (commands-only)
