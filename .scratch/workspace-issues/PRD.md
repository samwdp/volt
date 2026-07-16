# Workspace Issues Plugin

Status: ready-for-agent

## Problem Statement

Developers track work for the project open in Volt using ad hoc TODO comments and scattered notes. There is no workspace-native Issue Store: comments are not linked to durable records, status is not queryable, and there is no Board or commands to Create, Capture, Place, or navigate between code and Issues. Agent scratch tickets under `.scratch/` solve a different problem and must not be overloaded for project Issues.

## Solution

Ship a workspace-scoped Issues plugin backed by markdown files in `issues/` at the workspace root. An Issue is the markdown file; a Code Reference (`TODO(ISS-NNN):` / `FIXME(ISS-NNN):`) is only a link. Capture on save and Issue Scan mint Issues from unlinked TODOs/FIXMEs without blocking the UI. An Issue Board lists active Issues and is driven by commands. Users can Create Issues without code, Place Code References into the focused buffer, open an Issue from a Code Reference, and jump from an Issue to its Code References.

Domain language lives in `CONTEXT.md` (Issue, Issue Id, Issue Store, Issue Status, Code Reference, Capture, Create, Place, Issue Board, Issue Scan, Opened at, Closed at).

## User Stories

1. As a developer, I want an Issue Store under `issues/` in my workspace, so that project work is durable and shareable in git.
2. As a developer, I want each Issue to be one markdown file, so that I can edit notes in a normal buffer and review them in PRs.
3. As a developer, I want sequential Issue Ids (`ISS-NNN`), so that Code References stay stable when titles change.
4. As a developer, I want to write `// TODO: fix login` and have Capture mint an Issue and rewrite to `// TODO(ISS-042): fix login`, so that tracking starts from natural comments.
5. As a developer, I want the same for `FIXME`, so that urgent markers are tracked the same way.
6. As a developer, I want Capture to run when I save a file, so that I do not remember a separate promote step.
7. As a developer, I want save to return immediately while Capture runs in the background, so that I never wait on Issue IO.
8. As a developer, I want Capture to always mint the Issue even if my buffer changed before rewrite, so that work is not lost.
9. As a developer, I want the comment rewritten only if the line still matches what Capture saw, so that my keystrokes are not stomped.
10. As a developer, I want a diagnostic or clear signal when rewrite was skipped, so that I can link the comment later.
11. As a developer, I want an Issue Scan command over the workspace tree, so that I can catch unlinked TODOs after clone or bulk edits.
12. As a developer, I want Issue Scan to be non-blocking, so that large trees do not freeze the editor.
13. As a developer, I want Issue Scan to drop stale Code Reference locations from an Issue when the comment is gone, so that jump targets stay honest.
14. As a developer, I want Issue Scan never to delete an Issue when its last Code Reference disappears, so that markdown remains project truth.
15. As a developer, I want orphan `TODO(ISS-099):` comments (no matching file) reported without auto-creating a file, so that typos do not spawn ghost Issues.
16. As a developer, I want Status recorded only on the Issue file, so that comments do not drift from the Board.
17. As a developer, I want Statuses Open, Planning, In Progress, and Closed, so that I can reflect real workflow stages.
18. As a developer, I want to set any Status from any other via commands, so that I am not trapped in a pipeline.
19. As a developer, I want Opened at set once on Create/Capture, so that I know when work entered the store.
20. As a developer, I want Closed at set when entering Closed and cleared or absent when not Closed, so that completion time is visible.
21. As a developer, I want an Issue Board buffer listing active Issues, so that I can see current work at a glance.
22. As a developer, I want Closed Issues hidden on the Board by default, so that the Board stays an active work surface.
23. As a developer, I want a command to show Closed Issues on the Board, so that I can audit finished work.
24. As a developer, I want Board rows actionable by commands (not freeform write-through), so that Status and ids stay valid.
25. As a developer, I want to open the Issue markdown from the Board, so that I can edit title and body as text.
26. As a developer, I want Status changes via commands even while the Issue file is open, so that frontmatter is not hand-broken.
27. As a developer, I want Create with a title prompt and no code, so that I can track work that is not yet anchored in source.
28. As a developer, I want Place to insert a Code Reference at the cursor in the focused code buffer, so that I can link an existing Issue into code.
29. As a developer, I want Place to record the location on the Issue, so that jump-to-code works later.
30. As a developer, I want jump-to-code with one Code Reference to go straight there, so that navigation is fast.
31. As a developer, I want jump-to-code with many Code References to offer a picker, so that I choose the right site.
32. As a developer, I want jump-to-code with zero Code References to tell me to Place first, so that failure is clear.
33. As a developer, I want to open the correct Issue from a linked Code Reference under the cursor, so that I can move from code to the record.
34. As a developer, I want an Issue to allow zero, one, or many Code References, so that shared work and unlinked planning both work.
35. As a developer, I want deleting a Code Reference to leave the Issue intact, so that removing a comment does not erase project truth.
36. As a developer, I want workspace-scoped commands and keybindings for Issues, so that the feature feels like other Volt workspace packages.
37. As a developer, I want this feature distinct from `.scratch/` agent tickets, so that agent process and project Issues never collide.
38. As a developer, I want language-appropriate line comment markers when Capturing/Placing (`//`, `#`, `--`, etc.), so that non-Rust files work.
39. As a developer, I want Board refresh after Create/Capture/Status changes, so that the list matches the store.
40. As a plugin author / agent, I want a single Issue domain API test seam, so that behavior is proven without SDL.
41. As a developer, I want next Issue Id allocated from the max existing id in the store, so that numbering stays sequential without a separate service.
42. As a developer, I want Issue files named in a stable, human-readable way that includes the id (and optional slug), so that the store is browsable outside Volt.
43. As a developer, I want unlinked HACK/XXX ignored in v1, so that Capture noise stays low.
44. As a developer, I want Capture on save limited to the saved file, so that saving one buffer does not scan the whole tree.
45. As a developer, I want diagnostics or messages for Capture/Scan outcomes (minted, rewritten, skipped rewrite, orphan), so that background work is observable.

