use editor_fs::ProjectSearchRoot;
use editor_plugin_api::{
    OilDefaults, OilKeybindings, OilSortMode, PaneConfig, PickerTruncateStrategy,
    WorkspaceDockConfig, WorkspaceDockSide,
};
use serde::Deserialize;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const CONFIG_FILE_NAME: &str = "config.yaml";
const CONFIG_DIRECTORY_PARTS: [&str; 1] = ["user"];
const CONFIG_SEARCH_DEPTH: usize = 6;

pub fn load() -> UserConfig {
    let Some(root_dir) = config_root_dir() else {
        return UserConfig::default();
    };
    load_from_root(&root_dir)
}

fn load_from_root(root_dir: &Path) -> UserConfig {
    let master_path = root_dir.join(CONFIG_FILE_NAME);
    let master = read_yaml::<MasterConfig>(&master_path).unwrap_or_default();
    let mut config = UserConfig::default();

    if let Some(path) = master.workspace.as_deref() {
        config.workspace = read_section::<WorkspaceSection>(root_dir, path).unwrap_or_default();
    }
    if let Some(path) = master.acp.as_deref() {
        config.acp = read_section::<AcpSection>(root_dir, path).unwrap_or_default();
    }
    if let Some(path) = master.ui.as_deref() {
        config.ui = read_section::<UiSection>(root_dir, path).unwrap_or_default();
    }
    if let Some(path) = master.oil.as_deref() {
        config.oil = read_section::<OilSection>(root_dir, path).unwrap_or_default();
    }

    config
}

pub fn config_root_dir() -> Option<PathBuf> {
    let exe_path = env::current_exe().ok()?;
    let exe_dir = exe_path.parent()?;
    config_root_dir_from_exe_dir(exe_dir)
}

fn config_root_dir_from_exe_dir(exe_dir: &Path) -> Option<PathBuf> {
    let mut fallback = None;
    for ancestor in exe_dir.ancestors().take(CONFIG_SEARCH_DEPTH) {
        let mut candidate = PathBuf::from(ancestor);
        for part in CONFIG_DIRECTORY_PARTS {
            candidate = candidate.join(part);
        }
        if !candidate.is_dir() {
            continue;
        }
        if ancestor.join("Cargo.toml").is_file() {
            return Some(candidate);
        }
        fallback.get_or_insert(candidate);
    }
    fallback
}

pub fn config_source_files() -> Vec<PathBuf> {
    let Some(root_dir) = config_root_dir() else {
        return Vec::new();
    };
    config_source_files_from_root(&root_dir)
}

fn config_source_files_from_root(root_dir: &Path) -> Vec<PathBuf> {
    let master_path = root_dir.join(CONFIG_FILE_NAME);
    let mut files = Vec::new();
    if master_path.is_file() {
        files.push(master_path.clone());
    }
    let Some(master) = read_yaml::<MasterConfig>(&master_path) else {
        files.sort();
        files.dedup();
        return files;
    };
    for relative in [master.workspace, master.acp, master.ui, master.oil]
        .into_iter()
        .flatten()
    {
        let path = root_dir.join(relative);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
    files
}

fn read_section<T>(root_dir: &Path, relative_path: &str) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    let path = root_dir.join(relative_path);
    read_yaml::<T>(&path)
}

