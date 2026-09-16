use std::{collections::BTreeMap, path::Path};

use serde_json::{Number, Value};

use crate::install::InstallRecipe;
use crate::path::{PathMatcher, normalize_extension};

fn normalize_unique_entries<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for value in values {
        let value = value.into();
        let value = value.trim();
        if !value.is_empty() && !normalized.iter().any(|existing| existing == value) {
            normalized.push(value.to_owned());
        }
    }
    normalized
}

fn normalize_optional_string(value: String) -> Option<String> {
    let value = value.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
}

fn document_language_id_for_extension<'a>(
    document_language_ids: &'a BTreeMap<String, String>,
    extension: &str,
    default_language_id: &'a str,
) -> &'a str {
    let extension = normalize_extension(extension);
    document_language_ids
        .iter()
        .find_map(|(path_matcher, language_id)| {
            (normalize_extension(path_matcher) == extension).then_some(language_id.as_str())
        })
        .unwrap_or(default_language_id)
}

/// Workspace configuration metadata carried from declarative server specs into planned sessions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceConfiguration {
    pub(crate) section: Option<String>,
    pub(crate) settings: Option<WorkspaceConfigurationValue>,
}

impl WorkspaceConfiguration {
    /// Creates an empty workspace configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the configuration section queried for this server.
    pub fn with_section(mut self, section: impl Into<String>) -> Self {
        self.section = normalize_optional_string(section.into());
        self
    }

    /// Sets the server-specific workspace settings payload.
    pub fn with_settings(mut self, settings: impl Into<WorkspaceConfigurationValue>) -> Self {
        self.settings = Some(settings.into());
        self
    }

    /// Returns the workspace configuration section, if one is declared.
    pub fn section(&self) -> Option<&str> {
        self.section.as_deref()
    }

    /// Returns the workspace settings payload, if one is declared.
    pub fn settings(&self) -> Option<&WorkspaceConfigurationValue> {
        self.settings.as_ref()
    }

    /// Returns the workspace settings payload as a JSON value.
    pub fn settings_json(&self) -> Option<Value> {
        self.settings.as_ref().map(Value::from)
    }

    /// Returns whether both the section and settings are absent.
    pub fn is_empty(&self) -> bool {
        self.section.is_none() && self.settings.is_none()
    }
}

/// Recursive JSON-like workspace configuration value stored in declarative LSP specs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceConfigurationValue {
    Null,
    Bool(bool),
    Number(Number),
    String(String),
    Array(Vec<WorkspaceConfigurationValue>),
    Object(BTreeMap<String, WorkspaceConfigurationValue>),
}

impl WorkspaceConfigurationValue {
    /// Creates a null configuration value.
    pub const fn null() -> Self {
        Self::Null
    }

    /// Creates an integer configuration value.
    pub fn integer(value: i64) -> Self {
        Self::Number(value.into())
    }

    /// Creates an unsigned integer configuration value.
    pub fn unsigned(value: u64) -> Self {
        Self::Number(value.into())
    }

    /// Creates a floating-point configuration value when the input is finite.
    pub fn float(value: f64) -> Option<Self> {
        Number::from_f64(value).map(Self::Number)
    }

    /// Parses a JSON number string into a configuration value.
    pub fn from_number_text(value: impl AsRef<str>) -> Option<Self> {
        value.as_ref().parse::<Number>().ok().map(Self::Number)
    }

    /// Creates an array configuration value.
    pub fn array<I>(values: I) -> Self
    where
        I: IntoIterator<Item = Self>,
    {
        Self::Array(values.into_iter().collect())
    }

    /// Creates an object configuration value.
    pub fn object<I, K>(entries: I) -> Self
    where
        I: IntoIterator<Item = (K, Self)>,
        K: Into<String>,
    {
        let mut object = BTreeMap::new();
        for (key, value) in entries {
            object.insert(key.into(), value);
        }
        Self::Object(object)
    }

