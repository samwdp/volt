# Open Issue from Code Reference

Status: resolved
Blocked by: 01

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

A developer with the cursor on a linked Code Reference (`TODO(ISS-NNN):` / `FIXME(ISS-NNN):`) can open the matching Issue markdown buffer. Missing Issue files (orphan ids) are reported; no auto-create.

## Acceptance criteria

- [ ] Command/action from a linked Code Reference opens the correct Issue file
- [ ] Works for both TODO and FIXME linked forms
- [ ] Orphan Issue Id under the cursor reports clearly and does not mint a new Issue
- [ ] Opening an Issue uses the raw markdown buffer (Status still via commands from ticket 02 when available)
