/// Newtype wrapper so `Arc<dyn UserLibrary>` can be stored in the runtime's
/// type-erased service map.
struct UserLibraryService(Arc<dyn UserLibrary>);

#[derive(Debug, Default)]
struct UserLibraryReloadState {
    last_staged_path: Option<PathBuf>,
}

/// Returns a clone of the user library stored in the runtime service map.
fn shell_user_library(runtime: &EditorRuntime) -> Arc<dyn UserLibrary> {
    runtime
        .services()
        .get::<UserLibraryService>()
        .expect("UserLibraryService not registered in runtime")
        .0
        .clone()
}

struct DynamicUserLibrary {
    module: UserLibraryModuleRef,
    icon_symbols: &'static [editor_icons::IconFontSymbol],
}

impl DynamicUserLibrary {
    fn load_from_file(path: &Path) -> Result<Arc<dyn UserLibrary>, String> {
        let module =
            UserLibraryModuleRef::load_from_file(path).map_err(|error| error.to_string())?;
        let icon_symbols = module.icon_symbols()()
            .into_iter()
            .map(editor_icons::IconFontSymbol::from)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Ok(Arc::new(Self {
            module,
            icon_symbols: Box::leak(icon_symbols),
        }))
    }
}

impl UserLibrary for DynamicUserLibrary {
    fn packages(&self) -> Vec<editor_plugin_api::PluginPackage> {
        self.module.packages()().into_iter().collect()
    }

    fn themes(&self) -> Vec<editor_theme::Theme> {
        self.module.themes()().into_iter().map(Into::into).collect()
    }

