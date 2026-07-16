# Status commands

Status: resolved
Blocked by: 01

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

A developer can set an Issue’s Status to Open, Planning, In Progress, or Closed via workspace commands (from the Issue buffer or by Issue Id). Any Status may move to any other. Opened at stays immutable. Entering Closed sets Closed at; leaving Closed clears or omits Closed at. Status lives only on the Issue file—never in Code References.

## Acceptance criteria

- [ ] Commands can set each of Open, Planning, In Progress, Closed
- [ ] Any Status can be set from any other (no gated pipeline)
- [ ] Opened at is never rewritten by Status changes
- [ ] Closed at is set when entering Closed and absent when not Closed
- [ ] Domain API tests cover free moves and Closed at behavior
