//! Compiled user extension library.
//!
//! This crate intentionally keeps feature packages as Rust modules living directly
//! under the `user/` directory so the future extension model matches the planned
//! 4coder-style workflow.
//!
//! # Distribution model
//!
//! The user library is compiled as both a `cdylib` (for runtime loading by
//! `volt.exe`) and an `rlib` (for linking during development). `user/sdk`
//! is the only stable ABI crate; this crate sits on top of that ABI surface
//! and provides the compiled customization layer. The public [`UserLibraryImpl`]
//! struct implements the [`editor_plugin_host::UserLibrary`] trait so the host
//! can call into the user library without direct source-level coupling.

#[cfg(test)]
#[path = "build_output.rs"]
mod build_output;

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
/// Vim command-line enablement.
pub mod commandline;
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
/// Native image-viewer commands and keybindings.
pub mod image;
/// Bundled icon-font symbol modules (re-exported from editor-icons).
pub use editor_plugin_api::symbols as icon_font_symbols;
/// Interactive read-only buffer workflows.
pub mod interactive;
/// Language-specific registrations.
pub mod lang;
/// Text ligature configuration surfaced to the shell renderer.
pub mod ligatures;
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
/// Tree-sitter-backed ghost text context annotations.
pub mod treesittercontext_ghosttext;
/// Tree-sitter-backed sticky headerline context annotations.
pub mod treesittercontext_headerline;
mod treesittercontext_shared;
/// Undo tree picker and history navigation.
pub mod undotree;
/// Vim-style bindings and motions.
pub mod vim;
/// Workspace creation and project discovery.
pub mod workspace;

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    std_types::{ROption, RStr, RString, RVec},
};
use editor_plugin_api::PluginPackage;
use editor_plugin_api::{
    DebugAdapterSpec, LanguageConfiguration, LanguageServerSpec, Theme,
    abi::{
        AbiAcpClient, AbiAutocompleteProvider, AbiDebugAdapterSpec, AbiDirectoryEntry,
        AbiGhostTextContext, AbiGhostTextLine, AbiGitStatusPrefix, AbiGitStatusSnapshot,
        AbiHoverProvider, AbiIconFontSymbol, AbiLanguageConfiguration, AbiLanguageServerSpec,
        AbiLigatureConfig, AbiOilDefaults, AbiOilKeyAction, AbiOilKeybindings, AbiOilSortMode,
        AbiSectionTree, AbiStatuslineContext, AbiTerminalConfig, AbiTheme, AbiWorkspaceRoot,
        UserLibraryModule, UserLibraryModuleRef,
    },
};

