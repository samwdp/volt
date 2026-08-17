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
/// Database explorer commands, query buffers, and schema browsers.
pub mod db;
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
/// Workspace Issues plugin (Issue Store, Board, Capture, Place, Scan).
pub mod issues;
/// Bundled icon-font symbol modules (re-exported from editor-icons).
pub use editor_plugin_api::symbols as icon_font_symbols;
/// Runtime-loaded user configuration.
pub mod config;
/// Interactive read-only buffer workflows.
pub mod interactive;
/// Keymap tunables (`ui.keymap.*`).
pub mod keymap;
/// Language-specific registrations.
pub mod lang;
/// Text ligature configuration surfaced to the shell renderer.
pub mod ligatures;
/// Language server integration hooks and commands.
pub mod lsp;
/// User-editable doom-style modeline composition.
pub mod modeline;
/// Multiple cursor workflows.
pub mod multicursor;
/// Directory editing and navigation workflows.
pub mod oil;
/// Pane layout management.
pub mod pane;
/// Native PDF buffer commands and keybindings.
pub mod pdf;
/// Generic picker UI bindings and popup controls.
pub mod picker;
/// Rainbow delimiter highlighting for nested brackets.
pub mod rainbow_parens;
/// Compatibility alias for [`modeline`].
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
/// Vertical workspace dock (sibling to the bottom popup).
pub mod workspace_dock;

