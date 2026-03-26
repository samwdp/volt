- TODO i need a close split command. this should close the the split the cursor is in. also the horizontal split 
 seems to not scroll correctly. its as if the full screen height is being put in for the buffers in these splits 
 and then as you scroll, you do not see the entire code
- TODO the lsp log buffer keeps coming into focus and the error log. these should be background buffers that never 
 come to the front of the buffer stack unless explicitly called to by the buffer.switch commnad. these are 
 "background" buffers
- TODO can i have a switch split command. this should shift the splits so the split on the left becomes the split on 
 the right
- TODO i have a but in the vertical split, in [📷 vsplit.png] you can see the right buffer has gone 14 lines below 
 the visible area, something has broken in the height calculation for the buffer
- TODO the hardcoded color at line 1049 in @crates\editor-sdl\src\shell\picker.rs needs to be added to the theme 
 files. this should be ui.picker.highlight and then pick an appropriate color for each theme
- TODO git status needs to have a visual selection action. I should be able to visually select lines and do an action on all of them. Imagine i want to unstage a set of changes. I want to Ctrl+v to and select a number of lines and press "u" to unstage those changes. This shoudl be the same for staging "s" and deleting "x"

# Features
- color string support as a rendered square of that color. should support hex color strings and css color strings