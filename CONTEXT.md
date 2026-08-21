# Volt

Editor/runtime for a plugin-driven workspace shell. This context covers product language shared across core crates and user packages.

## Language

### Workspace

**Workspace**:
An open editing context in a Window: a name, optional project root, and its panes/buffers. Users switch among Workspaces in the same Window.
_Avoid_: project (when meaning the open editor context), tab (as the Workspace itself), session

**Default Workspace**:
The scratch Workspace with no project root, present for boot/scratch editing. It is not part of project Workspace cycling and cannot be Marked.
_Avoid_: unnamed workspace, empty workspace (as the spoken term)

**Project Workspace**:
A Workspace that has a project root path. Next/previous cycling and Marks apply only to these.
_Avoid_: rooted workspace (as the spoken term)

**Mark List**:
The ordered, app-wide list of project root paths used as Marked Workspaces. Canonical store is a plain-text file of one path per line; editing and saving that file manages order and membership beyond Mark/Unmark commands.
_Avoid_: favorites file, bookmark list, pin list, `.volt` marks (as product store)

**Marked Workspace**:
A project root path recorded in the Mark List for quick jump. List order matters; only the first four entries have dedicated key bindings.
_Avoid_: favorite, bookmark, pin, starred workspace

**Minor Mode**:
A keybinding layer that overrides Global bindings while it is active. Workspace editing, Popup, autocomplete, and hover are Minor Modes. Global bindings are the fallback when no active Minor Mode claims the chord.
_Avoid_: keymap scope (as the spoken product term), major mode (unless introduced later)

**Worktree**:
A git worktree checkout path that can be opened as a Project Workspace root. Distinct from the open editor Workspace itself.
_Avoid_: project (when meaning the checkout path), branch (when meaning the directory), workspace (when meaning the on-disk checkout)

**Workspace Dashboard**:
The picker that lists Worktrees for the active context so the user can open, create, or remove them. Opened by `workspace.dashboard` / `<leader> p d`.
_Avoid_: project dashboard, worktree list (as the spoken product name)

**Worktree Remove**:
Force-removing a Worktree from disk (`git worktree remove <path> --force`), after first closing every open Project Workspace whose root is that path (including the active one). Does not change the Mark List, and does not delete the branch by itself.
_Avoid_: delete workspace (when meaning disk remove), worktree delete (as the spoken product name)

**Picker Extra Keybind**:
A chord→command binding attached to one open picker instance (copied from its provider declaration), layered above the shared picker-plugin Popup bindings. Firing an extra always closes the picker after snapshotting the needed row/list context into the command invocation. Used for picker-specific actions such as Quickfix export or Worktree Remove.
_Avoid_: popup keymap (when meaning per-picker extras), secondary action (as the spoken product name)

### Issues

**Issue**:
A tracked unit of work whose canonical record is a markdown file in the Issue Store. Deleting a Code Reference does not delete the Issue. Opening an Issue opens that markdown file as a normal buffer; Status (and Closed at) change via commands, while title and body may be edited as text.
_Avoid_: Task, ticket, TODO (when meaning the stored record)

**Code Reference**:
A comment in source that points at an Issue. It is a link, not the Issue itself. An Issue may have zero, one, or many Code References. The join key is an Issue Id embedded in the comment. Linked form is `TODO(ISS-NNN):` or `FIXME(ISS-NNN):` (language-appropriate line comment). Unlinked form is `TODO:` / `FIXME:` with no id—Capture may promote these.
_Avoid_: TODO (when meaning the Issue), comment-issue, inline issue, status-in-comment

**Issue Id**:
A stable, sequential, human-readable identifier for an Issue (spoken form `ISS-NNN`). Independent of title or file path. Code References and Issue files both carry this id.
_Avoid_: slug (as identity), title key, path-as-id, UUID-as-primary-id

**Issue Store**:
The workspace-root directory `issues/` that holds Issue markdown files. Project truth for Issues; distinct from agent scratch tickets under `.scratch/`.
_Avoid_: `.volt/issues`, `.scratch/issues` (as the product Issue Store), ticket folder

**Issue Status**:
The workflow state of an Issue, recorded only on the Issue markdown. Code References do not carry or own status. The statuses are Open, Planning, In Progress, and Closed. Any Status may be set from any other; transitions are not gated.
_Avoid_: state-in-comment, TODO tag as status, triage label (agent `.scratch` vocabulary), Done (say Closed), Todo (say Open)

**Open**:
Issue Status meaning the Issue exists and is not yet being planned or worked.
_Avoid_: Todo, backlog (as Status name)

**Planning**:
Issue Status meaning the Issue is being scoped or designed before implementation.
_Avoid_: needs-triage, ready-for-agent

