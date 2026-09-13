## Parent

https://github.com/samwdp/volt/issues/86

## What to build

Finish the expand–contract migration: no Volt feature code bypasses Process Launch for OS children. Provide release-shaped verification that after Application Quit, no Volt-started processes remain and the project directory is not left locked by them.

## Acceptance criteria

- [ ] Audit finds no remaining product spawn paths that create OS children outside Process Launch
- [ ] Bypass paths from the expand phase are removed or made unusable
- [ ] Release-shaped verification: start terminals/jobs/tooling under Volt, quit, confirm no Volt-started descendants remain and the project directory is free of those locks
- [ ] Cross-platform kill-tree / crash-orphan expectations from the parent spec remain satisfied for Launch-created trees
- [ ] Parent epic acceptance bar (“no strays after quit”) is demonstrably met for this work

## Blocked by

- #88
- #89
