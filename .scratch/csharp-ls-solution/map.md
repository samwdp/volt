# csharp-ls solution binding

Labels: `wayfinder:map`

## Destination

Ship an in-repo fix so csharp-ls starts **one session per chosen `.sln`** (launch with `--solution`), not one process per nearest `.csproj`. Multi-`.sln` Workspaces get a picker; choice lives in session memory only. Map done when the implementation path is clear enough to build (or already decided and fog cleared).

## Notes

- Domain: Volt LSP session planning (`editor-lsp`) + SDL/user start path + picker; consult `CONTEXT.md` (Workspace), `/grilling`, `/domain-modeling` as needed.
- Standing preferences (chart grill):
  - Planned root = directory of the chosen `.sln` when one is selected; else nearest `.csproj` root, no `--solution`.
  - Candidates = every `.sln` under the **project Workspace root** (recursive). 0 → csproj fallback; 1 → auto; 2+ → picker before first start.
  - Cancel picker → do not start csharp-ls; no remembered choice; next start re-prompts.
  - Remember choice in **session memory only**, keyed by project Workspace root.
  - Tell server via **`--solution` on launch** (cwd = solution dir → relative name preferred); keep in-memory override as the remembered pick / one source of truth. No general dynamic-args ABI.
  - Command `lsp.csharp-pick-solution` always opens picker; submit stores choice and restarts csharp-ls for that Workspace.
- Skills: wayfinder, grilling, domain-modeling; research for csharp-ls upstream facts.

## Decisions so far

- [Research csharp-ls --solution semantics](issues/01-research-csharp-ls-solution-flag.md) — CLI `--solution`/`-s` → `Path.GetFullPath` (CWD); setting `csharp.solutionPathOverride` since 0.23.0; override = load that path only; null settings keep CLI; `.slnx` since 0.18.0; Volt floor ≥ 0.23. Details: [research/csharp-ls-solution-flag.md](research/csharp-ls-solution-flag.md).

## Not yet specified

- Exact picker provider id / package wiring once architecture seam locked.
- Session restart mechanics after re-pick (shutdown old SessionKey, start new).
- Test fixture shape for multi-sln Workspace.

## Out of scope

- Roslyn language server (`roslyn-language-server`) — same marker pain may return later; not this effort.
- Disk persistence of solution choice across editor restarts.
