use abi_stable::{
    StableAbi,
    library::RootModule,
    sabi_types::VersionStrings,
    std_types::{ROption, RStr, RString, RVec},
};
use editor_core::{Section, SectionAction, SectionItem, SectionTree};
use editor_dap::DebugAdapterSpec;
use editor_fs::{DirectoryEntry, DirectoryEntryKind};
use editor_git::{GitLogEntry, GitStashEntry, GitStatusSnapshot, RepositoryStatus, StatusEntry};
use editor_icons::{IconFontCategory, IconFontSymbol};
use editor_lsp::{LanguageServerRootStrategy, LanguageServerSpec};
use editor_syntax::{CaptureThemeMapping, GrammarSource, LanguageConfiguration};
use editor_theme::{Color, Theme, ThemeOption};

use crate::{
    AcpClient, AutocompleteProvider, AutocompleteProviderItem, GitStatusPrefix, HoverProvider,
    HoverProviderTopic, LigatureConfig, LspDiagnosticsInfo, OilDefaults, OilKeyAction,
    OilKeybindings, OilSortMode, StatuslineContext, TerminalConfig, WorkspaceRoot,
};

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiStringPair {
    key: RString,
    value: RString,
}

impl AbiStringPair {
    pub fn new(key: impl Into<RString>, value: impl Into<RString>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn key(&self) -> &str {
        self.key.as_str()
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub struct AbiColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Color> for AbiColor {
    fn from(value: Color) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

impl From<AbiColor> for Color {
    fn from(value: AbiColor) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, PartialEq, StableAbi)]
pub enum AbiThemeOption {
    Bool(bool),
    Number(f64),
    Text(RString),
}

impl From<ThemeOption> for AbiThemeOption {
    fn from(value: ThemeOption) -> Self {
        match value {
            ThemeOption::Bool(value) => Self::Bool(value),
            ThemeOption::Number(value) => Self::Number(value),
            ThemeOption::Text(value) => Self::Text(value.into()),
        }
    }
}

impl From<AbiThemeOption> for ThemeOption {
    fn from(value: AbiThemeOption) -> Self {
        match value {
            AbiThemeOption::Bool(value) => Self::Bool(value),
            AbiThemeOption::Number(value) => Self::Number(value),
            AbiThemeOption::Text(value) => Self::Text(value.into()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiThemeToken {
    token: RString,
    color: AbiColor,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, StableAbi)]
pub struct AbiThemeOptionEntry {
    option: RString,
    value: AbiThemeOption,
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, StableAbi)]
pub struct AbiTheme {
    id: RString,
    name: RString,
    tokens: RVec<AbiThemeToken>,
    options: RVec<AbiThemeOptionEntry>,
}

impl From<Theme> for AbiTheme {
    fn from(value: Theme) -> Self {
        Self {
            id: value.id().to_owned().into(),
            name: value.name().to_owned().into(),
            tokens: value
                .tokens()
                .iter()
                .map(|(token, color)| AbiThemeToken {
                    token: token.clone().into(),
                    color: (*color).into(),
                })
                .collect::<Vec<_>>()
                .into(),
            options: value
                .options()
                .iter()
                .map(|(option, value)| AbiThemeOptionEntry {
                    option: option.clone().into(),
                    value: value.clone().into(),
                })
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl From<AbiTheme> for Theme {
    fn from(value: AbiTheme) -> Self {
        let mut theme = Theme::new(value.id.into_string(), value.name.into_string());
        for token in value.tokens {
            theme = theme.with_token(token.token.into_string(), Color::from(token.color));
        }
        for option in value.options {
            theme = theme.with_option(option.option.into_string(), ThemeOption::from(option.value));
        }
        theme
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiCaptureThemeMapping {
    capture_name: RString,
    theme_token: RString,
}

impl From<CaptureThemeMapping> for AbiCaptureThemeMapping {
    fn from(value: CaptureThemeMapping) -> Self {
        Self {
            capture_name: value.capture_name().to_owned().into(),
            theme_token: value.theme_token().to_owned().into(),
        }
    }
}

impl From<AbiCaptureThemeMapping> for CaptureThemeMapping {
    fn from(value: AbiCaptureThemeMapping) -> Self {
        Self::new(
            value.capture_name.into_string(),
            value.theme_token.into_string(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiGrammarSource {
    repository_url: RString,
    grammar_dir: RString,
    source_dir: RString,
    install_dir_name: RString,
    symbol_name: RString,
}

impl From<GrammarSource> for AbiGrammarSource {
    fn from(value: GrammarSource) -> Self {
        Self {
            repository_url: value.repository_url().to_owned().into(),
            grammar_dir: value.grammar_dir().to_string_lossy().into_owned().into(),
            source_dir: value.source_dir().to_string_lossy().into_owned().into(),
            install_dir_name: value.install_dir_name().to_owned().into(),
            symbol_name: value.symbol_name().to_owned().into(),
        }
    }
}

impl From<AbiGrammarSource> for GrammarSource {
    fn from(value: AbiGrammarSource) -> Self {
        Self::new(
            value.repository_url.into_string(),
            value.grammar_dir.into_string(),
            value.source_dir.into_string(),
            value.install_dir_name.into_string(),
            value.symbol_name.into_string(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiLanguageConfiguration {
    id: RString,
    file_extensions: RVec<RString>,
    capture_mappings: RVec<AbiCaptureThemeMapping>,
    grammar: ROption<AbiGrammarSource>,
    extra_highlight_query: ROption<RString>,
    additional_highlight_languages: RVec<RString>,
}

impl From<LanguageConfiguration> for AbiLanguageConfiguration {
    fn from(value: LanguageConfiguration) -> Self {
        Self {
            id: value.id().to_owned().into(),
            file_extensions: value
                .file_extensions()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
            capture_mappings: value
                .capture_mappings()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            grammar: value.grammar().cloned().map(AbiGrammarSource::from).into(),
            extra_highlight_query: value
                .extra_highlight_query()
                .map(|query| RString::from(query.to_owned()))
                .into(),
            additional_highlight_languages: value
                .additional_highlight_languages()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
        }
    }
}

impl From<AbiLanguageConfiguration> for LanguageConfiguration {
    fn from(value: AbiLanguageConfiguration) -> Self {
        let language_id = value.id.clone();
        let grammar = value
            .grammar
            .into_option()
            .map(Into::into)
            .unwrap_or_else(|| {
                panic!(
                    "runtime-loaded user language `{}` must use LanguageConfiguration::from_grammar; static tree-sitter loaders are not supported across the shared-library ABI",
                    language_id.as_str()
                )
            });
        let mut language = LanguageConfiguration::from_grammar(
            value.id.into_string(),
            value.file_extensions.into_iter().map(RString::into_string),
            grammar,
            value
                .capture_mappings
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>(),
        );
        if let Some(query) = value.extra_highlight_query.into_option() {
            language = language.with_extra_highlight_query(query.into_string());
        }
        let additional = value
            .additional_highlight_languages
            .into_iter()
            .map(RString::into_string)
            .collect::<Vec<_>>();
        if !additional.is_empty() {
            language = language.with_additional_highlight_languages(additional);
        }
        language
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum AbiLanguageServerRootStrategy {
    Workspace,
    MarkersOrWorkspace,
}

impl From<LanguageServerRootStrategy> for AbiLanguageServerRootStrategy {
    fn from(value: LanguageServerRootStrategy) -> Self {
        match value {
            LanguageServerRootStrategy::Workspace => Self::Workspace,
            LanguageServerRootStrategy::MarkersOrWorkspace => Self::MarkersOrWorkspace,
        }
    }
}

impl From<AbiLanguageServerRootStrategy> for LanguageServerRootStrategy {
    fn from(value: AbiLanguageServerRootStrategy) -> Self {
        match value {
            AbiLanguageServerRootStrategy::Workspace => Self::Workspace,
            AbiLanguageServerRootStrategy::MarkersOrWorkspace => Self::MarkersOrWorkspace,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiLanguageServerSpec {
    id: RString,
    language_id: RString,
    file_extensions: RVec<RString>,
    document_language_ids: RVec<AbiStringPair>,
    program: RString,
    args: RVec<RString>,
    root_markers: RVec<RString>,
    root_strategy: AbiLanguageServerRootStrategy,
    env: RVec<AbiStringPair>,
}

impl From<LanguageServerSpec> for AbiLanguageServerSpec {
    fn from(value: LanguageServerSpec) -> Self {
        let document_language_ids = value
            .document_language_ids()
            .iter()
            .map(|(extension, language_id)| {
                AbiStringPair::new(extension.clone(), language_id.clone())
            })
            .collect::<Vec<_>>();
        let env = value
            .env()
            .iter()
            .map(|(key, value)| AbiStringPair::new(key.clone(), value.clone()))
            .collect::<Vec<_>>();
        Self {
            id: value.id().to_owned().into(),
            language_id: value.language_id().to_owned().into(),
            file_extensions: value
                .file_extensions()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
            document_language_ids: document_language_ids.into(),
            program: value.program().to_owned().into(),
            args: value
                .args()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
            root_markers: value
                .root_markers()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
            root_strategy: value.root_strategy().into(),
            env: env.into(),
        }
    }
}

impl From<AbiLanguageServerSpec> for LanguageServerSpec {
    fn from(value: AbiLanguageServerSpec) -> Self {
        let mut spec = LanguageServerSpec::new(
            value.id.into_string(),
            value.language_id.into_string(),
            value.file_extensions.into_iter().map(RString::into_string),
            value.program.into_string(),
            value.args.into_iter().map(RString::into_string),
        )
        .with_root_markers(value.root_markers.into_iter().map(RString::into_string))
        .with_root_strategy(value.root_strategy.into());
        let mappings = value
            .document_language_ids
            .into_iter()
            .map(|pair| (pair.key.into_string(), pair.value.into_string()))
            .collect::<Vec<_>>();
        if !mappings.is_empty() {
            spec = spec.with_document_language_ids(mappings);
        }
        for pair in value.env {
            spec = spec.with_env(pair.key.into_string(), pair.value.into_string());
        }
        spec
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiDebugAdapterSpec {
    id: RString,
    language_id: RString,
    file_extensions: RVec<RString>,
    program: RString,
    args: RVec<RString>,
}

impl From<DebugAdapterSpec> for AbiDebugAdapterSpec {
    fn from(value: DebugAdapterSpec) -> Self {
        Self {
            id: value.id().to_owned().into(),
            language_id: value.language_id().to_owned().into(),
            file_extensions: value
                .file_extensions()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
            program: value.program().to_owned().into(),
            args: value
                .args()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
        }
    }
}

impl From<AbiDebugAdapterSpec> for DebugAdapterSpec {
    fn from(value: AbiDebugAdapterSpec) -> Self {
        DebugAdapterSpec::new(
            value.id.into_string(),
            value.language_id.into_string(),
            value.file_extensions.into_iter().map(RString::into_string),
            value.program.into_string(),
            value.args.into_iter().map(RString::into_string),
        )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum AbiOilSortMode {
    TypeThenName,
    TypeThenNameDesc,
}

impl From<OilSortMode> for AbiOilSortMode {
    fn from(value: OilSortMode) -> Self {
        match value {
            OilSortMode::TypeThenName => Self::TypeThenName,
            OilSortMode::TypeThenNameDesc => Self::TypeThenNameDesc,
        }
    }
}

impl From<AbiOilSortMode> for OilSortMode {
    fn from(value: AbiOilSortMode) -> Self {
        match value {
            AbiOilSortMode::TypeThenName => Self::TypeThenName,
            AbiOilSortMode::TypeThenNameDesc => Self::TypeThenNameDesc,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum AbiOilKeyAction {
    OpenEntry,
    OpenVerticalSplit,
    OpenHorizontalSplit,
    OpenNewPane,
    PreviewEntry,
    Refresh,
    Close,
    StartPrefix,
    OpenParent,
    OpenWorkspaceRoot,
    SetRoot,
    ShowHelp,
    CycleSort,
    ToggleHidden,
    ToggleTrash,
    OpenExternal,
    SetTabLocalRoot,
}

impl From<OilKeyAction> for AbiOilKeyAction {
    fn from(value: OilKeyAction) -> Self {
        match value {
            OilKeyAction::OpenEntry => Self::OpenEntry,
            OilKeyAction::OpenVerticalSplit => Self::OpenVerticalSplit,
            OilKeyAction::OpenHorizontalSplit => Self::OpenHorizontalSplit,
            OilKeyAction::OpenNewPane => Self::OpenNewPane,
            OilKeyAction::PreviewEntry => Self::PreviewEntry,
            OilKeyAction::Refresh => Self::Refresh,
            OilKeyAction::Close => Self::Close,
            OilKeyAction::StartPrefix => Self::StartPrefix,
            OilKeyAction::OpenParent => Self::OpenParent,
            OilKeyAction::OpenWorkspaceRoot => Self::OpenWorkspaceRoot,
            OilKeyAction::SetRoot => Self::SetRoot,
            OilKeyAction::ShowHelp => Self::ShowHelp,
            OilKeyAction::CycleSort => Self::CycleSort,
            OilKeyAction::ToggleHidden => Self::ToggleHidden,
            OilKeyAction::ToggleTrash => Self::ToggleTrash,
            OilKeyAction::OpenExternal => Self::OpenExternal,
            OilKeyAction::SetTabLocalRoot => Self::SetTabLocalRoot,
        }
    }
}

impl From<AbiOilKeyAction> for OilKeyAction {
    fn from(value: AbiOilKeyAction) -> Self {
        match value {
            AbiOilKeyAction::OpenEntry => Self::OpenEntry,
            AbiOilKeyAction::OpenVerticalSplit => Self::OpenVerticalSplit,
            AbiOilKeyAction::OpenHorizontalSplit => Self::OpenHorizontalSplit,
            AbiOilKeyAction::OpenNewPane => Self::OpenNewPane,
            AbiOilKeyAction::PreviewEntry => Self::PreviewEntry,
            AbiOilKeyAction::Refresh => Self::Refresh,
            AbiOilKeyAction::Close => Self::Close,
            AbiOilKeyAction::StartPrefix => Self::StartPrefix,
            AbiOilKeyAction::OpenParent => Self::OpenParent,
            AbiOilKeyAction::OpenWorkspaceRoot => Self::OpenWorkspaceRoot,
            AbiOilKeyAction::SetRoot => Self::SetRoot,
            AbiOilKeyAction::ShowHelp => Self::ShowHelp,
            AbiOilKeyAction::CycleSort => Self::CycleSort,
            AbiOilKeyAction::ToggleHidden => Self::ToggleHidden,
            AbiOilKeyAction::ToggleTrash => Self::ToggleTrash,
            AbiOilKeyAction::OpenExternal => Self::OpenExternal,
            AbiOilKeyAction::SetTabLocalRoot => Self::SetTabLocalRoot,
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum AbiGitStatusPrefix {
    Commit,
    Push,
    Fetch,
    Pull,
    Branch,
    Diff,
    Log,
    Stash,
    Merge,
    Rebase,
    CherryPick,
    Revert,
    Reset,
}

impl From<GitStatusPrefix> for AbiGitStatusPrefix {
    fn from(value: GitStatusPrefix) -> Self {
        match value {
            GitStatusPrefix::Commit => Self::Commit,
            GitStatusPrefix::Push => Self::Push,
            GitStatusPrefix::Fetch => Self::Fetch,
            GitStatusPrefix::Pull => Self::Pull,
            GitStatusPrefix::Branch => Self::Branch,
            GitStatusPrefix::Diff => Self::Diff,
            GitStatusPrefix::Log => Self::Log,
            GitStatusPrefix::Stash => Self::Stash,
            GitStatusPrefix::Merge => Self::Merge,
            GitStatusPrefix::Rebase => Self::Rebase,
            GitStatusPrefix::CherryPick => Self::CherryPick,
            GitStatusPrefix::Revert => Self::Revert,
            GitStatusPrefix::Reset => Self::Reset,
        }
    }
}

impl From<AbiGitStatusPrefix> for GitStatusPrefix {
    fn from(value: AbiGitStatusPrefix) -> Self {
        match value {
            AbiGitStatusPrefix::Commit => Self::Commit,
            AbiGitStatusPrefix::Push => Self::Push,
            AbiGitStatusPrefix::Fetch => Self::Fetch,
            AbiGitStatusPrefix::Pull => Self::Pull,
            AbiGitStatusPrefix::Branch => Self::Branch,
            AbiGitStatusPrefix::Diff => Self::Diff,
            AbiGitStatusPrefix::Log => Self::Log,
            AbiGitStatusPrefix::Stash => Self::Stash,
            AbiGitStatusPrefix::Merge => Self::Merge,
            AbiGitStatusPrefix::Rebase => Self::Rebase,
            AbiGitStatusPrefix::CherryPick => Self::CherryPick,
            AbiGitStatusPrefix::Revert => Self::Revert,
            AbiGitStatusPrefix::Reset => Self::Reset,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiAutocompleteProviderItem {
    pub label: RString,
    pub replacement: RString,
    pub detail: ROption<RString>,
    pub documentation: ROption<RString>,
}

impl From<AutocompleteProviderItem> for AbiAutocompleteProviderItem {
    fn from(value: AutocompleteProviderItem) -> Self {
        Self {
            label: value.label.into(),
            replacement: value.replacement.into(),
            detail: value.detail.map(Into::into).into(),
            documentation: value.documentation.map(Into::into).into(),
        }
    }
}

impl From<AbiAutocompleteProviderItem> for AutocompleteProviderItem {
    fn from(value: AbiAutocompleteProviderItem) -> Self {
        Self {
            label: value.label.into(),
            replacement: value.replacement.into(),
            detail: value.detail.into_option().map(RString::into_string),
            documentation: value.documentation.into_option().map(RString::into_string),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiAutocompleteProvider {
    pub id: RString,
    pub label: RString,
    pub icon: RString,
    pub item_icon: RString,
    pub or_group: ROption<RString>,
    pub buffer_kind: ROption<RString>,
    pub items: RVec<AbiAutocompleteProviderItem>,
}

impl From<AutocompleteProvider> for AbiAutocompleteProvider {
    fn from(value: AutocompleteProvider) -> Self {
        Self {
            id: value.id.into(),
            label: value.label.into(),
            icon: value.icon.into(),
            item_icon: value.item_icon.into(),
            or_group: value.or_group.map(Into::into).into(),
            buffer_kind: value.buffer_kind.map(Into::into).into(),
            items: value
                .items
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl From<AbiAutocompleteProvider> for AutocompleteProvider {
    fn from(value: AbiAutocompleteProvider) -> Self {
        Self {
            id: value.id.into(),
            label: value.label.into(),
            icon: value.icon.into(),
            item_icon: value.item_icon.into(),
            or_group: value.or_group.into_option().map(RString::into_string),
            buffer_kind: value.buffer_kind.into_option().map(RString::into_string),
            items: value.items.into_iter().map(Into::into).collect(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiHoverProviderTopic {
    pub token: RString,
    pub lines: RVec<RString>,
}

impl From<HoverProviderTopic> for AbiHoverProviderTopic {
    fn from(value: HoverProviderTopic) -> Self {
        Self {
            token: value.token.into(),
            lines: value
                .lines
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl From<AbiHoverProviderTopic> for HoverProviderTopic {
    fn from(value: AbiHoverProviderTopic) -> Self {
        Self {
            token: value.token.into(),
            lines: value.lines.into_iter().map(RString::into_string).collect(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiHoverProvider {
    pub id: RString,
    pub label: RString,
    pub icon: RString,
    pub line_limit: usize,
    pub buffer_kind: ROption<RString>,
    pub topics: RVec<AbiHoverProviderTopic>,
}

impl From<HoverProvider> for AbiHoverProvider {
    fn from(value: HoverProvider) -> Self {
        Self {
            id: value.id.into(),
            label: value.label.into(),
            icon: value.icon.into(),
            line_limit: value.line_limit,
            buffer_kind: value.buffer_kind.map(Into::into).into(),
            topics: value
                .topics
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl From<AbiHoverProvider> for HoverProvider {
    fn from(value: AbiHoverProvider) -> Self {
        Self {
            id: value.id.into(),
            label: value.label.into(),
            icon: value.icon.into(),
            line_limit: value.line_limit,
            buffer_kind: value.buffer_kind.into_option().map(RString::into_string),
            topics: value.topics.into_iter().map(Into::into).collect(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiAcpClient {
    pub id: RString,
    pub label: RString,
    pub command: RString,
    pub args: RVec<RString>,
    pub env: RVec<AbiStringPair>,
    pub cwd: ROption<RString>,
}

impl From<AcpClient> for AbiAcpClient {
    fn from(value: AcpClient) -> Self {
        Self {
            id: value.id.into(),
            label: value.label.into(),
            command: value.command.into(),
            args: value
                .args
                .into_iter()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
            env: value
                .env
                .into_iter()
                .map(|(key, value)| AbiStringPair::new(key, value))
                .collect::<Vec<_>>()
                .into(),
            cwd: value.cwd.map(Into::into).into(),
        }
    }
}

impl From<AbiAcpClient> for AcpClient {
    fn from(value: AbiAcpClient) -> Self {
        Self {
            id: value.id.into(),
            label: value.label.into(),
            command: value.command.into(),
            args: value.args.into_iter().map(RString::into_string).collect(),
            env: value
                .env
                .into_iter()
                .map(|pair| (pair.key.into_string(), pair.value.into_string()))
                .collect(),
            cwd: value.cwd.into_option().map(RString::into_string),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiWorkspaceRoot {
    pub path: RString,
    pub max_depth: usize,
}

impl From<WorkspaceRoot> for AbiWorkspaceRoot {
    fn from(value: WorkspaceRoot) -> Self {
        Self {
            path: value.path.into(),
            max_depth: value.max_depth,
        }
    }
}

impl From<AbiWorkspaceRoot> for WorkspaceRoot {
    fn from(value: AbiWorkspaceRoot) -> Self {
        Self {
            path: value.path.into(),
            max_depth: value.max_depth,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiTerminalConfig {
    pub program: RString,
    pub args: RVec<RString>,
}

impl From<TerminalConfig> for AbiTerminalConfig {
    fn from(value: TerminalConfig) -> Self {
        Self {
            program: value.program.into(),
            args: value
                .args
                .into_iter()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
        }
    }
}

impl From<AbiTerminalConfig> for TerminalConfig {
    fn from(value: AbiTerminalConfig) -> Self {
        Self {
            program: value.program.into(),
            args: value.args.into_iter().map(RString::into_string).collect(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub struct AbiLigatureConfig {
    pub enabled: bool,
}

impl From<LigatureConfig> for AbiLigatureConfig {
    fn from(value: LigatureConfig) -> Self {
        Self {
            enabled: value.enabled,
        }
    }
}

impl From<AbiLigatureConfig> for LigatureConfig {
    fn from(value: AbiLigatureConfig) -> Self {
        Self {
            enabled: value.enabled,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub struct AbiLspDiagnosticsInfo {
    pub errors: usize,
    pub warnings: usize,
}

impl From<LspDiagnosticsInfo> for AbiLspDiagnosticsInfo {
    fn from(value: LspDiagnosticsInfo) -> Self {
        Self {
            errors: value.errors,
            warnings: value.warnings,
        }
    }
}

impl From<AbiLspDiagnosticsInfo> for LspDiagnosticsInfo {
    fn from(value: AbiLspDiagnosticsInfo) -> Self {
        Self {
            errors: value.errors,
            warnings: value.warnings,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub struct AbiOilDefaults {
    pub show_hidden: bool,
    pub sort_mode: AbiOilSortMode,
    pub trash_enabled: bool,
}

impl From<OilDefaults> for AbiOilDefaults {
    fn from(value: OilDefaults) -> Self {
        Self {
            show_hidden: value.show_hidden,
            sort_mode: value.sort_mode.into(),
            trash_enabled: value.trash_enabled,
        }
    }
}

impl From<AbiOilDefaults> for OilDefaults {
    fn from(value: AbiOilDefaults) -> Self {
        Self {
            show_hidden: value.show_hidden,
            sort_mode: value.sort_mode.into(),
            trash_enabled: value.trash_enabled,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub struct AbiOilKeybindings {
    pub open_entry: RStr<'static>,
    pub open_vertical_split: RStr<'static>,
    pub open_horizontal_split: RStr<'static>,
    pub open_new_pane: RStr<'static>,
    pub preview_entry: RStr<'static>,
    pub refresh: RStr<'static>,
    pub close: RStr<'static>,
    pub prefix: RStr<'static>,
    pub open_parent: RStr<'static>,
    pub open_workspace_root: RStr<'static>,
    pub set_root: RStr<'static>,
    pub show_help: RStr<'static>,
    pub cycle_sort: RStr<'static>,
    pub toggle_hidden: RStr<'static>,
    pub toggle_trash: RStr<'static>,
    pub open_external: RStr<'static>,
    pub set_tab_local_root: RStr<'static>,
}

impl From<OilKeybindings> for AbiOilKeybindings {
    fn from(value: OilKeybindings) -> Self {
        Self {
            open_entry: RStr::from_str(value.open_entry),
            open_vertical_split: RStr::from_str(value.open_vertical_split),
            open_horizontal_split: RStr::from_str(value.open_horizontal_split),
            open_new_pane: RStr::from_str(value.open_new_pane),
            preview_entry: RStr::from_str(value.preview_entry),
            refresh: RStr::from_str(value.refresh),
            close: RStr::from_str(value.close),
            prefix: RStr::from_str(value.prefix),
            open_parent: RStr::from_str(value.open_parent),
            open_workspace_root: RStr::from_str(value.open_workspace_root),
            set_root: RStr::from_str(value.set_root),
            show_help: RStr::from_str(value.show_help),
            cycle_sort: RStr::from_str(value.cycle_sort),
            toggle_hidden: RStr::from_str(value.toggle_hidden),
            toggle_trash: RStr::from_str(value.toggle_trash),
            open_external: RStr::from_str(value.open_external),
            set_tab_local_root: RStr::from_str(value.set_tab_local_root),
        }
    }
}

impl From<AbiOilKeybindings> for OilKeybindings {
    fn from(value: AbiOilKeybindings) -> Self {
        Self {
            open_entry: value.open_entry.as_str(),
            open_vertical_split: value.open_vertical_split.as_str(),
            open_horizontal_split: value.open_horizontal_split.as_str(),
            open_new_pane: value.open_new_pane.as_str(),
            preview_entry: value.preview_entry.as_str(),
            refresh: value.refresh.as_str(),
            close: value.close.as_str(),
            prefix: value.prefix.as_str(),
            open_parent: value.open_parent.as_str(),
            open_workspace_root: value.open_workspace_root.as_str(),
            set_root: value.set_root.as_str(),
            show_help: value.show_help.as_str(),
            cycle_sort: value.cycle_sort.as_str(),
            toggle_hidden: value.toggle_hidden.as_str(),
            toggle_trash: value.toggle_trash.as_str(),
            open_external: value.open_external.as_str(),
            set_tab_local_root: value.set_tab_local_root.as_str(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum AbiDirectoryEntryKind {
    File,
    Directory,
}

impl From<DirectoryEntryKind> for AbiDirectoryEntryKind {
    fn from(value: DirectoryEntryKind) -> Self {
        match value {
            DirectoryEntryKind::File => Self::File,
            DirectoryEntryKind::Directory => Self::Directory,
        }
    }
}

impl From<AbiDirectoryEntryKind> for DirectoryEntryKind {
    fn from(value: AbiDirectoryEntryKind) -> Self {
        match value {
            AbiDirectoryEntryKind::File => Self::File,
            AbiDirectoryEntryKind::Directory => Self::Directory,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiDirectoryEntry {
    name: RString,
    path: RString,
    kind: AbiDirectoryEntryKind,
}

impl From<DirectoryEntry> for AbiDirectoryEntry {
    fn from(value: DirectoryEntry) -> Self {
        Self {
            name: value.name().to_owned().into(),
            path: value.path().to_string_lossy().into_owned().into(),
            kind: value.kind().into(),
        }
    }
}

impl From<AbiDirectoryEntry> for DirectoryEntry {
    fn from(value: AbiDirectoryEntry) -> Self {
        Self::new(
            value.name.into_string(),
            value.path.into_string(),
            value.kind.into(),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiSectionAction {
    id: RString,
    detail: ROption<RString>,
}

impl From<SectionAction> for AbiSectionAction {
    fn from(value: SectionAction) -> Self {
        Self {
            id: value.id().to_owned().into(),
            detail: value
                .detail()
                .map(|detail| RString::from(detail.to_owned()))
                .into(),
        }
    }
}

impl From<AbiSectionAction> for SectionAction {
    fn from(value: AbiSectionAction) -> Self {
        match value.detail.into_option() {
            Some(detail) => {
                SectionAction::new(value.id.into_string()).with_detail(detail.into_string())
            }
            None => SectionAction::new(value.id.into_string()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiSectionItem {
    text: RString,
    action: ROption<AbiSectionAction>,
}

impl From<SectionItem> for AbiSectionItem {
    fn from(value: SectionItem) -> Self {
        Self {
            text: value.text().to_owned().into(),
            action: value.action().cloned().map(Into::into).into(),
        }
    }
}

impl From<AbiSectionItem> for SectionItem {
    fn from(value: AbiSectionItem) -> Self {
        match value.action.into_option() {
            Some(action) => SectionItem::new(value.text.into_string()).with_action(action.into()),
            None => SectionItem::new(value.text.into_string()),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiSection {
    id: RString,
    title: RString,
    items: RVec<AbiSectionItem>,
    children: RVec<AbiSection>,
}

impl From<Section> for AbiSection {
    fn from(value: Section) -> Self {
        Self {
            id: value.id().to_owned().into(),
            title: value.title().to_owned().into(),
            items: value
                .items()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            children: value
                .children()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl From<AbiSection> for Section {
    fn from(value: AbiSection) -> Self {
        Section::new(value.id.into_string(), value.title.into_string())
            .with_items(value.items.into_iter().map(Into::into).collect())
            .with_children(value.children.into_iter().map(Into::into).collect())
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Default, StableAbi)]
pub struct AbiSectionTree {
    sections: RVec<AbiSection>,
}

impl From<SectionTree> for AbiSectionTree {
    fn from(value: SectionTree) -> Self {
        Self {
            sections: value
                .sections()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        }
    }
}

impl From<AbiSectionTree> for SectionTree {
    fn from(value: AbiSectionTree) -> Self {
        SectionTree::new(value.sections.into_iter().map(Into::into).collect())
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiStatusEntry {
    path: RString,
    index_status: u32,
    worktree_status: u32,
}

impl From<StatusEntry> for AbiStatusEntry {
    fn from(value: StatusEntry) -> Self {
        Self {
            path: value.path().to_owned().into(),
            index_status: value.index_status() as u32,
            worktree_status: value.worktree_status() as u32,
        }
    }
}

impl From<AbiStatusEntry> for StatusEntry {
    fn from(value: AbiStatusEntry) -> Self {
        StatusEntry::new(
            value.path.into_string(),
            char::from_u32(value.index_status).unwrap_or('?'),
            char::from_u32(value.worktree_status).unwrap_or('?'),
        )
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiGitLogEntry {
    hash: RString,
    summary: RString,
}

impl From<GitLogEntry> for AbiGitLogEntry {
    fn from(value: GitLogEntry) -> Self {
        Self {
            hash: value.hash().to_owned().into(),
            summary: value.summary().to_owned().into(),
        }
    }
}

impl From<AbiGitLogEntry> for GitLogEntry {
    fn from(value: AbiGitLogEntry) -> Self {
        GitLogEntry::new(value.hash.into_string(), value.summary.into_string())
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiGitStashEntry {
    name: RString,
    summary: RString,
}

impl From<GitStashEntry> for AbiGitStashEntry {
    fn from(value: GitStashEntry) -> Self {
        Self {
            name: value.name().to_owned().into(),
            summary: value.summary().to_owned().into(),
        }
    }
}

impl From<AbiGitStashEntry> for GitStashEntry {
    fn from(value: AbiGitStashEntry) -> Self {
        GitStashEntry::new(value.name.into_string(), value.summary.into_string())
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, Default, StableAbi)]
pub struct AbiGitStatusSnapshot {
    branch: ROption<RString>,
    upstream: ROption<RString>,
    push_remote: ROption<RString>,
    ahead: usize,
    behind: usize,
    head: ROption<AbiGitLogEntry>,
    staged: RVec<AbiStatusEntry>,
    unstaged: RVec<AbiStatusEntry>,
    untracked: RVec<RString>,
    stashes: RVec<AbiGitStashEntry>,
    unpulled: RVec<AbiGitLogEntry>,
    unpushed: RVec<AbiGitLogEntry>,
    recent: RVec<AbiGitLogEntry>,
    in_progress: RVec<RString>,
}

impl From<GitStatusSnapshot> for AbiGitStatusSnapshot {
    fn from(value: GitStatusSnapshot) -> Self {
        Self {
            branch: value
                .branch()
                .map(|value| RString::from(value.to_owned()))
                .into(),
            upstream: value
                .upstream()
                .map(|value| RString::from(value.to_owned()))
                .into(),
            push_remote: value
                .push_remote()
                .map(|value| RString::from(value.to_owned()))
                .into(),
            ahead: value.ahead(),
            behind: value.behind(),
            head: value.head().cloned().map(Into::into).into(),
            staged: value
                .staged()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            unstaged: value
                .unstaged()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            untracked: value
                .untracked()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
            stashes: value
                .stashes()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            unpulled: value
                .unpulled()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            unpushed: value
                .unpushed()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            recent: value
                .recent()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
            in_progress: value
                .in_progress()
                .iter()
                .cloned()
                .map(Into::into)
                .collect::<Vec<RString>>()
                .into(),
        }
    }
}

impl From<AbiGitStatusSnapshot> for GitStatusSnapshot {
    fn from(value: AbiGitStatusSnapshot) -> Self {
        let status = RepositoryStatus::new(
            value.branch.clone().into_option().map(RString::into_string),
            value.ahead,
            value.behind,
            value.staged.clone().into_iter().map(Into::into).collect(),
            value.unstaged.clone().into_iter().map(Into::into).collect(),
            value
                .untracked
                .clone()
                .into_iter()
                .map(RString::into_string)
                .collect(),
        );
        GitStatusSnapshot::default()
            .with_status(status)
            .with_head(value.head.into_option().map(Into::into))
            .with_upstreams(
                value.upstream.into_option().map(RString::into_string),
                value.push_remote.into_option().map(RString::into_string),
            )
            .with_stashes(value.stashes.into_iter().map(Into::into).collect())
            .with_unpulled(value.unpulled.into_iter().map(Into::into).collect())
            .with_unpushed(value.unpushed.into_iter().map(Into::into).collect())
            .with_recent(value.recent.into_iter().map(Into::into).collect())
            .with_in_progress(
                value
                    .in_progress
                    .into_iter()
                    .map(RString::into_string)
                    .collect(),
            )
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub enum AbiIconFontCategory {
    Cod,
    Dev,
    Fa,
    Fae,
    Iec,
    Logos,
    Md,
    Oct,
    Ple,
    Pom,
    Seti,
    Weather,
}

impl From<IconFontCategory> for AbiIconFontCategory {
    fn from(value: IconFontCategory) -> Self {
        match value {
            IconFontCategory::Cod => Self::Cod,
            IconFontCategory::Dev => Self::Dev,
            IconFontCategory::Fa => Self::Fa,
            IconFontCategory::Fae => Self::Fae,
            IconFontCategory::Iec => Self::Iec,
            IconFontCategory::Logos => Self::Logos,
            IconFontCategory::Md => Self::Md,
            IconFontCategory::Oct => Self::Oct,
            IconFontCategory::Ple => Self::Ple,
            IconFontCategory::Pom => Self::Pom,
            IconFontCategory::Seti => Self::Seti,
            IconFontCategory::Weather => Self::Weather,
        }
    }
}

impl From<AbiIconFontCategory> for IconFontCategory {
    fn from(value: AbiIconFontCategory) -> Self {
        match value {
            AbiIconFontCategory::Cod => Self::Cod,
            AbiIconFontCategory::Dev => Self::Dev,
            AbiIconFontCategory::Fa => Self::Fa,
            AbiIconFontCategory::Fae => Self::Fae,
            AbiIconFontCategory::Iec => Self::Iec,
            AbiIconFontCategory::Logos => Self::Logos,
            AbiIconFontCategory::Md => Self::Md,
            AbiIconFontCategory::Oct => Self::Oct,
            AbiIconFontCategory::Ple => Self::Ple,
            AbiIconFontCategory::Pom => Self::Pom,
            AbiIconFontCategory::Seti => Self::Seti,
            AbiIconFontCategory::Weather => Self::Weather,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, StableAbi)]
pub struct AbiIconFontSymbol {
    pub name: RStr<'static>,
    pub glyph: RStr<'static>,
    pub category: AbiIconFontCategory,
}

impl From<IconFontSymbol> for AbiIconFontSymbol {
    fn from(value: IconFontSymbol) -> Self {
        Self {
            name: RStr::from_str(value.name),
            glyph: RStr::from_str(value.glyph),
            category: value.category.into(),
        }
    }
}

impl From<AbiIconFontSymbol> for IconFontSymbol {
    fn from(value: AbiIconFontSymbol) -> Self {
        Self {
            name: value.name.as_str(),
            glyph: value.glyph.as_str(),
            category: value.category.into(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, StableAbi)]
pub struct AbiStatuslineContext {
    pub vim_mode: RString,
    pub recording_macro: ROption<u32>,
    pub workspace_name: RString,
    pub buffer_name: RString,
    pub buffer_modified: bool,
    pub language_id: ROption<RString>,
    pub line: usize,
    pub column: usize,
    pub lsp_server: ROption<RString>,
    pub lsp_diagnostics: ROption<AbiLspDiagnosticsInfo>,
    pub acp_connected: bool,
    pub git_branch: ROption<RString>,
    pub git_added: usize,
    pub git_removed: usize,
}

impl From<StatuslineContext<'_>> for AbiStatuslineContext {
    fn from(value: StatuslineContext<'_>) -> Self {
        Self {
            vim_mode: value.vim_mode.to_owned().into(),
            recording_macro: value.recording_macro.map(|value| value as u32).into(),
            workspace_name: value.workspace_name.to_owned().into(),
            buffer_name: value.buffer_name.to_owned().into(),
            buffer_modified: value.buffer_modified,
            language_id: value
                .language_id
                .map(|value| RString::from(value.to_owned()))
                .into(),
            line: value.line,
            column: value.column,
            lsp_server: value
                .lsp_server
                .map(|value| RString::from(value.to_owned()))
                .into(),
            lsp_diagnostics: value.lsp_diagnostics.map(Into::into).into(),
            acp_connected: value.acp_connected,
            git_branch: value
                .git_branch
                .map(|value| RString::from(value.to_owned()))
                .into(),
            git_added: value.git_added,
            git_removed: value.git_removed,
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, StableAbi)]
#[sabi(kind(Prefix(prefix_ref = UserLibraryModuleRef, prefix_fields = UserLibraryModule_Prefix)))]
pub struct UserLibraryModule {
    pub packages: extern "C" fn() -> RVec<crate::PluginPackage>,
    pub themes: extern "C" fn() -> RVec<AbiTheme>,
    pub syntax_languages: extern "C" fn() -> RVec<AbiLanguageConfiguration>,
    pub language_servers: extern "C" fn() -> RVec<AbiLanguageServerSpec>,
    pub debug_adapters: extern "C" fn() -> RVec<AbiDebugAdapterSpec>,
    pub autocomplete_providers: extern "C" fn() -> RVec<AbiAutocompleteProvider>,
    pub autocomplete_result_limit: extern "C" fn() -> usize,
    pub autocomplete_token_icon: extern "C" fn() -> RStr<'static>,
    pub hover_providers: extern "C" fn() -> RVec<AbiHoverProvider>,
    pub hover_line_limit: extern "C" fn() -> usize,
    pub hover_token_icon: extern "C" fn() -> RStr<'static>,
    pub hover_signature_icon: extern "C" fn() -> RStr<'static>,
    pub acp_clients: extern "C" fn() -> RVec<AbiAcpClient>,
    pub acp_client_by_id: extern "C" fn(RString) -> ROption<AbiAcpClient>,
    pub workspace_roots: extern "C" fn() -> RVec<AbiWorkspaceRoot>,
    pub terminal_config: extern "C" fn() -> AbiTerminalConfig,
    pub commandline_enabled: extern "C" fn() -> bool,
    pub ligature_config: extern "C" fn() -> AbiLigatureConfig,
    pub oil_defaults: extern "C" fn() -> AbiOilDefaults,
    pub oil_keybindings: extern "C" fn() -> AbiOilKeybindings,
    pub oil_keydown_action: extern "C" fn(RString) -> ROption<AbiOilKeyAction>,
    pub oil_chord_action: extern "C" fn(bool, RString) -> ROption<AbiOilKeyAction>,
    pub oil_help_lines: extern "C" fn() -> RVec<RString>,
    pub oil_directory_sections: extern "C" fn(
        RString,
        RVec<AbiDirectoryEntry>,
        bool,
        AbiOilSortMode,
        bool,
    ) -> AbiSectionTree,
    pub oil_strip_entry_icon_prefix: extern "C" fn(RString) -> RString,
    pub git_status_sections: extern "C" fn(AbiGitStatusSnapshot) -> AbiSectionTree,
    pub git_commit_template: extern "C" fn() -> RVec<RString>,
    pub git_prefix_for_chord: extern "C" fn(RString) -> ROption<AbiGitStatusPrefix>,
    pub git_command_for_chord:
        extern "C" fn(ROption<AbiGitStatusPrefix>, RString) -> ROption<RStr<'static>>,
    pub browser_buffer_lines: extern "C" fn(ROption<RString>) -> RVec<RString>,
    pub browser_input_hint: extern "C" fn(ROption<RString>) -> RString,
    pub browser_url_prompt: extern "C" fn() -> RString,
    pub browser_url_placeholder: extern "C" fn() -> RString,
    pub statusline_render: extern "C" fn(AbiStatuslineContext) -> RString,
    pub statusline_lsp_connected_icon: extern "C" fn() -> RStr<'static>,
    pub statusline_lsp_error_icon: extern "C" fn() -> RStr<'static>,
    pub statusline_lsp_warning_icon: extern "C" fn() -> RStr<'static>,
    pub lsp_diagnostic_icon: extern "C" fn() -> RStr<'static>,
    pub lsp_diagnostic_line_limit: extern "C" fn() -> usize,
    pub lsp_show_buffer_diagnostics: extern "C" fn() -> bool,
    pub gitfringe_token_added: extern "C" fn() -> RStr<'static>,
    pub gitfringe_token_modified: extern "C" fn() -> RStr<'static>,
    pub gitfringe_token_removed: extern "C" fn() -> RStr<'static>,
    pub gitfringe_symbol: extern "C" fn() -> RStr<'static>,
    pub icon_symbols: extern "C" fn() -> RVec<AbiIconFontSymbol>,
    pub run_plugin_buffer_evaluator: extern "C" fn(RString, RString) -> RVec<RString>,
    pub default_build_command: extern "C" fn(RString) -> ROption<RString>,
    pub ligature_config_v1: extern "C" fn() -> AbiLigatureConfig,
    #[sabi(last_prefix_field)]
    pub commandline_enabled_v1: extern "C" fn() -> bool,
}

impl RootModule for UserLibraryModuleRef {
    abi_stable::declare_root_module_statics! {UserLibraryModuleRef}
    const BASE_NAME: &'static str = "user";
    const NAME: &'static str = "user";
    const VERSION_STRINGS: VersionStrings = abi_stable::package_version_strings!();
}
