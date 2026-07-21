# Research csharp-ls --solution semantics

Type: research
Status: resolved
Blocked by:

## Parent

[.scratch/csharp-ls-solution/map.md](../map.md)

## Question

What are csharp-ls's authoritative rules for `--solution` / `-s` and `csharp.solutionPathOverride` (path relative vs absolute, CWD coupling, `.slnx` support, multi-solution discovery when the flag is set, and any version caveats) from upstream docs/source — facts Volt must match when injecting `--solution` at launch?

## Answer

Findings: [research/csharp-ls-solution-flag.md](../research/csharp-ls-solution-flag.md) (upstream mirrors under `research/upstream/`, main @ `b714e27ca760` / NuGet 0.26.0).

Gist for implementers:

- `--solution`/`-s` → `Path.GetFullPath` at launch → `solutionPathOverride` (**≥ 0.23.0**). Relative means **process CWD**, not workspace folder. Absolute OK.
- When override set: load that path only (no recursive scan). Relative **config** values combine with workspace-folder dir; CLI is already absolute on ≥ 0.23.
- Setting renamed `csharp.solution` → `csharp.solutionPathOverride` in **0.23.0**. Stamped on **first** LSP workspace folder. Merge: client `Some` wins, client `null` keeps CLI.
- Discovery without override: recursive `*.sln`+`*.slnx`, skip `node_modules`, prefer most projects; else all csproj/fsproj.
- `.slnx` since 0.18.0. Practical Volt floor **≥ 0.23.0**; tool needs .NET 10; NuGet latest 0.26.0.
- Map’s plan (cwd = solution dir + relative `--solution`; keep settings override null / in-memory pick as Volt truth) matches upstream.

## Comments

- Claimed and resolved in wayfinder work-through session (2026-07-21).
