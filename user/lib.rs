//! Compiled user extension library.
//!
//! This crate intentionally keeps feature packages as Rust modules living directly
//! under the `user/` directory so the future extension model matches the planned
//! 4coder-style workflow.

/// Agent Client Protocol integrations.
pub mod acp;
/// Provider-backed autocomplete commands and configuration.
pub mod autocomplete;
/// Slim browser buffer groundwork.
pub mod browser;
/// Buffer management and save commands.
pub mod buffer;
/// Debug adapter integration hooks and commands.
pub mod dap;
/// Git workflows and repository-oriented commands.
pub mod git;
/// Git fringe configuration.
pub mod gitfringe;
/// Cursor-anchored hover commands and provider ordering.
pub mod hover;
/// Bundled icon-font symbols and metadata.
pub mod icon_font;
/// Bundled icon-font symbol modules.
#[path = "nerd_font_symbols/mod.rs"]
pub mod icon_font_symbols;
/// Interactive read-only buffer workflows.
pub mod interactive;
/// Language-specific registrations.
pub mod lang;
/// Language server integration hooks and commands.
pub mod lsp;
/// Multiple cursor workflows.
pub mod multicursor;
/// Directory editing and navigation workflows.
pub mod oil;
/// Pane layout management.
pub mod pane;
/// Generic picker UI bindings and popup controls.
pub mod picker;
/// User-editable statusline segment composition.
pub mod statusline;
/// Builtin terminal package surface.
pub mod terminal;
/// Code-defined themes compiled into the user library.
pub mod theme;
/// Tree-sitter installer and grammar management package.
pub mod treesitter;
/// Undo tree picker and history navigation.
pub mod undotree;
/// Vim-style bindings and motions.
pub mod vim;
/// Workspace creation and project discovery.
pub mod workspace;

use editor_dap::DebugAdapterSpec;
use editor_lsp::LanguageServerSpec;
use editor_plugin_api::PluginPackage;
use editor_syntax::LanguageConfiguration;
use editor_theme::Theme;

/// Returns the packages currently compiled into the user library.
pub fn packages() -> Vec<PluginPackage> {
    vec![
        buffer::package(),
        acp::package(),
        autocomplete::package(),
        browser::package(),
        interactive::package(),
        pane::package(),
        hover::package(),
        lsp::package(),
        dap::package(),
        oil::package(),
        multicursor::package(),
        picker::package(),
        treesitter::package(),
        undotree::package(),
        workspace::package(),
        git::package(),
        terminal::package(),
        vim::package(),
        lang::rust::package(),
        lang::markdown::package(),
    ]
}

/// Returns syntax languages currently compiled into the user library.
pub fn syntax_languages() -> Vec<LanguageConfiguration> {
    lang::syntax_languages()
}

/// Returns language-server specifications compiled into the user library.
pub fn language_servers() -> Vec<LanguageServerSpec> {
    lsp::language_servers()
}

/// Returns debug-adapter specifications compiled into the user library.
pub fn debug_adapters() -> Vec<DebugAdapterSpec> {
    dap::debug_adapters()
}

/// Returns themes compiled into the user library.
pub fn themes() -> Vec<Theme> {
    theme::themes()
}

#[cfg(test)]
mod tests {
    use super::{debug_adapters, language_servers, packages, syntax_languages, themes};
    use editor_syntax::LanguageConfiguration;