/// Returns the packages currently compiled into the user library.
pub fn packages() -> Vec<PluginPackage> {
    let mut pkgs = vec![
        buffer::package(),
        acp::package(),
        autocomplete::package(),
        browser::package(),
        calculator::package(),
        compile::package(),
        image::package(),
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
    AcpClient, AutocompleteProvider, GhostTextContext, GhostTextLine, GitStatusPrefix,
    HoverProvider, LigatureConfig, OilDefaults, OilKeyAction, OilKeybindings, StatuslineContext,
    TerminalConfig, UserLibrary, WorkspaceRoot,
};

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
                buffer_kind: p.buffer_kind,
                items: p.items,
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
                buffer_kind: p.buffer_kind,
                topics: p.topics,
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

    fn commandline_enabled(&self) -> bool {
        commandline::enabled()
    }

    fn ligature_config(&self) -> LigatureConfig {
        ligatures::config()
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
            editor_plugin_api::OilSortMode::TypeThenNameDesc => oil::OilSortMode::TypeThenNameDesc,
        };
        oil::directory_sections(root, entries, show_hidden, user_sort, trash_enabled)
    }

    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        oil::strip_entry_icon_prefix(label)
    }

    fn git_status_sections(
        &self,
        snapshot: &editor_git::GitStatusSnapshot,
    ) -> editor_core::SectionTree {
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

    fn ghost_text_lines(&self, context: &GhostTextContext<'_>) -> Vec<GhostTextLine> {
        treesittercontext_ghosttext::ghost_text_lines(context)
    }

    fn headerline_lines(&self, context: &GhostTextContext<'_>) -> Vec<String> {
        treesittercontext_headerline::headerline_lines(context)
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
            lsp_diagnostics: context
                .lsp_diagnostics
                .map(|d| statusline::LspDiagnosticsInfo {
                    errors: d.errors,
                    warnings: d.warnings,
                }),
            acp_connected: context.acp_connected,
            git: context
                .git_branch
                .map(|branch| statusline::GitStatuslineInfo {
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

    fn run_plugin_buffer_evaluator(&self, handler: &str, input: &str) -> Vec<String> {
        match handler {
            calculator::EVALUATE_HANDLER => calculator::evaluate(input),
            _ => vec![format!(
                "no plugin buffer evaluator registered for `{handler}`"
            )],
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

extern "C" fn exported_packages() -> RVec<PluginPackage> {
    packages().into()
}

extern "C" fn exported_themes() -> RVec<AbiTheme> {
    themes()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_syntax_languages() -> RVec<AbiLanguageConfiguration> {
    syntax_languages()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_language_servers() -> RVec<AbiLanguageServerSpec> {
    language_servers()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_debug_adapters() -> RVec<AbiDebugAdapterSpec> {
    debug_adapters()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_autocomplete_providers() -> RVec<AbiAutocompleteProvider> {
    UserLibraryImpl
        .autocomplete_providers()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_autocomplete_result_limit() -> usize {
    UserLibraryImpl.autocomplete_result_limit()
}

extern "C" fn exported_autocomplete_token_icon() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.autocomplete_token_icon())
}

extern "C" fn exported_hover_providers() -> RVec<AbiHoverProvider> {
    UserLibraryImpl
        .hover_providers()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_hover_line_limit() -> usize {
    UserLibraryImpl.hover_line_limit()
}

extern "C" fn exported_hover_token_icon() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.hover_token_icon())
}

extern "C" fn exported_hover_signature_icon() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.hover_signature_icon())
}

extern "C" fn exported_acp_clients() -> RVec<AbiAcpClient> {
    UserLibraryImpl
        .acp_clients()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_acp_client_by_id(id: RString) -> ROption<AbiAcpClient> {
    UserLibraryImpl
        .acp_client_by_id(id.as_str())
        .map(Into::into)
        .into()
}

extern "C" fn exported_workspace_roots() -> RVec<AbiWorkspaceRoot> {
    UserLibraryImpl
        .workspace_roots()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_terminal_config() -> AbiTerminalConfig {
    UserLibraryImpl.terminal_config().into()
}

extern "C" fn exported_commandline_enabled() -> bool {
    UserLibraryImpl.commandline_enabled()
}

extern "C" fn exported_ligature_config() -> AbiLigatureConfig {
    UserLibraryImpl.ligature_config().into()
}

extern "C" fn exported_oil_defaults() -> AbiOilDefaults {
    UserLibraryImpl.oil_defaults().into()
}

extern "C" fn exported_oil_keybindings() -> AbiOilKeybindings {
    UserLibraryImpl.oil_keybindings().into()
}

extern "C" fn exported_oil_keydown_action(chord: RString) -> ROption<AbiOilKeyAction> {
    UserLibraryImpl
        .oil_keydown_action(chord.as_str())
        .map(Into::into)
        .into()
}

extern "C" fn exported_oil_chord_action(
    had_prefix: bool,
    chord: RString,
) -> ROption<AbiOilKeyAction> {
    UserLibraryImpl
        .oil_chord_action(had_prefix, chord.as_str())
        .map(Into::into)
        .into()
}

extern "C" fn exported_oil_help_lines() -> RVec<RString> {
    UserLibraryImpl
        .oil_help_lines()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<RString>>()
        .into()
}

extern "C" fn exported_oil_directory_sections(
    root: RString,
    entries: RVec<AbiDirectoryEntry>,
    show_hidden: bool,
    sort_mode: AbiOilSortMode,
    trash_enabled: bool,
) -> AbiSectionTree {
    let entries = entries.into_iter().map(Into::into).collect::<Vec<_>>();
    UserLibraryImpl
        .oil_directory_sections(
            std::path::Path::new(root.as_str()),
            &entries,
            show_hidden,
            sort_mode.into(),
            trash_enabled,
        )
        .into()
}

extern "C" fn exported_oil_strip_entry_icon_prefix(label: RString) -> RString {
    UserLibraryImpl
        .oil_strip_entry_icon_prefix(label.as_str())
        .to_owned()
        .into()
}

extern "C" fn exported_git_status_sections(snapshot: AbiGitStatusSnapshot) -> AbiSectionTree {
    UserLibraryImpl.git_status_sections(&snapshot.into()).into()
}

extern "C" fn exported_git_commit_template() -> RVec<RString> {
    UserLibraryImpl
        .git_commit_template()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<RString>>()
        .into()
}

extern "C" fn exported_git_prefix_for_chord(chord: RString) -> ROption<AbiGitStatusPrefix> {
    UserLibraryImpl
        .git_prefix_for_chord(chord.as_str())
        .map(Into::into)
        .into()
}

extern "C" fn exported_git_command_for_chord(
    prefix: ROption<AbiGitStatusPrefix>,
    chord: RString,
) -> ROption<RStr<'static>> {
    UserLibraryImpl
        .git_command_for_chord(prefix.into_option().map(Into::into), chord.as_str())
        .map(RStr::from_str)
        .into()
}

extern "C" fn exported_browser_buffer_lines(url: ROption<RString>) -> RVec<RString> {
    let url = url.into_option();
    UserLibraryImpl
        .browser_buffer_lines(url.as_deref())
        .into_iter()
        .map(Into::into)
        .collect::<Vec<RString>>()
        .into()
}

extern "C" fn exported_browser_input_hint(url: ROption<RString>) -> RString {
    let url = url.into_option();
    UserLibraryImpl.browser_input_hint(url.as_deref()).into()
}

extern "C" fn exported_browser_url_prompt() -> RString {
    UserLibraryImpl.browser_url_prompt().into()
}

extern "C" fn exported_browser_url_placeholder() -> RString {
    UserLibraryImpl.browser_url_placeholder().into()
}

extern "C" fn exported_ghost_text_lines(context: AbiGhostTextContext) -> RVec<AbiGhostTextLine> {
    let context = GhostTextContext {
        buffer_id: context.buffer_id,
        buffer_revision: context.buffer_revision,
        buffer_name: context.buffer_name.as_str(),
        language_id: context
            .language_id
            .as_ref()
            .into_option()
            .map(|value| value.as_str()),
        buffer_text: context.buffer_text.as_str(),
        viewport_top_line: context.viewport_top_line,
        cursor_line: context.cursor_line,
        cursor_column: context.cursor_column,
    };
    UserLibraryImpl
        .ghost_text_lines(&context)
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_headerline_lines(context: AbiGhostTextContext) -> RVec<RString> {
    let context = GhostTextContext {
        buffer_id: context.buffer_id,
        buffer_revision: context.buffer_revision,
        buffer_name: context.buffer_name.as_str(),
        language_id: context
            .language_id
            .as_ref()
            .into_option()
            .map(|value| value.as_str()),
        buffer_text: context.buffer_text.as_str(),
        viewport_top_line: context.viewport_top_line,
        cursor_line: context.cursor_line,
        cursor_column: context.cursor_column,
    };
    UserLibraryImpl
        .headerline_lines(&context)
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_statusline_render(context: AbiStatuslineContext) -> RString {
    let context = StatuslineContext {
        vim_mode: context.vim_mode.as_str(),
        recording_macro: context
            .recording_macro
            .into_option()
            .and_then(char::from_u32),
        workspace_name: context.workspace_name.as_str(),
        buffer_name: context.buffer_name.as_str(),
        buffer_modified: context.buffer_modified,
        language_id: context
            .language_id
            .as_ref()
            .into_option()
            .map(|value| value.as_str()),
        line: context.line,
        column: context.column,
        lsp_server: context
            .lsp_server
            .as_ref()
            .into_option()
            .map(|value| value.as_str()),
        lsp_diagnostics: context.lsp_diagnostics.into_option().map(Into::into),
        acp_connected: context.acp_connected,
        git_branch: context
            .git_branch
            .as_ref()
            .into_option()
            .map(|value| value.as_str()),
        git_added: context.git_added,
        git_removed: context.git_removed,
    };
    UserLibraryImpl.statusline_render(&context).into()
}

extern "C" fn exported_statusline_lsp_connected_icon() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.statusline_lsp_connected_icon())
}

extern "C" fn exported_statusline_lsp_error_icon() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.statusline_lsp_error_icon())
}

extern "C" fn exported_statusline_lsp_warning_icon() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.statusline_lsp_warning_icon())
}

extern "C" fn exported_lsp_diagnostic_icon() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.lsp_diagnostic_icon())
}

extern "C" fn exported_lsp_diagnostic_line_limit() -> usize {
    UserLibraryImpl.lsp_diagnostic_line_limit()
}

extern "C" fn exported_lsp_show_buffer_diagnostics() -> bool {
    UserLibraryImpl.lsp_show_buffer_diagnostics()
}

extern "C" fn exported_gitfringe_token_added() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.gitfringe_token_added())
}

