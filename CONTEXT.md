# Volt

Editor/runtime for a plugin-driven workspace shell. This context covers product language shared across core crates and user packages.

## Language

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