**In Progress**:
Issue Status meaning active implementation work is underway.
_Avoid_: Active, started, claimed

**Closed**:
Issue Status meaning the Issue is no longer active work (completed, dropped, or superseded).
_Avoid_: Done, resolved, complete (as Status name)

**Capture**:
The act of turning a source comment that has no Issue Id into an Issue plus a Code Reference: mint id, write Issue file, rewrite the comment to embed the id. Capture runs when a buffer is saved (for that file) and when an explicit workspace-wide scan is requested. Capture must not block the user waiting on save or scan completion. The Issue is always minted; embedding the Issue Id into the comment happens only when that comment text still matches what Capture observed—otherwise the Issue stands and the comment stays unlinked until linked later.
_Avoid_: scrape (as the product verb), import, sync

**Create**:
Minting an Issue from a command and title, with no Code Reference required. Place may add links later.
_Avoid_: Capture (when no comment is involved), new task

**Issue Scan**:
A workspace-wide pass that Captures unlinked comments and refreshes Code Reference links across the project tree. It drops stale locations from an Issue when the comment is gone, but never deletes the Issue. A Code Reference whose Issue Id has no file is reported, not auto-recreated.
_Avoid_: full scrape, index rebuild (unless that is a separate concept)

**Issue Board**:
A generated workspace buffer that lists Issues from the Issue Store. It is a view, not the store; Issue markdown files remain canonical. The Board is actionable via commands on the current row, not via freeform write-through editing of the Board text. By default it lists Open, Planning, and In Progress; Closed Issues are hidden unless the user asks to show them.
_Avoid_: index file (as truth), task list buffer, TODO buffer, issues README

**Place**:
Inserting a Code Reference for an Issue into the focused code buffer at the cursor, and recording that location on the Issue. Jump-to-code uses recorded Code References (one → go; many → pick; zero → no jump, Place first).
_Avoid_: embed, inject, link-insert (as the product verb)

**Opened at**:
The timestamp when an Issue was first created. Set once; not edited later.
_Avoid_: created, created_at (as spoken term)

**Closed at**:
The timestamp when an Issue last entered Closed. Absent while not Closed.
_Avoid_: completed_at, resolved_at

### External commands

**External Command**:
A process the editor spawns outside its own address space (for example git or a build tool), distinct from an internal editor command id.
_Avoid_: job (as the spoken product term), shell command (as the umbrella), subprocess (as the spoken product term)

**Command Stream**:
Live stdout/stderr of an External Command written into a popup buffer while the process runs. Default presentation for user-facing long, network, or progress External Commands.
_Avoid_: terminal session (when meaning this popup), log buffer, streamed job

**Silent Command**:
An External Command run to completion with no Command Stream popup; the caller waits and receives the result. Used for silent prep (for example Workspace Dashboard `git fetch --prune`) and fast local mutators.
_Avoid_: background command (when meaning this sync wait), quiet job, hidden stream

**Git Editor Buffer**:
A normal editable pane buffer that stands in for `GIT_EDITOR` while an External Command waits. Confirm saves the file and exits the editor stub successfully; abort exits non-zero. Distinct from the dedicated commit message buffer, which never uses `GIT_EDITOR`.
_Avoid_: commit message buffer (when meaning the dedicated commit UI), stream popup (as the editor surface)

### Language servers

**Solution**:
A `.sln` file that groups one or more C# projects. When a Solution is available, csharp-ls uses one Language Server Session for that Solution — not one Session per `.csproj`.
_Avoid_: project (when meaning the `.sln`), workspace (when meaning the `.sln`)

**Language Server Session**:
One planned or live language-server process for a given server and root. For csharp-ls, the root is the Solution’s directory when a Solution applies; otherwise the nearest `.csproj` directory or the Project Workspace root.
_Avoid_: LSP instance (as the spoken product term), server process (when meaning the Session identity)

**Language Server Session Stop**:
Choosing one live Language Server Session from a picker scoped to the active Workspace and shutting that Session down for the whole app. `lsp.stop` means this — not detaching LSP from the active buffer only.
_Avoid_: stop buffer LSP, detach LSP (as the spoken product meaning of `lsp.stop`)

**Language Server Session Restart**:
Same picker grain as Language Server Session Stop: choose one live Session in the active Workspace and restart it. `lsp.restart` means this.
_Avoid_: restart buffer LSP (as the spoken product meaning of `lsp.restart`)

### Debugging

**Debug Adapter**:
An external Debug Adapter Protocol process the user installs on their machine (for example codelldb, gdb, or sharpdbg). Volt does not bundle adapters; compiled user specs only declare how to find and talk to them.
_Avoid_: debugger (when meaning the adapter process), DAP server (as the spoken product term)