    fn syntax_languages(&self) -> Vec<editor_syntax::LanguageConfiguration> {
        self.module.syntax_languages()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn language_servers(&self) -> Vec<editor_lsp::LanguageServerSpec> {
        self.module.language_servers()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn debug_adapters(&self) -> Vec<editor_dap::DebugAdapterSpec> {
        self.module.debug_adapters()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn autocomplete_providers(&self) -> Vec<editor_plugin_api::AutocompleteProvider> {
        self.module.autocomplete_providers()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn autocomplete_result_limit(&self) -> usize {
        self.module.autocomplete_result_limit()()
    }

    fn autocomplete_token_icon(&self) -> &'static str {
        self.module.autocomplete_token_icon()().as_str()
    }

    fn hover_providers(&self) -> Vec<editor_plugin_api::HoverProvider> {
        self.module.hover_providers()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn hover_line_limit(&self) -> usize {
        self.module.hover_line_limit()()
    }

    fn hover_token_icon(&self) -> &'static str {
        self.module.hover_token_icon()().as_str()
    }

    fn hover_signature_icon(&self) -> &'static str {
        self.module.hover_signature_icon()().as_str()
    }

    fn picker_providers(&self) -> Vec<editor_plugin_api::PickerProviderSpec> {
        self.module.picker_providers()().into_iter().collect()
    }

    fn picker_provider_items(
        &self,
        context: &editor_plugin_api::PickerProviderContext,
    ) -> Option<Vec<editor_plugin_api::PickerItemSpec>> {
        self.module.picker_provider_items()(context.clone())
            .into_option()
            .map(|items| items.into_iter().collect())
    }

    fn picker_truncate_strategy(&self) -> editor_plugin_api::PickerTruncateStrategy {
        self.module.picker_truncate_strategy_v1()().into()
    }

    fn picker_layout(&self) -> editor_plugin_api::PickerLayout {
        self.module.pane_config_v1()().picker_layout()
    }

    fn acp_clients(&self) -> Vec<editor_plugin_api::AcpClient> {
        self.module.acp_clients()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn acp_client_by_id(&self, id: &str) -> Option<editor_plugin_api::AcpClient> {
        self.module.acp_client_by_id()(id.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn acp_picker_items(
        &self,
        context: &editor_plugin_api::AcpPickerContext,
    ) -> Vec<editor_plugin_api::AcpPickerItemSpec> {
        self.module.acp_picker_items()(context.clone())
            .into_iter()
            .collect()
    }

    fn db_browser_items(
        &self,
        context: &editor_plugin_api::DbBrowserContext,
    ) -> Vec<editor_plugin_api::DbBrowserItemSpec> {
        self.module.db_browser_items()(context.clone())
            .into_iter()
            .collect()
    }

    fn workspace_roots(&self) -> Vec<editor_plugin_api::WorkspaceRoot> {
        self.module.workspace_roots()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn terminal_config(&self) -> editor_plugin_api::TerminalConfig {
        self.module.terminal_config()().into()
    }

    fn commandline_enabled(&self) -> bool {
        self.module.commandline_enabled()()
    }

    fn pane_config(&self) -> editor_plugin_api::PaneConfig {
        self.module.pane_config_v1()().pane_config()
    }

    fn workspace_dock_config(&self) -> editor_plugin_api::WorkspaceDockConfig {
        self.module.pane_config_v1()().workspace_dock_config()
    }

    fn markdown_pretty_config(&self) -> editor_plugin_api::MarkdownPrettyConfig {
        self.module.pane_config_v1()().markdown_pretty_config()
    }

    fn keymap_config(&self) -> editor_plugin_api::KeymapConfig {
        self.module.keymap_config_v1()().into()
    }

    fn ligature_config(&self) -> editor_plugin_api::LigatureConfig {
        self.module.ligature_config_v1()().into()
    }

    fn rainbow_parens_config(&self) -> editor_plugin_api::RainbowParensConfig {
        self.module.pane_config_v1()().rainbow_parens_config()
    }

    fn show_paren_config(&self) -> editor_plugin_api::ShowParenConfig {
        self.module.pane_config_v1()().show_paren_config()
    }

    fn oil_defaults(&self) -> editor_plugin_api::OilDefaults {
        self.module.oil_defaults()().into()
    }

    fn oil_keybindings(&self) -> editor_plugin_api::OilKeybindings {
        self.module.oil_keybindings()().into()
    }

    fn oil_keydown_action(&self, chord: &str) -> Option<editor_plugin_api::OilKeyAction> {
        self.module.oil_keydown_action()(chord.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn oil_chord_action(
        &self,
        had_prefix: bool,
        chord: &str,
    ) -> Option<editor_plugin_api::OilKeyAction> {
        self.module.oil_chord_action()(had_prefix, chord.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn oil_help_lines(&self) -> Vec<String> {
        self.module.oil_help_lines()()
            .into_iter()
            .map(|line| line.into_string())
            .collect()
    }

    fn oil_directory_sections(
        &self,
        root: &Path,
        entries: &[editor_fs::DirectoryEntry],
        show_hidden: bool,
        sort_mode: editor_plugin_api::OilSortMode,
        trash_enabled: bool,
    ) -> editor_core::SectionTree {
        let entries = entries
            .iter()
            .cloned()
            .map(AbiDirectoryEntry::from)
            .collect::<Vec<_>>();
        self.module.oil_directory_sections()(
            root.to_string_lossy().into_owned().into(),
            entries.into(),
            show_hidden,
            sort_mode.into(),
            trash_enabled,
        )
        .into()
    }

    fn oil_strip_entry_icon_prefix<'a>(&self, label: &'a str) -> &'a str {
        let stripped = self.module.oil_strip_entry_icon_prefix()(label.to_owned().into());
        if stripped.as_str() == label {
            label
        } else {
            label
                .find(stripped.as_str())
                .map(|start| &label[start..start + stripped.len()])
                .unwrap_or(label)
        }
    }

    fn git_status_sections(
        &self,
        snapshot: &editor_git::GitStatusSnapshot,
    ) -> editor_core::SectionTree {
        self.module.git_status_sections()(snapshot.clone().into()).into()
    }

    fn git_commit_template(&self, snapshot: &editor_git::GitStatusSnapshot) -> Vec<String> {
        self.module.git_commit_template()(snapshot.clone().into())
            .into_iter()
            .map(|line| line.into_string())
            .collect()
    }

    fn git_prefix_for_chord(&self, chord: &str) -> Option<editor_plugin_api::GitStatusPrefix> {
        self.module.git_prefix_for_chord()(chord.to_owned().into())
            .into_option()
            .map(Into::into)
    }

    fn git_command_for_chord(
        &self,
        prefix: Option<editor_plugin_api::GitStatusPrefix>,
        chord: &str,
    ) -> Option<&'static str> {
        let command = self.module.git_command_for_chord()(
            prefix.map(AbiGitStatusPrefix::from).into(),
            chord.to_owned().into(),
        );
        command.into_option().map(|command| command.as_str())
    }

    fn browser_buffer_lines(&self, url: Option<&str>) -> Vec<String> {
        let url = url.map(|value| value.to_owned().into());
        self.module.browser_buffer_lines()(url.into())
            .into_iter()
            .map(|line| line.into_string())
            .collect()
    }

    fn browser_input_hint(&self, url: Option<&str>) -> String {
        let url = url.map(|value| value.to_owned().into());
        self.module.browser_input_hint()(url.into()).into()
    }

    fn browser_url_prompt(&self) -> String {
        self.module.browser_url_prompt()().into()
    }

    fn browser_url_placeholder(&self) -> String {
        self.module.browser_url_placeholder()().into()
    }

    fn git_feature_spec(&self) -> editor_plugin_api::GitFeatureSpec {
        self.module.git_feature_spec()().into()
    }

    fn oil_feature_spec(&self) -> editor_plugin_api::OilFeatureSpec {
        self.module.oil_feature_spec()().into()
    }

    fn browser_feature_spec(&self) -> editor_plugin_api::BrowserFeatureSpec {
        self.module.browser_feature_spec()().into()
    }

    fn db_feature_spec(&self) -> editor_plugin_api::DbFeatureSpec {
        self.module.db_feature_spec()().into()
    }

    fn terminal_feature_spec(&self) -> editor_plugin_api::TerminalFeatureSpec {
        self.module.terminal_feature_spec()().into()
    }

    fn context_help_specs(&self) -> Vec<editor_plugin_api::ContextHelpSpec> {
        self.module.context_help_specs()()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn pdf_open_mode(&self) -> editor_plugin_api::PdfOpenMode {
        self.module.pdf_open_mode()().into()
    }

    fn ghost_text_lines(
        &self,
        context: &editor_plugin_api::GhostTextContext<'_>,
    ) -> Vec<editor_plugin_api::GhostTextLine> {
        self.module.ghost_text_lines()(AbiGhostTextContext::from(*context))
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn headerline_lines(&self, context: &editor_plugin_api::GhostTextContext<'_>) -> Vec<String> {
        self.module.headerline_lines()(AbiGhostTextContext::from(*context))
            .into_iter()
            .map(Into::into)
            .collect()
    }

    fn statusline_render(&self, context: &editor_plugin_api::StatuslineContext<'_>) -> String {
        flatten_modeline_text(&self.modeline_segments(context))
    }

    fn statusline_spans(
        &self,
        context: &editor_plugin_api::StatuslineContext<'_>,
    ) -> Vec<StatuslineSpan> {
        flatten_modeline_to_spans(&self.modeline_segments(context))
    }

    fn modeline_segments(
        &self,
        context: &editor_plugin_api::StatuslineContext<'_>,
    ) -> Vec<ModelineSegment> {
        decode_modeline(
            &self.module.statusline_render()(AbiStatuslineContext::from(*context)).into_string(),
        )
    }

    fn statusline_lsp_connected_icon(&self) -> &'static str {
        self.module.statusline_lsp_connected_icon()().as_str()
    }

    fn statusline_lsp_error_icon(&self) -> &'static str {
        self.module.statusline_lsp_error_icon()().as_str()
    }

    fn statusline_lsp_warning_icon(&self) -> &'static str {
        self.module.statusline_lsp_warning_icon()().as_str()
    }

    fn lsp_diagnostic_icon(&self) -> &'static str {
        self.module.lsp_diagnostic_icon()().as_str()
    }

    fn lsp_diagnostic_line_limit(&self) -> usize {
        self.module.lsp_diagnostic_line_limit()()
    }

    fn lsp_show_buffer_diagnostics(&self) -> bool {
        self.module.lsp_show_buffer_diagnostics()()
    }

    fn gitfringe_token_added(&self) -> &'static str {
        self.module.gitfringe_token_added()().as_str()
    }

    fn gitfringe_token_modified(&self) -> &'static str {
        self.module.gitfringe_token_modified()().as_str()
    }

    fn gitfringe_token_removed(&self) -> &'static str {
        self.module.gitfringe_token_removed()().as_str()
    }

    fn gitfringe_symbol(&self) -> &'static str {
        self.module.gitfringe_symbol()().as_str()
    }

    fn icon_symbols(&self) -> &'static [editor_icons::IconFontSymbol] {
        self.icon_symbols
    }

    fn run_plugin_buffer_evaluator(&self, handler: &str, input: &str) -> Vec<String> {
        self.module.run_plugin_buffer_evaluator()(
            handler.to_owned().into(),
            input.to_owned().into(),
        )
        .into_iter()
        .map(|line| line.into_string())
        .collect()
    }

    fn default_build_command(&self, language: &str) -> Option<String> {
        self.module.default_build_command()(language.to_owned().into())
            .into_option()
            .map(|command| command.into_string())
    }
}

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const THEME_DIRECTORY_PARTS: [&str; 2] = ["user", "themes"];
const THEME_FILE_EXTENSION: &str = "toml";
const THEME_SOURCE_SEARCH_DEPTH: usize = 6;
const USER_CONFIG_DIRECTORY_PARTS: [&str; 1] = ["user"];
const USER_CONFIG_FILE_NAME: &str = "config.yaml";
const THEME_SOURCE_POLL_INTERVAL: Duration = Duration::from_millis(250);