fn read_yaml<T>(path: &Path) -> Option<T>
where
    T: for<'de> Deserialize<'de>,
{
    if !path.is_file() {
        return None;
    }
    let contents = fs::read_to_string(path).ok()?;
    match serde_yaml::from_str::<T>(&contents) {
        Ok(value) => Some(value),
        Err(error) => {
            eprintln!("failed to parse user config `{}`: {error}", path.display());
            None
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UserConfig {
    pub workspace: WorkspaceSection,
    pub acp: AcpSection,
    pub ui: UiSection,
    pub oil: OilSection,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct MasterConfig {
    workspace: Option<String>,
    acp: Option<String>,
    ui: Option<String>,
    oil: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct WorkspaceRootConfig {
    pub path: String,
    #[serde(default = "default_workspace_max_depth")]
    pub max_depth: usize,
}

const fn default_workspace_max_depth() -> usize {
    4
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct WorkspaceSection {
    #[serde(default = "default_project_search_roots")]
    pub project_search_roots: Vec<WorkspaceRootConfig>,
}

impl Default for WorkspaceSection {
    fn default() -> Self {
        Self {
            project_search_roots: default_project_search_roots(),
        }
    }
}

fn default_project_search_roots() -> Vec<WorkspaceRootConfig> {
    #[cfg(target_os = "windows")]
    {
        vec![
            WorkspaceRootConfig {
                path: r"P:\".to_owned(),
                max_depth: 4,
            },
            WorkspaceRootConfig {
                path: r"W:\".to_owned(),
                max_depth: 4,
            },
            WorkspaceRootConfig {
                path: r"C:\Users\sam\".to_owned(),
                max_depth: 4,
            },
        ]
    }

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "macos"
    ))]
    {
        vec![
            WorkspaceRootConfig {
                path: "~/projects".to_owned(),
                max_depth: 4,
            },
            WorkspaceRootConfig {
                path: "~/work".to_owned(),
                max_depth: 4,
            },
        ]
    }
}

impl WorkspaceSection {
    pub fn project_search_roots(&self) -> Vec<ProjectSearchRoot> {
        let configured = self
            .project_search_roots
            .iter()
            .map(|root| ProjectSearchRoot::new(root.path.as_str(), root.max_depth))
            .filter(|root| root.root().exists())
            .collect::<Vec<_>>();
        if !configured.is_empty() {
            return configured;
        }
        default_project_search_roots()
            .into_iter()
            .map(|root| ProjectSearchRoot::new(root.path, root.max_depth))
            .filter(|root| root.root().exists())
            .collect()
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AcpClientConfig {
    pub id: String,
    pub label: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: Vec<StringPair>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct StringPair {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct AcpSection {
    #[serde(default = "default_acp_clients")]
    pub clients: Vec<AcpClientConfig>,
}

impl Default for AcpSection {
    fn default() -> Self {
        Self {
            clients: default_acp_clients(),
        }
    }
}

fn default_acp_clients() -> Vec<AcpClientConfig> {
    vec![
        AcpClientConfig {
            id: "agent".to_owned(),
            label: "Cursor Agent(ACP)".to_owned(),
            command: "agent".to_owned(),
            args: vec!["acp".to_owned(), "--yolo".to_owned()],
            env: Vec::new(),
            cwd: None,
        },
        AcpClientConfig {
            id: "codex".to_owned(),
            label: "Codex (ACP)".to_owned(),
            command: "codex-acp".to_owned(),
            args: Vec::new(),
            env: Vec::new(),
            cwd: None,
        },
        AcpClientConfig {
            id: "copilot".to_owned(),
            label: "GitHub Copilot (ACP)".to_owned(),
            command: "copilot".to_owned(),
            args: vec![
                "--acp".to_owned(),
                "--stdio".to_owned(),
                "--yolo".to_owned(),
            ],
            env: Vec::new(),
            cwd: None,
        },
        AcpClientConfig {
            id: "opencode".to_owned(),
            label: "OpenCode (ACP)".to_owned(),
            command: "opencode".to_owned(),
            args: vec!["acp".to_owned()],
            env: Vec::new(),
            cwd: None,
        },
        AcpClientConfig {
            id: "pi".to_owned(),
            label: "Pi (ACP)".to_owned(),
            command: "node".to_owned(),
            args: vec![crate::acp::PI_ACP_LOCATION.to_owned()],
            env: Vec::new(),
            cwd: None,
        },
    ]
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct UiSection {
    #[serde(default = "default_picker_truncate_strategy")]
    pub picker_truncate_strategy: ConfigPickerTruncateStrategy,
    #[serde(default = "default_ligatures_enabled")]
    pub ligatures_enabled: bool,
    #[serde(default = "default_rainbow_parens_enabled")]
    pub rainbow_parens_enabled: bool,
    #[serde(default)]
    pub pane: PaneSection,
    #[serde(default)]
    pub workspace_dock: WorkspaceDockSection,
    #[serde(default)]
    pub terminal: TerminalSection,
    #[serde(default)]
    pub keymap: KeymapSection,
}

impl Default for UiSection {
    fn default() -> Self {
        Self {
            picker_truncate_strategy: default_picker_truncate_strategy(),
            ligatures_enabled: default_ligatures_enabled(),
            rainbow_parens_enabled: default_rainbow_parens_enabled(),
            pane: PaneSection::default(),
            workspace_dock: WorkspaceDockSection::default(),
            terminal: TerminalSection::default(),
            keymap: KeymapSection::default(),
        }
    }
}

/// UI keymap tunables (`ui.keymap.*`).
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct KeymapSection {
    /// Ambiguous-prefix timeout in milliseconds.
    #[serde(default = "default_ambiguous_prefix_timeout_ms")]
    pub ambiguous_prefix_timeout_ms: u64,
}

impl Default for KeymapSection {
    fn default() -> Self {
        Self {
            ambiguous_prefix_timeout_ms: default_ambiguous_prefix_timeout_ms(),
        }
    }
}

const fn default_ambiguous_prefix_timeout_ms() -> u64 {
    editor_core::DEFAULT_AMBIGUOUS_PREFIX_TIMEOUT_MS
}

fn default_picker_truncate_strategy() -> ConfigPickerTruncateStrategy {
    ConfigPickerTruncateStrategy::Auto
}

const fn default_ligatures_enabled() -> bool {
    true
}

const fn default_rainbow_parens_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigPickerTruncateStrategy {
    EndEllipsis,
    StartEllipsis,
    MiddleEllipsis,
    ShrinkDirectories,
    ShrinkAll,
    FileName,
    FileNameWithParent,
    ParentInitialFileName,
    ShrinkLeadingKeepTail,
    Full,
    Auto,
}

impl From<ConfigPickerTruncateStrategy> for PickerTruncateStrategy {
    fn from(value: ConfigPickerTruncateStrategy) -> Self {
        match value {
            ConfigPickerTruncateStrategy::EndEllipsis => Self::EndEllipsis,
            ConfigPickerTruncateStrategy::StartEllipsis => Self::StartEllipsis,
            ConfigPickerTruncateStrategy::MiddleEllipsis => Self::MiddleEllipsis,
            ConfigPickerTruncateStrategy::ShrinkDirectories => Self::ShrinkDirectories,
            ConfigPickerTruncateStrategy::ShrinkAll => Self::ShrinkAll,
            ConfigPickerTruncateStrategy::FileName => Self::FileName,
            ConfigPickerTruncateStrategy::FileNameWithParent => Self::FileNameWithParent,
            ConfigPickerTruncateStrategy::ParentInitialFileName => Self::ParentInitialFileName,
            ConfigPickerTruncateStrategy::ShrinkLeadingKeepTail => Self::ShrinkLeadingKeepTail,
            ConfigPickerTruncateStrategy::Full => Self::Full,
            ConfigPickerTruncateStrategy::Auto => Self::Auto,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PaneSection {
    #[serde(default = "default_pane_golden_ratio")]
    pub golden_ratio: bool,
}

impl Default for PaneSection {
    fn default() -> Self {
        Self {
            golden_ratio: default_pane_golden_ratio(),
        }
    }
}

const fn default_pane_golden_ratio() -> bool {
    true
}

impl PaneSection {
    pub fn pane_config(&self) -> PaneConfig {
        PaneConfig {
            golden_ratio: self.golden_ratio,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConfigWorkspaceDockSide {
    #[default]
    Left,
    Right,
}

impl From<ConfigWorkspaceDockSide> for WorkspaceDockSide {
    fn from(value: ConfigWorkspaceDockSide) -> Self {
        match value {
            ConfigWorkspaceDockSide::Left => Self::Left,
            ConfigWorkspaceDockSide::Right => Self::Right,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct WorkspaceDockSection {
    #[serde(default)]
    pub side: ConfigWorkspaceDockSide,
    #[serde(default = "default_workspace_dock_docked")]
    pub docked: bool,
}

impl Default for WorkspaceDockSection {
    fn default() -> Self {
        Self {
            side: ConfigWorkspaceDockSide::Left,
            docked: default_workspace_dock_docked(),
        }
    }
}

const fn default_workspace_dock_docked() -> bool {
    false
}

impl WorkspaceDockSection {
    pub fn workspace_dock_config(&self) -> WorkspaceDockConfig {
        WorkspaceDockConfig {
            side: self.side.into(),
            docked: self.docked,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TerminalSection {
    #[serde(default = "default_terminal_program")]
    pub program: String,
    #[serde(default = "default_terminal_args")]
    pub args: Vec<String>,
}

impl Default for TerminalSection {
    fn default() -> Self {
        Self {
            program: default_terminal_program(),
            args: default_terminal_args(),
        }
    }
}

fn default_terminal_program() -> String {
    crate::terminal::default_shell_program_fallback()
}

fn default_terminal_args() -> Vec<String> {
    crate::terminal::default_shell_args_fallback()
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
pub struct OilSection {
    #[serde(default)]
    pub defaults: OilDefaultsSection,
    #[serde(default)]
    pub keybindings: OilKeybindingsSection,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct OilDefaultsSection {
    #[serde(default)]
    pub show_hidden: bool,
    #[serde(default = "default_oil_sort_mode")]
    pub sort_mode: ConfigOilSortMode,
    #[serde(default)]
    pub trash_enabled: bool,
}

impl Default for OilDefaultsSection {
    fn default() -> Self {
        Self {
            show_hidden: false,
            sort_mode: default_oil_sort_mode(),
            trash_enabled: false,
        }
    }
}

fn default_oil_sort_mode() -> ConfigOilSortMode {
    ConfigOilSortMode::TypeThenName
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigOilSortMode {
    TypeThenName,
    TypeThenNameDesc,
}

impl From<ConfigOilSortMode> for OilSortMode {
    fn from(value: ConfigOilSortMode) -> Self {
        match value {
            ConfigOilSortMode::TypeThenName => Self::TypeThenName,
            ConfigOilSortMode::TypeThenNameDesc => Self::TypeThenNameDesc,
        }
    }
}

impl OilDefaultsSection {
    pub fn oil_defaults(&self) -> OilDefaults {
        OilDefaults {
            show_hidden: self.show_hidden,
            sort_mode: self.sort_mode.clone().into(),
            trash_enabled: self.trash_enabled,
        }
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct OilKeybindingsSection {
    #[serde(default = "default_oil_open_entry")]
    pub open_entry: String,
    #[serde(default = "default_oil_open_vertical_split")]
    pub open_vertical_split: String,
    #[serde(default = "default_oil_open_horizontal_split")]
    pub open_horizontal_split: String,
    #[serde(default = "default_oil_open_new_pane")]
    pub open_new_pane: String,
    #[serde(default = "default_oil_preview_entry")]
    pub preview_entry: String,
    #[serde(default = "default_oil_refresh")]
    pub refresh: String,
    #[serde(default = "default_oil_close")]
    pub close: String,
    #[serde(default = "default_oil_prefix")]
    pub prefix: String,
    #[serde(default = "default_oil_open_parent")]
    pub open_parent: String,
    #[serde(default = "default_oil_open_workspace_root")]
    pub open_workspace_root: String,
    #[serde(default = "default_oil_set_root")]
    pub set_root: String,
    #[serde(default = "default_oil_show_help")]
    pub show_help: String,
    #[serde(default = "default_oil_cycle_sort")]
    pub cycle_sort: String,
    #[serde(default = "default_oil_toggle_hidden")]
    pub toggle_hidden: String,
    #[serde(default = "default_oil_toggle_trash")]
    pub toggle_trash: String,
    #[serde(default = "default_oil_open_external")]
    pub open_external: String,
    #[serde(default = "default_oil_set_tab_local_root")]
    pub set_tab_local_root: String,
    #[serde(default = "default_oil_create_git_worktree")]
    pub create_git_worktree: String,
}

impl Default for OilKeybindingsSection {
    fn default() -> Self {
        Self {
            open_entry: default_oil_open_entry(),
            open_vertical_split: default_oil_open_vertical_split(),
            open_horizontal_split: default_oil_open_horizontal_split(),
            open_new_pane: default_oil_open_new_pane(),
            preview_entry: default_oil_preview_entry(),
            refresh: default_oil_refresh(),
            close: default_oil_close(),
            prefix: default_oil_prefix(),
            open_parent: default_oil_open_parent(),
            open_workspace_root: default_oil_open_workspace_root(),
            set_root: default_oil_set_root(),
            show_help: default_oil_show_help(),
            cycle_sort: default_oil_cycle_sort(),
            toggle_hidden: default_oil_toggle_hidden(),
            toggle_trash: default_oil_toggle_trash(),
            open_external: default_oil_open_external(),
            set_tab_local_root: default_oil_set_tab_local_root(),
            create_git_worktree: default_oil_create_git_worktree(),
        }
    }
}

fn default_oil_open_entry() -> String {
    "Enter".to_owned()
}
fn default_oil_open_vertical_split() -> String {
    "Ctrl+\\".to_owned()
}
fn default_oil_open_horizontal_split() -> String {
    "Ctrl+|".to_owned()
}
fn default_oil_open_new_pane() -> String {
    "Ctrl+t".to_owned()
}
fn default_oil_preview_entry() -> String {
    "Ctrl+p".to_owned()
}
fn default_oil_refresh() -> String {
    "Ctrl+l".to_owned()
}
fn default_oil_close() -> String {
    "Ctrl+c".to_owned()
}
fn default_oil_prefix() -> String {
    "g".to_owned()
}
fn default_oil_open_parent() -> String {
    "-".to_owned()
}
fn default_oil_open_workspace_root() -> String {
    "_".to_owned()
}
fn default_oil_set_root() -> String {
    "`".to_owned()
}
fn default_oil_show_help() -> String {
    "?".to_owned()
}
fn default_oil_cycle_sort() -> String {
    "s".to_owned()
}
fn default_oil_toggle_hidden() -> String {
    ".".to_owned()
}
fn default_oil_toggle_trash() -> String {
    "\\".to_owned()
}
fn default_oil_open_external() -> String {
    "x".to_owned()
}
fn default_oil_set_tab_local_root() -> String {
    "~".to_owned()
}
fn default_oil_create_git_worktree() -> String {
    "wn".to_owned()
}

impl OilKeybindingsSection {
    pub fn oil_keybindings(&self) -> OilKeybindings {
        OilKeybindings {
            open_entry: Box::leak(self.open_entry.clone().into_boxed_str()),
            open_vertical_split: Box::leak(self.open_vertical_split.clone().into_boxed_str()),
            open_horizontal_split: Box::leak(self.open_horizontal_split.clone().into_boxed_str()),
            open_new_pane: Box::leak(self.open_new_pane.clone().into_boxed_str()),
            preview_entry: Box::leak(self.preview_entry.clone().into_boxed_str()),
            refresh: Box::leak(self.refresh.clone().into_boxed_str()),
            close: Box::leak(self.close.clone().into_boxed_str()),
            prefix: Box::leak(self.prefix.clone().into_boxed_str()),
            open_parent: Box::leak(self.open_parent.clone().into_boxed_str()),
            open_workspace_root: Box::leak(self.open_workspace_root.clone().into_boxed_str()),
            set_root: Box::leak(self.set_root.clone().into_boxed_str()),
            show_help: Box::leak(self.show_help.clone().into_boxed_str()),
            cycle_sort: Box::leak(self.cycle_sort.clone().into_boxed_str()),
            toggle_hidden: Box::leak(self.toggle_hidden.clone().into_boxed_str()),
            toggle_trash: Box::leak(self.toggle_trash.clone().into_boxed_str()),
            open_external: Box::leak(self.open_external.clone().into_boxed_str()),
            set_tab_local_root: Box::leak(self.set_tab_local_root.clone().into_boxed_str()),
            create_git_worktree: Box::leak(self.create_git_worktree.clone().into_boxed_str()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{config_root_dir_from_exe_dir, config_source_files_from_root, load_from_root};
    use editor_plugin_api::{OilSortMode, PickerTruncateStrategy};
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_root(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "volt-user-config-{label}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_millis()
        ))
    }

    #[test]
    fn load_uses_defaults_when_files_are_missing() {
        let root = temp_root("missing");
        fs::create_dir_all(&root).expect("create temp root");
        let config = load_from_root(&root);
        assert!(!config.workspace.project_search_roots.is_empty());
        assert!(!config.acp.clients.is_empty());
        assert_eq!(
            PickerTruncateStrategy::from(config.ui.picker_truncate_strategy.clone()),
            PickerTruncateStrategy::Auto
        );
        assert_eq!(config.ui.keymap.ambiguous_prefix_timeout_ms, 250);
        assert_eq!(
            config.oil.defaults.oil_defaults().sort_mode,
            OilSortMode::TypeThenName
        );
        fs::remove_dir_all(&root).expect("cleanup temp root");
    }

    #[test]
    fn load_reads_referenced_child_files() {
        let root = temp_root("children");
        let config_dir = root.join("config");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(
            root.join("config.yaml"),
            "workspace: config/workspace.yaml\nacp: config/acp.yaml\nui: config/ui.yaml\noil: config/oil.yaml\n",
        )
        .expect("write master config");
        fs::write(
            config_dir.join("workspace.yaml"),
            "project_search_roots:\n  - path: D:/code\n    max_depth: 2\n",
        )
        .expect("write workspace config");
        fs::write(
            config_dir.join("acp.yaml"),
            "clients:\n  - id: test\n    label: Test ACP\n    command: test-acp\n    args: [serve]\n",
        )
        .expect("write acp config");
        fs::write(
            config_dir.join("ui.yaml"),
            "picker_truncate_strategy: file-name\nligatures_enabled: false\npane:\n  golden_ratio: false\nkeymap:\n  ambiguous_prefix_timeout_ms: 100\nterminal:\n  program: bash\n  args: ['-i']\n",
        )
        .expect("write ui config");
        fs::write(
            config_dir.join("oil.yaml"),
            "defaults:\n  show_hidden: true\n  sort_mode: type-then-name-desc\n  trash_enabled: true\nkeybindings:\n  open_entry: Space\n",
        )
        .expect("write oil config");

        let config = load_from_root(&root);
        assert_eq!(config.workspace.project_search_roots[0].path, "D:/code");
        assert_eq!(config.workspace.project_search_roots[0].max_depth, 2);
        assert_eq!(config.acp.clients[0].id, "test");
        assert_eq!(
            PickerTruncateStrategy::from(config.ui.picker_truncate_strategy.clone()),
            PickerTruncateStrategy::FileName
        );
        assert!(!config.ui.ligatures_enabled);
        assert!(!config.ui.pane.golden_ratio);
        assert_eq!(config.ui.keymap.ambiguous_prefix_timeout_ms, 100);
        assert_eq!(config.ui.terminal.program, "bash");
        assert_eq!(config.ui.terminal.args, vec!["-i".to_owned()]);
        assert!(config.oil.defaults.show_hidden);
        assert_eq!(
            config.oil.defaults.oil_defaults().sort_mode,
            OilSortMode::TypeThenNameDesc
        );
        assert_eq!(config.oil.keybindings.open_entry, "Space");
        fs::remove_dir_all(&root).expect("cleanup temp root");
    }

    #[test]
    fn config_source_files_include_master_and_children() {
        let root = temp_root("sources");
        let config_dir = root.join("config");
        fs::create_dir_all(&config_dir).expect("create config dir");
        fs::write(
            root.join("config.yaml"),
            "workspace: config/workspace.yaml\nui: config/ui.yaml\n",
        )
        .expect("write master config");
        fs::write(
            config_dir.join("workspace.yaml"),
            "project_search_roots: []\n",
        )
        .expect("write workspace");
        fs::write(config_dir.join("ui.yaml"), "ligatures_enabled: true\n").expect("write ui");

        let files = config_source_files_from_root(&root);
        assert_eq!(files.len(), 3);
        assert!(files.contains(&root.join("config.yaml")));
        assert!(files.contains(&config_dir.join("workspace.yaml")));
        assert!(files.contains(&config_dir.join("ui.yaml")));
        fs::remove_dir_all(&root).expect("cleanup temp root");
    }

    #[test]
    fn config_root_prefers_workspace_user_directory() {
        let root = temp_root("dir");
        let exe_dir = root.join("target").join("debug").join("deps");
        let staged_user = root.join("target").join("debug").join("user");
        let source_user = root.join("user");
        fs::create_dir_all(&exe_dir).expect("create exe dir");
        fs::create_dir_all(&staged_user).expect("create staged user dir");
        fs::create_dir_all(&source_user).expect("create source user dir");
        fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("write manifest");

        let resolved = config_root_dir_from_exe_dir(&exe_dir).expect("resolve config root dir");
        assert_eq!(resolved, source_user);

        fs::remove_dir_all(&root).expect("cleanup temp root");
    }
}
