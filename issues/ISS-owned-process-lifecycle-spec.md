## Problem Statement

After opening and closing Volt (including a release build) with background and terminal work inside a project directory, Windows File Locksmith still shows many `OpenConsole.exe` / `cmd.exe` processes holding locks on that directory. Those children were started by Volt and were not cleaned up on Workspace Close or Application Quit. Users cannot delete or fully reclaim the project folder, and Volt leaves stray OS processes behind.

## Solution

Volt gains a single Process Launch path into a global Process Registry of Owned Processes (process-tree roots). Workspace Close tears down processes tagged for that Workspace (with Share Key refcounting for shared servers). Application Quit performs one global graceful-then-force sweep of the registry. Crash safety uses hybrid supervision: platform kill-trees (Windows Job Object / Unix process group) plus the existing process supervisor watchdog. No intentional survivors; no idle killer while the app is open. Visible nesting under Volt in Task Manager / Activity Monitor is best-effort, not a requirement.

Glossary: see root `CONTEXT.md`. Decision record: `docs/adr/0007-owned-process-hybrid-supervision.md`.

## User Stories

1. As an editor user, I want every process Volt started to die when I quit Volt, so that nothing holds locks on my project folder after exit.
2. As an editor user, I want every process tied to a Workspace to die when I close that Workspace, so that closing one project does not leave its shells and tools running.
3. As an editor user, I want interactive terminal sessions Volt opened to be torn down on Workspace Close and Application Quit, so that ConPTY/OpenConsole hosts do not outlive the editor.
4. As an editor user, I want background shell commands and compile-style jobs Volt started to be torn down the same way, so that one-shot and long-running job children do not orphan.
5. As an editor user, I want language servers Volt started to shut down when no Workspace still needs them, so that shared servers are not killed early and are not left forever.
6. As an editor user, I want debug adapters Volt started to follow the same shared-or-tagged ownership rules, so that debug children do not survive after the last interested Workspace closes.
7. As an editor user, I want ACP/agent-related processes Volt started to be owned and killed with the rest, so that agent helpers cannot lock my tree after quit.
8. As an editor user, I want tool-install and other helper processes Volt started to be owned and killed, so that install helpers cannot orphan either.
9. As an editor user, I want Application Quit to clean everything in one pass, so that quit is reliable and does not depend on iterating Workspace Close successfully.
10. As an editor user, I want Workspace Close to only kill processes tagged for that Workspace (and shared ones whose last tag was removed), so that unrelated Workspaces keep their tools.
11. As an editor user, I want a short graceful shutdown (protocol shutdown where available, plus tree signaling) before force-kill, so that servers and shells can exit cleanly when cheap.
12. As an editor user, I want a hard deadline after which force-kill always wins, so that “graceful” never means “maybe still running.”
13. As an editor user, I want no way for a normal session to detach a child to outlive Volt, so that “no stray processes” is absolute.
14. As an editor user, I want Volt not to idle-kill processes while I still have the app open, so that a waiting build or paused terminal is not murdered for being quiet.
15. As an editor user, I want crash of the Volt UI process to still result in descendant trees dying, so that a hard kill of Volt does not leave the same lock storm as a dirty quit.
16. As an editor user on Windows, I want kill-tree behavior that covers the full descendant tree of each Owned Process, so that OpenConsole/cmd grandchildren die with the root.
17. As an editor user on macOS, I want equivalent kill-tree and crash-orphan behavior, so that platform parity holds outside Windows.
18. As an editor user on Linux, I want equivalent kill-tree and crash-orphan behavior, so that platform parity holds outside Windows.
19. As an editor user, I want processes Volt starts to preferably appear nested under Volt in the OS process viewer when the platform allows, so that I can see what Volt is running — without blocking the cleanup guarantee if nesting is imperfect.
20. As a Volt contributor, I want every OS spawn to go through Process Launch into the Process Registry, so that forgotten spawn sites cannot recreate orphans.
21. As a Volt contributor, I want Owned Process to mean a tree root (Job Object / process group), so that teardown targets the unit that actually must die.
22. As a Volt contributor, I want a Share Key for multi-Workspace servers, so that refcount semantics are explicit in the domain model.
23. As a Volt contributor, I want Workspace Close matching to use explicit tags only (no cwd heuristics), so that teardown does not kill unrelated processes.
24. As a Volt contributor, I want hybrid supervision (platform kill-tree and process supervisor) documented and implemented together, so that Drop-only or supervisor-only gaps cannot regress quietly.
25. As a Volt contributor, I want tests at the Process Registry seam to prove launch, tag/refcount, Workspace Close, Application Quit, and crash-orphan behavior, so that agents can implement without guessing acceptance.
26. As a release-build user, I want the same ownership rules in release as in debug, so that shipping builds do not reintroduce directory locks.
27. As an editor user with two Workspaces on the same on-disk root, I want closing the first Workspace to keep a shared language server alive, so that the second Workspace keeps working.
28. As an editor user with two Workspaces on the same on-disk root, I want closing the last Workspace (or quitting) to kill that shared language server, so that nothing is left behind.
29. As an editor user, I want opening many terminals across sessions and then quitting once to clear all of them, so that repeated open/close cycles do not accumulate OpenConsole/cmd processes.
30. As an editor user, I want file locks on the project directory held only by live Owned Processes I still expect, so that after quit the directory is free to delete or move.

