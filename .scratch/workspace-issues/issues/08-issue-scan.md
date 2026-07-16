# Issue Scan

Status: ready-for-agent
Blocked by: 03, 04

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

A developer can run a workspace-wide Issue Scan that Captures unlinked TODO/FIXME comments across the project tree without blocking the UI. Scan refreshes Code Reference locations from what it finds, drops stale locations from Issues, never deletes Issues, and reports orphan Issue Ids in comments without auto-creating files. Outcomes (minted, rewritten, skipped, orphan, pruned) are observable.

## Acceptance criteria

- [ ] Issue Scan command processes the workspace tree without blocking the UI
- [ ] Unlinked TODO/FIXME across files are Captured with the same mint + rewrite-if-unchanged rules
- [ ] Stale Code Reference locations are removed from Issues when comments are gone
- [ ] Issues are never deleted by Scan when refs disappear
- [ ] Orphan linked comments are reported and do not create Issue files
- [ ] Domain API tests cover prune and orphan reporting; async non-blocking covered at adapter level