use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    std_types::{ROption, RStr, RString, RVec},
};
use editor_plugin_api::PluginPackage;
use editor_plugin_api::{
    DebugAdapterSpec, LanguageConfiguration, LanguageServerSpec, PdfOpenMode, Theme,
    abi::{
        AbiAcpClient, AbiAutocompleteProvider, AbiBrowserFeatureSpec, AbiContextHelpSpec,
        AbiDbFeatureSpec, AbiDebugAdapterSpec, AbiDirectoryEntry, AbiGhostTextContext,
        AbiGhostTextLine, AbiGitFeatureSpec, AbiGitStatusPrefix, AbiGitStatusSnapshot,
        AbiHoverProvider, AbiIconFontSymbol, AbiKeymapConfig, AbiLanguageConfiguration,
        AbiLanguageServerSpec, AbiLigatureConfig, AbiOilDefaults, AbiOilFeatureSpec,
        AbiOilKeyAction, AbiOilKeybindings, AbiOilSortMode, AbiPaneConfig, AbiPdfOpenMode,
        AbiPickerTruncateStrategy, AbiSectionTree, AbiStatuslineContext, AbiTerminalConfig,
        AbiTerminalFeatureSpec, AbiTheme, AbiWorkspaceRoot, UserLibraryModule,
        UserLibraryModuleRef,
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
        issues::package(),
        pane::package(),
        workspace_dock::package(),
        pdf::package(),
        hover::package(),
        lsp::package(),
        dap::package(),
        db::package(),
        oil::package(),
        multicursor::package(),
        picker::package(),
        rainbow_parens::package(),
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
    AcpClient, AcpPickerContext, AcpPickerItemSpec, AutocompleteProvider, BrowserFeatureSpec,
    ContextHelpSpec, DbBrowserContext, DbBrowserItemSpec, DbFeatureSpec, GhostTextContext,
    GhostTextLine, GitFeatureSpec, GitStatusPrefix, HoverProvider, KeymapConfig, LigatureConfig,
    MarkdownPrettyConfig, ModelineSegment, OilDefaults, OilFeatureSpec, OilKeyAction,
    OilKeybindings, PaneConfig, PickerItemSpec, PickerProviderContext, PickerProviderSpec,
    RainbowParensConfig, StatuslineContext, StatuslineSpan, TerminalConfig, TerminalFeatureSpec,
    UserLibrary, WorkspaceDockConfig, WorkspaceRoot, flatten_modeline_to_spans,
};

fn user_modeline_context<'a>(context: &StatuslineContext<'a>) -> modeline::ModelineContext<'a> {
    modeline::ModelineContext {
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
            .map(|diagnostics| modeline::LspDiagnosticsInfo {
                errors: diagnostics.errors,
                warnings: diagnostics.warnings,
            }),
        acp_connected: context.acp_connected,
        git: context.git_branch.map(|branch| modeline::GitModelineInfo {
            branch,
            added: context.git_added,
            removed: context.git_removed,
        }),
    }
}

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

    fn picker_providers(&self) -> Vec<PickerProviderSpec> {
        picker::providers()
    }

    fn picker_provider_items(
        &self,
        context: &PickerProviderContext,
    ) -> Option<Vec<PickerItemSpec>> {
        picker::provider_items(context)
    }

    fn picker_truncate_strategy(&self) -> editor_plugin_api::PickerTruncateStrategy {
        picker::truncate_strategy()
    }

    fn picker_layout(&self) -> editor_plugin_api::PickerLayout {
        picker::layout()
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

    fn acp_picker_items(&self, context: &AcpPickerContext) -> Vec<AcpPickerItemSpec> {
        acp::picker_items(context)
    }

    fn db_browser_items(&self, context: &DbBrowserContext) -> Vec<DbBrowserItemSpec> {
        db::browser_items(context)
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

    fn pane_config(&self) -> PaneConfig {
        pane::config()
    }

    fn workspace_dock_config(&self) -> WorkspaceDockConfig {
        workspace_dock::config()
    }

    fn markdown_pretty_config(&self) -> MarkdownPrettyConfig {
        lang::markdown::pretty_config()
    }

    fn keymap_config(&self) -> KeymapConfig {
        keymap::config()
    }

    fn ligature_config(&self) -> LigatureConfig {
        ligatures::config()
    }

    fn rainbow_parens_config(&self) -> RainbowParensConfig {
        rainbow_parens::config()
    }

    fn oil_defaults(&self) -> OilDefaults {
        oil::feature_spec().defaults
    }

    fn oil_keybindings(&self) -> OilKeybindings {
        oil::feature_spec().keybindings
    }

    fn oil_keydown_action(&self, chord: &str) -> Option<OilKeyAction> {
        oil::keydown_action(chord)
    }

    fn oil_chord_action(&self, had_prefix: bool, chord: &str) -> Option<OilKeyAction> {
        oil::chord_action(had_prefix, chord)
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
        oil::directory_sections(root, entries, show_hidden, sort_mode, trash_enabled)
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

    fn git_commit_template(&self, snapshot: &editor_git::GitStatusSnapshot) -> Vec<String> {
        git::commit_buffer_template(snapshot)
    }

    fn git_prefix_for_chord(&self, chord: &str) -> Option<GitStatusPrefix> {
        git::feature_spec().prefix_for_chord(chord)
    }

    fn git_command_for_chord(
        &self,
        prefix: Option<GitStatusPrefix>,
        chord: &str,
    ) -> Option<&'static str> {
        git::status_command_name(prefix, chord)
    }

    fn browser_buffer_lines(&self, url: Option<&str>) -> Vec<String> {
        browser::buffer_lines(url)
    }

    fn browser_input_hint(&self, url: Option<&str>) -> String {
        browser::input_hint(url)
    }

    fn browser_url_prompt(&self) -> String {
        browser::feature_spec().url_prompt
    }

    fn browser_url_placeholder(&self) -> String {
        browser::feature_spec().url_placeholder
    }

    fn git_feature_spec(&self) -> GitFeatureSpec {
        git::feature_spec()
    }

    fn oil_feature_spec(&self) -> OilFeatureSpec {
        oil::feature_spec()
    }

    fn browser_feature_spec(&self) -> BrowserFeatureSpec {
        browser::feature_spec()
    }

    fn db_feature_spec(&self) -> DbFeatureSpec {
        db::feature_spec()
    }

    fn terminal_feature_spec(&self) -> TerminalFeatureSpec {
        terminal::feature_spec()
    }

    fn context_help_specs(&self) -> Vec<ContextHelpSpec> {
        let mut specs = Vec::new();
        specs.extend(self.git_feature_spec().context_help_specs());
        specs.push(self.oil_feature_spec().help);
        specs.push(self.browser_feature_spec().help);
        specs.extend(self.db_feature_spec().context_help_specs());
        specs.push(self.terminal_feature_spec().help);
        specs
    }

    fn pdf_open_mode(&self) -> PdfOpenMode {
        pdf::open_mode()
    }

    fn ghost_text_lines(&self, context: &GhostTextContext<'_>) -> Vec<GhostTextLine> {
        treesittercontext_ghosttext::ghost_text_lines(context)
    }

    fn headerline_lines(&self, context: &GhostTextContext<'_>) -> Vec<String> {
        treesittercontext_headerline::headerline_lines(context)
    }

    fn statusline_render(&self, context: &StatuslineContext<'_>) -> String {
        modeline::compose(&user_modeline_context(context))
    }

    fn statusline_spans(&self, context: &StatuslineContext<'_>) -> Vec<StatuslineSpan> {
        flatten_modeline_to_spans(&self.modeline_segments(context))
    }

    fn modeline_segments(&self, context: &StatuslineContext<'_>) -> Vec<ModelineSegment> {
        modeline::compose_modeline(&user_modeline_context(context))
    }

    fn statusline_lsp_connected_icon(&self) -> &'static str {
        modeline::LSP_CONNECTED_ICON
    }

    fn statusline_lsp_error_icon(&self) -> &'static str {
        modeline::LSP_ERROR_ICON
    }

    fn statusline_lsp_warning_icon(&self) -> &'static str {
        modeline::LSP_WARNING_ICON
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

