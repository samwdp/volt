use crate::icon_font::symbols::{cod, md};
use editor_plugin_api::{
    PluginAction, PluginBuffer, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
    PluginVimMode,
};

pub const BROWSER_KIND: &str = "browser";
pub const BUFFER_NAME: &str = "*browser*";
pub const HOOK_BROWSER_URL: &str = "ui.browser.url";
pub const URL_PROMPT: &str = "URL > ";
pub const URL_PLACEHOLDER: &str = "https://example.com";
pub const INPUT_HINT: &str = "Ctrl+Enter navigate · F12 devtools · click page to browse";

/// Returns the metadata for the browser buffer package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "browser",
        true,
        "Embedded browser buffers and popup browsing.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "browser.open",
            "Opens the browser buffer in the active pane.",
            vec![PluginAction::open_buffer(
                BUFFER_NAME,
                BROWSER_KIND,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.open-popup",
            "Opens the browser buffer in the popup window.",
            vec![PluginAction::open_buffer(
                BUFFER_NAME,
                BROWSER_KIND,
                Some("Browser"),
            )],
        ),
        PluginCommand::new(
            "browser.url",
            "Detects a URL in the current buffer and opens it in the popup browser.",
            vec![PluginAction::emit_hook(HOOK_BROWSER_URL, None::<&str>)],
        ),
        PluginCommand::new(
            "browser.focus-input",
            "Focuses the browser input section and enters insert mode.",
            vec![PluginAction::emit_hook("ui.browser.focus-input", None::<&str>)],
        ),
    ])
    .with_buffers(vec![PluginBuffer::new(BROWSER_KIND, Vec::<String>::new()).with_key_bindings(vec![
        PluginKeyBinding::new("I", "browser.focus-input", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
    ])])
}

/// Returns the lines rendered into the current browser buffer state.
pub fn buffer_lines(url: Option<&str>) -> Vec<String> {
    match url {
        Some(url) => vec![
            format!("{} Browser buffer", cod::COD_BROWSER),
            format!("{} Current URL: {url}", md::MD_WEB),
            String::new(),
            "Click inside the page viewport to interact with the embedded browser.".to_owned(),
            "Press F12 or Ctrl+Shift+I to open DevTools.".to_owned(),
            format!(
                "{} Use the URL prompt below and press Ctrl+Enter to navigate again.",
                cod::COD_DEBUG_START
            ),
            format!(
                "{} Click the footer prompt area to return keyboard input to Volt's URL box.",
                cod::COD_OPEN_PREVIEW
            ),
        ],
        None => vec![
            format!("{} Browser buffer", cod::COD_BROWSER),
            format!(
                "{} Enter a URL in the prompt below and press Ctrl+Enter.",
                md::MD_LINK_VARIANT
            ),
            String::new(),
            "Once a page loads, click inside it to interact directly in the buffer body.".to_owned(),
            "Press F12 or Ctrl+Shift+I to open DevTools.".to_owned(),
            "Use browser.open for a full buffer or browser.open-popup/browser.url for popup browsing."
                .to_owned(),
        ],
    }
}

/// Returns the current input hint for browser buffers.
pub fn input_hint(url: Option<&str>) -> String {
    match url {
        Some(url) => format!("current {url} · {INPUT_HINT}"),
        None => INPUT_HINT.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_exports_browser_open_command() {
        let package = package();
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.open")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.open-popup")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.url")
        );
    }

    #[test]
    fn buffer_lines_include_current_url_when_present() {
        let lines = buffer_lines(Some("https://example.com"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("https://example.com"))
        );
        assert!(lines.iter().any(|line| line.contains("Browser buffer")));
        assert!(lines.iter().any(|line| line.contains("DevTools")));
    }

    #[test]
    fn input_hint_includes_current_url_when_present() {
        assert_eq!(
            input_hint(Some("https://example.com")),
            "current https://example.com · Ctrl+Enter navigate · F12 devtools · click page to browse"
        );
        assert_eq!(
            input_hint(None),
            "Ctrl+Enter navigate · F12 devtools · click page to browse"
        );
    }
}