    /// Returns true when the value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Returns the inner boolean when this value is a bool.
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns the inner number when this value is numeric.
    pub fn as_number(&self) -> Option<&Number> {
        match self {
            Self::Number(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the inner string when this value is textual.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the inner array when this value is an array.
    pub fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    /// Returns the inner object when this value is an object.
    pub fn as_object(&self) -> Option<&BTreeMap<String, Self>> {
        match self {
            Self::Object(values) => Some(values),
            _ => None,
        }
    }

    /// Converts this value into a JSON value.
    pub fn to_json_value(&self) -> Value {
        Value::from(self)
    }
}

impl From<bool> for WorkspaceConfigurationValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<i64> for WorkspaceConfigurationValue {
    fn from(value: i64) -> Self {
        Self::integer(value)
    }
}

impl From<u64> for WorkspaceConfigurationValue {
    fn from(value: u64) -> Self {
        Self::unsigned(value)
    }
}

impl From<Number> for WorkspaceConfigurationValue {
    fn from(value: Number) -> Self {
        Self::Number(value)
    }
}

impl From<String> for WorkspaceConfigurationValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for WorkspaceConfigurationValue {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl<T> From<Vec<T>> for WorkspaceConfigurationValue
where
    T: Into<WorkspaceConfigurationValue>,
{
    fn from(value: Vec<T>) -> Self {
        Self::Array(value.into_iter().map(Into::into).collect())
    }
}

impl<K, V> From<BTreeMap<K, V>> for WorkspaceConfigurationValue
where
    K: Into<String> + Ord,
    V: Into<WorkspaceConfigurationValue>,
{
    fn from(value: BTreeMap<K, V>) -> Self {
        Self::Object(
            value
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}

impl From<&WorkspaceConfigurationValue> for Value {
    fn from(value: &WorkspaceConfigurationValue) -> Self {
        match value {
            WorkspaceConfigurationValue::Null => Self::Null,
            WorkspaceConfigurationValue::Bool(value) => Self::Bool(*value),
            WorkspaceConfigurationValue::Number(value) => Self::Number(value.clone()),
            WorkspaceConfigurationValue::String(value) => Self::String(value.clone()),
            WorkspaceConfigurationValue::Array(values) => {
                Self::Array(values.iter().map(Value::from).collect())
            }
            WorkspaceConfigurationValue::Object(values) => Self::Object(
                values
                    .iter()
                    .map(|(key, value)| (key.clone(), Value::from(value)))
                    .collect(),
            ),
        }
    }
}

impl From<WorkspaceConfigurationValue> for Value {
    fn from(value: WorkspaceConfigurationValue) -> Self {
        match value {
            WorkspaceConfigurationValue::Null => Self::Null,
            WorkspaceConfigurationValue::Bool(value) => Self::Bool(value),
            WorkspaceConfigurationValue::Number(value) => Self::Number(value),
            WorkspaceConfigurationValue::String(value) => Self::String(value),
            WorkspaceConfigurationValue::Array(values) => {
                Self::Array(values.into_iter().map(Value::from).collect())
            }
            WorkspaceConfigurationValue::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Value::from(value)))
                    .collect(),
            ),
        }
    }
}

impl From<Value> for WorkspaceConfigurationValue {
    fn from(value: Value) -> Self {
        match value {
            Value::Null => Self::Null,
            Value::Bool(value) => Self::Bool(value),
            Value::Number(value) => Self::Number(value),
            Value::String(value) => Self::String(value),
            Value::Array(values) => Self::Array(values.into_iter().map(Into::into).collect()),
            Value::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, WorkspaceConfigurationValue::from(value)))
                    .collect(),
            ),
        }
    }
}

/// Declarative language-server specification compiled into the editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageServerSpec {
    pub(crate) id: String,
    pub(crate) language_id: String,
    pub(crate) file_extensions: Vec<String>,
    pub(crate) file_names: Vec<String>,
    pub(crate) file_globs: Vec<String>,
    pub(crate) document_language_ids: BTreeMap<String, String>,
    pub(crate) program: String,
    pub(crate) args: Vec<String>,
    pub(crate) root_markers: Vec<String>,
    pub(crate) activation_markers: Vec<String>,
    pub(crate) root_strategy: LanguageServerRootStrategy,
    pub(crate) env: Vec<(String, String)>,
    pub(crate) workspace_configuration: WorkspaceConfiguration,
    pub(crate) enabled_by_default: bool,
    pub(crate) install_recipe: Option<InstallRecipe>,
    pub(crate) path_matcher: PathMatcher,
}

/// Strategy used to choose the LSP workspace root for a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LanguageServerRootStrategy {
    /// Reuse the editor workspace root as-is.
    #[default]
    Workspace,
    /// Prefer the nearest configured root marker for the current file and fall back to the editor
    /// workspace root when no marker matches.
    MarkersOrWorkspace,
}