## Implementation Decisions

- **Primary seam:** one Issue domain module/crate (same layering spirit as `editor-git`): load/save Issue Store, mint Issue Id, Create, Capture over file text, Issue Scan over a file set, Status transitions (including Closed at), Place recording, Board listing with Closed filter, stale-path prune, orphan reporting. Inputs/outputs are paths, timestamps, and string snapshots/patches — not SDL types.
- **Adapters:** a `user` package declares workspace commands, hooks, and keybindings (EmitHook / open buffer patterns like `interactive` and `git`). The shell subscribes to hooks: enqueue Capture on `buffer.save`, run Issue Scan jobs, render/refresh Issue Board plugin buffer, open Issue files, apply comment rewrite patches when the line is unchanged, drive pickers for multi-ref jump and Create title input.
- **Async:** Capture (on save) and Issue Scan must not block the UI. Prefer `editor-jobs` (or equivalent background worker) to run domain work off the editor thread. Save completes before Capture finishes.
- **Rewrite-if-unchanged:** Capture returns a patch intent against a snapshot line; the adapter applies it only if the live buffer (or disk, if no dirty conflict policy says otherwise) still matches that snapshot; otherwise keep Issue and surface skipped-link.
- **Issue document fields:** Issue Id, Title, Status, Opened at, Closed at (when Closed), Code References list (path + line, optionally snippet/hash), free markdown body.
- **On-disk shape:** markdown under workspace-root `issues/`; sequential ids; filename includes numeric id and slug derived from title. Prefer structured header (YAML frontmatter or keyed lines) parseable by the domain API; body is free markdown below.
- **Code Reference syntax:** linked `TODO(ISS-NNN):` / `FIXME(ISS-NNN):`; unlinked `TODO:` / `FIXME:`. Line comments only in v1. Status never appears in the comment.
- **Status model:** Open, Planning, In Progress, Closed; any → any via commands; default on Create/Capture is Open.
- **Issue Board:** generated plugin/scratch-style buffer (commands-only; not a second store file). Default list excludes Closed; toggle command includes them. Cursor row selects the Issue for open / status / jump / Place context.
- **Navigation:** from Code Reference → open Issue file buffer; from Issue/Board → jump Code References (1 / many / zero rules); Place inserts at cursor in focused code buffer and appends location on the Issue.
- **Orphans / stale:** Scan refreshes locations from what it finds; removes missing locations from the Issue; never deletes Issue; orphan ids diagnostic only.
- **Id allocation:** next id = max existing `ISS-NNN` in store + 1 (create store dir if missing). Document merge-conflict risk on parallel branches as accepted for v1.
- **Scope of package:** workspace keymap scope; auto-load package unless product decides otherwise (default: auto-load so Board/Capture are available).
- **Non-goals in core crates:** do not reuse `.scratch/` paths or agent triage label vocabulary for product Issues.
- **Glossary:** keep `CONTEXT.md` as source of spoken terms; implementation must not invent parallel names (Task/ticket/Done) in UI strings.

