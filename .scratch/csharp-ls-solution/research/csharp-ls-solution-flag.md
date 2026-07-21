# csharp-ls `--solution` / `-s` and `csharp.solutionPathOverride`

Research date: 2026-07-21  
Upstream: [razzmatazz/csharp-language-server](https://github.com/razzmatazz/csharp-language-server)  
Source pin: `main` @ [`b714e27ca7608b1c77fa77caa5f0bb9330fdab49`](https://github.com/razzmatazz/csharp-language-server/commit/b714e27ca7608b1c77fa77caa5f0bb9330fdab49) (2026-07-15)  
NuGet package checked: [csharp-ls 0.26.0](https://www.nuget.org/packages/csharp-ls/0.26.0) (README on NuGet mirrors upstream README)

Every factual claim below cites a primary source. Gaps called out as **unknown**.

---

## 1. CLI flag names, help text, parsing

| Item | Fact | Source |
|------|------|--------|
| Long / short names | `--solution` / `-s` (Argu case `Solution`) | [`Program.fs` L20–L28](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Program.fs#L20-L28) |
| Help text | `"specify .sln file to load (relative to CWD)"` | [`Program.fs` L34](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Program.fs#L34); same text in [README Command Line Arguments](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/README.md) and [NuGet package README](https://www.nuget.org/packages/csharp-ls/0.26.0) |
| Usage line | `csharp-ls [--help] [--version] [--loglevel <level>] [--solution <solution>] ...` | [README](https://raw.githubusercontent.com/razzmatazz/csharp-language-server/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/README.md) |
| Parsing | Argu `ArgumentParser.Create` / `TryGetResult <@ Solution @>` | [`Program.fs` L99–L112](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Program.fs#L99-L112) |

Help text says `.sln` only. Code does **not** validate the extension at CLI parse time; it stores the path and later passes it to Roslyn `MSBuildWorkspace.OpenSolutionAsync`.

---

## 2. How the solution path is resolved (relative vs absolute, CWD)

### CLI path

1. CLI value is immediately run through `Path.GetFullPath`:
   ```fsharp
   let slnFullPath =
       serverArgs.TryGetResult <@ Solution @> |> Option.map Path.GetFullPath
   ```
   Source: [`Program.fs` L99–L100](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Program.fs#L99-L100).

2. That absolute path is stored as `CSharpConfiguration.solutionPathOverride` ([`Program.fs` L112](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Program.fs#L112)).

3. BCL `Path.GetFullPath`:
   - Relative paths resolve against the process current directory (CWD).
   - Absolute paths are accepted and normalized.
   - Official docs: [Path.GetFullPath(String)](https://learn.microsoft.com/en-us/dotnet/api/system.io.path.getfullpath).

**Implication for editors:** injecting `--solution` with a relative path couples to the **process CWD of the csharp-ls process**, not necessarily the LSP workspace folder URI. Prefer absolute paths if CWD may differ from the workspace root. Help text documents relative-to-CWD; absolute still works via `GetFullPath`.

### Load-time path (folder override / workspace config)

When a path is present on the folder override, load does:

```fsharp
| Some solutionPath ->
    let rootedSolutionPath =
        match Path.IsPathRooted solutionPath with
        | true -> solutionPath
        | false -> Path.Combine(dir, solutionPath)
    return! solutionTryLoadOnPath lspClient rootedSolutionPath
```

Source: [`Roslyn/Solution.fs` L387–L401](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L387-L401).  
`dir` is the workspace folder filesystem root ([`WorkspaceFolder.fs` L716–L730](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Lsp/WorkspaceFolder.fs#L716-L730)).

So:

| Origin | Relative meaning | Absolute |
|--------|------------------|----------|
| `--solution` / `-s` | vs process CWD (`GetFullPath` at startup) | OK |
| `csharp.solutionPathOverride` (if relative) | vs **workspace folder** root (`Path.Combine`) | OK (used as-is when rooted) |

CLI-supplied values are already absolute after startup, so the load-time `Path.Combine` branch is mainly for relative paths coming from workspace configuration.

`solutionTryLoadOnPath` asserts `Path.IsPathRooted` ([`Solution.fs` L241–L242](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L241-L242)).

---

## 3. Behavior when flag / override is **unset** (discovery)

Entry: `solutionLoadSolutionWithPathOrOnDir` with `None` → logs “attempting to find and load solution on path …” then `solutionFindAndLoadOnDir` ([`Solution.fs` L403–L410](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L403-L410)).

Discovery algorithm (`solutionFindAndLoadOnDir`, [`Solution.fs` L330–L385](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L330-L385)):

1. Recursively under workspace folder `dir` (`SearchOption.AllDirectories`):
   - `*.sln`
   - `*.slnx`
2. Filter out paths containing a `node_modules` path segment.
3. Log count + list of solutions found.
4. Pick one via `selectPreferredSolution`: parse each with `SolutionFile.Parse`, take the solution with the **highest** `ProjectsInOrder.Count` ([`Solution.fs` L224–L239](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L224-L239)). Changelog: “Apply simple heuristics to select a .sln/.slnx file when multiple are found” in **0.19.0** ([CHANGELOG](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/CHANGELOG.md); [PR #250](https://github.com/razzmatazz/csharp-language-server/pull/250)).
5. If no preferred solution: fall back to loading all `*.csproj` and `*.fsproj` under `dir` (same recursion + `node_modules` filter). If none, raise.

Automated coverage for `.slnx` discovery: `testSlnxSolutionFileWillBeFoundAndLoaded` ([`InitializationTests.fs`](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/tests/CSharpLanguageServer.Tests/InitializationTests.fs)).

---

## 4. Behavior when flag / override **is set**

When `solutionPathMaybe` is `Some`:

- **Does not** run `solutionFindAndLoadOnDir` (no recursive `*.sln`/`*.slnx` scan).
- Resolves to a rooted path (see §2).
- Calls `solutionTryLoadOnPath` only — single `OpenSolutionAsync` on that path ([`Solution.fs` L393–L401](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L393-L401)).

Folder field comment: “When set, the solution loader uses this path directly instead of auto-discovering a solution under Uri” ([`WorkspaceFolder.fs` L43–L45](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Lsp/WorkspaceFolder.fs#L43-L45)).

**Historical reports:** GitHub issues [#115](https://github.com/razzmatazz/csharp-language-server/issues/115) and [#257](https://github.com/razzmatazz/csharp-language-server/issues/257) describe `--solution` still scanning. Those are issue reports, not authoritative behavior. Against **current** `main` (`b714e27…`), the `Some` branch skips discovery. If an editor sees “attempting to find and load solution…”, that log line is **only** emitted on the `None` branch ([`Solution.fs` L403–L410](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L403-L410)) — meaning the override was not present on the folder at load time.

---

## 5. Relationship: `--solution` ↔ `csharp.solutionPathOverride`

### Documented equivalence

README:

> `csharp.solutionPathOverride` - override the solution path to load; useful for specifying an alternative solution when multiple exist in the workspace; **can also be set via the `--solution` CLI flag**; defaults to `null`

Sources: [README Settings](https://raw.githubusercontent.com/razzmatazz/csharp-language-server/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/README.md), [NuGet csharp-ls 0.26.0](https://www.nuget.org/packages/csharp-ls/0.26.0).

Settings are read from the `csharp` workspace configuration section (`workspace/configuration` / `workspace/didChangeConfiguration`) ([README](https://raw.githubusercontent.com/razzmatazz/csharp-language-server/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/README.md)).

Config type field: `solutionPathOverride: string option` ([`Types.fs` L14–L39](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Types.fs#L14-L39)).

Introduced / renamed into this shape in commit [`c957b73`](https://github.com/razzmatazz/csharp-language-server/commit/c957b73de08b7140b918844bf1968cfecc7a59e7) (“change how --solution works, add csharp.solutionPathOverride”), included in the **0.22.0 → 0.23.0** range ([compare](https://github.com/razzmatazz/csharp-language-server/compare/0.22.0...0.23.0); tag [0.23.0](https://github.com/razzmatazz/csharp-language-server/releases/tag/0.23.0) published 2026-04-08). CHANGELOG for 0.23.0 does **not** explicitly name this rename (gap in changelog prose; commit + README are authoritative).

### Merge / precedence (CLI vs client config)

`mergeCSharpConfiguration`:

```fsharp
solutionPathOverride =
    newConfig.solutionPathOverride
    |> Option.orElse oldConfig.solutionPathOverride
```

Source: [`Types.fs` L41–L59](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Types.fs#L41-L59).

Meaning:

| Client `solutionPathOverride` | Result |
|-------------------------------|--------|
| `Some path` | **Client path wins** over CLI |
| `None` / omitted | **CLI (or prior) value preserved** |

CLI seeds `oldConfig` before initialize ([`Program.fs` L99–L112](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Program.fs#L99-L112)). On `initialized`, server pulls `csharp` via `workspace/configuration` and merges ([`LifeCycle.fs` handleInitialized](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Handlers/LifeCycle.fs); pull helper [`Client.fs` `TryPullCSharpConfig`](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Lsp/Client.fs)).

Same merge on `workspace/didChangeConfiguration` ([`Handlers/Workspace.fs` L189–L226](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Handlers/Workspace.fs#L189-L226)).

**0.9.0 caveat (still relevant to merge design):** PR [#105](https://github.com/razzmatazz/csharp-language-server/pull/105) / CHANGELOG 0.9.0 — make `--solution` take effect even when the editor provides `csharp.` settings (avoid wiping CLI when client returns unset solution path). Current `Option.orElse` implements that preserve-CLI-when-absent behavior.

**Not “CLI always wins.”** If the client returns a non-null `solutionPathOverride`, that value overrides the CLI path.

### Stamping onto workspace folders

`workspaceSolutionPathOverride` applies `config.solutionPathOverride` to the **first** workspace folder only ([`Lsp/Workspace.fs` L121–L132](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Lsp/Workspace.fs#L121-L132)). Called when folders are configured / reconfigured ([`ServerStateLoop.fs` L270–L315, L492–L500](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Runtime/ServerStateLoop.fs#L270-L315)). Changing `solutionPathOverride` after load synthesizes a folder reconfiguration to tear down and reload ([`ServerStateLoop.fs` L270–L289](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Runtime/ServerStateLoop.fs#L270-L289)).

### VS Code settings schema (first-party?)

csharp-ls itself does **not** ship a VS Code `package.json` `contributes.configuration` schema. Authoritative setting name is the server README + `CSharpConfiguration` type.

The client listed in the csharp-ls README as [vscode-csharp-ls](https://github.com/vytautassurvila/vscode-csharp-ls) is a **separate** repo (not owned by razzmatazz). Its `package.json` does **not** declare `csharp.solutionPathOverride`; it injects `solutionPathOverride` via language-client `workspace.configuration` middleware under section `csharp`, typically as a path **relative to the workspace folder**, and does **not** pass `--solution` on the process argv ([`cSharpLsServer.ts`](https://raw.githubusercontent.com/vytautassurvila/vscode-csharp-ls/master/src/cSharpLsServer.ts)). Treat as a usage example of the LSP setting, not as csharp-ls’s schema.

---

## 6. `.slnx` support

| Aspect | Fact | Source |
|--------|------|--------|
| Discovery | Yes — `*.slnx` alongside `*.sln` | [`Solution.fs` L339–L341](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Roslyn/Solution.fs#L339-L341) |
| File watch reload | `.sln` and `.slnx` both trigger solution reload | [`Handlers/Workspace.fs` L149–L152](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Handlers/Workspace.fs#L149-L152); watch glob `**/*.{cs,cshtml,csproj,sln,slnx}` L44–L46 |
| CLI help | Still says “`.sln` file” only | [`Program.fs` L34](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Program.fs#L34) |
| Version added | **0.18.0** (2025-06-23): “Support loading slnx files” ([PR #226](https://github.com/razzmatazz/csharp-language-server/pull/226)) | [CHANGELOG 0.18.0](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/CHANGELOG.md) |

Passing `.slnx` via `--solution` / override is not specially branched; it is the same “open this path” path as `.sln`. Success depends on Roslyn/MSBuild accepting the file (same as discovery load).

`.slnf` support: **not** evidenced in current discovery patterns (`*.sln` / `*.slnx` only). Issue [#213](https://github.com/razzmatazz/csharp-language-server/issues/213) discusses slnf/slnx; it is not a substitute for reading the current loader code.

---

## 7. Multi-solution / multi-folder workspaces when override is set

1. **Single folder, override set:** only that one solution path is opened; sibling `.sln`/`.slnx` under the tree are **not** scanned (§4).

2. **Multi-folder workspace:** `workspaceSolutionPathOverride` stamps the override onto **`firstFolder` only**; remaining folders keep `SolutionPathOverride = None` and still run discovery independently ([`Lsp/Workspace.fs` L121–L132](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Lsp/Workspace.fs#L121-L132)). Multi-folder load without override is covered by `testMultiTargetWorkspace` ([`InitializationTests.fs`](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/tests/CSharpLanguageServer.Tests/InitializationTests.fs)).

3. There is **no** API for “list of solutions” or per-folder overrides via config — one global `csharp.solutionPathOverride` / one CLI `--solution`.

4. On-demand loading (from **0.23.0**, [PR #337](https://github.com/razzmatazz/csharp-language-server/pull/337)): solutions load when a folder is requested (e.g. document open → `LoadWorkspaceFolder`), not necessarily fully during `initialize`. Override must already be stamped on the folder before that load ([`ServerStateLoop.fs` `ProcessSolutionAwaiters` / `workspaceLoadingStarted`](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Runtime/ServerStateLoop.fs)).

---

## 8. Version caveats (NuGet / CHANGELOG)

| Version | Relevant change | Source |
|---------|-----------------|--------|
| **0.1.3** | `-s` / `--solution` added | [CHANGELOG](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/CHANGELOG.md) |
| **0.7.0** | LSP config `csharp.solution` (old name) | CHANGELOG 0.7.0 |
| **0.9.0** | `--solution` preserved when editor sends other `csharp.` settings ([PR #105](https://github.com/razzmatazz/csharp-language-server/pull/105)) | CHANGELOG 0.9.0 |
| **0.18.0** | `.slnx` loading/discovery ([PR #226](https://github.com/razzmatazz/csharp-language-server/pull/226)) | CHANGELOG 0.18.0; [release](https://github.com/razzmatazz/csharp-language-server/releases/tag/0.18.0) |
| **0.19.0** | Heuristic: prefer `.sln`/`.slnx` with most projects ([PR #250](https://github.com/razzmatazz/csharp-language-server/pull/250)) | CHANGELOG 0.19.0 |
| **0.23.0** (commits in 0.22→0.23) | `csharp.solutionPathOverride`; CLI value stored in config + stamped on first workspace folder ([`c957b73`](https://github.com/razzmatazz/csharp-language-server/commit/c957b73de08b7140b918844bf1968cfecc7a59e7), [`967df1d`](https://github.com/razzmatazz/csharp-language-server/commit/967df1d)); on-demand solution load | [compare 0.22.0...0.23.0](https://github.com/razzmatazz/csharp-language-server/compare/0.22.0...0.23.0); [0.23.0 release](https://github.com/razzmatazz/csharp-language-server/releases/tag/0.23.0) |
| **0.26.0** (current NuGet at research time) | Docs still document `--solution` + `csharp.solutionPathOverride` as above | [nuget.org/packages/csharp-ls/0.26.0](https://www.nuget.org/packages/csharp-ls/0.26.0) |

**Breaking / rename note:** Current `CSharpConfiguration` has only `solutionPathOverride`. There is **no** `solution` field in [`Types.fs`](https://github.com/razzmatazz/csharp-language-server/blob/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/src/CSharpLanguageServer/Types.fs). Editors sending legacy `csharp.solution` are **not** shown to bind that field in current source. Exact deserialization of unknown JSON properties: **unknown** without a dedicated test (likely ignored by the deserializer). Prefer `csharp.solutionPathOverride` for ≥0.23.0.

Runtime requirement (package/README at 0.26.0): **.NET 10 SDK or later** ([README](https://raw.githubusercontent.com/razzmatazz/csharp-language-server/b714e27ca7608b1c77fa77caa5f0bb9330fdab49/README.md)).

---

## Unknowns (checked, not established)

| Topic | What was checked | Status |
|-------|------------------|--------|
| Exact JSON casing / camelCase for `workspace/configuration` | Field name in F# is `solutionPathOverride`; README uses `csharp.solutionPathOverride`. Serializer options not re-audited end-to-end | Assume camelCase property under `csharp` object; not independently verified against a wire dump |
| Empty-string override (`""`) vs omit | Merge uses `option`; empty string would likely be `Some ""` if deserialized | **unknown** — no test found |
| Whether `csharp.solution` still accepted as alias | Only `solutionPathOverride` on `CSharpConfiguration` | **No alias in current source** |
| Pre-0.23 behavior of `--solution` vs workspace folder URI | Old Program built a synthetic folder from the sln directory ([`c957b73` diff](https://github.com/razzmatazz/csharp-language-server/commit/c957b73de08b7140b918844bf1968cfecc7a59e7)) | Different before 0.23; editors targeting old NuGet should not assume current stamping model |
| Issue #257 “still scans with --solution” on current 0.26 | Current `Some` branch skips scan; issue may be stale or override not stamped | Treat current source as authoritative |

---

## Implications for Volt

- Pass `--solution` / `-s` with an **absolute** path unless process CWD is guaranteed to be the workspace root (CLI uses `Path.GetFullPath` vs CWD).
- `--solution` and `csharp.solutionPathOverride` share one config field; they are documented equivalents, not independent knobs.
- Do **not** also send a conflicting non-null `csharp.solutionPathOverride` from `workspace/configuration` if CLI should win — client `Some` overrides CLI via `Option.orElse`.
- When override is set and stamped on the folder, csharp-ls loads **only that file** (no multi-sln scan for that folder).
- Override applies to the **first** workspace folder only; extra folders still auto-discover.
- Use setting name `csharp.solutionPathOverride` (not legacy `csharp.solution`) for csharp-ls ≥ 0.23.0.
- `.slnx` OK for discovery and load from **0.18.0+**; CLI help text still says `.sln`.
- Prefer csharp-ls **≥ 0.23.0** for current CLI→config→first-folder override model; **≥ 0.18.0** if `.slnx` required; **≥ 0.19.0** for multi-sln “most projects” heuristic when unset.
- If logs show “attempting to find and load solution…”, override did not reach the folder load path — check CWD, merge wipe, and first-folder stamping / on-demand load timing.
