# Volt-owned Language Server and Debug Adapter Install

PATH-visible programs still win, but Volt no longer requires the user to install every Language Server and Debug Adapter themselves. Specs may declare a typed Install Recipe (npm, dotnet tool, Go, pip, cargo, or an archive). Missing programs auto-install via Command Stream, then start without reload by appending `…/volt/lsp/bin` and `…/volt/dap/bin` to this process PATH. Toolchains (Node, Go, .NET, …) are never installed; Specs without a recipe stay PATH-only.