## Implementation Decisions

- Respect `CONTEXT.md` terms: Workspace, Owned Process, Process Registry, Share Key, Process Launch, Workspace Close, Application Quit.
- Respect ADR 0007: hybrid supervision — platform kill-tree plus `volt --process-supervisor` when applicable; single Process Launch into the Process Registry.
- Introduce (or extend) a Process Registry module as the sole ownership index: `OwnedProcessId` primary key; zero or more Workspace tags; optional Share Key for refcounted shared ownership.
- Process Launch is mandatory: terminals, jobs, LSP, DAP, ACP, and helpers must not `Command::spawn` (or equivalent) outside Launch. Bypass is a bug.
- An Owned Process record represents a tree root (Windows Job Object with kill-on-job-close; Unix process group / parent-death primitives), not a lone PID.
- Workspace Close: graceful-then-force for entries tagged with that WorkspaceId; for Share Key entries, remove the tag and kill only when no tags remain.
- Application Quit: one global graceful-then-force over the entire registry (not a loop of Workspace Closes).
- Grace phase: prefer protocol shutdown where it exists (LSP shutdown/exit, DAP disconnect, terminal close) in parallel with tree signaling; then force the tree on deadline. No survivors after the deadline. No detach/keep-alive.
- No idle auto-clean while the app is open.
- Extend/reuse the existing process supervisor rather than inventing a second watchdog story; ensure Launch places children into both the platform kill-tree and supervisor wrapping as required by ADR 0007.
- Visible process-viewer nesting is best-effort on all platforms; correctness of teardown wins.
- Migrate existing spawn sites (jobs supervision wrap, live terminal/PTY, LSP/DAP children, ACP background commands, tool helpers) onto Process Launch; remove reliance on subsystem `Drop` alone for orphan prevention.
- Wire Workspace Close and Application Quit to registry teardown (hooks/runtime shutdown paths as appropriate); do not depend solely on buffer close side effects.
- Cross-platform parity is required for kill-tree and crash-orphan behavior (Windows, macOS, Linux).

## Testing Decisions

- Good tests assert external behavior only: after Process Launch / tag changes / Workspace Close / Application Quit / simulated UI-process death, the Owned Process tree is either still alive (shared tag remains) or fully gone (no descendants holding the test workload). Do not assert internal map layouts, Job Object handle values, or supervisor argv shape except where that is the user-visible CLI contract of the supervisor entrypoint already under test.
- **Primary seam (confirmed):** Process Registry public surface — Process Launch, Workspace tagging / Share Key refcount, Workspace Close, Application Quit. Crash-orphan coverage is exercised through that seam (and Launch’s hybrid supervision) rather than through per-subsystem Drop tests.
- Modules under test: Process Registry / Process Launch; platform kill-tree integration as needed to observe tree death; supervisor only as an implementation detail behind Launch unless a scenario cannot be observed otherwise.
- Prior art: existing process-supervisor request parsing tests; job/LSP session tests that wait on child lifetime — prefer registry-level integration tests that spawn a short-lived descendant tree and assert liveness after teardown operations.
- Cover at least: single-Workspace tag kill; Share Key keep-on-first-close / kill-on-last-close; Application Quit clears all; graceful-then-force deadline; crash/parent-death leaves no orphans for a Launch-created tree.

## Out of Scope

- Idle / unused process auto-cleanup while Volt remains open.
- User-facing detach / “keep running after quit.”
- Guaranteed perfect nesting in Task Manager / Activity Monitor / `pstree` on every platform.
- Redesigning LSP/DAP protocol features unrelated to process ownership.
- Killing processes not started by Volt.
- Heuristic teardown based on working directory or buffer ancestry without registry tags.
- Changing Cargo workspace layout for its own sake.

## Further Notes

- Repro symptom: File Locksmith on a project directory after using a release Volt build shows many `OpenConsole.exe` / `cmd.exe` pairs; nothing else on the machine was using that directory.
- Sparse/local trees may be missing some editor crate sources; implementers should use the full repo / vendor mirrors as needed and keep behavior aligned with ADR 0007 and `CONTEXT.md`.
- Implementation should treat “no strays after quit” as a release-blocking acceptance bar for this ticket.
