# Capture focused file

Status: resolved
Blocked by: 01

## Parent

`.scratch/workspace-issues/PRD.md`

## What to build

A developer can Capture unlinked `TODO` / `FIXME` line comments in the focused file: each becomes an Issue (Open + Opened at) and a rewrite intent embeds `TODO(ISS-NNN):` / `FIXME(ISS-NNN):`. The Issue is always minted. The comment is rewritten only if the line still matches what Capture observed; otherwise the Issue stands and a skipped-link signal is shown. HACK/XXX are ignored. Line comments only; language-appropriate markers (`//`, `#`, `--`, …).

## Acceptance criteria

- [ ] Unlinked TODO/FIXME in the focused file mint Issues and produce rewrite intents with Issue Ids
- [ ] FIXME is handled the same as TODO
- [ ] HACK/XXX are not Captured
- [ ] Diverged line → Issue exists, rewrite not applied, skipped-link is observable
- [ ] Matching line → Code Reference rewrite applied and location recorded on the Issue
- [ ] Domain API tests cover mint + patch / skip without SDL
- [ ] A workspace command Captures the focused file end-to-end
