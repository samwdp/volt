---
name: update-volt-docs
description: >-
  Sync the volt-docs submodule after Volt product changes: user packages,
  plugins, themes, commands, technologies, help pages, and changelog. Use when
  finishing feature work that affects docs, when AGENTS.md requires a docs
  subagent, or when the user asks to update documentation.
compatibility: Requires the Volt repo with the docs submodule checked out; npm in docs/.
---

# Update Volt Docs

Bring `docs/` (submodule `samwdp/volt-docs`) in sync with the Volt change just made. Run as a **fresh subagent** — keep docs work out of the main product thread.

## Inputs (from parent)

Require a brief from the parent agent:

- What changed (packages, themes, commands, crates, tech)
- Paths touched in Volt
- GitHub issue numbers to link (if any)
- Scope: `drift-only` (default) or `full`

## Steps

1. **Confirm docs checkout** — `docs/` must be the `volt-docs` git worktree. Abort if missing or not a separate git repo.
2. **Inventory drift** against Volt source (prefer evidence over memory):
   - Packages: `user/lib.rs` `packages()` vs `docs/docs/plugins/*.md` (skip `catalog.md`)
   - Themes: `user/themes/*.toml` (skip `global.toml`) vs `docs/docs/guides/builtin-themes.md`
   - Default theme: `DEFAULT_THEME_ID` in `user/theme.rs`
   - New tech / architecture surfaces worth a short note on home or getting-started
3. **Apply updates** (only what the brief + inventory need):
   - **New plugin** — follow [create-volt-docs](../../docs/.agents/skills/create-volt-docs/SKILL.md); wire catalog + sidebar
   - **Removed package** — delete or deprecate page; drop catalog/sidebar entries
   - **Command / keybinding / behavior drift** — edit existing pages from source; do not invent APIs
   - **Theme list drift** — update `docs/docs/guides/builtin-themes.md` and lists in `docs/docs/guides/change-theme.md`
   - **Changelog** — update or create `docs/docs/guides/changelog.md` from recent Volt commits; link issues (`#N` → `https://github.com/samwdp/volt/issues/N`); keep entries filter-friendly (date, type, short summary). Add sidebar/nav link if the page is new
4. **Match site style** — reuse existing Markdown / VitePress patterns; no drive-by redesign.
5. **Reindex** from `docs/`:
   ```bash
   npm run index
   ```
6. **Build** when pages, sidebar, or theme components changed:
   ```bash
   npm run build
   ```
7. **Hand off** — list files changed + one-line summary. Do **not** commit here; next skill is [commit-volt-docs](../commit-volt-docs/SKILL.md).

**Done when:** docs match the scoped Volt change, search index regenerated, build OK if structure changed, and a file list is ready for commit.

## Scope control

- Default: only drift tied to the brief.
- Full sync: all plugin pages + themes + user-package workflows — only when brief says `full`.
- Skip unrelated prose polish.

## Authority

Source of truth is the Volt tree (`user/*.rs`, themes, config), not old HTML or guessed APIs. For new topic authoring details see create-volt-docs; for pure inventory refresh see [refresh-volt-docs](../../docs/.agents/skills/refresh-volt-docs/SKILL.md).
