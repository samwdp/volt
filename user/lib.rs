//! Compiled user extension library.
//!
//! This crate intentionally keeps feature packages as Rust modules living directly
//! under the `user/` directory so the future extension model matches the planned
//! 4coder-style workflow.

/// Debug adapter integration hooks and commands.
pub mod dap;
/// Git workflows and repository-oriented commands.
pub mod git;
/// Language-specific registrations.
pub mod lang;
/// Language server integration hooks and commands.
pub mod lsp;
/// Multiple cursor workflows.
pub mod multicursor;
/// Directory editing and navigation workflows.
pub mod oil;
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
        lsp::package(),
        dap::package(),
        vim::package(),
        multicursor::package(),
        picker::package(),
        treesitter::package(),
        workspace::package(),
        git::package(),
        oil::package(),
        terminal::package(),
        lang::rust::package(),
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
        assert_eq!(languages.len(), 1);
        assert_eq!(languages[0].id(), "rust");
        assert_eq!(languages[0].file_extensions(), ["rs"]);
    }

    #[test]
    fn user_library_exports_lsp_and_dap_defaults() {
        let servers = language_servers();
        let adapters = debug_adapters();

        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id(), "rust-analyzer");
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].id(), "codelldb");
    }

    #[test]
    fn user_library_exports_themes() {
        let themes = themes();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].id(), "volt-dark");
        assert!(themes[0].color("syntax.keyword").is_some());
        assert!(themes[0].color("ui.yank-flash").is_some());
    }
}