extern "C" fn exported_gitfringe_token_modified() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.gitfringe_token_modified())
}

extern "C" fn exported_gitfringe_token_removed() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.gitfringe_token_removed())
}

extern "C" fn exported_gitfringe_symbol() -> RStr<'static> {
    RStr::from_str(UserLibraryImpl.gitfringe_symbol())
}

extern "C" fn exported_icon_symbols() -> RVec<AbiIconFontSymbol> {
    UserLibraryImpl
        .icon_symbols()
        .iter()
        .copied()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_run_plugin_buffer_evaluator(
    handler: RString,
    input: RString,
) -> RVec<RString> {
    UserLibraryImpl
        .run_plugin_buffer_evaluator(handler.as_str(), input.as_str())
        .into_iter()
        .map(Into::into)
        .collect::<Vec<RString>>()
        .into()
}

extern "C" fn exported_default_build_command(language: RString) -> ROption<RString> {
    UserLibraryImpl
        .default_build_command(language.as_str())
        .map(Into::into)
        .into()
}

pub fn user_library_module() -> UserLibraryModuleRef {
    UserLibraryModule {
        packages: exported_packages,
        themes: exported_themes,
        syntax_languages: exported_syntax_languages,
        language_servers: exported_language_servers,
        debug_adapters: exported_debug_adapters,
        autocomplete_providers: exported_autocomplete_providers,
        autocomplete_result_limit: exported_autocomplete_result_limit,
        autocomplete_token_icon: exported_autocomplete_token_icon,
        hover_providers: exported_hover_providers,
        hover_line_limit: exported_hover_line_limit,
        hover_token_icon: exported_hover_token_icon,
        hover_signature_icon: exported_hover_signature_icon,
        acp_clients: exported_acp_clients,
        acp_client_by_id: exported_acp_client_by_id,
        workspace_roots: exported_workspace_roots,
        terminal_config: exported_terminal_config,
        commandline_enabled: exported_commandline_enabled,
        ligature_config: exported_ligature_config,
        oil_defaults: exported_oil_defaults,
        oil_keybindings: exported_oil_keybindings,
        oil_keydown_action: exported_oil_keydown_action,
        oil_chord_action: exported_oil_chord_action,
        oil_help_lines: exported_oil_help_lines,
        oil_directory_sections: exported_oil_directory_sections,
        oil_strip_entry_icon_prefix: exported_oil_strip_entry_icon_prefix,
        git_status_sections: exported_git_status_sections,
        git_commit_template: exported_git_commit_template,
        git_prefix_for_chord: exported_git_prefix_for_chord,
        git_command_for_chord: exported_git_command_for_chord,
        browser_buffer_lines: exported_browser_buffer_lines,
        browser_input_hint: exported_browser_input_hint,
        browser_url_prompt: exported_browser_url_prompt,
        browser_url_placeholder: exported_browser_url_placeholder,
        statusline_render: exported_statusline_render,
        statusline_lsp_connected_icon: exported_statusline_lsp_connected_icon,
        statusline_lsp_error_icon: exported_statusline_lsp_error_icon,
        statusline_lsp_warning_icon: exported_statusline_lsp_warning_icon,
        lsp_diagnostic_icon: exported_lsp_diagnostic_icon,
        lsp_diagnostic_line_limit: exported_lsp_diagnostic_line_limit,
        lsp_show_buffer_diagnostics: exported_lsp_show_buffer_diagnostics,
        gitfringe_token_added: exported_gitfringe_token_added,
        gitfringe_token_modified: exported_gitfringe_token_modified,
        gitfringe_token_removed: exported_gitfringe_token_removed,
        gitfringe_symbol: exported_gitfringe_symbol,
        icon_symbols: exported_icon_symbols,
        run_plugin_buffer_evaluator: exported_run_plugin_buffer_evaluator,
        default_build_command: exported_default_build_command,
        ligature_config_v1: exported_ligature_config,
        ghost_text_lines: exported_ghost_text_lines,
        headerline_lines: exported_headerline_lines,
    }
    .leak_into_prefix()
}

#[export_root_module]
pub fn exported_user_library_module() -> UserLibraryModuleRef {
    user_library_module()
}

#[cfg(test)]
mod tests {
    use super::{
        UserLibraryImpl, debug_adapters, language_servers, packages, syntax_languages, themes,
    };
    use crate::calculator;
    use crate::lsp::{
        SERVER_CLANGD, SERVER_CSHARP_LS, SERVER_GOPLS, SERVER_MAKEFILE_LANGUAGE_SERVER,
        SERVER_MARKSMAN, SERVER_OLS, SERVER_PYRIGHT_LANGSERVER, SERVER_RUST_ANALYZER, SERVER_SQLS,
        SERVER_TOMBI, SERVER_TYPESCRIPT_LANGUAGE_SERVER, SERVER_VSCODE_CSS_LANGUAGE_SERVER,
        SERVER_VSCODE_HTML_LANGUAGE_SERVER, SERVER_VSCODE_JSON_LANGUAGE_SERVER,
        SERVER_YAML_LANGUAGE_SERVER, SERVER_ZLS,
    };
    use editor_buffer::TextBuffer;
    use editor_plugin_api::UserLibrary;
    use editor_syntax::{LanguageConfiguration, SyntaxRegistry};
    use std::collections::BTreeSet;

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
    fn user_library_keybindings_do_not_conflict() {
        let mut seen = BTreeSet::new();
        for package in packages() {
            for keybinding in package.key_bindings() {
                let identity = (
                    format!("{:?}", keybinding.scope()),
                    format!("{:?}", keybinding.vim_mode()),
                    keybinding.chord().to_owned(),
                );
                assert!(
                    seen.insert(identity.clone()),
                    "duplicate keybinding {:?} in package `{}`",
                    identity,
                    package.name()
                );
            }
        }
    }

    #[test]
    fn user_library_derives_plugin_buffer_behavior_from_package_metadata() {
        let library = UserLibraryImpl;
        assert!(library.supports_plugin_evaluate(calculator::CALCULATOR_KIND));
        assert_eq!(
            library.plugin_buffer_initial_lines(calculator::CALCULATOR_KIND),
            calculator::initial_buffer_lines()
        );
        assert_eq!(
            library
                .plugin_buffer_sections(calculator::CALCULATOR_KIND)
                .and_then(|sections| {
                    sections
                        .items()
                        .last()
                        .map(|section| section.name().to_owned())
                }),
            Some("Output".to_owned())
        );
        assert_eq!(
            library.handle_plugin_evaluate(calculator::CALCULATOR_KIND, "1 + 1"),
            vec!["2".to_owned()]
        );
    }

    #[test]
    fn user_library_packages_exclude_tree_sitter_context_renderers() {
        let packages = packages();
        assert!(
            packages
                .iter()
                .all(|package| package.name() != "treesittercontext_headerline")
        );
        assert!(
            packages
                .iter()
                .all(|package| package.name() != "treesittercontext_ghosttext")
        );
    }

    #[test]
    fn exported_user_library_module_matches_static_library() {
        let module = super::user_library_module();
        assert_eq!(module.packages()().len(), packages().len());
        assert_eq!(module.themes()().len(), themes().len());
        assert_eq!(module.language_servers()().len(), language_servers().len());
    }

    #[test]
    fn user_library_exports_calculator_manual_providers() {
        let library = UserLibraryImpl;
        let autocomplete = library.autocomplete_providers();
        let calculator_autocomplete = autocomplete
            .iter()
            .find(|provider| provider.id == calculator::PROVIDER_CALCULATOR)
            .expect("calculator autocomplete provider should be exported");
        assert_eq!(
            calculator_autocomplete.buffer_kind.as_deref(),
            Some(calculator::CALCULATOR_KIND)
        );
        assert!(
            calculator_autocomplete
                .items
                .iter()
                .any(|item| item.replacement == "sqrt")
        );

        let hover = library.hover_providers();
        let calculator_hover = hover
            .iter()
            .find(|provider| provider.id == calculator::PROVIDER_CALCULATOR)
            .expect("calculator hover provider should be exported");
        assert_eq!(
            calculator_hover.buffer_kind.as_deref(),
            Some(calculator::CALCULATOR_KIND)
        );
        assert!(
            calculator_hover
                .topics
                .iter()
                .any(|topic| topic.token == "pi")
        );
    }

    #[test]
    fn user_library_exports_language_registrations() {
        let languages = syntax_languages();
        assert!(languages.len() >= 23);
        let ids = languages
            .iter()
            .map(|language| language.id())
            .collect::<Vec<_>>();
        assert!(ids.contains(&"c"));
        assert!(ids.contains(&"csharp"));
        assert!(ids.contains(&"cpp"));
        assert!(ids.contains(&"css"));
        assert!(ids.contains(&"rust"));
        assert!(ids.contains(&"gitcommit"));
        assert!(ids.contains(&"go"));
        assert!(ids.contains(&"html"));
        assert!(ids.contains(&"javascript"));
        assert!(ids.contains(&"jsx"));
        assert!(ids.contains(&"json"));
        assert!(ids.contains(&"make"));
        assert!(ids.contains(&"markdown"));
        assert!(ids.contains(&"markdown-inline"));
        assert!(ids.contains(&"odin"));
        assert!(ids.contains(&"python"));
        assert!(ids.contains(&"scss"));
        assert!(ids.contains(&"sql"));
        assert!(ids.contains(&"toml"));
        assert!(ids.contains(&"typescript"));
        assert!(ids.contains(&"tsx"));
        assert!(ids.contains(&"yaml"));
        assert!(ids.contains(&"zig"));

        assert_eq!(
            language_extensions(&languages, "c"),
            Some(vec!["c".to_owned(), "h".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "csharp"),
            Some(vec!["cs".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "cpp"),
            Some(vec![
                "cc".to_owned(),
                "cpp".to_owned(),
                "cxx".to_owned(),
                "hpp".to_owned(),
                "hh".to_owned(),
                "hxx".to_owned(),
            ])
        );
        assert_eq!(
            language_extensions(&languages, "css"),
            Some(vec!["css".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "go"),
            Some(vec!["go".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "html"),
            Some(vec!["html".to_owned(), "htm".to_owned()])
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
            language_extensions(&languages, "make"),
            Some(vec!["mk".to_owned(), "mak".to_owned(), "make".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "markdown"),
            Some(vec!["md".to_owned(), "markdown".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "odin"),
            Some(vec!["odin".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "python"),
            Some(vec!["py".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "scss"),
            Some(vec!["scss".to_owned()])
        );
        assert_eq!(
            language_extensions(&languages, "sql"),
            Some(vec!["sql".to_owned()])
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
        assert_eq!(
            language_extensions(&languages, "zig"),
            Some(vec!["zig".to_owned()])
        );
    }

    #[test]
    fn user_library_exports_lsp_and_dap_defaults() {
        let servers = language_servers();
        let server_ids = servers.iter().map(|server| server.id()).collect::<Vec<_>>();
        let adapters = debug_adapters();

        assert_eq!(servers.len(), 16);
        assert!(server_ids.contains(&SERVER_CLANGD));
        assert!(server_ids.contains(&SERVER_RUST_ANALYZER));
        assert!(server_ids.contains(&SERVER_MARKSMAN));
        assert!(server_ids.contains(&SERVER_CSHARP_LS));
        assert!(server_ids.contains(&SERVER_GOPLS));
        assert!(server_ids.contains(&SERVER_MAKEFILE_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_OLS));
        assert!(server_ids.contains(&SERVER_PYRIGHT_LANGSERVER));
        assert!(server_ids.contains(&SERVER_SQLS));
        assert!(server_ids.contains(&SERVER_TYPESCRIPT_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_VSCODE_CSS_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_VSCODE_HTML_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_VSCODE_JSON_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_TOMBI));
        assert!(server_ids.contains(&SERVER_YAML_LANGUAGE_SERVER));
        assert!(server_ids.contains(&SERVER_ZLS));
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

        let css = servers
            .iter()
            .find(|server| server.id() == SERVER_VSCODE_CSS_LANGUAGE_SERVER)
            .expect("vscode-css-language-server missing");
        assert_eq!(css.document_language_id_for_extension(".scss"), "scss");

        let clangd = servers
            .iter()
            .find(|server| server.id() == SERVER_CLANGD)
            .expect("clangd missing");
        assert_eq!(clangd.document_language_id_for_extension(".c"), "c");
        assert_eq!(clangd.document_language_id_for_extension(".cpp"), "cpp");

        let html = servers
            .iter()
            .find(|server| server.id() == SERVER_VSCODE_HTML_LANGUAGE_SERVER)
            .expect("vscode-html-language-server missing");
        assert_eq!(
            html.file_extensions()
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            vec!["html", "htm"]
        );
    }

    #[test]
    fn user_library_exports_themes() {
        let themes = themes();
        let ids = themes.iter().map(|theme| theme.id()).collect::<Vec<_>>();
        assert_eq!(themes.len(), 7);
        assert!(ids.contains(&"volt-dark"));
        assert!(ids.contains(&"volt-light"));
        assert!(ids.contains(&"gruvbox-dark"));
        assert!(ids.contains(&"gruvbox-light"));
        assert!(ids.contains(&"rosepine-dark"));
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
