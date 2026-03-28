//! Compiled user extension library.
//!
//! This crate intentionally keeps feature packages as Rust modules living directly
//! under the `user/` directory so the future extension model matches the planned
//! 4coder-style workflow.
//!
//! # Distribution model
//!
//! The user library is compiled as both a `cdylib` (for runtime loading by
//! `volt.exe`) and an `rlib` (for linking during development).  The public
//! [`UserLibraryImpl`] struct implements the [`editor_plugin_host::UserLibrary`]
//! trait so the host can call into the user library without direct source-level
//! coupling.

/// Agent Client Protocol integrations.
pub mod acp;
/// Provider-backed autocomplete commands and configuration.
pub mod autocomplete;
/// Slim browser buffer groundwork.
pub mod browser;
/// Buffer management and save commands.
pub mod buffer;
/// Expression evaluator buffer plugin.
pub mod calculator;
/// Workspace build/compile commands.
pub mod compile;
/// Debug adapter integration hooks and commands.
pub mod dap;
/// Git workflows and repository-oriented commands.
pub mod git;
/// Git fringe configuration.
pub mod gitfringe;
/// Cursor-anchored hover commands and provider ordering.
pub mod hover;
/// Bundled icon-font symbols and metadata (backed by editor-icons).
pub mod icon_font;
/// Bundled icon-font symbol modules (re-exported from editor-icons).
pub use editor_icons::symbols as icon_font_symbols;
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
    let mut pkgs = vec![
        buffer::package(),
        acp::package(),
        autocomplete::package(),
        browser::package(),
        calculator::package(),
        compile::package(),
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
    ];
    // Language packages are managed entirely inside user/lang/mod.rs so that
    // adding a new language only requires changes in that one file.
    pkgs.extend(lang::packages());
    pkgs
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

// ─── UserLibrary trait implementation ────────────────────────────────────────

/// Concrete implementation of [`editor_plugin_host::UserLibrary`] backed by
/// the modules compiled into this crate.
///
/// # Static vs. dynamic loading
///
/// During development, create an instance of this struct and pass it to
/// `ShellConfig` (via `Box::new(UserLibraryImpl)` or `Arc::new`).
///
/// For distribution, the same functions are also exported as C-ABI symbols
/// (see the bottom of this file) so that `volt.exe` can load `user.dll` /
/// `libuser.so` dynamically at runtime without recompiling the editor binary.
#[derive(Debug, Clone, Copy)]
pub struct UserLibraryImpl;

use editor_plugin_api::{
    AcpClient, AutocompleteProvider, GitStatusPrefix, HoverProvider, OilDefaults, OilKeyAction,
    OilKeybindings, TerminalConfig, WorkspaceRoot,
};
use editor_plugin_host::{StatuslineContext, UserLibrary};

impl UserLibrary for UserLibraryImpl {
    fn packages(&self) -> Vec<PluginPackage> {
        packages()
    }

    fn themes(&self) -> Vec<Theme> {
        themes()
    }

    fn syntax_languages(&self) -> Vec<LanguageConfiguration> {
        syntax_languages()
    }

    fn language_servers(&self) -> Vec<LanguageServerSpec> {
        language_servers()
    }

    fn debug_adapters(&self) -> Vec<DebugAdapterSpec> {
        debug_adapters()
    }

    fn autocomplete_providers(&self) -> Vec<AutocompleteProvider> {
        autocomplete::providers()
            .into_iter()
            .map(|p| AutocompleteProvider {
                id: p.id,
                label: p.label,
                icon: p.icon,
                item_icon: p.item_icon,
                or_group: p.or_group,
            })
            .collect()
    }

    fn autocomplete_result_limit(&self) -> usize {
        autocomplete::RESULT_LIMIT
    }

