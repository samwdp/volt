# Async Capture on save

Status: ready-for-agent
Blocked by: 03

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

Saving a file returns immediately. Capture for that file runs in the background (editor jobs or equivalent). When finished, rewrite-if-unchanged still applies: mint always; rewrite only if the line still matches the snapshot. The user never waits on Issue IO for save.

## Acceptance criteria

- [ ] `buffer.save` completes without waiting for Capture to finish
- [ ] Capture of the saved file still mints Issues and applies rewrite-if-unchanged
- [ ] Background failure or skip still surfaces an observable outcome (no silent loss of minted Issues)
- [ ] Saving one file does not Scan the whole workspace tree
- [ ] At least one adapter-level test or demo proves save is non-blocking relative to Capture