impl LanguageServerSpec {
    /// Creates a new language-server specification.
    pub fn new(
        id: impl Into<String>,
        language_id: impl Into<String>,
        file_extensions: impl IntoIterator<Item = impl Into<String>>,
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let file_extensions = file_extensions
            .into_iter()
            .map(|extension| normalize_extension(&extension.into()))
            .collect::<Vec<_>>();
        Self {
            id: id.into(),
            language_id: language_id.into(),
            file_extensions: file_extensions.clone(),
            file_names: Vec::new(),
            file_globs: Vec::new(),
            document_language_ids: BTreeMap::new(),
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            root_markers: Vec::new(),
            activation_markers: Vec::new(),
            root_strategy: LanguageServerRootStrategy::Workspace,
            env: Vec::new(),
            workspace_configuration: WorkspaceConfiguration::default(),
            enabled_by_default: true,
            install_recipe: None,
            path_matcher: PathMatcher::from_parts(
                &file_extensions,
                [] as [&str; 0],
                [] as [&str; 0],
            ),
        }
    }

    /// Adds root markers used for workspace discovery.
    ///
    /// Marker order is preference priority: earlier markers win over later ones across the full
    /// ancestor walk (for example `*.sln` before `*.csproj`). Solution globs (`*.sln`, `*.slnx`)
    /// also walk above a nested Project Workspace and search that workspace for a unique match.
    pub fn with_root_markers(
        mut self,
        markers: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.root_markers = markers.into_iter().map(Into::into).collect();
        self
    }

