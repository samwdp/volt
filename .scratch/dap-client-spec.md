## Problem Statement

Volt can register Debug Adapters and prepare Debug Session plans, but cannot run a real Debug Adapter Protocol client. Users cannot start debugging from the editor, toggle Breakpoints with fringe feedback, step through code, or inspect Locals the way they can already use language servers. Debugging today means leaving Volt for another tool.

## Solution

Ship a first-class DAP host: a Volt-owned client (typed protocol plus session lifecycle), compiled Debug Adapter specs in the user package (LSP-shaped), hybrid Debug Configurations, and a system Debug Layout (Breakpoints | editor | Locals with Expressions) with Popup for REPL/pickers. One Debug Session per Workspace; Debug Stop ends it cleanly. Adapters stay user-installed.

## User Stories

1. As a developer, I want to run `dap.start` from Volt, so that I can debug without switching editors.
2. As a developer, I want Volt to pick a Debug Adapter from project type and preference order, so that I am not forced to name the adapter every time.
3. As a developer, I want a picker when multiple Debug Adapters match, so that I can choose between e.g. codelldb and gdb for Rust.
4. As a developer, I want per-adapter start commands (e.g. `dap.start-codelldb`), so that I can bypass bad auto-detect.
5. As a developer, I want `dap.start-last` and `dap.start-recent`, so that I can re-run a previous Debug Configuration quickly.
6. As a developer, I want optional project Debug Configurations plus compiled defaults, so that simple projects work with inference and complex ones can be explicit.
7. As a developer, I want missing launch fields filled via picker or minibuffer, so that I am not blocked by incomplete config.
8. As a developer, I want launch vs attach chosen by the Debug Configuration’s request kind, so that I do not need a separate attach command.
9. As a developer, I want attach templates (e.g. pick process) available as configurations, so that attach workflows still work under `dap.start`.
10. As a developer, I want an optional compile-before-debug step on a Debug Configuration, so that binaries exist before launch.
11. As a developer, I want a confirm picker when Volt infers a build command, so that it does not silently run unexpected builds.
12. As a developer, I want Debug Adapter programs to be my responsibility to install, so that Volt stays thin like LSP.
13. As a developer, I want Debug Adapter specs to feel like Language Server Specs (id, language, extensions, program, args, roots, priority, enabled, transport), so that configuring DAP matches mental models I already have.
14. As a developer, I want stdio and TCP transports, so that adapters like codelldb work alongside stdio-based ones.
15. As a developer, I want at most one Debug Session per Workspace, so that layout and commands stay unambiguous.
16. As a developer, I want switching Workspace to hide the Debug Layout but keep the Debug Session alive, so that I can look at another Workspace mid-debug.
17. As a developer, I want returning to a Workspace with a live Debug Session to rebuild the Debug Layout from Session state, so that I can resume the same debug UI.
18. As a developer, I want `dap.start` to open the Debug Layout (Breakpoints | editor | Locals+Expressions), so that I always have an editor to set Breakpoints.
19. As a developer, I want golden-ratio sizing forced off during the Debug Layout, so that the three panes stay usable.
20. As a developer, I want user split commands blocked while the Debug Layout is active, so that system panes are not fighting user splits.
21. As a developer, I want Debug Stop to restore my prior pane layout and golden-ratio default, so that leaving debug returns me to normal editing.
22. As a developer, I want the REPL, evaluate prompts, and pickers in Popup/minibuffer, so that chrome stays three panes plus transient UI.
23. As a developer, I want to toggle a Breakpoint without a live Debug Session, so that I can set them before start.
24. As a developer, I want Breakpoints stored for the Workspace in memory across buffer closes (for the app lifetime), so that multi-file Breakpoints survive navigation.
25. As a developer, I want Breakpoints synced and verified when a Debug Session starts, so that the adapter and fringe agree.
26. As a developer, I want Breakpoint condition, hit condition, and log message commands, so that I get dap-mode-level Breakpoint control.
27. As a developer, I want a Breakpoints pane listing Workspace Breakpoints, so that I can review and jump to them.
28. As a developer, I want the Debug Fringe to show Breakpoint and execution markers beside git fringe (two cells while a Session is live), so that I do not lose git gutter info.
29. As a developer, I want `dap.continue`, `dap.pause`, `dap.step`, `dap.step_into`, and `dap.step_out`, so that I can control execution.
30. As a developer, I want `dap.restart`, so that I can relaunch the same Debug Configuration without rebuilding the whole workflow by hand.
31. As a developer, I want a single `dap.stop` (Debug Stop) with no separate detach command, so that ending debug is one action.
32. As a developer, I want Debug Stop after launch to terminate the debugee, so that launched processes do not linger.
33. As a developer, I want Debug Stop after attach to leave the process running, so that I do not kill attached processes by accident.
34. As a developer, I want the editor to jump to and focus the stopped source line on a stop event, so that I always see where execution paused.
35. As a developer, I want Locals refreshed when stopped, so that I can inspect variables.
36. As a developer, I want Watch Expressions in the Expressions section under Locals, so that I can track values across stops.
37. As a developer, I want `dap.expressions_add` and `dap.expressions_remove`, so that I can manage watches.
38. As a developer, I want `dap.eval` and `dap.eval_at_point`, so that I can evaluate once without adding a watch.
39. As a developer, I want `dap.repl` as a Popup, so that I get an interactive debug shell when needed.
40. As a developer, I want `dap.switch_thread` and `dap.switch_stack_frame`, so that I can change evaluation/stop context inside one Session.
41. As a developer, I want `dap.log` for transport traffic, so that I can diagnose adapter handshake failures like LSP.
42. As a developer in the Default Workspace, I want to debug when I supply an explicit program/config, so that scratch debugging still works without deep project inference.
43. As a Rust developer, I want preferred adapters ordered codelldb then gdb, so that the common path wins when both are installed.
44. As a C/C++ developer, I want gdb preferred by default, so that native debugging works with a familiar adapter.
45. As a C# developer, I want sharpdbg as the preferred adapter, so that .NET debugging matches the intended default.
46. As a developer, I want missing Debug Adapter binaries to fail with a clear message, so that I know to install the tool myself.
47. As a developer, I want compile-before-debug failures to abort start and surface output, so that I am not launched against a stale or missing binary.

