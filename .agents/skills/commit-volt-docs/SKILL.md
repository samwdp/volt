---
name: commit-volt-docs
description: >-
  Commit documentation changes inside the docs submodule (volt-docs), then bump
  the submodule pointer in the Volt parent repo. Use after update-volt-docs, or
  when the user asks to commit docs-only changes.
compatibility: Requires git; docs/ must be the volt-docs submodule worktree.
---

# Commit Volt Docs

Commit docs work in the **docs submodule first**, then record the new gitlink in the Volt parent repo. Run as a **fresh subagent** (or the same docs subagent after update-volt-docs finishes).

## Safety

- Commit **only** documentation changes under `docs/` (and the parent `docs` gitlink). Never stage secrets, `user.dll`, or unrelated Volt source.
- Do **not** push unless the user or parent brief explicitly asks.
- Do **not** amend, force-push, or skip hooks.
- Do **not** update git config.

## Steps

### 1. Commit inside `docs/` (volt-docs)

From `docs/`:

```bash
git status
git diff
git log -5 --oneline
```

- Stage intended paths only (`git add` specific files).
- Skip generated artifacts that are gitignored (e.g. `search-index.json`, `node_modules`, dist).
- Draft a Conventional Commits message focused on **why**:
  - `docs: …` for content
  - `feat(docs): …` when adding a new help surface users will discover
  - Reference issues when known: `Refs #N` / `Closes #N`
- Commit (example; adapt to shell):

```bash
git commit -m "$(cat <<'EOF'
docs: sync plugin pages after volt package changes

Refs #123
EOF
)"
```

On Windows PowerShell without HEREDOC:

```powershell
git commit -m "docs: sync plugin pages after volt package changes`n`nRefs #123"
```

**Done when:** `git status` in `docs/` is clean (or only unrelated local files remain).

### 2. Bump submodule pointer in Volt

From the Volt repo root:

```bash
git status
git diff docs
git add docs
git commit -m "$(cat <<'EOF'
docs: bump volt-docs submodule

EOF
)"
```

Skip the parent commit if the gitlink did not change or the user asked for submodule-only commit.

### 3. Report

Return to the parent agent:

- Docs commit SHA + subject
- Parent commit SHA + subject (if any)
- Whether push was skipped (default) or performed
- Remaining dirty paths, if any

## Push only the docs site

```bash
# inside docs/
git push origin HEAD
# then Volt root, if parent commit exists
git push origin HEAD
```

## Empty diff

If there is nothing to commit in `docs/`, report `no-docs-changes` and do not create an empty commit.
