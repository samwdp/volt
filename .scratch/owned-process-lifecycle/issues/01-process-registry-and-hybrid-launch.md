## Parent

https://github.com/samwdp/volt/issues/86

## What to build

A Process Registry and mandatory Process Launch path that starts Owned Process trees under hybrid supervision (platform kill-tree plus process supervisor). Callers can tag Workspace ownership and Share Keys. Workspace Close and Application Quit perform graceful-then-force teardown. Registry-seam tests prove launch, refcount, close, quit, and crash/parent-death leave no orphans — using synthetic trees, before product features are migrated.

## Acceptance criteria

- [ ] Process Launch registers an Owned Process tree root (not a lone PID) in the Process Registry
- [ ] Workspace tags and Share Key refcount behave as specified (keep while tags remain; kill when last tag drops or on Application Quit)
- [ ] Workspace Close and Application Quit run graceful-then-force against the correct registry subset / entire registry
- [ ] Hybrid supervision is in place for Launch-created trees (kill-tree + supervisor) so parent death does not leave orphans
- [ ] Registry-seam tests cover launch, Share Key keep/kill, Workspace Close, Application Quit, graceful-then-force deadline, and crash/parent-death
- [ ] Existing ad-hoc product spawns may still exist (migrate comes later); Launch sits beside them as the expand step

## Blocked by

None (can start immediately)