    fn autocomplete_token_icon(&self) -> &'static str {
        autocomplete::TOKEN_ICON
    }

    fn hover_providers(&self) -> Vec<HoverProvider> {
        hover::providers()
            .into_iter()
            .map(|p| HoverProvider {
                id: p.id,
                label: p.label,
                icon: p.icon,
                line_limit: hover::LINE_LIMIT,
            })
            .collect()
    }

    fn hover_line_limit(&self) -> usize {
        hover::LINE_LIMIT
    }

    fn hover_token_icon(&self) -> &'static str {
        hover::TOKEN_ICON
    }

    fn hover_signature_icon(&self) -> &'static str {
        hover::SIGNATURE_ICON
    }

    fn acp_clients(&self) -> Vec<AcpClient> {
        acp::clients()
            .into_iter()
            .map(|c| AcpClient {
                id: c.id,
                label: c.label,
                command: c.command,
                args: c.args,
                env: c.env,
                cwd: c.cwd,
            })
            .collect()
    }

    fn acp_client_by_id(&self, id: &str) -> Option<AcpClient> {
        acp::client_by_id(id).map(|c| AcpClient {
            id: c.id,
            label: c.label,
            command: c.command,
            args: c.args,
            env: c.env,
            cwd: c.cwd,
        })
    }

    fn workspace_roots(&self) -> Vec<WorkspaceRoot> {
        workspace::project_search_roots()
            .into_iter()
            .map(|r| WorkspaceRoot {
                path: r.root().display().to_string(),
                max_depth: r.max_depth(),
            })
            .collect()
    }

    fn terminal_config(&self) -> TerminalConfig {
        TerminalConfig {
            program: terminal::default_shell_program(),
            args: terminal::default_shell_args(),
        }
    }

    fn oil_defaults(&self) -> OilDefaults {
        let d = oil::defaults();
        OilDefaults {
            show_hidden: d.show_hidden,
            sort_mode: match d.sort_mode {
                oil::OilSortMode::TypeThenName => editor_plugin_api::OilSortMode::TypeThenName,
                oil::OilSortMode::TypeThenNameDesc => {
                    editor_plugin_api::OilSortMode::TypeThenNameDesc
                }
            },
            trash_enabled: d.trash_enabled,
        }
    }

    fn oil_keybindings(&self) -> OilKeybindings {
        let k = oil::keybindings();
        OilKeybindings {
            open_entry: k.open_entry,
            open_vertical_split: k.open_vertical_split,
            open_horizontal_split: k.open_horizontal_split,
            open_new_pane: k.open_new_pane,
            preview_entry: k.preview_entry,
            refresh: k.refresh,
            close: k.close,
            prefix: k.prefix,
            open_parent: k.open_parent,
            open_workspace_root: k.open_workspace_root,
            set_root: k.set_root,
            show_help: k.show_help,
            cycle_sort: k.cycle_sort,
            toggle_hidden: k.toggle_hidden,
            toggle_trash: k.toggle_trash,
            open_external: k.open_external,
            set_tab_local_root: k.set_tab_local_root,
        }
    }

    fn oil_keydown_action(&self, chord: &str) -> Option<OilKeyAction> {
        oil::keydown_action(chord).map(map_oil_key_action)
    }

    fn oil_chord_action(&self, had_prefix: bool, chord: &str) -> Option<OilKeyAction> {
        oil::chord_action(had_prefix, chord).map(map_oil_key_action)
    }

    fn oil_help_lines(&self) -> Vec<String> {
        oil::help_lines()
    }

    fn oil_directory_sections(
        &self,
        root: &std::path::Path,
        entries: &[editor_fs::DirectoryEntry],
        show_hidden: bool,
        sort_mode: editor_plugin_api::OilSortMode,
        trash_enabled: bool,
    ) -> editor_core::SectionTree {
        let user_sort = match sort_mode {
            editor_plugin_api::OilSortMode::TypeThenName => oil::OilSortMode::TypeThenName,
            editor_plugin_api::OilSortMode::TypeThenNameDesc => {
                oil::OilSortMode::TypeThenNameDesc
            }
        };
        oil::directory_sections(root, entries, show_hidden, user_sort, trash_enabled)
    }

    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        oil::strip_entry_icon_prefix(label)
    }

    fn git_status_sections(&self, snapshot: &editor_git::GitStatusSnapshot) -> editor_core::SectionTree {
        git::status_sections(snapshot)
    }

    fn git_commit_template(&self) -> Vec<String> {
        git::commit_buffer_template()
    }

    fn git_prefix_for_chord(&self, chord: &str) -> Option<GitStatusPrefix> {
        git::status_prefix_for_chord(chord).map(|p| match p {
            git::GitStatusPrefix::Commit => GitStatusPrefix::Commit,
            git::GitStatusPrefix::Push => GitStatusPrefix::Push,
            git::GitStatusPrefix::Fetch => GitStatusPrefix::Fetch,
            git::GitStatusPrefix::Pull => GitStatusPrefix::Pull,
            git::GitStatusPrefix::Branch => GitStatusPrefix::Branch,
            git::GitStatusPrefix::Diff => GitStatusPrefix::Diff,
            git::GitStatusPrefix::Log => GitStatusPrefix::Log,
            git::GitStatusPrefix::Stash => GitStatusPrefix::Stash,
            git::GitStatusPrefix::Merge => GitStatusPrefix::Merge,
            git::GitStatusPrefix::Rebase => GitStatusPrefix::Rebase,
            git::GitStatusPrefix::CherryPick => GitStatusPrefix::CherryPick,
            git::GitStatusPrefix::Revert => GitStatusPrefix::Revert,
            git::GitStatusPrefix::Reset => GitStatusPrefix::Reset,
        })
    }

    fn git_command_for_chord(
        &self,
        prefix: Option<GitStatusPrefix>,
        chord: &str,
    ) -> Option<&'static str> {
        let user_prefix = prefix.map(|p| match p {
            GitStatusPrefix::Commit => git::GitStatusPrefix::Commit,
            GitStatusPrefix::Push => git::GitStatusPrefix::Push,
            GitStatusPrefix::Fetch => git::GitStatusPrefix::Fetch,
            GitStatusPrefix::Pull => git::GitStatusPrefix::Pull,
            GitStatusPrefix::Branch => git::GitStatusPrefix::Branch,
            GitStatusPrefix::Diff => git::GitStatusPrefix::Diff,
            GitStatusPrefix::Log => git::GitStatusPrefix::Log,
            GitStatusPrefix::Stash => git::GitStatusPrefix::Stash,
            GitStatusPrefix::Merge => git::GitStatusPrefix::Merge,
            GitStatusPrefix::Rebase => git::GitStatusPrefix::Rebase,
            GitStatusPrefix::CherryPick => git::GitStatusPrefix::CherryPick,
            GitStatusPrefix::Revert => git::GitStatusPrefix::Revert,
            GitStatusPrefix::Reset => git::GitStatusPrefix::Reset,
        });
        git::status_command_name(user_prefix, chord)
    }

    fn browser_buffer_lines(&self, url: Option<&str>) -> Vec<String> {
        browser::buffer_lines(url)
    }

    fn browser_input_hint(&self, url: Option<&str>) -> String {
        browser::input_hint(url)
    }

    fn browser_url_prompt(&self) -> String {
        browser::URL_PROMPT.to_owned()
    }

    fn browser_url_placeholder(&self) -> String {
        browser::URL_PLACEHOLDER.to_owned()
    }

    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String {
        statusline::compose(&statusline::StatuslineContext {
            vim_mode: context.vim_mode,
            recording_macro: context.recording_macro,
            workspace_name: context.workspace_name,
            buffer_name: context.buffer_name,
            buffer_modified: context.buffer_modified,
            language_id: context.language_id,
            line: context.line,
            column: context.column,
            lsp_server: context.lsp_server,
            lsp_diagnostics: context.lsp_diagnostics.map(|d| statusline::LspDiagnosticsInfo {
                errors: d.errors,
                warnings: d.warnings,
            }),
            acp_connected: context.acp_connected,
            git: context.git_branch.map(|branch| statusline::GitStatuslineInfo {
                branch,
                added: context.git_added,
                removed: context.git_removed,
            }),
        })
    }

    fn statusline_lsp_connected_icon(&self) -> &'static str {
        statusline::LSP_CONNECTED_ICON
    }

    fn statusline_lsp_error_icon(&self) -> &'static str {
        statusline::LSP_ERROR_ICON
    }

    fn statusline_lsp_warning_icon(&self) -> &'static str {
        statusline::LSP_WARNING_ICON
    }

    fn lsp_diagnostic_icon(&self) -> &'static str {
        lsp::DIAGNOSTIC_ICON
    }

    fn lsp_diagnostic_line_limit(&self) -> usize {
        lsp::DIAGNOSTIC_LINE_LIMIT
    }

    fn lsp_show_buffer_diagnostics(&self) -> bool {
        lsp::SHOW_BUFFER_DIAGNOSTICS
    }

    fn gitfringe_token_added(&self) -> &'static str {
        gitfringe::TOKEN_ADDED
    }

    fn gitfringe_token_modified(&self) -> &'static str {
        gitfringe::TOKEN_MODIFIED
    }

    fn gitfringe_token_removed(&self) -> &'static str {
        gitfringe::TOKEN_REMOVED
    }

    fn gitfringe_symbol(&self) -> &'static str {
        gitfringe::SYMBOL
    }

    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol] {
        editor_icons::all_symbols()
    }

    fn supports_plugin_evaluate(&self, kind: &str) -> bool {
        matches!(kind, calculator::CALCULATOR_KIND)
    }

    fn handle_plugin_evaluate(&self, kind: &str, input: &str) -> Vec<String> {
        match kind {
            calculator::CALCULATOR_KIND => calculator::evaluate(input),
            _ => vec![format!("no evaluator registered for plugin kind `{kind}`")],
        }
    }

    fn plugin_buffer_initial_lines(&self, kind: &str) -> Vec<String> {
        match kind {
            calculator::CALCULATOR_KIND => calculator::initial_buffer_lines(),
            _ => Vec::new(),
        }
    }

    fn default_build_command(&self, language: &str) -> Option<String> {
        compile::default_build_command(language).map(str::to_owned)
    }
}

