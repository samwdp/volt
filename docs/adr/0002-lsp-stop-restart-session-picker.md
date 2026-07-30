# lsp.stop / lsp.restart pick a Language Server Session

`lsp.stop` and `lsp.restart` used to act on the active buffer’s attachments. They now open a picker of live Language Server Sessions for the active Workspace, and the chosen Session is stopped (or stopped then restarted) for the whole app.

Membership is hybrid: a Session is listed if it serves an open buffer in the active Workspace, or—when the active Workspace is a Project Workspace—if its root equals or lies under that Workspace root. Default Workspace listing is buffer-served only. Empty scope fails with a message instead of an empty picker. One match still opens the picker so the user confirms before killing a Session.