    fn mapped_theme_token<'a>(
        language: &'a LanguageConfiguration,
        capture: &str,
    ) -> Option<&'a str> {
        language
            .capture_mappings()
            .iter()
            .find(|mapping| mapping.capture_name() == capture)
            .map(|mapping| mapping.theme_token())
    }

    #[test]
    fn user_library_contains_auto_loaded_packages() {
        let packages = packages();
        assert!(packages.iter().any(|package| package.auto_load()));
        assert!(
            packages
                .iter()
                .any(|package| !package.commands().is_empty())
        );
        assert!(
            packages
                .iter()
                .any(|package| !package.key_bindings().is_empty())
        );
    }

    #[test]
    fn user_library_exports_language_registrations() {
        let languages = syntax_languages();
        assert!(languages.len() >= 4);
        let ids = languages
            .iter()
            .map(|language| language.id())
            .collect::<Vec<_>>();
        assert!(ids.contains(&"rust"));
        assert!(ids.contains(&"gitcommit"));
        assert!(ids.contains(&"markdown"));
        assert!(ids.contains(&"markdown-inline"));
        let rust = languages.iter().find(|language| language.id() == "rust");
        assert_eq!(
            rust.map(|language| {
                language
                    .file_extensions()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            }),
            Some(vec!["rs"])
        );
        let markdown = languages
            .iter()
            .find(|language| language.id() == "markdown");
        assert_eq!(
            markdown.map(|language| {
                language
                    .file_extensions()
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
            }),
            Some(vec!["md", "markdown"])
        );
    }

    #[test]
    fn user_library_exports_lsp_and_dap_defaults() {
        let servers = language_servers();
        let adapters = debug_adapters();

        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].id(), "rust-analyzer");
        assert_eq!(servers[1].id(), "marksman");
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].id(), "codelldb");
    }

    #[test]
    fn user_library_exports_themes() {
        let themes = themes();
        let ids = themes.iter().map(|theme| theme.id()).collect::<Vec<_>>();
        assert_eq!(themes.len(), 6);
        assert!(ids.contains(&"volt-dark"));
        assert!(ids.contains(&"volt-light"));
        assert!(ids.contains(&"gruvbox-dark"));
        assert!(ids.contains(&"gruvbox-light"));
        assert!(ids.contains(&"vscode-dark"));
        assert!(ids.contains(&"vscode-light"));
        assert!(
            themes
                .iter()
                .any(|theme| theme.color("syntax.keyword").is_some())
        );
        assert!(
            themes
                .iter()
                .any(|theme| theme.color("ui.yank-flash").is_some())
        );
    }

    #[test]
    fn user_library_themes_cover_core_editor_ui_tokens() {
        let themes = themes();
        const TOKENS: &[&str] = &["ui.cursor", "ui.selection", "ui.yank-flash"];
        for theme in themes {
            for token in TOKENS {
                assert!(
                    theme.color(token).is_some(),
                    "theme `{}` is missing `{token}`",
                    theme.id()
                );
            }
        }
    }

    #[test]
    fn user_library_themes_cover_extended_capture_families() {
        let themes = themes();
        const TOKENS: &[&str] = &[
            "syntax.none",
            "syntax.preproc",
            "syntax.string.escape",
            "syntax.number",
            "syntax.method",
            "syntax.parameter",
            "syntax.keyword.directive",
            "syntax.markup.heading",
            "syntax.markup.list.checked",
            "syntax.comment.error",
            "syntax.diff.plus",
            "syntax.text.title",
            "syntax.text.diff.add",
            "syntax.tag.attribute",
            "syntax.punctuation.special",
            "syntax.lsp.type.function",
        ];
        for theme in themes {
            for token in TOKENS {
                assert!(
                    theme.color(token).is_some(),
                    "theme `{}` is missing `{token}`",
                    theme.id()
                );
            }
        }
    }

    #[test]
    fn rich_markdown_and_gitcommit_captures_preserve_exact_theme_tokens() {
        let languages = syntax_languages();
        let markdown = languages
            .iter()
            .find(|language| language.id() == "markdown")
            .expect("markdown language missing");
        let markdown_inline = languages
            .iter()
            .find(|language| language.id() == "markdown-inline")
            .expect("markdown-inline language missing");
        let gitcommit = languages
            .iter()
            .find(|language| language.id() == "gitcommit")
            .expect("gitcommit language missing");

        assert_eq!(
            mapped_theme_token(markdown, "text.title"),
            Some("syntax.text.title")
        );
        assert_eq!(
            mapped_theme_token(markdown, "punctuation.special"),
            Some("syntax.punctuation.special")
        );
        assert_eq!(
            mapped_theme_token(markdown, "string.escape"),
            Some("syntax.string.escape")
        );
        assert_eq!(
            mapped_theme_token(markdown_inline, "text.emphasis"),
            Some("syntax.text.emphasis")
        );
        assert_eq!(
            mapped_theme_token(markdown_inline, "text.strong"),
            Some("syntax.text.strong")
        );
        assert_eq!(
            mapped_theme_token(gitcommit, "markup.heading"),
            Some("syntax.markup.heading")
        );
        assert_eq!(
            mapped_theme_token(gitcommit, "markup.link"),
            Some("syntax.markup.link")
        );
        assert_eq!(
            mapped_theme_token(gitcommit, "comment.error"),
            Some("syntax.comment.error")
        );
        assert_eq!(
            mapped_theme_token(gitcommit, "variable.parameter"),
            Some("syntax.variable.parameter")
        );
    }
}