fn map_oil_key_action(action: oil::OilKeyAction) -> OilKeyAction {
    match action {
        oil::OilKeyAction::OpenEntry => OilKeyAction::OpenEntry,
        oil::OilKeyAction::OpenVerticalSplit => OilKeyAction::OpenVerticalSplit,
        oil::OilKeyAction::OpenHorizontalSplit => OilKeyAction::OpenHorizontalSplit,
        oil::OilKeyAction::OpenNewPane => OilKeyAction::OpenNewPane,
        oil::OilKeyAction::PreviewEntry => OilKeyAction::PreviewEntry,
        oil::OilKeyAction::Refresh => OilKeyAction::Refresh,
        oil::OilKeyAction::Close => OilKeyAction::Close,
        oil::OilKeyAction::StartPrefix => OilKeyAction::StartPrefix,
        oil::OilKeyAction::OpenParent => OilKeyAction::OpenParent,
        oil::OilKeyAction::OpenWorkspaceRoot => OilKeyAction::OpenWorkspaceRoot,
        oil::OilKeyAction::SetRoot => OilKeyAction::SetRoot,
        oil::OilKeyAction::ShowHelp => OilKeyAction::ShowHelp,
        oil::OilKeyAction::CycleSort => OilKeyAction::CycleSort,
        oil::OilKeyAction::ToggleHidden => OilKeyAction::ToggleHidden,
        oil::OilKeyAction::ToggleTrash => OilKeyAction::ToggleTrash,
        oil::OilKeyAction::OpenExternal => OilKeyAction::OpenExternal,
        oil::OilKeyAction::SetTabLocalRoot => OilKeyAction::SetTabLocalRoot,
    }
}


