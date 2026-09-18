use crate::icon_font::symbols::{cod, md};
use editor_plugin_api::{
    BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, PluginAction, PluginBuffer,
    PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage, PluginVimMode,
    browser_hooks, buffer_kinds,
};

pub const BROWSER_KIND: &str = buffer_kinds::BROWSER;
pub const BUFFER_NAME: &str = "*browser*";
pub const URL_PROMPT: &str = "";
pub const URL_PLACEHOLDER: &str = "https://example.com";
pub const INPUT_HINT: &str = "Enter/Ctrl+Enter navigate · F12 devtools · click page to browse";

/// Public browser feature contract used by first-party and third-party code.
pub fn feature_spec() -> BrowserFeatureSpec {
    BrowserFeatureSpec {
        buffer_name: BUFFER_NAME.to_owned(),
        url_prompt: URL_PROMPT.to_owned(),
        url_placeholder: URL_PLACEHOLDER.to_owned(),
        input_hint: INPUT_HINT.to_owned(),
        help: ContextHelpSpec::new(
            "Browser",
            "Browser",
            vec![
                ContextHelpEntry::new(
                    "I",
                    "focus input",
                    "Focuses browser URL prompt and enters insert mode.",
                ),
                ContextHelpEntry::new(
                    "Enter",
                    "navigate",
                    "Navigates using current browser URL prompt text.",
                ),
                ContextHelpEntry::new(
                    "Ctrl+Enter",
                    "navigate",
                    "Navigates using current browser URL prompt text.",
                ),
            ],
        ),
    }
}

/// Returns the metadata for the browser buffer package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        "browser",
        true,
        "Embedded browser buffers with tabs, bookmarks, and dock.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "browser.open",
            "Always opens a new browser buffer in a split pane alongside the active buffer.",
            vec![PluginAction::emit_hook(browser_hooks::OPEN, None::<&str>)],
        ),
        PluginCommand::new(
            "browser.add-tab",
            "Creates a new tab in the browser buffer currently in view.",
            vec![PluginAction::emit_hook(
                browser_hooks::ADD_TAB,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.bookmarks",
            "Opens a picker of saved bookmarks. Uses add-tab when a browser is in view, otherwise open.",
            vec![PluginAction::emit_hook(
                browser_hooks::BOOKMARKS,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.bookmark-add",
            "Saves the current browser page URL as a named bookmark in the Volt data folder.",
            vec![PluginAction::emit_hook(
                browser_hooks::BOOKMARK_ADD,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.bookmark-rename",
            "Renames the selected bookmark in the bookmarks picker (Ctrl+r).",
            vec![PluginAction::emit_hook(
                browser_hooks::BOOKMARK_RENAME,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.tabs",
            "Opens a picker of tabs in the active browser buffer.",
            vec![PluginAction::emit_hook(browser_hooks::TABS, None::<&str>)],
        ),
        PluginCommand::new(
            "browser.dock",
            "Toggles the browser dock listing open browser buffers and tabs.",
            vec![PluginAction::emit_hook(browser_hooks::DOCK, None::<&str>)],
        ),
        PluginCommand::new(
            "browser.dock.previous",
            "Moves to the previous entry in the browser dock.",
            vec![PluginAction::emit_hook(
                browser_hooks::DOCK_PREVIOUS,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.dock.next",
            "Moves to the next entry in the browser dock.",
            vec![PluginAction::emit_hook(
                browser_hooks::DOCK_NEXT,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.dock.close",
            "Closes the selected browser dock buffer or tab.",
            vec![PluginAction::emit_hook(
                browser_hooks::DOCK_CLOSE,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.open-popup",
            "Opens the browser buffer in the popup window.",
            vec![
                PluginAction::open_buffer(BUFFER_NAME, BROWSER_KIND, Some("Browser")),
                PluginAction::emit_hook(browser_hooks::OPEN_POPUP, None::<&str>),
            ],
        ),
        PluginCommand::new(
            "browser.url",
            "Detects a URL in the current buffer and opens it in a split browser buffer.",
            vec![PluginAction::emit_hook(browser_hooks::URL, None::<&str>)],
        ),
        PluginCommand::new(
            "browser.open-buffer",
            "Opens the active file in a split browser buffer.",
            vec![PluginAction::emit_hook(
                browser_hooks::OPEN_BUFFER,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.focus-input",
            "Focuses the browser input section and enters insert mode.",
            vec![PluginAction::emit_hook(
                browser_hooks::FOCUS_INPUT,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "browser.navigate",
            "Navigates the browser using the current URL prompt text.",
            vec![PluginAction::emit_hook(browser_hooks::SUBMIT, None::<&str>)],
        ),
    ])
    .with_buffers(vec![
        PluginBuffer::new(BROWSER_KIND, Vec::<String>::new()).with_key_bindings(vec![
            PluginKeyBinding::new("I", "browser.focus-input", PluginKeymapScope::Workspace)
                .with_vim_mode(PluginVimMode::Normal),
            PluginKeyBinding::new("Enter", "browser.navigate", PluginKeymapScope::Workspace)
                .with_vim_mode(PluginVimMode::Insert),
            PluginKeyBinding::new(
                "Ctrl+Enter",
                "browser.navigate",
                PluginKeymapScope::Workspace,
            )
            .with_vim_mode(PluginVimMode::Insert),
        ]),
    ])
    .with_key_bindings(vec![
        PluginKeyBinding::new("j", "browser.dock.next", PluginKeymapScope::BrowserDock),
        PluginKeyBinding::new("k", "browser.dock.previous", PluginKeymapScope::BrowserDock),
        PluginKeyBinding::new(
            "Ctrl+d",
            "browser.dock.close",
            PluginKeymapScope::BrowserDock,
        ),
    ])
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
            format!("{} Use the URL prompt below and press Enter or Ctrl+Enter to navigate again.", cod::COD_DEBUG_START),
            format!(
                "{} Click the footer prompt area to return keyboard input to Volt's URL box.",
                cod::COD_OPEN_PREVIEW
            ),
        ],
        None => vec![
            format!("{} Browser buffer", cod::COD_BROWSER),
            format!(
                "{} Enter a URL in the prompt below and press Enter or Ctrl+Enter.",
                md::MD_LINK_VARIANT
            ),
            String::new(),
            "Once a page loads, click inside it to interact directly in the buffer body.".to_owned(),
            "Press F12 or Ctrl+Shift+I to open DevTools.".to_owned(),
            "Use browser.open/browser.url for split browsing or browser.open-popup for popup browsing."
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
                .any(|command| command.name() == "browser.add-tab")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.bookmarks")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.bookmark-rename")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.tabs")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.dock")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.dock.next")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.dock.previous")
        );
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.dock.close")
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
        assert!(
            package
                .commands()
                .iter()
                .any(|command| command.name() == "browser.open-buffer")
        );
    }

    #[test]
    fn package_binds_j_and_k_in_browser_dock_scope() {
        let package = package();
        for (chord, command) in [
            ("j", "browser.dock.next"),
            ("k", "browser.dock.previous"),
            ("Ctrl+d", "browser.dock.close"),
        ] {
            assert!(
                package.key_bindings().iter().any(|binding| {
                    binding.chord() == chord
                        && binding.command_name() == command
                        && binding.scope() == PluginKeymapScope::BrowserDock
                }),
                "missing binding for {chord} -> {command}"
            );
        }
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
            "current https://example.com · Enter/Ctrl+Enter navigate · F12 devtools · click page to browse"
        );
        assert_eq!(
            input_hint(None),
            "Enter/Ctrl+Enter navigate · F12 devtools · click page to browse"
        );
    }
}