## Testing Decisions

- **Good tests** assert externally visible domain behavior: Issue files created/updated, ids assigned, Status/Opened at/Closed at, Code Reference list membership, returned rewrite patches, Board listing filter, orphan/stale diagnostics. Do not assert private helpers, job thread internals, or SDL draw calls.
- **Primary tested module:** the Issue domain API (the single seam above). Prefer in-memory or temp-directory fixtures for the Issue Store and sample source files.
- **Prior art:** crate-level tests in `editor-git` / `editor-fs` / `editor-jobs` (temp dirs, pure results); shell tests in `editor-sdl` only when proving adapter contracts that domain API cannot express (e.g. save returns before Capture completes, rewrite skipped when buffer dirty diverged) — keep those few.
- **Suggested domain cases:** Capture unlinked TODO → file + patch; Capture with diverged line → file + no apply; Create without refs; Status free moves and Closed at; Scan prunes stale path; Scan reports orphan id; Board hides Closed; Place records location; jump target selection inputs (0/1/many) as pure helpers if exposed.
- **CI:** domain crate covered by `cargo xtask ci` like other workspace crates.

## Out of Scope

- Agent / skill issue tracker under `.scratch/` (unchanged; separate product).
- Write-through editing of the Issue Board text.
- Status mirrored inside Code Reference comments.
- Block comment Captures (`/* */`), and HACK/XXX markers.
- Assignees, priority, labels, milestones, dependencies between Issues.
- Strict Status pipeline / gated transitions.
- UUID or hash Issue Ids; configurable Issue Store path (non-`issues/` root).
- Sync with GitHub/GitLab/Jira or export formats beyond the markdown store.
- Multi-root workspace store merging policy beyond “active workspace root”.
- Deleting Issues (hard delete / soft-delete policy) — deferred; Closed is the v1 terminal Status.
- Real-time collaborative editing of the Issue Store.
- Gutter/diagnostics UI polish beyond minimal orphan/skipped-link signals (can follow-up).

## Further Notes

- Grilling session locked shared understanding; glossary is in root `CONTEXT.md`.
- Recommended follow-up ADR (not yet filed): Capture and Issue Scan run asynchronously via editor jobs and never block save — hard to reverse, surprising, and a real trade-off against simpler sync Capture.
- Issue delete policy was explicitly left foggy: v1 should not require delete commands; Closing is enough.
- Implementation should stay declarative in `user` where possible; any shell subscription for `buffer.save` / Board must be wired or the package will register but do nothing (existing Volt hook lesson).
