## graphify

This project has a graphify knowledge graph at graphify-out/.

Rules:
- Before answering architecture or codebase questions, read graphify-out/GRAPH_REPORT.md for god nodes and community structure
- If graphify-out/wiki/index.md exists, navigate it instead of reading raw files
- For cross-module "how does X relate to Y" questions, prefer `graphify query "<question>"`, `graphify path "<A>" "<B>"`, or `graphify explain "<concept>"` over grep — these traverse the graph's EXTRACTED + INFERRED edges instead of scanning files
- After modifying code files in this session, run `graphify update .` to keep the graph current (AST-only, no API cost)

## Agent skills

### Issue tracker

Issues live in GitHub Issues for `samwdp/volt` (via `gh`). See `docs/agents/issue-tracker.md`.

### Triage labels

Canonical role names used as-is (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `CONTEXT.md` + `docs/adr/`. See `docs/agents/domain.md`.

### Docs sync (subagent)

After product changes that affect user-facing docs, spawn a fresh subagent. Pass what changed, paths, and issue numbers. Subagent: (1) `.agents/skills/update-volt-docs/SKILL.md` (2) `.agents/skills/commit-volt-docs/SKILL.md`. Do not update or commit docs on the main thread. Authoring skills: `docs/.agents/skills/`.
