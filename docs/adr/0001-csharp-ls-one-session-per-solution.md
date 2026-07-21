# csharp-ls one Language Server Session per Solution

csharp-ls Session roots used nearest `*.csproj`, so multi-project trees spawned one process per project. We prefer ordered root markers (`*.sln` before `*.csproj`), key the Session by the Solution directory, and pass `--solution <relative-name>` when that directory contains exactly one `.sln`. Auto `csharp.solutionPathOverride` for unique solutions is dropped; the CLI owns auto binding and the override API remains for manual multi-sln picks. Roslyn is out of scope.
