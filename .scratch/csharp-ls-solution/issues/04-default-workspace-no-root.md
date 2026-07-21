# Default Workspace and no-root csharp-ls

Type: grilling
Blocked by:

## Parent

[.scratch/csharp-ls-solution/map.md](../map.md)

## Question

When the active context is the Default Workspace (no project root) or otherwise has no Workspace root for candidate search, what should csharp-ls do — skip solution binding and keep nearest-`.csproj` / file-dir behavior, refuse start, or use another root — given destination assumes project Workspace root for discovery and memory key?