## Implementation Decisions

- Own the DAP client inside the existing debug-planning module (extend it from registry/plans into live client), mirroring the language-server host pattern: schema-oriented types crate plus Volt transport/session code—not a third-party session manager.
- Key live Debug Sessions by Workspace; enforce at most one Session per Workspace.
- Extend Debug Adapter specs toward Language Server Spec parity: identity, language, extensions, program/args, root markers/strategy, enabled-by-default, preference/priority, and transport (stdio or TCP). Launch/attach argument bodies and compile-before-debug live in the Debug Configuration / inference layer, not packed into every adapter row.
- Change the adapter registry from one-extension-one-adapter to multi-adapter-per-extension with preference ordering; ambiguous matches open a picker.
- Hybrid Debug Configuration resolution: compiled user defaults + optional project configs + inference; picker/minibuffer fills holes. `request` on the configuration selects launch vs attach; no dedicated attach command.
- preLaunch: honor explicit compilation on the configuration; otherwise offer a language/project heuristic behind a confirm picker; failure aborts start and surfaces command output via the existing External Command / Command Stream patterns where appropriate.
- On Debug Session start: install Debug Layout (three vertical panes), force golden-ratio off via the same override style used by Database Multiview, snapshot prior layout for restore, block user split commands while the layout is active.
- On Workspace switch away: tear down Debug Layout only; keep Session process/state. On return: rebuild Debug Layout from Session + Breakpoint store.
- Debug Stop: one command—disconnect adapter, teardown layout, restore golden-ratio/layout; `terminateDebuggee` true for launch, false for attach.
- Breakpoint store: Workspace-scoped, in-memory for v1; toggle allowed without Session; sync/`setBreakpoints` on start and on toggle while live; verified vs pending reflected in fringe/pane.
- Right pane: one specialised buffer with Buffer Section Layout—Locals above, Expressions below (empty until watches added).
- Popup hosts REPL, one-shot eval UX as needed, configuration/adapter/process pickers, and confirmations; minibuffer for short prompts.
- Debug Fringe: widen to two cells while the Workspace has a live Session (DAP markers | git fringe); collapse to current single-cell git fringe when idle.
- On `stopped`: open/jump source in center pane, focus editor, update execution fringe, refresh Locals (and watches).
- User package owns compiled adapter list, command/hook surface (dap-mode-inspired v1 set already agreed), and preference defaults; host subscribes to hooks and runs the client—same package→hook→shell pattern as LSP.
- Default Workspace: allow Debug Sessions but require explicit program/configuration; do not deep-infer like a Project Workspace.
- Prefer fake in-process/stdio Debug Adapter fixtures for automated tests; do not require real codelldb/gdb/sharpdbg in CI.

## Testing Decisions

- Good tests assert external behavior: Session lifecycle outcomes, layout/golden-ratio/split policy, Breakpoint store sync, fringe width/markers, command effects, and protocol conversations against a fake adapter—not private helper structure.
- Primary seam: debug module—registry multi-map/preference, configuration planning, Workspace→Session map, Breakpoint store, transport (stdio/TCP), and client request/event handling with a fake adapter.
- Secondary seam: shell—Debug Layout install/teardown, golden-ratio override, user-split blocking, Workspace switch hide/rebuild, Debug Fringe width and markers, command/hook wiring visible at shell level.
- Thin seam: user DAP package exports (commands, hooks, adapter specs)—same style as the user LSP package tests.
- Prior art: existing debug registry tests; language-server client tests; Database Multiview golden-ratio/split tests; git fringe tests; user LSP package tests.
- Out of automated CI: real vendor adapters and pixel-perfect SDL glyph screenshots unless a cheap existing render harness already covers fringe cells.

## Out of Scope

- Multiple Debug Sessions in one Workspace
- Separate detach command
- Persisting Breakpoints to disk
- Exception breakpoint UI, loaded-sources browser, restart-frame, stop-thread, eval-region
- Full VS Code `launch.json` variable expansion and `tasks.json` / `preLaunchTask` compatibility (beyond explicit compile command + heuristic confirm)
- Bundling or auto-installing Debug Adapters
- Always-on two-cell fringe when no Session
- Permanent bottom debug strip (Emacs-style) as a fourth Workspace pane
- Replacing git fringe with DAP-only gutter

## Further Notes

- Domain language: see CONTEXT.md Debugging terms; architecture rationale: ADR-0005.
- Agreed v1 command cut includes start family, stop/restart, continue/pause, step/into/out, Breakpoint toggle/delete/conditions/logpoint, eval/eval_at_point, switch thread/frame, repl, log, expressions add/remove, per-adapter starts, start-last/recent—drawn from dap-mode features with Volt’s one-Session-per-Workspace filter.
- Suggested implementation order for later ticketing: (1) spec/registry/transport + fake adapter client, (2) Breakpoint store + fringe, (3) Debug Layout + golden-ratio/split policy, (4) start/stop/step/locals wiring, (5) watches/REPL/polish commands.