    /// Adds project markers required before this server may start for a file.
    pub fn with_activation_markers(
        mut self,
        markers: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.activation_markers = markers.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the workspace-root strategy for this server.
    pub fn with_root_strategy(mut self, strategy: LanguageServerRootStrategy) -> Self {
        self.root_strategy = strategy;
        self
    }

    /// Overrides the LSP document language id for specific extensions, basenames, or globs.
    pub fn with_document_language_ids<I, E, L>(mut self, mappings: I) -> Self
    where
        I: IntoIterator<Item = (E, L)>,
        E: Into<String>,
        L: Into<String>,
    {
        for (path_matcher, language_id) in mappings {
            let path_matcher = path_matcher.into();
            let path_matcher = path_matcher.trim();
            let language_id = language_id.into();
            if path_matcher.is_empty() || language_id.is_empty() {
                continue;
            }
            self.document_language_ids
                .insert(path_matcher.to_owned(), language_id);
        }
        self
    }

    /// Adds an environment override.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    /// Sets the workspace configuration section used for server-specific settings.
    pub fn with_workspace_configuration_section(mut self, section: impl Into<String>) -> Self {
        self.workspace_configuration = self.workspace_configuration.with_section(section);
        self
    }

    /// Sets the server-specific workspace settings payload.
    pub fn with_workspace_configuration_settings(
        mut self,
        settings: impl Into<WorkspaceConfigurationValue>,
    ) -> Self {
        self.workspace_configuration = self.workspace_configuration.with_settings(settings);
        self
    }

    /// Sets both the workspace configuration section and settings payload.
    pub fn with_workspace_configuration(
        mut self,
        section: impl Into<String>,
        settings: impl Into<WorkspaceConfigurationValue>,
    ) -> Self {
        self.workspace_configuration = self
            .workspace_configuration
            .with_section(section)
            .with_settings(settings);
        self
    }

    /// Controls whether generic LSP startup should include this server by default.
    pub fn with_enabled_by_default(mut self, enabled_by_default: bool) -> Self {
        self.enabled_by_default = enabled_by_default;
        self
    }

    /// Sets the optional Install Recipe used when the program is not on PATH.
    pub fn with_install_recipe(mut self, recipe: InstallRecipe) -> Self {
        self.install_recipe = Some(recipe);
        self
    }

    /// Creates a workspace settings object without importing the underlying value type.
    pub fn workspace_settings_object<I, K>(entries: I) -> WorkspaceConfigurationValue
    where
        I: IntoIterator<Item = (K, WorkspaceConfigurationValue)>,
        K: Into<String>,
    {
        WorkspaceConfigurationValue::object(entries)
    }

    /// Creates a workspace settings array without importing the underlying value type.
    pub fn workspace_settings_array<I>(values: I) -> WorkspaceConfigurationValue
    where
        I: IntoIterator<Item = WorkspaceConfigurationValue>,
    {
        WorkspaceConfigurationValue::array(values)
    }

    /// Creates a null workspace setting value.
    pub const fn workspace_settings_null() -> WorkspaceConfigurationValue {
        WorkspaceConfigurationValue::Null
    }

    /// Creates a floating-point workspace setting value when the input is finite.
    pub fn workspace_settings_float(value: f64) -> Option<WorkspaceConfigurationValue> {
        WorkspaceConfigurationValue::float(value)
    }

    /// Returns the server identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the language identifier.
    pub fn language_id(&self) -> &str {
        &self.language_id
    }

    /// Returns the file extensions handled by this server.
    pub fn file_extensions(&self) -> &[String] {
        &self.file_extensions
    }

    /// Adds exact basenames handled by this server.
    pub fn with_file_names<I, S>(mut self, file_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_names = normalize_unique_entries(file_names);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the exact basenames handled by this server.
    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    /// Adds basename globs handled by this server.
    pub fn with_file_globs<I, S>(mut self, file_globs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_globs = normalize_unique_entries(file_globs);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the basename globs handled by this server.
    pub fn file_globs(&self) -> &[String] {
        &self.file_globs
    }

    /// Returns the LSP document language id for a file extension.
    pub fn document_language_id_for_extension(&self, extension: &str) -> &str {
        document_language_id_for_extension(
            &self.document_language_ids,
            extension,
            &self.language_id,
        )
    }

    /// Returns the program executable.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Returns the program arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Returns root markers for workspace discovery.
    pub fn root_markers(&self) -> &[String] {
        &self.root_markers
    }

    /// Returns project markers required before this server may start.
    pub fn activation_markers(&self) -> &[String] {
        &self.activation_markers
    }

    /// Returns the path-matcher-to-language-id overrides.
    pub fn document_language_ids(&self) -> &BTreeMap<String, String> {
        &self.document_language_ids
    }

    /// Returns the workspace-root strategy for this server.
    pub const fn root_strategy(&self) -> LanguageServerRootStrategy {
        self.root_strategy
    }

    /// Returns the environment overrides used when launching the server.
    pub fn env(&self) -> &[(String, String)] {
        &self.env
    }

    /// Returns the declared workspace configuration.
    pub fn workspace_configuration(&self) -> &WorkspaceConfiguration {
        &self.workspace_configuration
    }

    /// Returns the workspace configuration section, if one is declared.
    pub fn workspace_configuration_section(&self) -> Option<&str> {
        self.workspace_configuration.section()
    }

    /// Returns the workspace settings payload, if one is declared.
    pub fn workspace_configuration_settings(&self) -> Option<&WorkspaceConfigurationValue> {
        self.workspace_configuration.settings()
    }

    /// Returns the workspace settings payload as a JSON value.
    pub fn workspace_configuration_settings_json(&self) -> Option<Value> {
        self.workspace_configuration.settings_json()
    }

    /// Returns whether generic LSP startup should include this server by default.
    pub const fn enabled_by_default(&self) -> bool {
        self.enabled_by_default
    }

    /// Returns the Install Recipe, if this server is Volt-installable.
    pub fn install_recipe(&self) -> Option<&InstallRecipe> {
        self.install_recipe.as_ref()
    }

    /// Returns whether this server should attach to the provided path.
    pub fn matches_path(&self, path: &Path) -> bool {
        self.path_match_score(path).is_some()
    }

    fn rebuild_path_matcher(&mut self) {
        self.path_matcher =
            PathMatcher::from_parts(&self.file_extensions, &self.file_names, &self.file_globs);
    }

    /// Returns the best filename-match score for this server, if any.
    pub fn path_match_score(&self, path: &Path) -> Option<usize> {
        self.path_matcher.best_match_score(path)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LspCompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

impl LspCompletionKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Text => "Text",
            Self::Method => "Method",
            Self::Function => "Function",
            Self::Constructor => "Constructor",
            Self::Field => "Field",
            Self::Variable => "Variable",
            Self::Class => "Class",
            Self::Interface => "Interface",
            Self::Module => "Module",
            Self::Property => "Property",
            Self::Unit => "Unit",
            Self::Value => "Value",
            Self::Enum => "Enum",
            Self::Keyword => "Keyword",
            Self::Snippet => "Snippet",
            Self::Color => "Color",
            Self::File => "File",
            Self::Reference => "Reference",
            Self::Folder => "Folder",
            Self::EnumMember => "Enum Member",
            Self::Constant => "Constant",
            Self::Struct => "Struct",
            Self::Event => "Event",
            Self::Operator => "Operator",
            Self::TypeParameter => "Type Parameter",
        }
    }
}
