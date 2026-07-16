# Place and jump to code

Status: resolved
Blocked by: 01

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

A developer can Place a Code Reference for an Issue into the focused code buffer at the cursor (language-appropriate `TODO(ISS-NNN):` / title) and have that location recorded on the Issue. Jump-to-code: zero refs → clear message to Place first; one ref → go there; many → picker. An Issue may have zero, one, or many Code References.

## Acceptance criteria

- [ ] Place inserts a linked Code Reference at the cursor in the focused code buffer
- [ ] Place records path/line on the Issue
- [ ] Jump with zero Code References shows a clear message (no crash / silent no-op)
- [ ] Jump with one Code Reference navigates to that location
- [ ] Jump with many Code References offers a picker and navigates to the chosen location
- [ ] Domain helpers for 0/1/many selection are tested where exposed
