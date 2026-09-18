/// [`InputPromptOverlay`] id used for the `help` command prompt.
const HELP_PROMPT_ID: &str = "help";

/// Open an [`InputPromptOverlay`] asking for a docs search question.
fn open_help_prompt(runtime: &mut EditorRuntime) -> Result<(), String> {
    let overlay = InputPromptOverlay::new(HELP_PROMPT_ID, "Help: ", "");
    shell_ui_mut(runtime)?.open_input_prompt(overlay);
    Ok(())
}

/// Build the docs search URL and open it in a browser split.
fn confirm_help_question(runtime: &mut EditorRuntime, text: &str) -> Result<(), String> {
    let question = text.trim();
    if question.is_empty() {
        return Ok(());
    }
    let url = help_hooks::docs_search_url(question);
    open_browser_buffer_in_split(runtime, Some(&url))
}
