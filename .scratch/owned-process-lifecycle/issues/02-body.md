## Parent

https://github.com/samwdp/volt/issues/86

## What to build

Interactive terminals and background/compile-style jobs Volt starts become Owned Processes via Process Launch. Closing a Workspace or quitting the application tears down their full process trees so OpenConsole/cmd (and job children) do not hold locks on the project directory after exit.

## Acceptance criteria

- [ ] Interactive terminal sessions are started only through Process Launch and appear in the Process Registry with Workspace tags
- [ ] Background/compile-style jobs are started only through Process Launch and are registry-owned
- [ ] Workspace Close tears down terminal and job trees tagged for that Workspace
- [ ] Application Quit clears remaining terminal and job Owned Processes in one global registry sweep
- [ ] Repro-class behavior: after using terminals/jobs and quitting, no Volt-started shell host/job descendants remain locking the project directory
- [ ] Behavior is covered at the Process Registry seam and/or end-to-end checks appropriate to this slice

## Blocked by

- #87