**Debug Session**:
At most one live debug connection owned by a Workspace. Leaving the Workspace tears down the Debug Layout but keeps the Session alive; returning rebuilds the layout from Session state.
_Avoid_: DAP instance, debug process (when meaning Session identity), multi-session-per-workspace

**Debug Configuration**:
A named launch or attach setup: request kind, program or process target, cwd, args, and optional compile-before-debug. Comes from compiled defaults, optional project files, inference, and picker/minibuffer fills for missing pieces.
_Avoid_: launch.json (as the product name), run config (as the spoken term)

**Debug Layout**:
The system-owned three-pane Workspace split used while debugging: Breakpoints, editor, and Locals (with Expressions as a section). User splits are blocked while it is active; ending the Session restores the prior layout and golden-ratio default. REPL, console, and pickers use Popup, not a fourth pane.
_Avoid_: debug UI (vague), dap multiview (as the spoken product name), four-pane debug

**Debug Stop**:
The single command that ends the active Workspace’s Debug Session: disconnect the Debug Adapter, tear down the Debug Layout, and—when the Session was a launch—terminate the debugee; attach Sessions leave the process running. There is no separate detach command.
_Avoid_: dap.detach, disconnect (as a second user-facing command)

**Breakpoint**:
A line marker stored for a Workspace (in memory for now) that can be toggled without a live Debug Session and is synced when a Session starts. Shown in the Debug Fringe; the Breakpoints pane lists them.
_Avoid_: debug point, red dot (as the product term)

**Watch Expression**:
A user-owned expression kept in the Expressions section of the Locals pane and re-evaluated while stopped. Distinct from one-shot evaluate via minibuffer or the debug REPL Popup.
_Avoid_: watch (alone), expression breakpoint

**Debug Fringe**:
While a Workspace has a live Debug Session, the editor gutter widens so DAP markers (breakpoint and execution position) sit beside git fringe marks instead of replacing them.
_Avoid_: debug gutter (as the spoken product term), breakpoint column

### Markdown presentation

**Markdown Pretty**:
The beautified on-screen presentation of markdown (icons, concealed markers, inline images, and related chrome) while the buffer’s underlying text stays canonical markdown source.
_Avoid_: preview pane (when meaning in-buffer pretty), rendered mode (as the spoken product term), WYSIWYG

**Markdown Raw**:
Showing the canonical markdown source text with Pretty chrome suppressed for that region or buffer.
_Avoid_: source mode (when meaning this), plain markdown (as the spoken product term)

**Anti-conceal**:
Temporarily showing Markdown Raw for the cursor line, and for lines under a Visual selection, so navigation and yank operate on real source text.
_Avoid_: unpretty, reveal syntax, disable pretty (when meaning only the cursor/selection reveal)

**Forced Language**:
A sticky language id on a buffer that is not inferred from path or extension, so non-file buffers (for example ACP output) can still be treated as markdown for highlighting and Pretty.
_Avoid_: fake filetype, hardcoded filetype (as the spoken product term), override extension

**Markdown Pipeline**:
The single presentation path that turns markdown source (from a buffer or an ephemeral string) into highlight + Pretty output, shared by file buffers, hover, and ACP markdown payloads.
_Avoid_: markdown renderer (when meaning several ad-hoc paths), hover-only markdown, ACP-only markdown paint

**Pretty Kill-switch**:
An optional size threshold that disables Markdown Pretty for an oversized source and falls back to Markdown Raw; default off.
_Avoid_: pretty disable (when meaning the global/buffer enable toggle), render budget (as the spoken product term)

### Database

**Database Dashboard**:
A specialised plugin buffer (`db.dashboard`) whose sections are laid out as Connections and Tables on the left, Editor and Output on the right. The Editor is an embedded writable buffer with the same keybinds as a normal query buffer. `Ctrl+c Ctrl+c` in Editor runs SQL and replaces Output; multiple statements concatenate into one scrollable Output. Opening a file while Dashboard is focused loads it into Editor; write/save targets that file.
_Avoid_: query popup (when meaning this 2D buffer), schema browser (as the whole surface)

**Database Multiview**:
A workspace split opened by `db.multiview`: a narrow left sidebar (Connections above Tables) plus normal query buffers on the right. Golden-ratio sizing is forced off; left weight is smaller than right. Query output stays a popup, not an in-buffer Output section.
_Avoid_: dashboard (when meaning this split), sidebar-only view

**Buffer Section Layout**:
A named split tree on plugin buffer sections (rows/columns with weights) used to place specialised panes. Absent a layout, sections stack vertically as before (Calculator Input/Output).
_Avoid_: workspace split (when meaning in-buffer panes), pane layout (when meaning section chrome)
