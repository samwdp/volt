## Parent

https://github.com/samwdp/volt/issues/86

## What to build

Language servers, debug adapters, ACP/agent children, and tool-install helpers Volt starts become Owned Processes via Process Launch. Shared servers use Share Key refcount so the first Workspace Close keeps them alive and the last close (or Application Quit) kills them after graceful protocol shutdown plus force.

## Acceptance criteria

- [ ] LSP, DAP, ACP, and tool-helper OS spawns go through Process Launch into the Process Registry
- [ ] Shared servers use Share Key / multi-Workspace tags: first Workspace Close keeps them; last tag removal or Application Quit kills them
- [ ] Grace phase prefers protocol shutdown where it exists, in parallel with tree signaling, then force on deadline
- [ ] Workspace Close only affects tagged (and last-tag shared) entries; Application Quit clears all of these Owned Processes
- [ ] No reliance on subsystem Drop alone for orphan prevention for these spawns
- [ ] Registry-seam or slice-level tests cover Share Key keep/kill and quit teardown for at least one shared and one single-tagged helper path

## Blocked by

- Process Registry and hybrid Process Launch (parent epic #86 child)
