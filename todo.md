## Fixes
- [ ] TODO: search the entire codbase and look speifically for performance problems. you should use /rust-skills to look for obvious code smells and perfomance bottlenecks
- [ ] TODO: Ctrl-n Ctrl-p should repeat when pressed
- [ ] TODO: investigate why typeing feels sluggish when a buffer with treesitter enabled, i feel like something is happenint on each key stroke and should probably be asyncronous. 
- [ ] TODO: Workspace wide search. Should be ripgrep and grep fallback this should have a debounce so that it is not really slow to type in
- [ ] TODO: fix buffer search. It is really slow. can we do this asyncronously.
- [ ] TODO: Picker - all pickers should be able to search fuzzy, also search if there is a space. for example, if i seacrch "acp mode" in the command picker this should return acp.pick-mode and acp.cyle-mode
- [ ] TODO: add a right pane/window padding of a few pixels as the curor goes right to the end when typing
- [ ] TODO: the vim.move-word-forward and vim.move-word-backward should ignore punctuation "'`.,<[{()}]>/\| etc
- [ ] TODO: the vim.move-big-word-end only goes 1 word forward. If i press this multiple times it should be constatly moving to the end of the next word
- [ ] TODO: acp client sessions should have their own buffers. If i select a session with acp.pick-session this should open a new buffer. I want a command acp.new-session that invokes the session/new acp command into a new buffer
- [ ] TODO: treat the ouput of the acp client as markdown. area of the buffer that the client puts the responses from the server should be like a markdown file, treesitter should be enabled for that area for some fancyer highlighting
## Features
- [ ] TODO: flexible auto complete. This should spawn a window of the top x results (configurable in user/autocomplete.rs). We should allow multiple things to register in the autocomplete. Lets start with a buffer autocomplete which shows results of tokens in the buffer. This should be fast and not block the main thread while getting and displaying the results. We should have a configurable key number that triggers the autocomplete window. When the window is active, we should still be able to type normally, but Ctrl-n, Ctrl-p should cycle the results in the window. Then Ctrl-y to accept the result which puts completes the text in the buffer. We should have an insert_mode command of Ctrl-SPACE to manually trigger the autocomplete window to show. We should have the ability to have the token that it is trying to complete, an icon for the completion provider. Looking forwards when we integrate lsp, we should be able to register this as a provider, and it should be able to submit its own results to the window. 
- [ ] TODO: add lsp integration, we should allow a buffer to start multiple lsp servers if it has been setup to do so. Im thinking markdown has an lsp and we could use harper for spelling and grammar. We should have diagnostics avalible to be shown in the buffer (configurable in user/lsp.rs). I want a hover toggle bound to "K" in the user/vim.rs layer. This should show the result
- [ ] TODO: add a flexible hover window bound to user/vim.rs "K" in normal mode. We should allow multiple providers to be registered as a hover provider. We will start by putting lsp doc hover as a provider, and lsp diagnostics as a hover provider. Allow Ctrl-n Ctrl-p to to cycle beween providers. The keybind should be configurable in user/hover.rs
- [ ] TODO: add slim browser buffer
- [ ] TODO: mouse support
- [ ] TODO: add icons to the git status buffer