extern "C" fn exported_picker_providers() -> RVec<PickerProviderSpec> {
    UserLibraryImpl.picker_providers().into()
}

extern "C" fn exported_picker_provider_items(
    context: PickerProviderContext,
) -> ROption<RVec<PickerItemSpec>> {
    UserLibraryImpl
        .picker_provider_items(&context)
        .map(Into::into)
        .into()
}

extern "C" fn exported_picker_truncate_strategy() -> AbiPickerTruncateStrategy {
    UserLibraryImpl.picker_truncate_strategy().into()
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

extern "C" fn exported_acp_picker_items(context: AcpPickerContext) -> RVec<AcpPickerItemSpec> {
    UserLibraryImpl.acp_picker_items(&context).into()
}

extern "C" fn exported_db_browser_items(context: DbBrowserContext) -> RVec<DbBrowserItemSpec> {
    UserLibraryImpl.db_browser_items(&context).into()
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

extern "C" fn exported_pane_config() -> AbiPaneConfig {
    AbiPaneConfig::from_parts(
        UserLibraryImpl.pane_config(),
        UserLibraryImpl.workspace_dock_config(),
        UserLibraryImpl.markdown_pretty_config(),
        UserLibraryImpl.picker_layout(),
        UserLibraryImpl.rainbow_parens_config(),
    )
}

extern "C" fn exported_keymap_config() -> AbiKeymapConfig {
    UserLibraryImpl.keymap_config().into()
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

extern "C" fn exported_git_commit_template(snapshot: AbiGitStatusSnapshot) -> RVec<RString> {
    UserLibraryImpl
        .git_commit_template(&snapshot.into())
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

extern "C" fn exported_git_feature_spec() -> AbiGitFeatureSpec {
    UserLibraryImpl.git_feature_spec().into()
}

extern "C" fn exported_oil_feature_spec() -> AbiOilFeatureSpec {
    UserLibraryImpl.oil_feature_spec().into()
}

extern "C" fn exported_browser_feature_spec() -> AbiBrowserFeatureSpec {
    UserLibraryImpl.browser_feature_spec().into()
}

extern "C" fn exported_db_feature_spec() -> AbiDbFeatureSpec {
    UserLibraryImpl.db_feature_spec().into()
}

extern "C" fn exported_terminal_feature_spec() -> AbiTerminalFeatureSpec {
    UserLibraryImpl.terminal_feature_spec().into()
}

extern "C" fn exported_context_help_specs() -> RVec<AbiContextHelpSpec> {
    UserLibraryImpl
        .context_help_specs()
        .into_iter()
        .map(Into::into)
        .collect::<Vec<_>>()
        .into()
}

extern "C" fn exported_pdf_open_mode() -> AbiPdfOpenMode {
    UserLibraryImpl.pdf_open_mode().into()
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

fn statusline_context_from_abi(context: &AbiStatuslineContext) -> StatuslineContext<'_> {
    StatuslineContext {
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
    }
}

extern "C" fn exported_statusline_render(context: AbiStatuslineContext) -> RString {
    editor_plugin_api::encode_modeline(
        &UserLibraryImpl.modeline_segments(&statusline_context_from_abi(&context)),
    )
    .into()
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
        picker_providers: exported_picker_providers,
        picker_provider_items: exported_picker_provider_items,
        acp_clients: exported_acp_clients,
        acp_client_by_id: exported_acp_client_by_id,
        acp_picker_items: exported_acp_picker_items,
        db_browser_items: exported_db_browser_items,
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
        git_feature_spec: exported_git_feature_spec,
        oil_feature_spec: exported_oil_feature_spec,
        browser_feature_spec: exported_browser_feature_spec,
        db_feature_spec: exported_db_feature_spec,
        terminal_feature_spec: exported_terminal_feature_spec,
        context_help_specs: exported_context_help_specs,
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
        pdf_open_mode: exported_pdf_open_mode,
        pane_config_v1: exported_pane_config,
        picker_truncate_strategy_v1: exported_picker_truncate_strategy,
        keymap_config_v1: exported_keymap_config,
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
    use editor_buffer::TextBuffer;
    use editor_plugin_api::UserLibrary;
    use editor_syntax::{LanguageConfiguration, SyntaxRegistry};
    use std::{
        collections::BTreeSet,
        fs,
        path::{Path, PathBuf},
    };

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

    fn highlight_query_asset_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("crates")
            .join("volt")
            .join("assets")
            .join("grammars")
            .join("queries")
    }

    fn bundled_highlight_query(language_id: &str) -> String {
        let path = highlight_query_asset_root()
            .join(language_id)
            .join("highlights.scm");
        fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
    }

    fn query_capture_names(source: &str) -> BTreeSet<String> {
        let bytes = source.as_bytes();
        let mut capture_names = BTreeSet::new();
        let mut index = 0;
        while index < bytes.len() {
            match bytes[index] {
                b';' => {
                    while index < bytes.len() && bytes[index] != b'\n' {
                        index += 1;
                    }
                }
                b'"' => {
                    index += 1;
                    while index < bytes.len() {
                        if bytes[index] == b'\\' && index + 1 < bytes.len() {
                            index += 2;
                            continue;
                        }
                        let is_string_end = bytes[index] == b'"';
                        index += 1;
                        if is_string_end {
                            break;
                        }
                    }
                }
                b'@' => {
                    let start = index + 1;
                    let mut end = start;
                    while end < bytes.len()
                        && matches!(
                            bytes[end],
                            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'.' | b'_' | b'-'
                        )
                    {
                        end += 1;
                    }
                    if end > start {
                        capture_names.insert(source[start..end].to_owned());
                        index = end;
                    } else {
                        index += 1;
                    }
                }
                _ => {
                    index += 1;
                }
            }
        }
        capture_names
    }

    fn capture_requires_theme_token(capture_name: &str) -> bool {
        !capture_name.starts_with('_')
            && !matches!(
                capture_name,
                "spell" | "nospell" | "conceal" | "conceal_lines"
            )
    }

    fn registered_highlight_theme_tokens(languages: &[LanguageConfiguration]) -> BTreeSet<String> {
        let mut tokens = BTreeSet::new();
        for language in languages {
            let bundled_query = bundled_highlight_query(language.id());
            for query_source in
                std::iter::once(bundled_query.as_str()).chain(language.extra_highlight_query())
            {
                for capture_name in query_capture_names(query_source) {
                    if !capture_requires_theme_token(&capture_name) {
                        continue;
                    }
                    let theme_token = mapped_theme_token(language, &capture_name)
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("syntax.{capture_name}"));
                    tokens.insert(theme_token);
                }
            }
        }
        tokens
    }

    #[test]
    fn user_library_contains_unique_packages_with_behavior() {
        let packages = packages();
        let mut names = BTreeSet::new();
        for package in &packages {
            assert!(!package.name().is_empty());
            assert!(
                names.insert(package.name().to_owned()),
                "duplicate package `{}`",
                package.name()
            );
        }
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
    fn user_library_exports_acp_clients_by_id_consistently() {
        let library = UserLibraryImpl;
        let clients = library.acp_clients();
        let mut ids = BTreeSet::new();

        for client in &clients {
            assert!(!client.id.is_empty());
            assert!(
                ids.insert(client.id.clone()),
                "duplicate ACP client `{}`",
                client.id
            );

            let resolved = library
                .acp_client_by_id(&client.id)
                .expect("ACP client should round-trip by id");
            assert_eq!(resolved.id, client.id);
            assert_eq!(resolved.command, client.command);
            assert_eq!(resolved.args, client.args);
        }

        assert!(library.acp_client_by_id("__missing__").is_none());
    }

    #[test]
    fn user_library_exports_picker_providers() {
        let library = UserLibraryImpl;
        let providers = library.picker_providers();
        let ids = providers
            .iter()
            .map(|provider| provider.id())
            .collect::<BTreeSet<_>>();
        assert!(ids.contains("commands"));
        assert!(ids.contains("workspace.files"));
        assert!(ids.contains("themes"));
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
        let mut ids = BTreeSet::new();

        for language in &languages {
            assert!(!language.id().is_empty());
            assert!(
                ids.insert(language.id().to_owned()),
                "duplicate language `{}`",
                language.id()
            );
            assert_eq!(
                language_extensions(&languages, language.id()),
                Some(language.file_extensions().to_vec())
            );

            let grammar = language
                .grammar()
                .unwrap_or_else(|| panic!("language `{}` missing grammar metadata", language.id()));
            assert!(!grammar.repository_url().is_empty());
            assert!(!grammar.install_dir_name().is_empty());
            assert!(!grammar.symbol_name().is_empty());
        }
    }

    #[test]
    fn user_library_exports_lsp_and_dap_registrations() {
        let servers = language_servers();
        let adapters = debug_adapters();
        let mut server_ids = BTreeSet::new();
        for server in &servers {
            assert!(!server.id().is_empty());
            assert!(
                server_ids.insert(server.id().to_owned()),
                "duplicate language server `{}`",
                server.id()
            );
            assert!(!server.language_id().is_empty());
            assert!(!server.program().is_empty());
        }

        let mut adapter_ids = BTreeSet::new();
        for adapter in &adapters {
            assert!(!adapter.id().is_empty());
            assert!(
                adapter_ids.insert(adapter.id().to_owned()),
                "duplicate debug adapter `{}`",
                adapter.id()
            );
        }
    }

    #[test]
    fn user_library_exports_themes() {
        let themes = themes();
        let mut ids = BTreeSet::new();
        for theme in &themes {
            assert!(!theme.id().is_empty());
            assert!(
                ids.insert(theme.id().to_owned()),
                "duplicate theme `{}`",
                theme.id()
            );
        }
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
            "ui.current-line",
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
            "ui.workspace-dock.background",
            "ui.workspace-dock.foreground",
            "ui.workspace-dock.muted",
            "ui.workspace-dock.selection",
            "ui.workspace-dock.accent",
            "ui.statusline.foreground",
            "ui.statusline.inactive.foreground",
            "ui.statusline.mode",
            "ui.statusline.muted",
            "ui.modeline.foreground",
            "ui.modeline.muted",
            "ui.modeline.mode.normal.foreground",
            "ui.modeline.mode.normal.background",
            "ui.modeline.mode.insert.foreground",
            "ui.modeline.mode.insert.background",
            "ui.modeline.mode.replace.foreground",
            "ui.modeline.mode.replace.background",
            "ui.modeline.mode.visual.foreground",
            "ui.modeline.mode.visual.background",
            "ui.modeline.git.branch",
            "ui.modeline.git.added",
            "ui.modeline.git.removed",
            "ui.picker.background",
            "ui.picker.foreground",
            "ui.picker.muted",
            "ui.picker.subtle",
            "ui.picker.border",
            "ui.picker.selection",
            "ui.diagnostic.error",
            "ui.diagnostic.warning",
            "ui.diagnostic.info",
            "ui.line-number",
            "ui.line-number.current",
            "ui.pane.inactive",
            "ui.pane.border",
            "ui.pane.active-border",
            "ui.ghost-text",
            "ui.headerline",
            "ui.headerline.background",
            "ui.modal.scrim",
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
    fn user_library_themes_cover_rainbow_paren_tokens() {
        let themes = themes();
        const TOKENS: &[&str] = &[
            "rainbow.paren.depth.1",
            "rainbow.paren.depth.2",
            "rainbow.paren.depth.3",
            "rainbow.paren.depth.4",
            "rainbow.paren.depth.5",
            "rainbow.paren.depth.6",
            "rainbow.paren.depth.7",
            "rainbow.paren.depth.8",
            "rainbow.paren.depth.9",
            "rainbow.paren.unmatched",
            "rainbow.paren.mismatched",
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
    fn user_library_themes_cover_registered_highlight_query_tokens() {
        let languages = syntax_languages();
        let required_tokens = registered_highlight_theme_tokens(&languages);

        for theme in themes() {
            for token in &required_tokens {
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
        if std::env::var_os("VOLT_TEST_INSTALLED_GRAMMARS").is_none() {
            eprintln!(
                "skipping installed grammar query compile test; set VOLT_TEST_INSTALLED_GRAMMARS=1"
            );
            return;
        }
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

    #[test]
    fn every_installed_grammar_highlight_query_compiles() {
        if std::env::var_os("VOLT_TEST_INSTALLED_GRAMMARS").is_none() {
            eprintln!(
                "skipping installed grammar query compile test; set VOLT_TEST_INSTALLED_GRAMMARS=1"
            );
            return;
        }

        let languages = syntax_languages();
        let requested_languages = std::env::var("VOLT_TEST_GRAMMAR_ID").ok().map(|value| {
            value
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .collect::<BTreeSet<_>>()
        });
        let mut checked = Vec::new();
        let mut failures = Vec::new();
        for language in languages {
            if requested_languages
                .as_ref()
                .is_some_and(|requested| !requested.contains(language.id()))
            {
                continue;
            }
            let mut registry = SyntaxRegistry::new();
            registry
                .register_all(syntax_languages())
                .expect("registering syntax languages");
            if !registry
                .is_installed(language.id())
                .expect("checking grammar install")
            {
                continue;
            }
            eprintln!("checking installed grammar highlight: {}", language.id());
            let snapshot = match registry
                .highlight_buffer_for_language(language.id(), &TextBuffer::from_text("value = 1\n"))
            {
                Ok(snapshot) => snapshot,
                Err(error) => {
                    failures.push(format!("{}: {error}", language.id()));
                    std::mem::forget(registry);
                    continue;
                }
            };
            assert_eq!(snapshot.language_id, language.id());
            checked.push(language.id().to_owned());
            // Runtime syntax registries live for the worker's lifetime. Keep this installed-grammar
            // stress test equivalent; unloading tree-sitter DLL/query state mid-process is unsafe.
            std::mem::forget(registry);
        }
        assert!(
            failures.is_empty(),
            "installed grammar highlight failures:\n{}",
            failures.join("\n")
        );
        assert!(!checked.is_empty(), "no installed grammars were checked");
        eprintln!(
            "checked installed grammar highlights: {}",
            checked.join(", ")
        );
    }

    #[test]
    fn recompile_installed_tree_sitter_grammars() {
        if std::env::var_os("VOLT_RECOMPILE_INSTALLED_GRAMMARS").is_none() {
            eprintln!(
                "skipping installed grammar recompile; set VOLT_RECOMPILE_INSTALLED_GRAMMARS=1"
            );
            return;
        }

        let mut registry = SyntaxRegistry::new();
        registry
            .register_all(syntax_languages())
            .expect("registering syntax languages");
        let report = registry.recompile_installed_languages_best_effort();
        eprintln!(
            "recompiled installed grammars: {}",
            report.recompiled().join(", ")
        );
        if !report.failed().is_empty() {
            eprintln!(
                "failed installed grammars: {}",
                report
                    .failed()
                    .iter()
                    .map(|failure| format!("{} ({})", failure.language_id(), failure.message()))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
}