#[cfg(test)]
mod tests {
    use super::{debug_adapters, language_servers, packages, syntax_languages, themes};
    use crate::lsp::{
        SERVER_CSHARP_LS, SERVER_MARKSMAN, SERVER_RUST_ANALYZER, SERVER_TOMBI,
        SERVER_TYPESCRIPT_LANGUAGE_SERVER, SERVER_VSCODE_JSON_LANGUAGE_SERVER,
        SERVER_YAML_LANGUAGE_SERVER,
    };
    use editor_buffer::TextBuffer;
    use editor_syntax::{LanguageConfiguration, SyntaxRegistry};

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

    fn language_extensions(
        languages: &[LanguageConfiguration],
        language_id: &str,
    ) -> Option<Vec<String>> {
        languages
            .iter()
            .find(|language| language.id() == language_id)
            .map(|language| language.file_extensions().to_vec())
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
        assert!(languages.len() >= 12);
        let ids = languages
            .iter()
            .map(|language| language.id())
            .collect::<Vec<_>>();
        assert!(ids.contains(&"csharp"));
        assert!(ids.contains(&"rust"));
        assert!(ids.contains(&"gitcommit"));
        assert!(ids.contains(&"javascript"));
        assert!(ids.contains(&"jsx"));
        assert!(ids.contains(&"json"));
        assert!(ids.contains(&"markdown"));
        assert!(ids.contains(&"markdown-inline"));
        assert!(ids.contains(&"toml"));
        assert!(ids.contains(&"typescript"));
        assert!(ids.contains(&"tsx"));
        assert!(ids.contains(&"yaml"));

        assert_eq!(
            language_extensions(&languages, "csharp"),
            Some(vec!["cs".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "rust"),
            Some(vec!["rs".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "javascript"),
            Some(vec!["js".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "jsx"),
            Some(vec!["jsx".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "json"),
            Some(vec!["json".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "markdown"),
            Some(vec!["md".to_owned(), "markdown".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "toml"),
            Some(vec!["toml".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "typescript"),
            Some(vec!["ts".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "tsx"),
            Some(vec!["tsx".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "yaml"),
            Some(vec!["yaml".to_owned(), "yml".to_owned()])
        );
    }

    #[test]
    fn user_library_exports_lsp_and_dap_defaults() {
        let servers = language_servers();
        let server_ids = servers.iter().map(|server| server.id()).collect::<Vec<_>>();
        let adapters = debug_adapters();

        assert_eq!(servers.len(), 7);
        assert!(server_ids.contains(&SERVER_RUST_ANALYZER));
        assert!(server_ids.contains(&SERVER_MARKSMAN));
        assert!(server_ids.contains(&SERVER_CSHARP_LS));
        assert!(server_ids.contains(&SERVER_TYPESCRIPT_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_VSCODE_JSON_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_TOMBI));
        assert!(server_ids.contains(&SERVER_YAML_LANGUAGE_SERVER));
        assert_eq!(adapters.len(), 1);
        assert_eq!(adapters[0].id(), "codelldb");

        let typescript = servers
            .iter()
            .find(|server| server.id() == SERVER_TYPESCRIPT_LANGUAGE_SERVER)
            .expect("typescript-language-server missing");
        assert_eq!(
            typescript
                .file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["ts", "tsx", "js", "jsx"]
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".ts"),
            "typescript"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".tsx"),
            "typescriptreact"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".js"),
            "javascript"
        );
        assert_eq!(
            typescript.document_language_id_for_extension(".jsx"),
            "javascriptreact"
        );
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
        const TOKENS: &[&str] = &[
            "ui.cursor",
            "ui.selection",
            "ui.yank-flash",
            "ui.notification.background",
            "ui.notification.foreground",
            "ui.notification.title",
            "ui.notification.muted",
            "ui.notification.border",
            "ui.notification.progress.background",
            "ui.notification.progress.fill",
            "ui.notification.info",
            "ui.notification.success",
            "ui.notification.warning",
            "ui.notification.error",
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
            mapped_theme_token(markdown, "text.literal"),
            Some("syntax.text.literal")
        );
        assert_eq!(
            mapped_theme_token(markdown, "text.uri"),
            Some("syntax.text.uri")
        );
        assert_eq!(
            mapped_theme_token(markdown, "punctuation.special"),
            Some("syntax.punctuation.special")
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

    #[test]
    fn tsx_highlight_query_compiles() {
        let mut registry = SyntaxRegistry::new();
        for language in syntax_languages()
            .into_iter()
            .filter(|language| matches!(language.id(), "typescript" | "tsx"))
        {
            registry
                .register(language)
                .expect("registering TypeScript language");
        }

        if registry
            .is_installed("typescript")
            .expect("checking TypeScript install")
        {
            registry
                .highlight_buffer_for_language(
                    "typescript",
                    &TextBuffer::from_text(
                        "const describe = ({ title }: { title: string }) => title;\n",
                    ),
                )
                .expect("typescript query should compile");
        }
        if !registry.is_installed("tsx").expect("checking TSX install") {
            eprintln!("skipping TSX highlight query compile test: grammar is not installed");
            return;
        }
        registry
            .highlight_buffer_for_language(
                "tsx",
                &TextBuffer::from_text(
                    "const App = ({ title }: { title: string }) => <div>{title}</div>;\n",
                ),
            )
            .expect("tsx query should compile");
    }
}
