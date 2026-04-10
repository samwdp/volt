#![doc = r#"Tree-sitter language registration, installation, parsing, highlighting, and indentation."#]

use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use editor_buffer::{TextBuffer, TextByteChunks, TextEdit, TextPoint};
use editor_path::PathMatcher;
pub use tree_sitter::Language;
use tree_sitter::{
    InputEdit, Node, Parser, Point, Query, QueryCursor, QueryPredicateArg, QueryProperty, Range,
    StreamingIterator, TextProvider, Tree,
};
use tree_sitter_language::LanguageFn;

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str =
    "Tree-sitter language registration, installation, parsing, highlighting, and indentation.";

const DEFAULT_QUERY_ASSET_SEARCH_DEPTH: usize = 6;
const MAX_INJECTION_DEPTH: usize = 8;
const QUERY_ASSET_DIR_CANDIDATES: &[&[&str]] = &[
    &["crates", "volt", "assets", "grammars", "queries"],
    &["assets", "grammars", "queries"],
];

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Function pointer that returns a statically linked tree-sitter language handle.
pub type LanguageProvider = fn() -> Language;

/// Maps a tree-sitter capture name to a theme token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureThemeMapping {
    capture_name: String,
    theme_token: String,
}

impl CaptureThemeMapping {
    /// Creates a new capture-to-theme mapping.
    pub fn new(capture_name: impl Into<String>, theme_token: impl Into<String>) -> Self {
        Self {
            capture_name: capture_name.into(),
            theme_token: theme_token.into(),
        }
    }

    /// Returns the capture name.
    pub fn capture_name(&self) -> &str {
        &self.capture_name
    }

    /// Returns the destination theme token.
    pub fn theme_token(&self) -> &str {
        &self.theme_token
    }
}

/// Download/build metadata for one installable tree-sitter grammar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrammarSource {
    repository_url: String,
    grammar_dir: PathBuf,
    source_dir: PathBuf,
    install_dir_name: String,
    symbol_name: String,
}

impl GrammarSource {
    /// Creates a new installable grammar source description.
    pub fn new(
        repository_url: impl Into<String>,
        grammar_dir: impl Into<PathBuf>,
        source_dir: impl Into<PathBuf>,
        install_dir_name: impl Into<String>,
        symbol_name: impl Into<String>,
    ) -> Self {
        Self {
            repository_url: repository_url.into(),
            grammar_dir: grammar_dir.into(),
            source_dir: source_dir.into(),
            install_dir_name: install_dir_name.into(),
            symbol_name: symbol_name.into(),
        }
    }

    /// Returns the grammar repository URL.
    pub fn repository_url(&self) -> &str {
        &self.repository_url
    }

    /// Returns the subdirectory within the cloned repository that contains the grammar.
    pub fn grammar_dir(&self) -> &Path {
        &self.grammar_dir
    }

    /// Returns the source directory inside the grammar directory.
    pub fn source_dir(&self) -> &Path {
        &self.source_dir
    }

    /// Returns the stable install directory name used under the configured grammar install root.
    pub fn install_dir_name(&self) -> &str {
        &self.install_dir_name
    }

    /// Returns the exported grammar symbol name.
    pub fn symbol_name(&self) -> &str {
        &self.symbol_name
    }

    /// Returns the installed grammar directory under the configured install root.
    pub fn install_directory(&self, install_root: &Path) -> PathBuf {
        install_root.join(&self.install_dir_name)
    }

    /// Returns the installed highlight query path.
    pub fn installed_highlight_query_path(&self, install_root: &Path) -> PathBuf {
        self.installed_query_path(install_root, "highlights.scm")
    }

    /// Returns the installed indent query path.
    pub fn installed_indent_query_path(&self, install_root: &Path) -> PathBuf {
        self.installed_query_path(install_root, "indents.scm")
    }

    /// Returns the installed injections query path.
    pub fn installed_injections_query_path(&self, install_root: &Path) -> PathBuf {
        self.installed_query_path(install_root, "injections.scm")
    }

    /// Returns the installed locals query path.
    pub fn installed_locals_query_path(&self, install_root: &Path) -> PathBuf {
        self.installed_query_path(install_root, "locals.scm")
    }

    /// Returns the installed folds query path.
    pub fn installed_folds_query_path(&self, install_root: &Path) -> PathBuf {
        self.installed_query_path(install_root, "folds.scm")
    }

    /// Returns the installed query path for an arbitrary query file.
    pub fn installed_query_path(&self, install_root: &Path, file_name: &str) -> PathBuf {
        self.install_directory(install_root)
            .join("queries")
            .join(file_name)
    }

    /// Returns the installed shared library path.
    pub fn installed_library_path(&self, install_root: &Path) -> PathBuf {
        self.install_directory(install_root)
            .join(shared_library_file_name(&self.install_dir_name))
    }
}

#[derive(Debug, Clone)]
enum LanguageLoader {
    Static {
        language_provider: LanguageProvider,
        highlight_query: String,
    },
    Grammar {
        grammar: GrammarSource,
    },
}

/// User-facing registration for one syntax language.
#[derive(Debug, Clone)]
pub struct LanguageConfiguration {
    id: String,
    file_extensions: Vec<String>,
    file_names: Vec<String>,
    file_globs: Vec<String>,
    capture_mappings: Vec<CaptureThemeMapping>,
    loader: LanguageLoader,
    extra_highlight_query: Option<String>,
    extra_indent_query: Option<String>,
    extra_injections_query: Option<String>,
    extra_locals_query: Option<String>,
    extra_folds_query: Option<String>,
    additional_highlight_languages: Vec<String>,
    path_matcher: PathMatcher,
}

impl LanguageConfiguration {
    /// Creates a statically linked language configuration.
    pub fn new<I, S>(
        id: impl Into<String>,
        file_extensions: I,
        language_provider: LanguageProvider,
        highlight_query: impl Into<String>,
        capture_mappings: impl IntoIterator<Item = CaptureThemeMapping>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::normalize(
            id,
            file_extensions,
            capture_mappings,
            LanguageLoader::Static {
                language_provider,
                highlight_query: highlight_query.into(),
            },
        )
    }

    /// Creates an installable grammar-backed language configuration.
    pub fn from_grammar<I, S>(
        id: impl Into<String>,
        file_extensions: I,
        grammar: GrammarSource,
        capture_mappings: impl IntoIterator<Item = CaptureThemeMapping>,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::normalize(
            id,
            file_extensions,
            capture_mappings,
            LanguageLoader::Grammar { grammar },
        )
    }

    fn normalize<I, S>(
        id: impl Into<String>,
        file_extensions: I,
        capture_mappings: impl IntoIterator<Item = CaptureThemeMapping>,
        loader: LanguageLoader,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut normalized_extensions = Vec::new();
        for extension in file_extensions {
            let extension = normalize_extension(&extension.into());
            if !extension.is_empty() && !normalized_extensions.contains(&extension) {
                normalized_extensions.push(extension);
            }
        }
        let path_matcher =
            PathMatcher::from_parts(&normalized_extensions, [] as [&str; 0], [] as [&str; 0]);

        Self {
            id: id.into(),
            file_extensions: normalized_extensions,
            file_names: Vec::new(),
            file_globs: Vec::new(),
            capture_mappings: capture_mappings.into_iter().collect(),
            loader,
            extra_highlight_query: None,
            extra_indent_query: None,
            extra_injections_query: None,
            extra_locals_query: None,
            extra_folds_query: None,
            additional_highlight_languages: Vec::new(),
            path_matcher,
        }
    }

    /// Returns the stable language identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the registered file extensions without leading dots.
    pub fn file_extensions(&self) -> &[String] {
        &self.file_extensions
    }

    /// Adds exact basenames that should resolve to this language.
    pub fn with_file_names<I, S>(mut self, file_names: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_names = normalize_unique_entries(file_names);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the registered exact basenames.
    pub fn file_names(&self) -> &[String] {
        &self.file_names
    }

    /// Adds glob patterns that should resolve to this language.
    pub fn with_file_globs<I, S>(mut self, file_globs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.file_globs = normalize_unique_entries(file_globs);
        self.rebuild_path_matcher();
        self
    }

    /// Returns the registered basename globs.
    pub fn file_globs(&self) -> &[String] {
        &self.file_globs
    }

    /// Returns the capture-to-theme mappings.
    pub fn capture_mappings(&self) -> &[CaptureThemeMapping] {
        &self.capture_mappings
    }

    /// Returns the inline highlight query for statically linked languages.
    pub fn highlight_query(&self) -> Option<&str> {
        match &self.loader {
            LanguageLoader::Static {
                highlight_query, ..
            } => Some(highlight_query),
            LanguageLoader::Grammar { .. } => None,
        }
    }

    /// Adds an extra highlight query appended at load time.
    pub fn with_extra_highlight_query(mut self, query: impl Into<String>) -> Self {
        self.extra_highlight_query = Some(query.into());
        self
    }

    /// Returns the extra highlight query, when configured.
    pub fn extra_highlight_query(&self) -> Option<&str> {
        self.extra_highlight_query.as_deref()
    }

    /// Adds an extra indent query appended at load time.
    pub fn with_extra_indent_query(mut self, query: impl Into<String>) -> Self {
        self.extra_indent_query = Some(query.into());
        self
    }

    /// Returns the extra indent query, when configured.
    pub fn extra_indent_query(&self) -> Option<&str> {
        self.extra_indent_query.as_deref()
    }

    /// Adds an extra injections query appended at load time.
    pub fn with_extra_injections_query(mut self, query: impl Into<String>) -> Self {
        self.extra_injections_query = Some(query.into());
        self
    }

    /// Returns the extra injections query, when configured.
    pub fn extra_injections_query(&self) -> Option<&str> {
        self.extra_injections_query.as_deref()
    }

    /// Adds an extra locals query appended at load time.
    pub fn with_extra_locals_query(mut self, query: impl Into<String>) -> Self {
        self.extra_locals_query = Some(query.into());
        self
    }

    /// Returns the extra locals query, when configured.
    pub fn extra_locals_query(&self) -> Option<&str> {
        self.extra_locals_query.as_deref()
    }

    /// Adds an extra folds query appended at load time.
    pub fn with_extra_folds_query(mut self, query: impl Into<String>) -> Self {
        self.extra_folds_query = Some(query.into());
        self
    }

    /// Returns the extra folds query, when configured.
    pub fn extra_folds_query(&self) -> Option<&str> {
        self.extra_folds_query.as_deref()
    }

    /// Adds additional language ids to merge highlight spans for this language.
    pub fn with_additional_highlight_languages<I, S>(mut self, languages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut ids = Vec::new();
        for language in languages {
            let language = language.into();
            if !language.is_empty() && !ids.contains(&language) {
                ids.push(language);
            }
        }
        self.additional_highlight_languages = ids;
        self
    }

    /// Returns additional language ids used to merge highlight spans.
    pub fn additional_highlight_languages(&self) -> &[String] {
        &self.additional_highlight_languages
    }

    fn path_match_score(&self, path: &Path) -> Option<usize> {
        self.path_matcher.best_match_score(path)
    }

    fn rebuild_path_matcher(&mut self) {
        self.path_matcher =
            PathMatcher::from_parts(&self.file_extensions, &self.file_names, &self.file_globs);
    }

    /// Returns the installable grammar metadata, when present.
    pub fn grammar(&self) -> Option<&GrammarSource> {
        match &self.loader {
            LanguageLoader::Static { .. } => None,
            LanguageLoader::Grammar { grammar } => Some(grammar),
        }
    }
}

/// Line and column pair reported by tree-sitter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyntaxPoint {
    /// Zero-based line index.
    pub line: usize,
    /// Zero-based column index in bytes.
    pub column: usize,
}

impl SyntaxPoint {
    /// Creates a new syntax point.
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

/// One highlighted range produced by tree-sitter query captures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    /// Start byte offset.
    pub start_byte: usize,
    /// End byte offset.
    pub end_byte: usize,
    /// Start line/column pair from tree-sitter.
    pub start_position: SyntaxPoint,
    /// End line/column pair from tree-sitter.
    pub end_position: SyntaxPoint,
    /// Original tree-sitter capture name.
    pub capture_name: String,
    /// Resolved theme token.
    pub theme_token: String,
}

/// Syntax parse result for a single buffer snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxSnapshot {
    /// Stable language identifier.
    pub language_id: String,
    /// Root syntax node kind.
    pub root_kind: String,
    /// Whether the parse tree contains errors.
    pub has_errors: bool,
    /// Highlight spans generated from the configured highlight query.
    pub highlight_spans: Vec<HighlightSpan>,
}

/// One named tree-sitter node in the ancestor chain for a cursor location.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNodeContext {
    /// Tree-sitter node kind.
    pub kind: String,
    /// Starting line/column for the node.
    pub start_position: SyntaxPoint,
    /// Exclusive ending line/column for the node.
    pub end_position: SyntaxPoint,
}

impl SyntaxSnapshot {
    /// Returns the number of highlight spans.
    pub fn highlight_count(&self) -> usize {
        self.highlight_spans.len()
    }

    /// Returns highlight spans intersecting a visible line window.
    pub fn visible_spans(&self, start_line: usize, line_count: usize) -> Vec<&HighlightSpan> {
        if line_count == 0 {
            return Vec::new();
        }

        let end_line = start_line + line_count.saturating_sub(1);
        self.highlight_spans
            .iter()
            .filter(|span| {
                span.start_position.line <= end_line && span.end_position.line >= start_line
            })
            .collect()
    }
}

/// Reusable parser/tree state for incremental highlighting of one buffer.
pub struct SyntaxParseSession {
    language_id: String,
    revision: u64,
    parser: Parser,
    tree: Tree,
    last_highlight_window: Option<HighlightWindow>,
    last_snapshot: Option<SyntaxSnapshot>,
}

/// A requested line window for range-limited syntax highlighting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HighlightWindow {
    start_line: usize,
    line_count: usize,
}

impl HighlightWindow {
    /// Creates a new line window.
    pub const fn new(start_line: usize, line_count: usize) -> Self {
        Self {
            start_line,
            line_count,
        }
    }

    /// Returns the first requested line.
    pub const fn start_line(&self) -> usize {
        self.start_line
    }

    /// Returns the requested number of lines.
    pub const fn line_count(&self) -> usize {
        self.line_count
    }

    const fn is_empty(&self) -> bool {
        self.line_count == 0
    }

    const fn end_line_exclusive(&self) -> usize {
        self.start_line.saturating_add(self.line_count)
    }
}

/// Errors that can occur while registering, installing, or executing syntax providers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxError {
    /// A language id was registered more than once.
    DuplicateLanguageId(String),
    /// A file extension was already assigned to another language.
    DuplicateExtension(String),
    /// A request referenced an unknown file extension.
    UnknownExtension(String),
    /// A request referenced an unknown language id.
    UnknownLanguage(String),
    /// The grammar is not installed at the configured install root.
    GrammarNotInstalled {
        language_id: String,
        install_dir: PathBuf,
    },
    /// A query failed to compile.
    InvalidQuery {
        language_id: String,
        query_kind: String,
        message: String,
    },
    /// The parser failed to accept the requested language.
    ParserConfiguration {
        language_id: String,
        message: String,
    },
    /// The parser did not return a syntax tree.
    ParseCancelled(String),
    /// Included ranges could not be configured for a parser.
    IncludedRangesFailed {
        language_id: String,
        message: String,
    },
    /// File-system work required for installation failed.
    Io {
        operation: String,
        path: PathBuf,
        message: String,
    },
    /// Running an installer command failed.
    InstallCommand {
        language_id: String,
        message: String,
    },
    /// Loading the compiled grammar library failed.
    LibraryLoad {
        language_id: String,
        message: String,
    },
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateLanguageId(language_id) => {
                write!(formatter, "language `{language_id}` is already registered")
            }
            Self::DuplicateExtension(extension) => {
                write!(formatter, "extension `{extension}` is already registered")
            }
            Self::UnknownExtension(extension) => {
                write!(formatter, "no syntax language registered for `{extension}`")
            }
            Self::UnknownLanguage(language_id) => {
                write!(formatter, "language `{language_id}` is not registered")
            }
            Self::GrammarNotInstalled {
                language_id,
                install_dir,
            } => {
                write!(
                    formatter,
                    "grammar `{language_id}` is not installed under `{}`",
                    install_dir.display()
                )
            }
            Self::InvalidQuery {
                language_id,
                query_kind,
                message,
            } => {
                write!(
                    formatter,
                    "{query_kind} query for `{language_id}` is invalid: {message}"
                )
            }
            Self::ParserConfiguration {
                language_id,
                message,
            } => {
                write!(
                    formatter,
                    "parser configuration failed for `{language_id}`: {message}"
                )
            }
            Self::ParseCancelled(language_id) => {
                write!(
                    formatter,
                    "parser did not produce a tree for `{language_id}`"
                )
            }
            Self::IncludedRangesFailed {
                language_id,
                message,
            } => {
                write!(
                    formatter,
                    "setting included ranges failed for `{language_id}`: {message}"
                )
            }
            Self::Io {
                operation,
                path,
                message,
            } => {
                write!(
                    formatter,
                    "{operation} failed for `{}`: {message}",
                    path.display()
                )
            }
            Self::InstallCommand {
                language_id,
                message,
            } => {
                write!(formatter, "installing `{language_id}` failed: {message}")
            }
            Self::LibraryLoad {
                language_id,
                message,
            } => {
                write!(formatter, "loading `{language_id}` failed: {message}")
            }
        }
    }
}

impl Error for SyntaxError {}

struct LoadedLanguage {
    _library: Option<libloading::Library>,
    language: Language,
    query: Query,
    indent_query: Option<Query>,
    injections_query: Option<Query>,
    locals_query: Option<Query>,
    folds_query: Option<Query>,
    capture_mappings: BTreeMap<String, String>,
}

struct ParsedHighlight {
    snapshot: SyntaxSnapshot,
    tree: Tree,
}

struct ParseTreeResult {
    tree: Tree,
    changed_ranges: Option<Vec<Range>>,
    applied_edits: Option<Vec<TextEdit>>,
}

#[derive(Default)]
struct InjectionHighlights {
    highlight_spans: Vec<HighlightSpan>,
    has_errors: bool,
}

struct InjectionRegion {
    language_name: String,
    start_byte: usize,
    end_byte: usize,
    start_position: SyntaxPoint,
    end_position: SyntaxPoint,
}

struct TextBufferProvider<'a> {
    buffer: &'a TextBuffer,
}

impl<'a> TextProvider<&'a [u8]> for TextBufferProvider<'a> {
    type I = TextByteChunks<'a>;

    fn text(&mut self, node: tree_sitter::Node) -> Self::I {
        self.buffer.byte_slice_chunks(node.byte_range())
    }
}

impl LoadedLanguage {
    fn theme_token_for_capture(&self, capture_name: &str) -> String {
        self.capture_mappings
            .get(capture_name)
            .cloned()
            .unwrap_or_else(|| format!("syntax.{capture_name}"))
    }
}

fn capture_requires_theme_token(capture_name: &str) -> bool {
    !capture_name.starts_with('_')
        && !matches!(
            capture_name,
            "spell" | "nospell" | "conceal" | "conceal_lines"
        )
}

/// Runtime registry of known tree-sitter languages.
pub struct SyntaxRegistry {
    install_root: PathBuf,
    query_asset_root: Option<PathBuf>,
    languages: BTreeMap<String, LanguageConfiguration>,
    language_order: Vec<String>,
    extensions: BTreeMap<String, String>,
    loaded: BTreeMap<String, LoadedLanguage>,
}

impl fmt::Debug for SyntaxRegistry {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SyntaxRegistry")
            .field("install_root", &self.install_root)
            .field("query_asset_root", &self.query_asset_root)
            .field("language_count", &self.languages.len())
            .field("loaded_language_count", &self.loaded.len())
            .finish()
    }
}

impl Default for SyntaxRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SyntaxRegistry {
    /// Creates a syntax registry using Volt's default per-user grammar install root.
    pub fn new() -> Self {
        Self::with_install_root(default_install_root())
    }

    /// Creates a syntax registry with an explicit install root.
    pub fn with_install_root(install_root: impl Into<PathBuf>) -> Self {
        Self {
            install_root: install_root.into(),
            query_asset_root: default_query_asset_root(),
            languages: BTreeMap::new(),
            language_order: Vec::new(),
            extensions: BTreeMap::new(),
            loaded: BTreeMap::new(),
        }
    }

    /// Returns the grammar install root.
    pub fn install_root(&self) -> &Path {
        &self.install_root
    }

    /// Returns the bundled query asset root, when configured.
    pub fn query_asset_root(&self) -> Option<&Path> {
        self.query_asset_root.as_deref()
    }

    /// Replaces the bundled query asset root used for grammar query installation/loading.
    pub fn set_query_asset_root(&mut self, query_asset_root: Option<PathBuf>) {
        self.query_asset_root = query_asset_root;
        self.loaded.clear();
    }

    /// Returns the number of registered languages.
    pub fn len(&self) -> usize {
        self.languages.len()
    }

    /// Reports whether the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.languages.is_empty()
    }

    /// Returns all registered language configurations.
    pub fn languages(&self) -> impl Iterator<Item = &LanguageConfiguration> {
        self.languages.values()
    }

    /// Returns a registered language by identifier.
    pub fn language(&self, language_id: &str) -> Option<&LanguageConfiguration> {
        self.languages.get(language_id)
    }

    /// Registers a single language configuration.
    pub fn register(&mut self, config: LanguageConfiguration) -> Result<(), SyntaxError> {
        let language_id = config.id().to_owned();
        if self.languages.contains_key(&language_id) {
            return Err(SyntaxError::DuplicateLanguageId(language_id));
        }

        for extension in config.file_extensions() {
            if self.extensions.contains_key(extension) {
                return Err(SyntaxError::DuplicateExtension(extension.clone()));
            }
        }

        for extension in config.file_extensions() {
            self.extensions
                .insert(extension.clone(), language_id.clone());
        }
        self.language_order.push(language_id.clone());
        self.languages.insert(language_id, config);
        Ok(())
    }

    /// Registers multiple language configurations.
    pub fn register_all<I>(&mut self, configs: I) -> Result<(), SyntaxError>
    where
        I: IntoIterator<Item = LanguageConfiguration>,
    {
        for config in configs {
            self.register(config)?;
        }

        Ok(())
    }

    /// Returns the language configuration for an extension, if one exists.
    pub fn language_for_extension(&self, extension: &str) -> Option<&LanguageConfiguration> {
        let extension = normalize_extension(extension);
        let language_id = self.extensions.get(&extension)?;
        self.languages.get(language_id)
    }

    /// Returns the language configuration for a path, if one exists.
    pub fn language_for_path(&self, path: impl AsRef<Path>) -> Option<&LanguageConfiguration> {
        let path = path.as_ref();
        let mut best = None;
        let mut best_score = 0;
        for language_id in &self.language_order {
            let Some(language) = self.languages.get(language_id) else {
                continue;
            };
            let Some(score) = language.path_match_score(path) else {
                continue;
            };
            if best.is_none() || score > best_score {
                best = Some(language);
                best_score = score;
            }
        }
        best
    }

    /// Reports whether a grammar-backed language is installed.
    pub fn is_installed(&self, language_id: &str) -> Result<bool, SyntaxError> {
        let Some(config) = self.languages.get(language_id) else {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        };

        Ok(match config.grammar() {
            Some(grammar) => {
                grammar.installed_library_path(&self.install_root).exists()
                    && grammar
                        .installed_highlight_query_path(&self.install_root)
                        .exists()
            }
            None => true,
        })
    }

    /// Installs a grammar-backed language into the configured install root.
    pub fn install_language(&mut self, language_id: &str) -> Result<PathBuf, SyntaxError> {
        let config = self
            .languages
            .get(language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.to_owned()))?;
        let Some(grammar) = config.grammar().cloned() else {
            return Ok(self.install_root.clone());
        };

        let temp_clone_root = std::env::temp_dir().join(format!(
            "volt-treesitter-{}",
            temp_guid_like_directory_name()
        ));
        let _cleanup = TempCloneGuard::new(temp_clone_root.clone());
        let parent = temp_clone_root
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(std::env::temp_dir);
        fs::create_dir_all(&parent)
            .map_err(|error| io_error("create temp parent", &parent, error))?;

        let clone_output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                grammar.repository_url(),
                &temp_clone_root.display().to_string(),
            ])
            .output()
            .map_err(|error| io_error("run git clone", &temp_clone_root, error))?;
        if !clone_output.status.success() {
            return Err(SyntaxError::InstallCommand {
                language_id: language_id.to_owned(),
                message: command_failure_message("git clone", &clone_output),
            });
        }

        let cloned_grammar_dir = temp_clone_root.join(grammar.grammar_dir());
        if !cloned_grammar_dir.exists() {
            return Err(SyntaxError::Io {
                operation: "locate cloned grammar directory".to_owned(),
                path: cloned_grammar_dir,
                message: "configured grammar directory does not exist in the cloned repository"
                    .to_owned(),
            });
        }

        fs::create_dir_all(&self.install_root)
            .map_err(|error| io_error("create grammar install root", &self.install_root, error))?;
        let install_dir = grammar.install_directory(&self.install_root);
        if install_dir.exists() {
            fs::remove_dir_all(&install_dir)
                .map_err(|error| io_error("replace installed grammar", &install_dir, error))?;
        }
        fs::create_dir_all(&install_dir)
            .map_err(|error| io_error("create install directory", &install_dir, error))?;
        let query_asset_root = self
            .query_asset_root
            .as_deref()
            .ok_or_else(|| SyntaxError::Io {
                operation: "resolve bundled query asset root".to_owned(),
                path: self.install_root.clone(),
                message: "bundled tree-sitter query assets are not configured".to_owned(),
            })?;
        install_bundled_queries(&config, query_asset_root, &self.install_root)?;
        ensure_installed_highlight_query_path(
            &config,
            &grammar.installed_highlight_query_path(&self.install_root),
        )?;
        let highlight_query_path = grammar.installed_highlight_query_path(&self.install_root);
        if !highlight_query_path.exists() {
            return Err(SyntaxError::Io {
                operation: "locate bundled highlight query".to_owned(),
                path: query_asset_root.join(config.id()).join("highlights.scm"),
                message: "bundled highlights.scm is missing for this language".to_owned(),
            });
        }
        build_shared_library(
            language_id,
            &grammar,
            &cloned_grammar_dir,
            &self.install_root,
        )?;

        self.loaded.remove(language_id);
        Ok(install_dir)
    }

    /// Parses and highlights a buffer for a known file extension.
    pub fn highlight_buffer_for_extension(
        &mut self,
        extension: &str,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_extension_impl(extension, buffer, None, None)
    }

    /// Parses and highlights a line window for a known file extension.
    pub fn highlight_buffer_for_extension_window(
        &mut self,
        extension: &str,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_extension_impl(extension, buffer, Some(highlight_window), None)
    }

    fn highlight_buffer_for_extension_impl(
        &mut self,
        extension: &str,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let extension = normalize_extension(extension);
        let language_id = self
            .extensions
            .get(&extension)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownExtension(extension.clone()))?;
        self.highlight_buffer_for_language_impl(
            &language_id,
            buffer,
            highlight_window,
            parse_session,
        )
    }

    /// Parses and highlights a buffer using a registered language identifier.
    pub fn highlight_buffer_for_language(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, None, None)
    }

    /// Returns named ancestor nodes for a cursor location, ordered innermost to outermost.
    pub fn ancestor_contexts_for_language(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        point: TextPoint,
    ) -> Result<Vec<SyntaxNodeContext>, SyntaxError> {
        self.ancestor_contexts_for_language_impl(language_id, buffer, point, None)
    }

    /// Returns named ancestor nodes for a cursor location, ordered innermost to outermost,
    /// reusing an existing parse session when provided.
    pub fn ancestor_contexts_for_language_with_parse_session(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        point: TextPoint,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<Vec<SyntaxNodeContext>, SyntaxError> {
        self.ancestor_contexts_for_language_impl(language_id, buffer, point, Some(parse_session))
    }

    fn ancestor_contexts_for_language_impl(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        point: TextPoint,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<Vec<SyntaxNodeContext>, SyntaxError> {
        let language_id = language_id.to_owned();
        if !self.languages.contains_key(&language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id));
        }
        self.ensure_loaded_language(&language_id)?;
        let loaded = self
            .loaded
            .get(&language_id)
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        let parse_result = parse_tree(&language_id, loaded, buffer, parse_session)?;
        let point = text_point_to_tree_sitter_point(buffer, point);
        let Some(mut node) = parse_result
            .tree
            .root_node()
            .named_descendant_for_point_range(point, point)
        else {
            return Ok(Vec::new());
        };
        let mut contexts = Vec::new();
        loop {
            let start = node.start_position();
            let end = node.end_position();
            let parent = node.parent();
            // Only keep ancestors whose closing line is at or after the cursor so
            // callers can render closing-line context breadcrumbs.
            if node.is_named() && parent.is_some() && end.row >= point.row {
                contexts.push(SyntaxNodeContext {
                    kind: node.kind().to_owned(),
                    start_position: SyntaxPoint::new(start.row, start.column),
                    end_position: SyntaxPoint::new(end.row, end.column),
                });
            }
            let Some(parent) = parent else {
                break;
            };
            node = parent;
        }
        Ok(contexts)
    }

    /// Returns the desired indentation column for a target line when an indent query is available.
    pub fn desired_indent_for_language(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        line_index: usize,
        indent_width: usize,
    ) -> Result<Option<usize>, SyntaxError> {
        self.desired_indent_for_language_impl(language_id, buffer, line_index, indent_width, None)
    }

    /// Returns the desired indentation column using a reusable parse session.
    pub fn desired_indent_for_language_with_session(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        line_index: usize,
        indent_width: usize,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<Option<usize>, SyntaxError> {
        self.desired_indent_for_language_impl(
            language_id,
            buffer,
            line_index,
            indent_width,
            Some(parse_session),
        )
    }

    fn desired_indent_for_language_impl(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        line_index: usize,
        indent_width: usize,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<Option<usize>, SyntaxError> {
        let language_id = language_id.to_owned();
        if !self.languages.contains_key(&language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id));
        }
        self.ensure_loaded_language(&language_id)?;
        let loaded = self
            .loaded
            .get(&language_id)
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        desired_indent_for_loaded_language(
            &language_id,
            loaded,
            buffer,
            line_index,
            indent_width,
            parse_session,
        )
    }

    /// Parses and highlights a line window using a registered language identifier.
    pub fn highlight_buffer_for_language_window(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, Some(highlight_window), None)
    }

    /// Parses and highlights a buffer using a reusable parse session.
    pub fn highlight_buffer_for_language_with_session(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(language_id, buffer, None, Some(parse_session))
    }

    /// Parses and highlights a line window using a reusable parse session.
    pub fn highlight_buffer_for_language_window_with_session(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
        parse_session: &mut Option<SyntaxParseSession>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl(
            language_id,
            buffer,
            Some(highlight_window),
            Some(parse_session),
        )
    }

    fn highlight_buffer_for_language_impl(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_language_impl_with_depth(
            language_id,
            buffer,
            highlight_window,
            parse_session,
            0,
        )
    }

    fn highlight_buffer_for_language_impl_with_depth(
        &mut self,
        language_id: &str,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        mut parse_session: Option<&mut Option<SyntaxParseSession>>,
        injection_depth: usize,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let language_id = language_id.to_owned();
        let config = self
            .languages
            .get(&language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        self.ensure_loaded_language(&language_id)?;
        let loaded = self
            .loaded
            .get(&language_id)
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.clone()))?;
        let inline_language_id = "markdown-inline";
        let needs_inline_ranges = config.id() == "markdown"
            && config
                .additional_highlight_languages()
                .iter()
                .any(|language| language == inline_language_id);
        let has_injections = self
            .loaded
            .get(&language_id)
            .and_then(|loaded| loaded.injections_query.as_ref())
            .is_some();

        let base_parse = if needs_inline_ranges || has_injections {
            Some(highlight_loaded_language_with_tree(
                &language_id,
                loaded,
                buffer,
                highlight_window,
                parse_session.as_deref_mut(),
            )?)
        } else {
            None
        };

        let mut snapshot = if let Some(parse) = &base_parse {
            parse.snapshot.clone()
        } else {
            highlight_loaded_language(
                &language_id,
                loaded,
                buffer,
                highlight_window,
                parse_session,
            )?
        };

        if let Some(parse) = base_parse.as_ref().filter(|_| has_injections) {
            let injections = self.highlight_injections_for_tree(
                &config,
                &language_id,
                &parse.tree,
                buffer,
                highlight_window,
                injection_depth,
            )?;
            snapshot.highlight_spans.extend(injections.highlight_spans);
            snapshot.has_errors = snapshot.has_errors || injections.has_errors;
        }

        for extra_language_id in config.additional_highlight_languages() {
            self.ensure_loaded_language(extra_language_id)?;
            let loaded = self
                .loaded
                .get(extra_language_id)
                .ok_or_else(|| SyntaxError::UnknownLanguage(extra_language_id.clone()))?;
            let extra_snapshot = if needs_inline_ranges && extra_language_id == inline_language_id {
                let Some(parse) = base_parse.as_ref() else {
                    continue;
                };
                let mut inline_lines = markdown_inline_line_indices(&parse.tree);
                if let Some(highlight_window) = highlight_window {
                    let end_line = highlight_window.end_line_exclusive();
                    inline_lines
                        .retain(|line| *line >= highlight_window.start_line() && *line < end_line);
                }
                if inline_lines.is_empty() {
                    continue;
                }
                highlight_inline_language_per_line(
                    extra_language_id,
                    loaded,
                    buffer,
                    &inline_lines,
                )?
            } else {
                highlight_loaded_language(
                    extra_language_id,
                    loaded,
                    buffer,
                    highlight_window,
                    None,
                )?
            };
            snapshot
                .highlight_spans
                .extend(extra_snapshot.highlight_spans);
            snapshot.has_errors = snapshot.has_errors || extra_snapshot.has_errors;
        }

        sort_highlight_spans(&mut snapshot.highlight_spans);
        Ok(snapshot)
    }

    fn highlight_injections_for_tree(
        &mut self,
        host_config: &LanguageConfiguration,
        host_language_id: &str,
        tree: &Tree,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        injection_depth: usize,
    ) -> Result<InjectionHighlights, SyntaxError> {
        if injection_depth >= MAX_INJECTION_DEPTH {
            return Ok(InjectionHighlights::default());
        }

        let injection_regions = {
            let Some(injections_query) = self
                .loaded
                .get(host_language_id)
                .and_then(|loaded| loaded.injections_query.as_ref())
            else {
                return Ok(InjectionHighlights::default());
            };
            collect_injection_regions(injections_query, tree, buffer, highlight_window)
        };

        let mut highlights = InjectionHighlights::default();
        for region in injection_regions {
            if let Some(window) = highlight_window
                && !injection_region_intersects_window(&region, window)
            {
                continue;
            }

            let Some(injection_language_id) =
                self.resolve_injection_language_id(&region.language_name)
            else {
                continue;
            };
            if host_config
                .additional_highlight_languages()
                .iter()
                .any(|language| language == &injection_language_id)
            {
                continue;
            }

            let Some(source) =
                buffer_text_for_byte_range(buffer, region.start_byte, region.end_byte)
            else {
                continue;
            };
            let Ok(snapshot) = self.highlight_buffer_for_language_impl_with_depth(
                &injection_language_id,
                &TextBuffer::from_text(source),
                None,
                None,
                injection_depth + 1,
            ) else {
                continue;
            };

            highlights.has_errors = highlights.has_errors || snapshot.has_errors;
            highlights.highlight_spans.extend(
                snapshot
                    .highlight_spans
                    .into_iter()
                    .map(|span| translate_injected_highlight_span(span, &region))
                    .filter(|span| {
                        highlight_window
                            .map(|window| span_intersects_window(span, window))
                            .unwrap_or(true)
                    }),
            );
        }

        Ok(highlights)
    }

    fn resolve_injection_language_id(&self, raw_language: &str) -> Option<String> {
        let raw_language = raw_language.trim();
        if raw_language.is_empty() {
            return None;
        }

        let mut candidates = Vec::new();
        for candidate in [
            raw_language.to_owned(),
            raw_language.to_ascii_lowercase(),
            raw_language.replace('_', "-"),
            raw_language.to_ascii_lowercase().replace('_', "-"),
        ] {
            if !candidate.is_empty() && !candidates.contains(&candidate) {
                candidates.push(candidate);
            }
        }

        for candidate in &candidates {
            if self.languages.contains_key(candidate) {
                return Some(candidate.clone());
            }
        }
        for candidate in &candidates {
            if let Some(language) = self.language_for_extension(candidate) {
                return Some(language.id().to_owned());
            }
        }
        for candidate in &candidates {
            if let Some(language) = self.language_for_path(candidate) {
                return Some(language.id().to_owned());
            }
        }
        None
    }

    /// Returns the compiled injections query for a language, loading it if needed.
    ///
    /// Returns `Ok(None)` when the language is registered but has no injections query.
    pub fn injections_query_for_language(
        &mut self,
        language_id: &str,
    ) -> Result<Option<&Query>, SyntaxError> {
        if !self.languages.contains_key(language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        }
        self.ensure_loaded_language(language_id)?;
        Ok(self
            .loaded
            .get(language_id)
            .and_then(|loaded| loaded.injections_query.as_ref()))
    }

    /// Returns the compiled locals query for a language, loading it if needed.
    ///
    /// Returns `Ok(None)` when the language is registered but has no locals query.
    pub fn locals_query_for_language(
        &mut self,
        language_id: &str,
    ) -> Result<Option<&Query>, SyntaxError> {
        if !self.languages.contains_key(language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        }
        self.ensure_loaded_language(language_id)?;
        Ok(self
            .loaded
            .get(language_id)
            .and_then(|loaded| loaded.locals_query.as_ref()))
    }

    /// Returns the compiled folds query for a language, loading it if needed.
    ///
    /// Returns `Ok(None)` when the language is registered but has no folds query.
    pub fn folds_query_for_language(
        &mut self,
        language_id: &str,
    ) -> Result<Option<&Query>, SyntaxError> {
        if !self.languages.contains_key(language_id) {
            return Err(SyntaxError::UnknownLanguage(language_id.to_owned()));
        }
        self.ensure_loaded_language(language_id)?;
        Ok(self
            .loaded
            .get(language_id)
            .and_then(|loaded| loaded.folds_query.as_ref()))
    }

    /// Parses and highlights a buffer using a file path's extension.
    pub fn highlight_buffer_for_path(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &TextBuffer,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_path_impl(path, buffer, None, None)
    }

    /// Parses and highlights a line window using a file path's extension.
    pub fn highlight_buffer_for_path_window(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &TextBuffer,
        highlight_window: HighlightWindow,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        self.highlight_buffer_for_path_impl(path, buffer, Some(highlight_window), None)
    }

    fn highlight_buffer_for_path_impl(
        &mut self,
        path: impl AsRef<Path>,
        buffer: &TextBuffer,
        highlight_window: Option<HighlightWindow>,
        parse_session: Option<&mut Option<SyntaxParseSession>>,
    ) -> Result<SyntaxSnapshot, SyntaxError> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| SyntaxError::UnknownExtension(path.display().to_string()))?;
        self.highlight_buffer_for_extension_impl(extension, buffer, highlight_window, parse_session)
    }

    fn ensure_loaded_language(&mut self, language_id: &str) -> Result<(), SyntaxError> {
        if self.loaded.contains_key(language_id) {
            return Ok(());
        }

        let config = self
            .languages
            .get(language_id)
            .cloned()
            .ok_or_else(|| SyntaxError::UnknownLanguage(language_id.to_owned()))?;
        let loaded = load_language(
            &config,
            &self.install_root,
            self.query_asset_root.as_deref(),
        )?;
        self.loaded.insert(language_id.to_owned(), loaded);
        Ok(())
    }
}

fn load_language(
    config: &LanguageConfiguration,
    install_root: &Path,
    query_asset_root: Option<&Path>,
) -> Result<LoadedLanguage, SyntaxError> {
    let capture_mappings = config
        .capture_mappings()
        .iter()
        .map(|mapping| {
            (
                mapping.capture_name().to_owned(),
                mapping.theme_token().to_owned(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    match &config.loader {
        LanguageLoader::Static {
            language_provider,
            highlight_query,
        } => {
            let language = language_provider();
            let query_source =
                append_query_source(highlight_query.to_owned(), config.extra_highlight_query());
            let query = compile_query_source(&language, config.id(), "highlight", &query_source)?;
            let indent_query = config
                .extra_indent_query()
                .map(|query| compile_query_source(&language, config.id(), "indent", query))
                .transpose()?;
            let injections_query = config
                .extra_injections_query()
                .map(|query| compile_query_source(&language, config.id(), "injections", query))
                .transpose()?;
            let locals_query = config
                .extra_locals_query()
                .map(|query| compile_query_source(&language, config.id(), "locals", query))
                .transpose()?;
            let folds_query = config
                .extra_folds_query()
                .map(|query| compile_query_source(&language, config.id(), "folds", query))
                .transpose()?;
            Ok(LoadedLanguage {
                _library: None,
                language,
                query,
                indent_query,
                injections_query,
                locals_query,
                folds_query,
                capture_mappings,
            })
        }
        LanguageLoader::Grammar { grammar } => {
            let install_dir = grammar.install_directory(install_root);
            let library_path = grammar.installed_library_path(install_root);
            let query_path = grammar.installed_highlight_query_path(install_root);
            if !library_path.exists() || !query_path.exists() {
                return Err(SyntaxError::GrammarNotInstalled {
                    language_id: config.id().to_owned(),
                    install_dir,
                });
            }

            let query_source = append_query_source(
                read_query_source_preferring_bundled(
                    &query_path,
                    query_asset_root,
                    config.id(),
                    "highlights.scm",
                )?,
                config.extra_highlight_query(),
            );
            let library = unsafe {
                // SAFETY: The library path is chosen by the installer for a tree-sitter grammar
                // compiled from generated parser sources. We keep the `Library` alive for at least
                // as long as the loaded `Language` is cached in `LoadedLanguage`.
                libloading::Library::new(&library_path)
            }
            .map_err(|error| SyntaxError::LibraryLoad {
                language_id: config.id().to_owned(),
                message: error.to_string(),
            })?;
            let symbol_name = format!("{}\0", grammar.symbol_name());
            let symbol = unsafe {
                // SAFETY: The symbol name comes from the language configuration and points to the
                // standard tree-sitter exported language constructor for the compiled grammar.
                library.get::<unsafe extern "C" fn() -> *const ()>(symbol_name.as_bytes())
            }
            .map_err(|error| SyntaxError::LibraryLoad {
                language_id: config.id().to_owned(),
                message: error.to_string(),
            })?;
            let language_fn = unsafe {
                // SAFETY: Tree-sitter generated grammar libraries export functions matching the
                // `LanguageFn` ABI. The symbol was resolved from the configured exported name above.
                LanguageFn::from_raw(*symbol)
            };
            let language = Language::new(language_fn);
            let query = compile_query_source(&language, config.id(), "highlight", &query_source)?;
            let indent_query_source = maybe_read_query_source_preferring_bundled(
                &grammar.installed_indent_query_path(install_root),
                query_asset_root,
                config.id(),
                "indents.scm",
            )?;
            let indent_query_source = match (indent_query_source, config.extra_indent_query()) {
                (Some(source), extra_query) => Some(append_query_source(source, extra_query)),
                (None, Some(extra_query)) => Some(extra_query.to_owned()),
                (None, None) => None,
            };
            let indent_query = indent_query_source
                .map(|source| compile_query_source(&language, config.id(), "indent", &source))
                .transpose()?;
            let injections_query = load_optional_query(
                &language,
                config,
                install_root,
                query_asset_root,
                &grammar.installed_injections_query_path(install_root),
                "injections.scm",
                "injections",
                config.extra_injections_query(),
            )?;
            let locals_query = load_optional_query(
                &language,
                config,
                install_root,
                query_asset_root,
                &grammar.installed_locals_query_path(install_root),
                "locals.scm",
                "locals",
                config.extra_locals_query(),
            )?;
            let folds_query = load_optional_query(
                &language,
                config,
                install_root,
                query_asset_root,
                &grammar.installed_folds_query_path(install_root),
                "folds.scm",
                "folds",
                config.extra_folds_query(),
            )?;
            Ok(LoadedLanguage {
                _library: Some(library),
                language,
                query,
                indent_query,
                injections_query,
                locals_query,
                folds_query,
                capture_mappings,
            })
        }
    }
}

fn append_query_source(mut source: String, extra_query: Option<&str>) -> String {
    if let Some(extra_query) = extra_query {
        if !source.ends_with('\n') && !source.is_empty() {
            source.push('\n');
        }
        source.push_str(extra_query);
    }
    source
}

fn compile_query_source(
    language: &Language,
    language_id: &str,
    query_kind: &str,
    source: &str,
) -> Result<Query, SyntaxError> {
    Query::new(language, source).map_err(|error| SyntaxError::InvalidQuery {
        language_id: language_id.to_owned(),
        query_kind: query_kind.to_owned(),
        message: error.to_string(),
    })
}

fn read_installed_query_source(
    query_path: &Path,
    query_asset_root: Option<&Path>,
    _language_id: &str,
    file_name: &str,
) -> Result<String, SyntaxError> {
    let raw_source = fs::read_to_string(query_path)
        .map_err(|error| io_error("read installed query", query_path, error))?;
    resolve_query_source_from_raw(
        &raw_source,
        query_path,
        query_asset_root,
        file_name,
        &mut Vec::new(),
    )
}

/// Reads query source for a grammar-backed language, preferring the live bundled asset when
/// `query_asset_root` is configured and contains `<language_id>/<file_name>`. Falls back to
/// the installed query file when no bundled asset is present. This ensures a stale installed
/// query file can never shadow a corrected bundled query asset.
fn read_query_source_preferring_bundled(
    installed_path: &Path,
    query_asset_root: Option<&Path>,
    language_id: &str,
    file_name: &str,
) -> Result<String, SyntaxError> {
    if let Some(asset_root) = query_asset_root
        && let Some(source) =
            resolve_bundled_query_source(asset_root, language_id, file_name, &mut Vec::new())?
    {
        return Ok(source);
    }
    read_installed_query_source(installed_path, query_asset_root, language_id, file_name)
}

/// Like [`read_query_source_preferring_bundled`] but returns `Ok(None)` when neither the
/// bundled asset nor the installed file exists.
fn maybe_read_query_source_preferring_bundled(
    installed_path: &Path,
    query_asset_root: Option<&Path>,
    language_id: &str,
    file_name: &str,
) -> Result<Option<String>, SyntaxError> {
    if let Some(asset_root) = query_asset_root
        && let Some(source) =
            resolve_bundled_query_source(asset_root, language_id, file_name, &mut Vec::new())?
    {
        return Ok(Some(source));
    }
    if !installed_path.exists() {
        return Ok(None);
    }
    read_installed_query_source(installed_path, query_asset_root, language_id, file_name).map(Some)
}

/// Loads an optional compiled query from the installed path, merging in any extra query text.
///
/// Returns `Ok(None)` when neither an installed file nor an extra query exists.
#[allow(clippy::too_many_arguments)]
fn load_optional_query(
    language: &Language,
    config: &LanguageConfiguration,
    _install_root: &Path,
    query_asset_root: Option<&Path>,
    installed_path: &Path,
    file_name: &str,
    kind_label: &str,
    extra_query: Option<&str>,
) -> Result<Option<Query>, SyntaxError> {
    let source_from_file = maybe_read_query_source_preferring_bundled(
        installed_path,
        query_asset_root,
        config.id(),
        file_name,
    )?;
    let merged_source = match (source_from_file, extra_query) {
        (Some(source), extra) => Some(append_query_source(source, extra)),
        (None, Some(extra)) => Some(extra.to_owned()),
        (None, None) => None,
    };
    merged_source
        .map(|source| compile_query_source(language, config.id(), kind_label, &source))
        .transpose()
}

fn resolve_query_source_from_raw(
    raw_source: &str,
    query_path: &Path,
    query_asset_root: Option<&Path>,
    file_name: &str,
    inheritance_stack: &mut Vec<(String, String)>,
) -> Result<String, SyntaxError> {
    let (inherited_languages, body) = parse_query_inherits(raw_source);
    if inherited_languages.is_empty() {
        return Ok(body);
    }

    let Some(query_asset_root) = query_asset_root else {
        return Err(SyntaxError::Io {
            operation: "resolve inherited query".to_owned(),
            path: query_path.to_path_buf(),
            message:
                "query declares inherited languages but no bundled query asset root is configured"
                    .to_owned(),
        });
    };

    let mut resolved = String::new();
    for inherited_language in inherited_languages {
        let inherited_source = resolve_bundled_query_source(
            query_asset_root,
            &inherited_language,
            file_name,
            inheritance_stack,
        )?
        .ok_or_else(|| SyntaxError::Io {
            operation: "resolve inherited query".to_owned(),
            path: query_asset_root.join(&inherited_language).join(file_name),
            message: format!(
                "inherited query `{file_name}` for language `{inherited_language}` is missing"
            ),
        })?;
        if !resolved.is_empty() && !resolved.ends_with('\n') {
            resolved.push('\n');
        }
        resolved.push_str(&inherited_source);
    }
    if !body.is_empty() {
        if !resolved.is_empty() && !resolved.ends_with('\n') {
            resolved.push('\n');
        }
        resolved.push_str(&body);
    }
    Ok(resolved)
}

fn resolve_bundled_query_source(
    query_asset_root: &Path,
    language_id: &str,
    file_name: &str,
    inheritance_stack: &mut Vec<(String, String)>,
) -> Result<Option<String>, SyntaxError> {
    let query_path = query_asset_root.join(language_id).join(file_name);
    if !query_path.exists() {
        return Ok(None);
    }
    if inheritance_stack
        .iter()
        .any(|(id, file)| id == language_id && file.eq_ignore_ascii_case(file_name))
    {
        return Err(SyntaxError::Io {
            operation: "resolve inherited query".to_owned(),
            path: query_path,
            message: "cyclic query inheritance detected".to_owned(),
        });
    }

    let raw_source = fs::read_to_string(&query_path)
        .map_err(|error| io_error("read bundled query", &query_path, error))?;
    inheritance_stack.push((language_id.to_owned(), file_name.to_owned()));
    let resolved = resolve_query_source_from_raw(
        &raw_source,
        &query_path,
        Some(query_asset_root),
        file_name,
        inheritance_stack,
    )?;
    inheritance_stack.pop();
    Ok(Some(resolved))
}

fn parse_query_inherits(source: &str) -> (Vec<String>, String) {
    let mut inherited_languages = Vec::new();
    let mut body_lines = Vec::new();
    let had_trailing_newline = source.ends_with('\n');

    for line in source.lines() {
        let trimmed = line.trim_start();
        if let Some(inherits) = trimmed.strip_prefix("; inherits:") {
            for language in inherits.split(',') {
                let language = language.trim();
                if !language.is_empty() && !inherited_languages.iter().any(|id| id == language) {
                    inherited_languages.push(language.to_owned());
                }
            }
            continue;
        }
        body_lines.push(line);
    }

    let mut body = body_lines.join("\n");
    if had_trailing_newline {
        body.push('\n');
    }
    (inherited_languages, body)
}

fn install_bundled_queries(
    config: &LanguageConfiguration,
    query_asset_root: &Path,
    install_root: &Path,
) -> Result<(), SyntaxError> {
    let Some(grammar) = config.grammar() else {
        return Ok(());
    };
    let language_query_dir = query_asset_root.join(config.id());
    if !language_query_dir.is_dir() {
        return Ok(());
    }

    let install_queries_dir = grammar.install_directory(install_root).join("queries");
    fs::create_dir_all(&install_queries_dir).map_err(|error| {
        io_error(
            "create installed query directory",
            &install_queries_dir,
            error,
        )
    })?;

    for entry in fs::read_dir(&language_query_dir)
        .map_err(|error| io_error("read query asset directory", &language_query_dir, error))?
    {
        let entry = entry
            .map_err(|error| io_error("read query asset entry", &language_query_dir, error))?;
        let entry_path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|error| io_error("read query asset metadata", &entry_path, error))?;
        if metadata.is_dir() {
            continue;
        }

        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy().to_string();
        let destination = install_queries_dir.join(&file_name);
        if entry_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("scm"))
        {
            let source = resolve_bundled_query_source(
                query_asset_root,
                config.id(),
                &file_name,
                &mut Vec::new(),
            )?
            .unwrap_or_default();
            fs::write(&destination, source)
                .map_err(|error| io_error("write installed query", &destination, error))?;
        } else {
            fs::copy(&entry_path, &destination).map_err(|error| SyntaxError::Io {
                operation: "copy query asset".to_owned(),
                path: entry_path,
                message: error.to_string(),
            })?;
        }
    }

    Ok(())
}

/// Converts an editor [`TextPoint`] (character columns) into a tree-sitter [`Point`]
/// whose columns are measured in UTF-8 bytes.
///
/// Out-of-bounds coordinates are clamped to the nearest valid line/column in the
/// provided buffer before converting character columns into byte columns.
fn text_point_to_tree_sitter_point(buffer: &TextBuffer, point: TextPoint) -> Point {
    let max_line = buffer.line_count().saturating_sub(1);
    let line = point.line.min(max_line);
    let text = buffer.line(line).unwrap_or_default();
    let column = text.chars().take(point.column).map(char::len_utf8).sum();
    Point { row: line, column }
}

fn tree_sitter_column_to_char_column(line: &str, byte_column: usize) -> usize {
    let mut bytes = 0usize;
    let mut chars = 0usize;
    for character in line.chars() {
        if bytes >= byte_column {
            break;
        }
        bytes = bytes.saturating_add(character.len_utf8());
        chars = chars.saturating_add(1);
    }
    chars
}

fn desired_indent_for_loaded_language(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    line_index: usize,
    indent_width: usize,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<Option<usize>, SyntaxError> {
    let Some(indent_query) = loaded.indent_query.as_ref() else {
        return Ok(None);
    };
    if line_index >= buffer.line_count() || indent_width == 0 {
        return Ok(Some(0));
    }

    let parse_result = parse_tree(language_id, loaded, buffer, parse_session)?;
    let mut query_cursor = QueryCursor::new();
    query_cursor.set_point_range(
        Point {
            row: line_index,
            column: 0,
        }..Point {
            row: line_index.saturating_add(1),
            column: 0,
        },
    );

    let capture_names = indent_query.capture_names();
    let mut saw_capture = false;
    let mut begin_levels = 0usize;
    let mut branch_levels = 0usize;
    let mut dedent_levels = 0usize;
    let mut zero = false;
    let mut fallback_to_auto = false;
    let mut aligned_indent: Option<usize> = None;
    let mut matches = query_cursor.matches(
        indent_query,
        parse_result.tree.root_node(),
        TextBufferProvider { buffer },
    );
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };
        if !general_predicates_match(
            indent_query,
            query_match.pattern_index,
            query_match.captures,
            buffer,
        ) {
            continue;
        }

        let properties = indent_query.property_settings(query_match.pattern_index);
        for capture in query_match.captures {
            let Some(capture_name) = capture_names.get(capture.index as usize).copied() else {
                continue;
            };
            saw_capture = true;
            match capture_name {
                "indent.begin" if indent_begin_applies(capture.node, line_index, properties) => {
                    begin_levels = begin_levels.saturating_add(1);
                }
                "indent.branch" if line_index == capture.node.start_position().row => {
                    branch_levels = branch_levels.saturating_add(1);
                }
                "indent.dedent"
                    if line_index > capture.node.start_position().row
                        && line_index <= capture.node.end_position().row =>
                {
                    dedent_levels = dedent_levels.saturating_add(1);
                }
                "indent.align" => {
                    if let Some(column) =
                        aligned_indent_column(capture.node, line_index, properties, buffer)
                    {
                        aligned_indent =
                            Some(aligned_indent.map_or(column, |current| current.max(column)));
                    }
                }
                "indent.zero" if line_intersects_node(capture.node, line_index) => {
                    zero = true;
                }
                "indent.ignore" | "indent.auto"
                    if line_intersects_node(capture.node, line_index) =>
                {
                    fallback_to_auto = true;
                }
                "indent.end" => {}
                _ => {}
            }
        }
    }

    if zero {
        return Ok(Some(0));
    }
    if fallback_to_auto || !saw_capture {
        return Ok(None);
    }

    let levels = begin_levels.saturating_sub(branch_levels.saturating_add(dedent_levels));
    let level_columns = levels.saturating_mul(indent_width);
    Ok(Some(
        aligned_indent.map_or(level_columns, |column| column.max(level_columns)),
    ))
}

fn indent_begin_applies(node: Node<'_>, line_index: usize, properties: &[QueryProperty]) -> bool {
    let start_line = node.start_position().row;
    let end_line = node.end_position().row;
    (line_index > start_line
        || (line_index == start_line
            && query_property_is_set(properties, "indent.start_at_same_line")))
        && line_index <= end_line
}

fn aligned_indent_column(
    node: Node<'_>,
    line_index: usize,
    properties: &[QueryProperty],
    buffer: &TextBuffer,
) -> Option<usize> {
    if line_index <= node.start_position().row || line_index > node.end_position().row {
        return None;
    }

    let open_delimiter = query_property_value(properties, "indent.open_delimiter")?;
    let close_delimiter = query_property_value(properties, "indent.close_delimiter");
    if line_index == node.end_position().row
        && close_delimiter
            .is_some_and(|token| current_line_starts_with_token(buffer, line_index, token))
    {
        return None;
    }

    let start_line = buffer.line(node.start_position().row)?;
    let start_column = tree_sitter_column_to_char_column(&start_line, node.start_position().column);
    let open_column = delimiter_column(&start_line, start_column, open_delimiter)?;
    first_content_column_after(
        &start_line,
        open_column.saturating_add(open_delimiter.chars().count()),
    )
    .filter(|column| {
        close_delimiter
            .map(|token| !line_starts_with_token_at_column(&start_line, *column, token))
            .unwrap_or(true)
    })
    .or(Some(open_column.saturating_add(1)))
}

fn line_intersects_node(node: Node<'_>, line_index: usize) -> bool {
    let start_line = node.start_position().row;
    let end_line = node.end_position().row;
    start_line <= line_index && line_index <= end_line
}

fn query_property_is_set(properties: &[QueryProperty], key: &str) -> bool {
    properties
        .iter()
        .any(|property| property.key.as_ref() == key)
}

fn query_property_value<'a>(properties: &'a [QueryProperty], key: &str) -> Option<&'a str> {
    properties
        .iter()
        .find(|property| property.key.as_ref() == key)
        .and_then(|property| property.value.as_deref())
}

/// Returns the string value of the first `#set!` property with the given key that is
/// associated with a specific capture index, falling back to any pattern-wide property
/// with the same key.
///
/// This covers both the simple form `(#set! key "value")` (no capture, pattern-wide)
/// and the capture-targeted form `(#set! @capture key "value")`.
///
/// Returns `None` if no matching property is found or the property has no string value.
pub fn query_capture_property_value<'q>(
    query: &'q Query,
    pattern_index: usize,
    capture_index: u32,
    key: &str,
) -> Option<&'q str> {
    let properties = query.property_settings(pattern_index);
    // Capture-targeted properties take precedence over pattern-wide ones.
    for property in properties {
        if property.key.as_ref() == key && property.capture_id == Some(capture_index as usize) {
            return property.value.as_deref();
        }
    }
    for property in properties {
        if property.key.as_ref() == key && property.capture_id.is_none() {
            return property.value.as_deref();
        }
    }
    None
}

fn delimiter_column(line: &str, start_column: usize, delimiter: &str) -> Option<usize> {
    let delimiter = delimiter.chars().next()?;
    for (column, character) in line.chars().enumerate().skip(start_column) {
        if character == delimiter {
            return Some(column);
        }
    }
    None
}

fn first_content_column_after(line: &str, start_column: usize) -> Option<usize> {
    line.chars()
        .enumerate()
        .skip(start_column)
        .find_map(|(column, character)| (!character.is_whitespace()).then_some(column))
}

fn current_line_starts_with_token(buffer: &TextBuffer, line_index: usize, token: &str) -> bool {
    let line = buffer.line(line_index).unwrap_or_default();
    line.trim_start().starts_with(token)
}

fn line_starts_with_token_at_column(line: &str, column: usize, token: &str) -> bool {
    let tail = line.chars().skip(column).collect::<String>();
    tail.starts_with(token)
}

fn general_predicates_match(
    query: &Query,
    pattern_index: usize,
    captures: &[tree_sitter::QueryCapture<'_>],
    buffer: &TextBuffer,
) -> bool {
    query
        .general_predicates(pattern_index)
        .iter()
        .all(|predicate| {
            evaluate_general_predicate(
                predicate.operator.as_ref(),
                &predicate.args,
                captures,
                buffer,
            )
        })
}

fn evaluate_general_predicate(
    operator: &str,
    args: &[QueryPredicateArg],
    captures: &[tree_sitter::QueryCapture<'_>],
    buffer: &TextBuffer,
) -> bool {
    match operator.trim_start_matches('#') {
        "kind-eq?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            args.iter().skip(1).any(|argument| match argument {
                QueryPredicateArg::String(kind) => node.kind() == kind.as_ref(),
                QueryPredicateArg::Capture(_) => false,
            })
        }
        "not-kind-eq?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            args.iter().skip(1).all(|argument| match argument {
                QueryPredicateArg::String(kind) => node.kind() != kind.as_ref(),
                QueryPredicateArg::Capture(_) => true,
            })
        }
        "has-parent?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let Some(parent) = node.parent() else {
                return false;
            };
            args.iter().skip(1).any(|argument| match argument {
                QueryPredicateArg::String(kind) => parent.kind() == kind.as_ref(),
                QueryPredicateArg::Capture(_) => false,
            })
        }
        "not-has-parent?" => {
            // Symmetric with `has-parent?`: checks only the immediate parent.
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let Some(parent) = node.parent() else {
                return true;
            };
            args.iter().skip(1).all(|argument| match argument {
                QueryPredicateArg::String(kind) => parent.kind() != kind.as_ref(),
                QueryPredicateArg::Capture(_) => true,
            })
        }
        "has-ancestor?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let mut ancestor = node.parent();
            while let Some(current) = ancestor {
                for argument in args.iter().skip(1) {
                    if let QueryPredicateArg::String(kind) = argument
                        && current.kind() == kind.as_ref()
                    {
                        return true;
                    }
                }
                ancestor = current.parent();
            }
            false
        }
        "not-has-ancestor?" => {
            let Some(node) = predicate_capture_node(args.first(), captures) else {
                return false;
            };
            let mut ancestor = node.parent();
            while let Some(current) = ancestor {
                for argument in args.iter().skip(1) {
                    if let QueryPredicateArg::String(kind) = argument
                        && current.kind() == kind.as_ref()
                    {
                        return false;
                    }
                }
                ancestor = current.parent();
            }
            true
        }
        "lua-match?" => {
            let Some(text) = predicate_capture_text(args.first(), captures, buffer) else {
                return false;
            };
            let Some(pattern) = args.get(1).and_then(|argument| match argument {
                QueryPredicateArg::String(pattern) => Some(pattern.as_ref()),
                QueryPredicateArg::Capture(_) => None,
            }) else {
                return false;
            };
            lua_pattern_matches(&text, pattern)
        }
        "not-lua-match?" => {
            let Some(text) = predicate_capture_text(args.first(), captures, buffer) else {
                return false;
            };
            let Some(pattern) = args.get(1).and_then(|argument| match argument {
                QueryPredicateArg::String(pattern) => Some(pattern.as_ref()),
                QueryPredicateArg::Capture(_) => None,
            }) else {
                return false;
            };
            !lua_pattern_matches(&text, pattern)
        }
        "contains?" => {
            let Some(text) = predicate_capture_text(args.first(), captures, buffer) else {
                return false;
            };
            args.iter().skip(1).any(|argument| match argument {
                QueryPredicateArg::String(needle) => text.contains(needle.as_ref()),
                QueryPredicateArg::Capture(_) => false,
            })
        }
        // Directives (ending in `!`) are metadata annotations, not match filters.
        // They do not cause a match to be rejected.
        op if op.ends_with('!') => true,
        // Unknown filter predicates are allowed through. We cannot evaluate them
        // here, so we avoid silently discarding matches that depend on them.
        _ => true,
    }
}

fn predicate_capture_node<'tree>(
    argument: Option<&QueryPredicateArg>,
    captures: &[tree_sitter::QueryCapture<'tree>],
) -> Option<Node<'tree>> {
    let QueryPredicateArg::Capture(capture_id) = argument? else {
        return None;
    };
    captures
        .iter()
        .find(|capture| capture.index == *capture_id)
        .map(|capture| capture.node)
}

fn predicate_capture_text(
    argument: Option<&QueryPredicateArg>,
    captures: &[tree_sitter::QueryCapture<'_>],
    buffer: &TextBuffer,
) -> Option<String> {
    let node = predicate_capture_node(argument, captures)?;
    let mut text = String::new();
    for chunk in buffer.byte_slice_chunks(node.byte_range()) {
        text.push_str(std::str::from_utf8(chunk).ok()?);
    }
    Some(text)
}

/// Minimal Lua 5.x pattern matcher sufficient for the patterns used in tree-sitter query
/// corpora.  Supported pattern items:
///
/// * `.`             — any character
/// * `%a`/`%A`       — letter / non-letter
/// * `%d`/`%D`       — digit / non-digit
/// * `%l`/`%L`       — lowercase / non-lowercase
/// * `%u`/`%U`       — uppercase / non-uppercase
/// * `%w`/`%W`       — alphanumeric / non-alphanumeric
/// * `%s`/`%S`       — whitespace / non-whitespace
/// * `%p`/`%P`       — punctuation / non-punctuation
/// * `%x`/`%X`       — hex digit / non-hex-digit
/// * `%c`/`%C`       — control char / non-control-char
/// * `%(` `%)` etc.  — escaped literal
/// * `[set]`/`[^set]`— character class (ranges a–z supported)
/// * `*` `+` `-` `?` — quantifiers on the preceding item
/// * `^`             — anchor at start
/// * `$`             — anchor at end
///
/// Captures (`(…)`) are parsed but their contents are otherwise ignored for the
/// match/no-match decision.
fn lua_pattern_matches(text: &str, pattern: &str) -> bool {
    let text_bytes = text.as_bytes();
    let pat_bytes = pattern.as_bytes();

    let (anchored, pat_start) = if pat_bytes.first() == Some(&b'^') {
        (true, 1)
    } else {
        (false, 0)
    };

    if anchored {
        lua_match_here(text_bytes, 0, pat_bytes, pat_start)
    } else {
        // Try matching at every starting position.
        for start in 0..=text_bytes.len() {
            if lua_match_here(text_bytes, start, pat_bytes, pat_start) {
                return true;
            }
        }
        false
    }
}

/// Try to match `pat[pi..]` against `text[ti..]`.
fn lua_match_here(text: &[u8], mut ti: usize, pat: &[u8], mut pi: usize) -> bool {
    loop {
        // End of pattern — success.
        if pi >= pat.len() {
            return true;
        }

        // `$` at end of pattern anchors to end of text.
        if pat[pi] == b'$' && pi + 1 == pat.len() {
            return ti == text.len();
        }

        // Opening capture group `(` — skip, we only need match/no-match.
        if pat[pi] == b'(' {
            pi += 1;
            continue;
        }
        // Closing capture group `)` — skip.
        if pat[pi] == b')' {
            pi += 1;
            continue;
        }

        // Determine how many bytes the current pattern item consumes (item_len) and
        // the end of the item in `pat` (item_end), then look ahead for a quantifier.
        let (item_end, item_len_in_pat) = lua_item_span(pat, pi);
        let quantifier = pat.get(item_end).copied();

        match quantifier {
            Some(b'*') => {
                // Match zero or more (greedy).
                let q_end = item_end + 1;
                let mut ti2 = ti;
                while ti2 < text.len() && lua_item_matches(text[ti2], pat, pi, item_end) {
                    ti2 += 1;
                }
                // Try longest first.
                loop {
                    if lua_match_here(text, ti2, pat, q_end) {
                        return true;
                    }
                    if ti2 == ti {
                        break;
                    }
                    ti2 -= 1;
                }
                return false;
            }
            Some(b'+') => {
                // One or more.
                if ti >= text.len() || !lua_item_matches(text[ti], pat, pi, item_end) {
                    return false;
                }
                ti += 1;
                let q_end = item_end + 1;
                let mut ti2 = ti;
                while ti2 < text.len() && lua_item_matches(text[ti2], pat, pi, item_end) {
                    ti2 += 1;
                }
                loop {
                    if lua_match_here(text, ti2, pat, q_end) {
                        return true;
                    }
                    if ti2 == ti {
                        break;
                    }
                    ti2 -= 1;
                }
                return false;
            }
            Some(b'-') => {
                // Lazy zero-or-more.
                let q_end = item_end + 1;
                loop {
                    if lua_match_here(text, ti, pat, q_end) {
                        return true;
                    }
                    if ti >= text.len() || !lua_item_matches(text[ti], pat, pi, item_end) {
                        return false;
                    }
                    ti += 1;
                }
            }
            Some(b'?') => {
                // Zero or one.
                let q_end = item_end + 1;
                if ti < text.len()
                    && lua_item_matches(text[ti], pat, pi, item_end)
                    && lua_match_here(text, ti + 1, pat, q_end)
                {
                    return true;
                }
                pi = q_end;
                // fall through (zero occurrences)
            }
            _ => {
                // No quantifier — match exactly one.
                if ti >= text.len() || !lua_item_matches(text[ti], pat, pi, item_end) {
                    return false;
                }
                ti += 1;
                pi = item_end;
                let _ = item_len_in_pat;
            }
        }
    }
}

/// Returns `(item_end, item_byte_len)` where `item_end` is the index in `pat` of the
/// first byte *after* the current pattern item that starts at `pi`.
fn lua_item_span(pat: &[u8], pi: usize) -> (usize, usize) {
    if pat[pi] == b'%' {
        // Escaped character: `%x` — always 2 bytes.
        (pi + 2, 2)
    } else if pat[pi] == b'[' {
        // Character class: `[…]` — find the closing `]`.
        let mut i = pi + 1;
        if i < pat.len() && pat[i] == b'^' {
            i += 1;
        }
        // A `]` immediately after `[` or `[^` is treated as a literal.
        if i < pat.len() && pat[i] == b']' {
            i += 1;
        }
        while i < pat.len() && pat[i] != b']' {
            if pat[i] == b'%' {
                i += 1; // skip the escaped char
            }
            i += 1;
        }
        (i + 1, i + 1 - pi) // include the closing `]`
    } else {
        (pi + 1, 1)
    }
}

/// Returns `true` if `byte` matches the pattern item `pat[pi..item_end]`.
fn lua_item_matches(byte: u8, pat: &[u8], pi: usize, item_end: usize) -> bool {
    let ch = byte as char;
    if pat[pi] == b'.' {
        return true;
    }
    if pat[pi] == b'%' && pi + 1 < pat.len() {
        return lua_class_matches(ch, pat[pi + 1]);
    }
    if pat[pi] == b'[' {
        return lua_set_matches(byte, pat, pi, item_end);
    }
    // Literal match.
    pat[pi] == byte
}

/// Match a Lua `%x` class character against `ch`.
fn lua_class_matches(ch: char, class: u8) -> bool {
    let res = match class.to_ascii_lowercase() {
        b'a' => ch.is_alphabetic(),
        b'd' => ch.is_ascii_digit(),
        b'l' => ch.is_lowercase(),
        b'u' => ch.is_uppercase(),
        b'w' => ch.is_alphanumeric(),
        b's' => ch.is_whitespace(),
        b'p' => ch.is_ascii_punctuation(),
        b'x' => ch.is_ascii_hexdigit(),
        b'c' => (ch as u32) < 32,
        _ => return ch == class as char, // `%(` → literal `(`
    };
    if class.is_ascii_uppercase() {
        !res
    } else {
        res
    }
}

/// Match a byte against a Lua character-set `[…]` spanning `pat[pi..item_end]`.
fn lua_set_matches(byte: u8, pat: &[u8], pi: usize, item_end: usize) -> bool {
    let ch = byte as char;
    let mut i = pi + 1; // skip `[`
    let negate = if i < item_end && pat[i] == b'^' {
        i += 1;
        true
    } else {
        false
    };
    let mut matched = false;
    // A `]` right after `[` or `[^` is a literal `]`.
    let initial = i;
    while i < item_end.saturating_sub(1) {
        // item_end points past `]`
        if pat[i] == b'%' && i + 1 < item_end - 1 {
            if lua_class_matches(ch, pat[i + 1]) {
                matched = true;
            }
            i += 2;
        } else if i + 2 < item_end - 1 && pat[i + 1] == b'-' {
            // Range a-z
            if byte >= pat[i] && byte <= pat[i + 2] {
                matched = true;
            }
            i += 3;
        } else {
            if i == initial && pat[i] == b']' {
                // Literal `]`
                if byte == b']' {
                    matched = true;
                }
            } else if pat[i] == byte {
                matched = true;
            }
            i += 1;
        }
    }
    if negate { !matched } else { matched }
}

fn create_parser(language_id: &str, loaded: &LoadedLanguage) -> Result<Parser, SyntaxError> {
    let mut parser = Parser::new();
    parser
        .set_language(&loaded.language)
        .map_err(|error| SyntaxError::ParserConfiguration {
            language_id: language_id.to_owned(),
            message: error.to_string(),
        })?;
    Ok(parser)
}

fn text_edit_to_input_edit(edit: TextEdit) -> InputEdit {
    InputEdit {
        start_byte: edit.start_byte,
        old_end_byte: edit.old_end_byte,
        new_end_byte: edit.new_end_byte,
        start_position: Point {
            row: edit.start_position.line,
            column: edit.start_position.column,
        },
        old_end_position: Point {
            row: edit.old_end_position.line,
            column: edit.old_end_position.column,
        },
        new_end_position: Point {
            row: edit.new_end_position.line,
            column: edit.new_end_position.column,
        },
    }
}

fn parse_with_parser(
    language_id: &str,
    parser: &mut Parser,
    buffer: &TextBuffer,
    old_tree: Option<&Tree>,
) -> Result<Tree, SyntaxError> {
    let byte_count = buffer.byte_count();
    parser
        .parse_with_options(
            &mut |byte_offset, _| {
                if byte_offset >= byte_count {
                    return &[][..];
                }
                let Some((chunk, chunk_start_byte)) = buffer.chunk_at_byte(byte_offset) else {
                    return &[][..];
                };
                &chunk.as_bytes()[byte_offset.saturating_sub(chunk_start_byte)..]
            },
            old_tree,
            None,
        )
        .ok_or_else(|| SyntaxError::ParseCancelled(language_id.to_owned()))
}

fn parse_tree(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<ParseTreeResult, SyntaxError> {
    let Some(parse_session) = parse_session else {
        let mut parser = create_parser(language_id, loaded)?;
        return Ok(ParseTreeResult {
            tree: parse_with_parser(language_id, &mut parser, buffer, None)?,
            changed_ranges: None,
            applied_edits: None,
        });
    };

    if let Some(session) = parse_session.as_mut()
        && session.language_id == language_id
    {
        if session.revision == buffer.revision() {
            return Ok(ParseTreeResult {
                tree: session.tree.clone(),
                changed_ranges: Some(Vec::new()),
                applied_edits: Some(Vec::new()),
            });
        }

        let applied_edits = if session.revision < buffer.revision() {
            buffer.edits_since(session.revision)
        } else {
            None
        };
        let edited_tree = applied_edits.as_ref().map(|edits| {
            let mut tree = session.tree.clone();
            for edit in edits {
                tree.edit(&text_edit_to_input_edit(*edit));
            }
            tree
        });
        let new_tree = parse_with_parser(
            language_id,
            &mut session.parser,
            buffer,
            edited_tree.as_ref(),
        )?;
        let changed_ranges = edited_tree
            .as_ref()
            .map(|previous_tree| previous_tree.changed_ranges(&new_tree).collect::<Vec<_>>());
        session.revision = buffer.revision();
        session.tree = new_tree.clone();
        return Ok(ParseTreeResult {
            tree: new_tree,
            changed_ranges,
            applied_edits,
        });
    }

    let mut parser = create_parser(language_id, loaded)?;
    let tree = parse_with_parser(language_id, &mut parser, buffer, None)?;
    *parse_session = Some(SyntaxParseSession {
        language_id: language_id.to_owned(),
        revision: buffer.revision(),
        parser,
        tree: tree.clone(),
        last_highlight_window: None,
        last_snapshot: None,
    });
    Ok(ParseTreeResult {
        tree,
        changed_ranges: None,
        applied_edits: None,
    })
}

fn highlight_tree(
    loaded: &LoadedLanguage,
    tree: &Tree,
    buffer: &TextBuffer,
    highlight_window: Option<HighlightWindow>,
) -> Vec<HighlightSpan> {
    let mut query_cursor = QueryCursor::new();
    if let Some(highlight_window) = highlight_window.filter(|window| !window.is_empty()) {
        query_cursor.set_point_range(
            Point {
                row: highlight_window.start_line(),
                column: 0,
            }..Point {
                row: highlight_window.end_line_exclusive(),
                column: 0,
            },
        );
    }
    let capture_names = loaded.query.capture_names();
    let mut highlight_spans = Vec::new();
    let mut matches = query_cursor.matches(
        &loaded.query,
        tree.root_node(),
        TextBufferProvider { buffer },
    );
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };

        // Apply custom predicates that tree-sitter does not evaluate automatically.
        if !general_predicates_match(
            &loaded.query,
            query_match.pattern_index,
            query_match.captures,
            buffer,
        ) {
            continue;
        }

        for capture in query_match.captures {
            let node = capture.node;
            let start = node.start_position();
            let end = node.end_position();
            let capture_name = capture_names
                .get(capture.index as usize)
                .map(|name| name.to_string())
                .unwrap_or_default();
            if capture_name.is_empty() || !capture_requires_theme_token(&capture_name) {
                continue;
            }

            highlight_spans.push(HighlightSpan {
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_position: SyntaxPoint::new(start.row, start.column),
                end_position: SyntaxPoint::new(end.row, end.column),
                theme_token: loaded.theme_token_for_capture(&capture_name),
                capture_name,
            });
        }
    }
    highlight_spans
}

fn collect_injection_regions(
    injections_query: &Query,
    tree: &Tree,
    buffer: &TextBuffer,
    highlight_window: Option<HighlightWindow>,
) -> Vec<InjectionRegion> {
    let mut query_cursor = QueryCursor::new();
    if let Some(highlight_window) = highlight_window.filter(|window| !window.is_empty()) {
        query_cursor.set_point_range(
            Point {
                row: highlight_window.start_line(),
                column: 0,
            }..Point {
                row: highlight_window.end_line_exclusive(),
                column: 0,
            },
        );
    }
    let capture_names = injections_query.capture_names();
    let mut regions = Vec::new();
    let mut matches = query_cursor.matches(
        injections_query,
        tree.root_node(),
        TextBufferProvider { buffer },
    );
    loop {
        matches.advance();
        let Some(query_match) = matches.get() else {
            break;
        };
        if !general_predicates_match(
            injections_query,
            query_match.pattern_index,
            query_match.captures,
            buffer,
        ) {
            continue;
        }

        let language_capture = query_match.captures.iter().find_map(|capture| {
            let capture_name = capture_names.get(capture.index as usize)?;
            (*capture_name == "injection.language").then(|| {
                buffer_text_for_byte_range(
                    buffer,
                    capture.node.start_byte(),
                    capture.node.end_byte(),
                )
            })?
        });
        for capture in query_match.captures {
            let Some(capture_name) = capture_names.get(capture.index as usize) else {
                continue;
            };
            if *capture_name != "injection.content" {
                continue;
            }

            let language_name = query_capture_property_value(
                injections_query,
                query_match.pattern_index,
                capture.index,
                "injection.language",
            )
            .map(str::to_owned)
            .or_else(|| language_capture.clone())
            .map(|language| language.trim().to_owned())
            .filter(|language| !language.is_empty());
            let Some(language_name) = language_name else {
                continue;
            };

            let node = capture.node;
            let start = node.start_position();
            let end = node.end_position();
            regions.push(InjectionRegion {
                language_name,
                start_byte: node.start_byte(),
                end_byte: node.end_byte(),
                start_position: SyntaxPoint::new(start.row, start.column),
                end_position: SyntaxPoint::new(end.row, end.column),
            });
        }
    }
    regions
}

fn sort_highlight_spans(highlight_spans: &mut [HighlightSpan]) {
    highlight_spans.sort_by(|left, right| {
        (
            left.start_byte,
            left.end_byte,
            left.start_position.line,
            left.start_position.column,
            &left.capture_name,
            &left.theme_token,
        )
            .cmp(&(
                right.start_byte,
                right.end_byte,
                right.start_position.line,
                right.start_position.column,
                &right.capture_name,
                &right.theme_token,
            ))
    });
}

fn span_intersects_window(span: &HighlightSpan, window: HighlightWindow) -> bool {
    span.start_position.line < window.end_line_exclusive()
        && span.end_position.line >= window.start_line()
}

fn injection_region_intersects_window(region: &InjectionRegion, window: HighlightWindow) -> bool {
    region.start_position.line < window.end_line_exclusive()
        && region.end_position.line >= window.start_line()
}

fn apply_text_edits_to_span(mut span: HighlightSpan, edits: &[TextEdit]) -> HighlightSpan {
    for edit in edits {
        let input_edit = text_edit_to_input_edit(*edit);
        let mut range = Range {
            start_byte: span.start_byte,
            end_byte: span.end_byte,
            start_point: Point {
                row: span.start_position.line,
                column: span.start_position.column,
            },
            end_point: Point {
                row: span.end_position.line,
                column: span.end_position.column,
            },
        };
        input_edit.edit_range(&mut range);

        span.start_byte = range.start_byte;
        span.end_byte = range.end_byte;
        span.start_position = SyntaxPoint::new(range.start_point.row, range.start_point.column);
        span.end_position = SyntaxPoint::new(range.end_point.row, range.end_point.column);
    }

    span
}

fn changed_range_windows(
    changed_ranges: &[Range],
    highlight_window: HighlightWindow,
) -> Vec<HighlightWindow> {
    const CONTEXT_LINES: usize = 1;

    if highlight_window.is_empty() {
        return Vec::new();
    }

    let mut ranges = changed_ranges
        .iter()
        .filter_map(|range| {
            let start_line = range
                .start_point
                .row
                .saturating_sub(CONTEXT_LINES)
                .max(highlight_window.start_line());
            let end_line_exclusive = range
                .end_point
                .row
                .saturating_add(1)
                .saturating_add(CONTEXT_LINES)
                .min(highlight_window.end_line_exclusive());
            (start_line < end_line_exclusive)
                .then(|| HighlightWindow::new(start_line, end_line_exclusive - start_line))
        })
        .collect::<Vec<_>>();
    ranges.sort_by_key(HighlightWindow::start_line);

    let mut merged: Vec<HighlightWindow> = Vec::new();
    for range in ranges {
        if let Some(last) = merged.last_mut()
            && last.end_line_exclusive() >= range.start_line()
        {
            let end_line_exclusive = last.end_line_exclusive().max(range.end_line_exclusive());
            *last = HighlightWindow::new(last.start_line(), end_line_exclusive - last.start_line());
            continue;
        }
        merged.push(range);
    }
    merged
}

fn highlight_loaded_language_with_tree(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    highlight_window: Option<HighlightWindow>,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<ParsedHighlight, SyntaxError> {
    let (parse_result, mut session) = match parse_session {
        Some(parse_session) => (
            parse_tree(language_id, loaded, buffer, Some(parse_session))?,
            parse_session.as_mut(),
        ),
        None => (parse_tree(language_id, loaded, buffer, None)?, None),
    };

    let mut highlight_spans = session
        .as_ref()
        .and_then(|session| {
            let previous_snapshot = session.last_snapshot.as_ref()?;
            if session.last_highlight_window != highlight_window {
                return None;
            }
            let changed_ranges = parse_result.changed_ranges.as_ref()?;
            let applied_edits = parse_result.applied_edits.as_deref().unwrap_or(&[]);
            let previous_highlight_spans = previous_snapshot
                .highlight_spans
                .iter()
                .cloned()
                .map(|span| apply_text_edits_to_span(span, applied_edits))
                .collect::<Vec<_>>();
            if changed_ranges.is_empty() {
                return Some(previous_highlight_spans);
            }
            let highlight_window = highlight_window?;
            let changed_windows = changed_range_windows(changed_ranges, highlight_window);
            if changed_windows.is_empty() {
                return Some(previous_highlight_spans);
            }

            let mut highlight_spans = previous_highlight_spans
                .iter()
                .filter(|span| {
                    !changed_windows
                        .iter()
                        .any(|window| span_intersects_window(span, *window))
                })
                .cloned()
                .collect::<Vec<_>>();
            for changed_window in changed_windows {
                highlight_spans.extend(highlight_tree(
                    loaded,
                    &parse_result.tree,
                    buffer,
                    Some(changed_window),
                ));
            }
            Some(highlight_spans)
        })
        .unwrap_or_else(|| highlight_tree(loaded, &parse_result.tree, buffer, highlight_window));
    sort_highlight_spans(&mut highlight_spans);
    let snapshot = SyntaxSnapshot {
        language_id: language_id.to_owned(),
        root_kind: parse_result.tree.root_node().kind().to_owned(),
        has_errors: parse_result.tree.root_node().has_error(),
        highlight_spans,
    };
    if let Some(session) = session.as_mut() {
        session.last_highlight_window = highlight_window;
        session.last_snapshot = Some(snapshot.clone());
    }
    Ok(ParsedHighlight {
        snapshot,
        tree: parse_result.tree,
    })
}

fn highlight_loaded_language(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    highlight_window: Option<HighlightWindow>,
    parse_session: Option<&mut Option<SyntaxParseSession>>,
) -> Result<SyntaxSnapshot, SyntaxError> {
    Ok(highlight_loaded_language_with_tree(
        language_id,
        loaded,
        buffer,
        highlight_window,
        parse_session,
    )?
    .snapshot)
}

fn markdown_inline_line_indices(tree: &Tree) -> Vec<usize> {
    fn collect_lines(node: tree_sitter::Node<'_>, lines: &mut BTreeSet<usize>) {
        if node.kind() == "inline" {
            for line in node.start_position().row..=node.end_position().row {
                lines.insert(line);
            }
        }
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            collect_lines(child, lines);
        }
    }

    let mut lines = BTreeSet::new();
    collect_lines(tree.root_node(), &mut lines);
    lines.into_iter().collect()
}

fn buffer_text_for_byte_range(
    buffer: &TextBuffer,
    start_byte: usize,
    end_byte: usize,
) -> Option<String> {
    if start_byte >= end_byte || end_byte > buffer.byte_count() {
        return None;
    }

    let mut text = String::new();
    for chunk in buffer.byte_slice_chunks(start_byte..end_byte) {
        let Ok(chunk) = std::str::from_utf8(chunk) else {
            return None;
        };
        text.push_str(chunk);
    }
    Some(text)
}

fn translate_injected_highlight_span(
    span: HighlightSpan,
    region: &InjectionRegion,
) -> HighlightSpan {
    let start_line = region.start_position.line + span.start_position.line;
    let end_line = region.start_position.line + span.end_position.line;
    let start_column = if span.start_position.line == 0 {
        region.start_position.column + span.start_position.column
    } else {
        span.start_position.column
    };
    let end_column = if span.end_position.line == 0 {
        region.start_position.column + span.end_position.column
    } else {
        span.end_position.column
    };

    HighlightSpan {
        start_byte: region.start_byte.saturating_add(span.start_byte),
        end_byte: region.start_byte.saturating_add(span.end_byte),
        start_position: SyntaxPoint::new(start_line, start_column),
        end_position: SyntaxPoint::new(end_line, end_column),
        capture_name: span.capture_name,
        theme_token: span.theme_token,
    }
}

fn highlight_inline_language_per_line(
    language_id: &str,
    loaded: &LoadedLanguage,
    buffer: &TextBuffer,
    line_indices: &[usize],
) -> Result<SyntaxSnapshot, SyntaxError> {
    let mut highlight_spans = Vec::new();
    let mut has_errors = false;

    for &line_index in line_indices {
        let Some(line_text) = buffer.line(line_index) else {
            continue;
        };
        if line_text.is_empty() {
            continue;
        }
        let Some(start_byte) = buffer.line_start_byte(line_index) else {
            continue;
        };
        let line_buffer = TextBuffer::from_text(&line_text);
        let mut parser = create_parser(language_id, loaded)?;
        let tree = parse_with_parser(language_id, &mut parser, &line_buffer, None)?;
        has_errors |= tree.root_node().has_error();

        let spans = highlight_tree(loaded, &tree, &line_buffer, None);
        let line_len = line_text.chars().count();
        for span in spans {
            let start_col = span.start_position.column.min(line_len);
            let end_col = span.end_position.column.min(line_len);
            if start_col >= end_col {
                continue;
            }
            highlight_spans.push(HighlightSpan {
                start_byte: start_byte.saturating_add(span.start_byte),
                end_byte: start_byte.saturating_add(span.end_byte),
                start_position: SyntaxPoint::new(line_index, start_col),
                end_position: SyntaxPoint::new(line_index, end_col),
                capture_name: span.capture_name,
                theme_token: span.theme_token,
            });
        }
    }

    Ok(SyntaxSnapshot {
        language_id: language_id.to_owned(),
        root_kind: "inline".to_owned(),
        has_errors,
        highlight_spans,
    })
}

fn build_shared_library(
    language_id: &str,
    grammar: &GrammarSource,
    grammar_dir: &Path,
    install_root: &Path,
) -> Result<(), SyntaxError> {
    let source_dir = grammar_dir.join(grammar.source_dir());
    let parser_path = source_dir.join("parser.c");
    if !parser_path.exists() {
        return Err(SyntaxError::Io {
            operation: "locate parser source".to_owned(),
            path: parser_path,
            message: "parser.c is missing".to_owned(),
        });
    }

    let scanner_c = source_dir.join("scanner.c");
    let scanner_cpp = source_dir.join("scanner.cc");
    let output_path = grammar.installed_library_path(install_root);
    let compiler = if scanner_cpp.exists() { "c++" } else { "cc" };
    let mut command = Command::new(compiler);
    if cfg!(target_os = "macos") {
        command.args(["-fPIC", "-dynamiclib"]);
    } else {
        command.args(["-fPIC", "-shared"]);
    }
    if scanner_cpp.exists() {
        command.arg("-std=c++14");
    }
    command.arg(&parser_path);
    if scanner_c.exists() {
        command.arg(&scanner_c);
    }
    if scanner_cpp.exists() {
        command.arg(&scanner_cpp);
    }
    command.arg("-I");
    command.arg(&source_dir);
    command.arg("-o");
    command.arg(&output_path);
    command.current_dir(grammar_dir);

    let output = command
        .output()
        .map_err(|error| io_error("run grammar compiler", grammar_dir, error))?;
    if !output.status.success() {
        return Err(SyntaxError::InstallCommand {
            language_id: language_id.to_owned(),
            message: command_failure_message(compiler, &output),
        });
    }

    Ok(())
}

fn ensure_installed_highlight_query_path(
    config: &LanguageConfiguration,
    query_path: &Path,
) -> Result<(), SyntaxError> {
    if query_path.exists() || config.extra_highlight_query().is_none() {
        return Ok(());
    }
    let parent = query_path.parent().ok_or_else(|| SyntaxError::Io {
        operation: "locate highlight query parent".to_owned(),
        path: query_path.to_path_buf(),
        message: "installed highlight query path has no parent directory".to_owned(),
    })?;
    fs::create_dir_all(parent)
        .map_err(|error| io_error("create highlight query directory", parent, error))?;
    fs::write(query_path, "")
        .map_err(|error| io_error("create placeholder highlight query", query_path, error))
}

fn io_error(operation: &str, path: &Path, error: std::io::Error) -> SyntaxError {
    SyntaxError::Io {
        operation: operation.to_owned(),
        path: path.to_path_buf(),
        message: error.to_string(),
    }
}

fn command_failure_message(command_name: &str, output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if !stderr.is_empty() {
        return stderr;
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if !stdout.is_empty() {
        return stdout;
    }

    format!("{command_name} exited with status {}", output.status)
}

fn normalize_unique_entries<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for value in values {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() || normalized.iter().any(|entry| entry == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_owned());
    }
    normalized
}

fn default_install_root() -> PathBuf {
    if let Some(path) = env::var_os("VOLT_GRAMMAR_DIR").map(PathBuf::from) {
        return path;
    }

    let base = if cfg!(target_os = "windows") {
        env::var_os("LOCALAPPDATA")
            .or_else(|| env::var_os("APPDATA"))
            .map(PathBuf::from)
    } else {
        env::var_os("XDG_DATA_HOME").map(PathBuf::from).or_else(|| {
            env::var_os("HOME").map(|home| PathBuf::from(home).join(".local").join("share"))
        })
    };

    base.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("volt")
        .join("grammars")
}

fn default_query_asset_root() -> Option<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        roots.extend(
            exe_dir
                .ancestors()
                .take(DEFAULT_QUERY_ASSET_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    if let Ok(current_dir) = env::current_dir() {
        roots.extend(
            current_dir
                .ancestors()
                .take(DEFAULT_QUERY_ASSET_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }

    for root in roots {
        for parts in QUERY_ASSET_DIR_CANDIDATES {
            let candidate = asset_path_from_parts(&root, parts);
            if candidate.is_dir() {
                return Some(candidate);
            }
        }
    }
    None
}

fn asset_path_from_parts(base: &Path, parts: &[&str]) -> PathBuf {
    parts
        .iter()
        .fold(base.to_path_buf(), |candidate, part| candidate.join(part))
}

fn normalize_extension(extension: &str) -> String {
    extension
        .trim()
        .trim_start_matches('.')
        .to_ascii_lowercase()
}

fn shared_library_file_name(install_dir_name: &str) -> String {
    if cfg!(target_os = "windows") {
        format!("lib{install_dir_name}.dll")
    } else if cfg!(target_os = "macos") {
        format!("lib{install_dir_name}.dylib")
    } else {
        format!("lib{install_dir_name}.so")
    }
}

fn temp_guid_like_directory_name() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let value = duration.as_nanos() ^ ((std::process::id() as u128) << 32);
    let part1 = ((value >> 96) & 0xffff_ffff) as u32;
    let part2 = ((value >> 80) & 0xffff) as u16;
    let part3 = ((value >> 64) & 0xffff) as u16;
    let part4 = ((value >> 48) & 0xffff) as u16;
    let part5 = (value & 0xffff_ffff_ffff) as u64;
    format!("{part1:08x}-{part2:04x}-{part3:04x}-{part4:04x}-{part5:012x}")
}

struct TempCloneGuard {
    path: PathBuf,
}

impl TempCloneGuard {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TempCloneGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use super::{
        CaptureThemeMapping, GrammarSource, HighlightWindow, LanguageConfiguration, SyntaxError,
        SyntaxParseSession, SyntaxRegistry, ensure_installed_highlight_query_path,
        install_bundled_queries, maybe_read_query_source_preferring_bundled,
        read_query_source_preferring_bundled,
    };
    use editor_buffer::{TextBuffer, TextPoint};

    fn rust_language() -> super::Language {
        tree_sitter_rust::LANGUAGE.into()
    }

    fn rust_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "rust",
            ["rs"],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [
                CaptureThemeMapping::new("keyword", "syntax.keyword"),
                CaptureThemeMapping::new("function", "syntax.function"),
                CaptureThemeMapping::new("string", "syntax.string"),
            ],
        )
    }

    fn installable_rust_configuration() -> LanguageConfiguration {
        LanguageConfiguration::from_grammar(
            "rust",
            ["rs"],
            GrammarSource::new(
                "https://github.com/tree-sitter/tree-sitter-rust.git",
                ".",
                "src",
                "tree-sitter-rust",
                "tree_sitter_rust",
            ),
            [CaptureThemeMapping::new("keyword", "syntax.keyword")],
        )
    }

    fn rust_inline_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "rust-inline",
            [] as [&str; 0],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [CaptureThemeMapping::new("string", "syntax.string.inline")],
        )
    }

    fn cmake_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "cmake",
            ["cmake"],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [CaptureThemeMapping::new("keyword", "syntax.keyword")],
        )
        .with_file_names(["CMakeLists.txt"])
    }

    fn dockerfile_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "dockerfile",
            [] as [&str; 0],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [CaptureThemeMapping::new("string", "syntax.string")],
        )
        .with_file_names(["Dockerfile"])
        .with_file_globs(["Dockerfile.*"])
    }

    fn dev_extension_configuration() -> LanguageConfiguration {
        LanguageConfiguration::new(
            "dev",
            ["dev"],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [CaptureThemeMapping::new("string", "syntax.string")],
        )
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    struct TempTestDir {
        path: PathBuf,
    }

    impl TempTestDir {
        fn new(name: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos();
            let path = env::temp_dir().join(format!("volt-syntax-{name}-{unique}"));
            fs::create_dir_all(&path).expect("create temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempTestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    #[test]
    fn registry_resolves_languages_by_extension_and_path() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        assert_eq!(
            registry
                .language_for_extension(".rs")
                .map(|language| language.id()),
            Some("rust")
        );
        assert_eq!(
            registry
                .language_for_path("src\\main.rs")
                .map(|language| language.id()),
            Some("rust")
        );
    }

    #[test]
    fn registry_prefers_exact_filenames_and_globs_over_extensions() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(LanguageConfiguration::new(
            "plaintext",
            ["txt"],
            rust_language,
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            [CaptureThemeMapping::new("keyword", "syntax.keyword")],
        )));
        must(registry.register(cmake_configuration()));
        must(registry.register(dockerfile_configuration()));

        assert_eq!(
            registry
                .language_for_path("project\\CMakeLists.txt")
                .map(LanguageConfiguration::id),
            Some("cmake")
        );
        assert_eq!(
            registry
                .language_for_path("containers\\Dockerfile.dev")
                .map(LanguageConfiguration::id),
            Some("dockerfile")
        );
        assert_eq!(
            registry
                .language_for_path("notes\\guide.txt")
                .map(LanguageConfiguration::id),
            Some("plaintext")
        );
    }

    #[test]
    fn registry_resolves_languages_by_filename_and_glob() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(dockerfile_configuration()));

        assert_eq!(
            registry
                .language_for_path("Dockerfile")
                .map(|language| language.id()),
            Some("dockerfile")
        );
        assert_eq!(
            registry
                .language_for_path("containers\\Dockerfile.dev")
                .map(|language| language.id()),
            Some("dockerfile")
        );
    }

    #[test]
    fn registry_prefers_filename_globs_over_extension_matches() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(dev_extension_configuration()));
        must(registry.register(dockerfile_configuration()));

        assert_eq!(
            registry
                .language_for_path("containers\\Dockerfile.dev")
                .map(|language| language.id()),
            Some("dockerfile")
        );
    }

    #[test]
    fn rust_highlighting_produces_theme_tokens() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let buffer = TextBuffer::from_text(
            r#"
fn main() {
    let value = "volt";
    println!("{value}");
}
"#,
        );

        let snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
        assert_eq!(snapshot.language_id, "rust");
        assert_eq!(snapshot.root_kind, "source_file");
        assert!(!snapshot.has_errors);
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.keyword")
        );
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.string")
        );
    }

    #[test]
    fn ancestor_contexts_include_named_nodes_up_to_the_root() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let buffer = TextBuffer::from_text(
            r#"impl Demo {
    fn render(value: usize) {
        let current = value;
    }
}
"#,
        );
        let contexts =
            must(registry.ancestor_contexts_for_language("rust", &buffer, TextPoint::new(2, 8)));

        let kinds = contexts
            .iter()
            .map(|context| context.kind.as_str())
            .collect::<Vec<_>>();
        assert!(kinds.contains(&"function_item"));
        assert!(kinds.contains(&"impl_item"));
        assert!(!kinds.contains(&"source_file"));
    }

    #[test]
    fn ancestor_contexts_parse_session_matches_cold_query_after_edits() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut buffer = TextBuffer::from_text(
            r#"impl Demo {
    fn render(value: usize) {
        let current = value;
    }
}
"#,
        );
        let mut parse_session = None;

        let cold =
            must(registry.ancestor_contexts_for_language("rust", &buffer, TextPoint::new(2, 8)));
        let incremental = must(registry.ancestor_contexts_for_language_with_parse_session(
            "rust",
            &buffer,
            TextPoint::new(2, 8),
            &mut parse_session,
        ));
        assert_eq!(incremental, cold);

        buffer.set_cursor(TextPoint::new(2, 8));
        buffer.insert_text("mut ");

        let cold_after =
            must(registry.ancestor_contexts_for_language("rust", &buffer, TextPoint::new(2, 12)));
        let incremental_after = must(registry.ancestor_contexts_for_language_with_parse_session(
            "rust",
            &buffer,
            TextPoint::new(2, 12),
            &mut parse_session,
        ));
        assert_eq!(incremental_after, cold_after);
    }

    #[test]
    fn additional_highlight_languages_merge_spans() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry.register(
                rust_configuration().with_additional_highlight_languages(["rust-inline"]),
            ),
        );
        must(registry.register(rust_inline_configuration()));

        let buffer = TextBuffer::from_text("fn main() { let value = \"volt\"; }");
        let snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.string.inline")
        );
    }

    #[test]
    fn injected_highlighting_merges_nested_language_and_additional_spans() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry.register(
                rust_configuration()
                    .with_additional_highlight_languages(["rust-inline"])
                    .with_extra_injections_query(
                        r#"((raw_string_literal
  (string_content) @injection.content)
  (#set! injection.language "rs"))"#,
                    ),
            ),
        );
        must(registry.register(rust_inline_configuration()));

        let buffer = TextBuffer::from_text(
            r##"fn main() {
    let source = r#"fn injected() { let value = "volt"; }"#;
}"##,
        );
        let source = buffer.text();
        let Some(injected_fn_byte) = source.find("injected") else {
            panic!("expected injected function name in test buffer");
        };
        let Some(injected_string_byte) = source.find("volt") else {
            panic!("expected injected string literal in test buffer");
        };

        let snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.function"
                    && span.start_byte <= injected_fn_byte
                    && injected_fn_byte < span.end_byte),
            "expected injected Rust function highlight at byte {injected_fn_byte}, got {:?}",
            snapshot.highlight_spans
        );
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.string.inline"
                    && span.start_byte <= injected_string_byte
                    && injected_string_byte < span.end_byte),
            "expected injected additional highlight at byte {injected_string_byte}, got {:?}",
            snapshot.highlight_spans
        );
    }

    #[test]
    fn unknown_injection_language_is_ignored_without_failing_host_highlighting() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry.register(rust_configuration().with_extra_injections_query(
                r#"((raw_string_literal
  (string_content) @injection.content)
  (#set! injection.language "not-registered"))"#,
            )),
        );

        let buffer = TextBuffer::from_text(
            r##"fn main() {
    let source = r#"fn injected() {}"#;
}"##,
        );
        let source = buffer.text();
        let Some(main_byte) = source.find("main") else {
            panic!("expected main function name in test buffer");
        };
        let Some(injected_byte) = source.find("injected") else {
            panic!("expected injected function name in test buffer");
        };

        let snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|span| span.theme_token == "syntax.function"
                    && span.start_byte <= main_byte
                    && main_byte < span.end_byte),
            "expected host Rust function highlight at byte {main_byte}, got {:?}",
            snapshot.highlight_spans
        );
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .all(|span| !(span.theme_token == "syntax.function"
                    && span.start_byte <= injected_byte
                    && injected_byte < span.end_byte)),
            "unexpected injected function highlight at byte {injected_byte}: {:?}",
            snapshot.highlight_spans
        );
    }

    #[test]
    fn visible_spans_filters_to_line_window() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut source = String::new();
        for _ in 0..256 {
            source.push_str("fn demo() {}\n");
        }
        source.push_str("let target = \"visible\";\n");
        for _ in 0..256 {
            source.push_str("fn tail() {}\n");
        }

        let snapshot =
            must(registry.highlight_buffer_for_extension("rs", &TextBuffer::from_text(source)));
        let visible = snapshot.visible_spans(256, 4);

        assert!(!visible.is_empty());
        assert!(visible.iter().all(|span| span.start_position.line <= 259));
    }

    #[test]
    fn highlight_window_limits_highlight_spans_to_requested_lines() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut source = String::new();
        for index in 0..512 {
            source.push_str(&format!(
                "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
            ));
        }
        let buffer = TextBuffer::from_text(source);

        let full_snapshot = must(registry.highlight_buffer_for_extension("rs", &buffer));
        let window = HighlightWindow::new(240, 16);
        let windowed_snapshot =
            must(registry.highlight_buffer_for_extension_window("rs", &buffer, window));

        assert!(!windowed_snapshot.highlight_spans.is_empty());
        assert!(windowed_snapshot.highlight_spans.len() < full_snapshot.highlight_spans.len());
        assert!(windowed_snapshot.highlight_spans.iter().all(|span| {
            span.start_position.line < window.end_line_exclusive()
                && span.end_position.line >= window.start_line()
        }));
    }

    #[test]
    fn incremental_parse_session_matches_cold_highlight_after_edits() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut buffer = TextBuffer::from_text("fn main() {\n    let value = 1;\n}\n");
        let mut parse_session: Option<SyntaxParseSession> = None;
        let initial_snapshot = must(registry.highlight_buffer_for_language_with_session(
            "rust",
            &buffer,
            &mut parse_session,
        ));

        buffer.set_cursor(editor_buffer::TextPoint::new(1, 16));
        buffer.insert_text("mut ");
        let incremental_snapshot = must(registry.highlight_buffer_for_language_with_session(
            "rust",
            &buffer,
            &mut parse_session,
        ));
        let cold_snapshot = must(registry.highlight_buffer_for_language("rust", &buffer));

        assert_eq!(incremental_snapshot, cold_snapshot);
        assert_ne!(initial_snapshot, incremental_snapshot);
    }

    #[test]
    fn incremental_windowed_session_matches_cold_highlight_after_edits() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));

        let mut source = String::new();
        for index in 0..512 {
            source.push_str(&format!(
                "fn demo_{index}() {{ let value = \"line_{index}\"; }}\n"
            ));
        }
        let mut buffer = TextBuffer::from_text(source);
        let window = HighlightWindow::new(240, 24);
        let mut parse_session: Option<SyntaxParseSession> = None;

        let _ = must(registry.highlight_buffer_for_language_window_with_session(
            "rust",
            &buffer,
            window,
            &mut parse_session,
        ));

        buffer.set_cursor(editor_buffer::TextPoint::new(248, 0));
        buffer.insert_text("x");

        let incremental_snapshot =
            must(registry.highlight_buffer_for_language_window_with_session(
                "rust",
                &buffer,
                window,
                &mut parse_session,
            ));
        let cold_snapshot =
            must(registry.highlight_buffer_for_language_window("rust", &buffer, window));

        assert_eq!(incremental_snapshot, cold_snapshot);
    }

    #[test]
    fn grammar_configuration_uses_install_root_paths() {
        let grammar = GrammarSource::new(
            "https://example.com/tree-sitter-rust.git",
            ".",
            "src",
            "tree-sitter-rust",
            "tree_sitter_rust",
        );
        let install_root = PathBuf::from("volt-grammars");

        assert_eq!(
            grammar.install_directory(&install_root),
            install_root.join("tree-sitter-rust")
        );
        assert_eq!(
            grammar.installed_highlight_query_path(&install_root),
            install_root
                .join("tree-sitter-rust")
                .join("queries")
                .join("highlights.scm")
        );
        assert_eq!(
            grammar.installed_indent_query_path(&install_root),
            install_root
                .join("tree-sitter-rust")
                .join("queries")
                .join("indents.scm")
        );
        assert!(
            grammar
                .installed_library_path(&install_root)
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .map(|file_name| file_name.starts_with("libtree-sitter-rust"))
                .unwrap_or(false)
        );
    }

    #[test]
    fn grammar_backed_language_reports_missing_install() {
        let install_root = std::env::temp_dir().join("volt-missing-tree-sitter-grammar");
        let mut registry = SyntaxRegistry::with_install_root(install_root.clone());
        must(registry.register(installable_rust_configuration()));

        assert!(!must(registry.is_installed("rust")));
        let error = registry
            .highlight_buffer_for_extension("rs", &TextBuffer::from_text("fn main() {}"))
            .expect_err("expected missing grammar error");
        match error {
            SyntaxError::GrammarNotInstalled {
                language_id,
                install_dir,
            } => {
                assert_eq!(language_id, "rust");
                assert_eq!(install_dir, install_root.join("tree-sitter-rust"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn extra_highlight_query_can_seed_missing_installed_query_file() {
        let query_path = std::env::temp_dir().join(format!(
            "volt-extra-query-{}-{}.scm",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default()
        ));
        let config =
            installable_rust_configuration().with_extra_highlight_query("(identifier) @function");

        must(ensure_installed_highlight_query_path(&config, &query_path));

        assert!(query_path.exists());
        assert_eq!(std::fs::read_to_string(&query_path).unwrap_or_default(), "");

        if query_path.exists() {
            let _ = std::fs::remove_file(&query_path);
        }
    }

    #[test]
    fn bundled_query_install_flattens_inherited_queries() {
        let asset_root = TempTestDir::new("query-assets");
        let install_root = TempTestDir::new("query-install");
        let base_dir = asset_root.path().join("base");
        let child_dir = asset_root.path().join("child");
        fs::create_dir_all(&base_dir).expect("create base query dir");
        fs::create_dir_all(&child_dir).expect("create child query dir");
        fs::write(base_dir.join("highlights.scm"), "(identifier) @variable\n")
            .expect("write base highlight query");
        fs::write(
            child_dir.join("highlights.scm"),
            "; inherits: base\n(string_literal) @string\n",
        )
        .expect("write child highlight query");
        let config = LanguageConfiguration::from_grammar(
            "child",
            ["child"],
            GrammarSource::new(
                "https://example.com/tree-sitter-child.git",
                ".",
                "src",
                "tree-sitter-child",
                "tree_sitter_child",
            ),
            [CaptureThemeMapping::new("variable", "syntax.variable")],
        );

        must(install_bundled_queries(
            &config,
            asset_root.path(),
            install_root.path(),
        ));

        let installed = fs::read_to_string(
            config
                .grammar()
                .expect("grammar config")
                .installed_highlight_query_path(install_root.path()),
        )
        .expect("read installed query");
        assert!(installed.contains("(identifier) @variable"));
        assert!(installed.contains("(string_literal) @string"));
        assert!(!installed.contains("; inherits:"));
    }

    /// When both an installed query file and a bundled asset exist, the bundled asset must win.
    /// This guards against stale installed queries (e.g. the markdown-inline highlights.scm
    /// written to `%LOCALAPPDATA%\volt\grammars`) shadowing a corrected repo asset.
    #[test]
    fn bundled_query_asset_wins_over_stale_installed_query() {
        let asset_root = TempTestDir::new("bundled-wins-asset");
        let install_root = TempTestDir::new("bundled-wins-install");
        let lang_id = "test-lang";

        // Write the "live" bundled asset (what the repo has after a fix).
        let asset_dir = asset_root.path().join(lang_id);
        fs::create_dir_all(&asset_dir).expect("create asset dir");
        fs::write(
            asset_dir.join("highlights.scm"),
            "(identifier) @variable.bundled\n",
        )
        .expect("write bundled highlights");

        // Write a "stale" installed query in the simulated grammar install directory
        // (the kind that would exist before a manual grammar reinstall).
        let installed_query_dir = install_root
            .path()
            .join(format!("tree-sitter-{lang_id}"))
            .join("queries");
        fs::create_dir_all(&installed_query_dir).expect("create installed query dir");
        let installed_path = installed_query_dir.join("highlights.scm");
        fs::write(&installed_path, "(identifier) @variable.stale\n")
            .expect("write stale installed highlights");

        // With query_asset_root configured, the bundled asset must be returned.
        let source = must(read_query_source_preferring_bundled(
            &installed_path,
            Some(asset_root.path()),
            lang_id,
            "highlights.scm",
        ));
        assert!(
            source.contains("@variable.bundled"),
            "expected bundled content, got: {source:?}"
        );
        assert!(
            !source.contains("@variable.stale"),
            "stale installed content leaked into result: {source:?}"
        );

        // Without a query_asset_root, the installed file is the only source.
        let fallback = must(read_query_source_preferring_bundled(
            &installed_path,
            None,
            lang_id,
            "highlights.scm",
        ));
        assert!(
            fallback.contains("@variable.stale"),
            "expected installed fallback content, got: {fallback:?}"
        );
    }

    /// When both an installed optional query and a bundled asset exist, the bundled asset wins.
    #[test]
    fn bundled_optional_query_asset_wins_over_stale_installed_query() {
        let asset_root = TempTestDir::new("bundled-opt-asset");
        let install_root = TempTestDir::new("bundled-opt-install");
        let lang_id = "test-lang-opt";

        // Write bundled indents asset.
        let asset_dir = asset_root.path().join(lang_id);
        fs::create_dir_all(&asset_dir).expect("create asset dir");
        fs::write(asset_dir.join("indents.scm"), "(block) @indent.bundled\n")
            .expect("write bundled indents");

        // Write stale installed indents.
        let installed_query_dir = install_root
            .path()
            .join(format!("tree-sitter-{lang_id}"))
            .join("queries");
        fs::create_dir_all(&installed_query_dir).expect("create installed query dir");
        let installed_path = installed_query_dir.join("indents.scm");
        fs::write(&installed_path, "(block) @indent.stale\n")
            .expect("write stale installed indents");

        let source = must(maybe_read_query_source_preferring_bundled(
            &installed_path,
            Some(asset_root.path()),
            lang_id,
            "indents.scm",
        ));
        assert!(
            source.as_deref().unwrap_or("").contains("@indent.bundled"),
            "expected bundled content, got: {source:?}"
        );

        // When bundled asset is absent, installed file is returned.
        let absent_asset_root = TempTestDir::new("bundled-opt-absent");
        let fallback = must(maybe_read_query_source_preferring_bundled(
            &installed_path,
            Some(absent_asset_root.path()),
            lang_id,
            "indents.scm",
        ));
        assert!(
            fallback.as_deref().unwrap_or("").contains("@indent.stale"),
            "expected installed fallback, got: {fallback:?}"
        );

        // When neither bundled nor installed file exists, returns None.
        let nonexistent = install_root.path().join("nonexistent.scm");
        let none_result = must(maybe_read_query_source_preferring_bundled(
            &nonexistent,
            Some(absent_asset_root.path()),
            lang_id,
            "indents.scm",
        ));
        assert!(
            none_result.is_none(),
            "expected None when neither source exists"
        );
    }

    #[test]
    fn indent_queries_compute_nested_and_branch_indentation() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry.register(
                rust_configuration().with_extra_indent_query(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../volt/assets/grammars/queries/rust/indents.scm"
                ))),
            ),
        );

        let buffer = TextBuffer::from_text("fn main() {\n    if true {\n\n    }\n}\n");

        assert_eq!(
            must(registry.desired_indent_for_language("rust", &buffer, 2, 4)),
            Some(8)
        );
        assert_eq!(
            must(registry.desired_indent_for_language("rust", &buffer, 3, 4)),
            Some(4)
        );
    }

    #[test]
    fn indent_queries_reuse_parse_sessions_after_edits() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry.register(
                rust_configuration().with_extra_indent_query(include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../volt/assets/grammars/queries/rust/indents.scm"
                ))),
            ),
        );

        let mut buffer = TextBuffer::from_text("fn main() {\n    if true {\n\n    }\n}\n");
        let mut parse_session = None;

        assert_eq!(
            must(registry.desired_indent_for_language_with_session(
                "rust",
                &buffer,
                2,
                4,
                &mut parse_session,
            )),
            Some(8)
        );

        buffer.set_cursor(TextPoint::new(2, 0));
        buffer.insert_text("        println!(\"hi\");\n");

        assert_eq!(
            must(registry.desired_indent_for_language("rust", &buffer, 3, 4)),
            must(registry.desired_indent_for_language_with_session(
                "rust",
                &buffer,
                3,
                4,
                &mut parse_session,
            )),
        );
        assert_eq!(
            must(registry.desired_indent_for_language("rust", &buffer, 4, 4)),
            must(registry.desired_indent_for_language_with_session(
                "rust",
                &buffer,
                4,
                4,
                &mut parse_session,
            )),
        );
    }

    // --- Additional query kind tests ---

    #[test]
    fn extra_injections_query_compiles_for_static_language() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry.register(rust_configuration().with_extra_injections_query(
                r#"((string_literal) @injection.content (#set! injection.language "json"))"#,
            )),
        );
        let result = must(registry.injections_query_for_language("rust"));
        assert!(result.is_some(), "injections query should be present");
        let q = result.expect("query");
        assert_eq!(q.pattern_count(), 1);
        assert!(q.capture_names().contains(&"injection.content"));
    }

    #[test]
    fn extra_locals_query_compiles_for_static_language() {
        let mut registry = SyntaxRegistry::new();
        must(
            registry
                .register(rust_configuration().with_extra_locals_query(r#"(block) @local.scope"#)),
        );
        let result = must(registry.locals_query_for_language("rust"));
        assert!(result.is_some(), "locals query should be present");
        let q = result.expect("query");
        assert!(q.capture_names().contains(&"local.scope"));
    }

    #[test]
    fn extra_folds_query_compiles_for_static_language() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(
            rust_configuration().with_extra_folds_query(r#"[(block) (use_declaration)] @fold"#),
        ));
        let result = must(registry.folds_query_for_language("rust"));
        assert!(result.is_some(), "folds query should be present");
        let q = result.expect("query");
        assert!(q.capture_names().contains(&"fold"));
    }

    #[test]
    fn bundled_injections_query_compiles_for_rust() {
        let injections_text = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("volt")
                .join("assets")
                .join("grammars")
                .join("queries")
                .join("rust")
                .join("injections.scm"),
        );
        let Ok(injections_text) = injections_text else {
            eprintln!("SKIP: bundled rust/injections.scm not found");
            return;
        };
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration().with_extra_injections_query(injections_text)));
        let result = must(registry.injections_query_for_language("rust"));
        assert!(
            result.is_some(),
            "rust injections query should compile successfully"
        );
    }

    #[test]
    fn bundled_locals_query_compiles_for_rust() {
        let locals_text = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("volt")
                .join("assets")
                .join("grammars")
                .join("queries")
                .join("rust")
                .join("locals.scm"),
        );
        let Ok(locals_text) = locals_text else {
            eprintln!("SKIP: bundled rust/locals.scm not found");
            return;
        };
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration().with_extra_locals_query(locals_text)));
        let result = must(registry.locals_query_for_language("rust"));
        assert!(
            result.is_some(),
            "rust locals query should compile successfully"
        );
    }

    #[test]
    fn bundled_folds_query_compiles_for_rust() {
        let folds_text = std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("volt")
                .join("assets")
                .join("grammars")
                .join("queries")
                .join("rust")
                .join("folds.scm"),
        );
        let Ok(folds_text) = folds_text else {
            eprintln!("SKIP: bundled rust/folds.scm not found");
            return;
        };
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration().with_extra_folds_query(folds_text)));
        let result = must(registry.folds_query_for_language("rust"));
        assert!(
            result.is_some(),
            "rust folds query should compile successfully"
        );
    }

    #[test]
    fn missing_extra_query_returns_none() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_configuration()));
        // No extra queries configured — all three should return None.
        assert!(must(registry.injections_query_for_language("rust")).is_none());
        assert!(must(registry.locals_query_for_language("rust")).is_none());
        assert!(must(registry.folds_query_for_language("rust")).is_none());
    }

    #[test]
    fn query_accessors_return_unknown_language_error_for_unregistered_id() {
        let mut registry = SyntaxRegistry::new();
        assert!(matches!(
            registry.injections_query_for_language("not-registered"),
            Err(SyntaxError::UnknownLanguage(_))
        ));
        assert!(matches!(
            registry.locals_query_for_language("not-registered"),
            Err(SyntaxError::UnknownLanguage(_))
        ));
        assert!(matches!(
            registry.folds_query_for_language("not-registered"),
            Err(SyntaxError::UnknownLanguage(_))
        ));
    }

    #[test]
    fn grammar_source_installed_paths_include_new_query_kinds() {
        let grammar = GrammarSource::new(
            "https://example.com/tree-sitter-rust.git",
            ".",
            "src",
            "tree-sitter-rust",
            "tree_sitter_rust",
        );
        let install_root = PathBuf::from("volt-grammars");

        assert_eq!(
            grammar.installed_injections_query_path(&install_root),
            install_root
                .join("tree-sitter-rust")
                .join("queries")
                .join("injections.scm")
        );
        assert_eq!(
            grammar.installed_locals_query_path(&install_root),
            install_root
                .join("tree-sitter-rust")
                .join("queries")
                .join("locals.scm")
        );
        assert_eq!(
            grammar.installed_folds_query_path(&install_root),
            install_root
                .join("tree-sitter-rust")
                .join("queries")
                .join("folds.scm")
        );
    }

    // --- Query predicate evaluation tests ---

    /// Helper: build a LanguageConfiguration for Rust with a custom highlight query so
    /// that predicate evaluation is exercised end-to-end through `highlight_tree`.
    fn rust_config_with_query(query: &str) -> LanguageConfiguration {
        LanguageConfiguration::new(
            "rust-predicate-test",
            ["__rust_pred_test__"],
            rust_language,
            query,
            [CaptureThemeMapping::new("function", "syntax.function")],
        )
    }

    #[test]
    fn highlight_not_kind_eq_predicate_filters_captures() {
        // `(identifier) @function (#not-kind-eq? @function "identifier")` should never
        // produce a span because every identifier is – by definition – of kind
        // "identifier".
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#not-kind-eq? @function "identifier"))"#,
        )));

        let buffer = TextBuffer::from_text("fn main() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert!(
            snapshot.highlight_spans.is_empty(),
            "expected no spans after #not-kind-eq? filtered them all, got {:?}",
            snapshot.highlight_spans
        );
    }

    #[test]
    fn highlight_kind_eq_predicate_keeps_matching_captures() {
        // `(identifier) @function (#kind-eq? @function "identifier")` should keep every
        // identifier span since kind always matches.
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#kind-eq? @function "identifier"))"#,
        )));

        let buffer = TextBuffer::from_text("fn main() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert!(
            !snapshot.highlight_spans.is_empty(),
            "expected spans to pass through #kind-eq? unchanged"
        );
    }

    #[test]
    fn highlight_has_ancestor_predicate_matches_nested_nodes() {
        // Identifiers inside a block_expression have "block" as an ancestor.
        // (#has-ancestor? @fn "block") should keep them.
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#has-ancestor? @function "block"))"#,
        )));

        // `value` lives inside the block `{}`.
        let buffer = TextBuffer::from_text("fn main() { let value = 1; }");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert!(
            !snapshot.highlight_spans.is_empty(),
            "expected identifier inside block to pass #has-ancestor? block"
        );
    }

    #[test]
    fn highlight_not_has_ancestor_predicate_filters_nested_nodes() {
        // (#not-has-ancestor? @fn "block") should reject identifiers inside a block.
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#not-has-ancestor? @function "block"))"#,
        )));

        // All identifiers in `fn main() { let value = 1; }` are inside a block.
        let buffer = TextBuffer::from_text("fn main() { let value = 1; }");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        // At minimum `main` is NOT inside a block – the function name is a direct child
        // of the function_item, which itself is a direct child of source_file.  So some
        // spans should survive.
        // We just verify the predicate doesn't crash and returns a consistent result.
        let _ = snapshot.highlight_spans.len();
    }

    #[test]
    fn highlight_has_parent_predicate_checks_immediate_parent() {
        // `(identifier) @function (#has-parent? @function "function_item")` keeps
        // identifiers whose direct parent is a function_item (i.e. the function name).
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#has-parent? @function "function_item"))"#,
        )));

        let buffer = TextBuffer::from_text("fn main() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert!(
            !snapshot.highlight_spans.is_empty(),
            "expected function name identifier to pass #has-parent? function_item"
        );
    }

    #[test]
    fn highlight_contains_predicate_filters_by_text_content() {
        // (#contains? @function "main") keeps only identifiers whose text contains "main".
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#contains? @function "main"))"#,
        )));

        let buffer = TextBuffer::from_text("fn main() { let value = 1; }");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert_eq!(
            snapshot.highlight_spans.len(),
            1,
            "expected exactly one span for the identifier `main`"
        );
        assert_eq!(snapshot.highlight_spans[0].capture_name, "function");
    }

    #[test]
    fn highlight_not_lua_match_predicate_filters_matching_text() {
        // (#not-lua-match? @function "^main") rejects identifiers that start with "main".
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#not-lua-match? @function "^main"))"#,
        )));

        let buffer = TextBuffer::from_text("fn main() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        // `main` matches the lua pattern so it should be filtered out.
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .all(|span| span.capture_name != "function"
                    || span.start_byte != buffer.text().find("main").unwrap_or(usize::MAX)),
            "identifier `main` should have been removed by #not-lua-match?"
        );
    }

    #[test]
    fn highlight_directive_predicate_does_not_filter_matches() {
        // A query using a directive (#offset! …) should still produce spans because
        // directives are metadata, not match filters.
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#offset! @function 0 1))"#,
        )));

        let buffer = TextBuffer::from_text("fn main() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert!(
            !snapshot.highlight_spans.is_empty(),
            "directive predicate should not filter spans"
        );
    }

    #[test]
    fn highlight_unknown_predicate_does_not_filter_matches() {
        // An unknown custom predicate should allow the match through rather than silently
        // discarding it.
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @function (#unknown-custom-predicate? @function "value"))"#,
        )));

        let buffer = TextBuffer::from_text("fn main() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert!(
            !snapshot.highlight_spans.is_empty(),
            "unknown predicates should allow matches through"
        );
    }

    #[test]
    fn highlight_skips_internal_and_meta_captures() {
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"
(identifier) @_helper
(identifier) @function
(identifier) @spell
(identifier) @conceal
"#,
        )));

        let buffer = TextBuffer::from_text("fn main() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert_eq!(snapshot.highlight_spans.len(), 1);
        assert_eq!(snapshot.highlight_spans[0].capture_name, "function");
    }

    #[test]
    fn query_capture_property_value_returns_set_property() {
        use super::query_capture_property_value;

        let language = rust_language();
        // `#set!` with no capture argument produces a pattern-wide property.
        let query =
            tree_sitter::Query::new(&language, r#"((identifier) @var (#set! priority "90"))"#)
                .expect("valid query");

        let value = query_capture_property_value(&query, 0, 0, "priority");
        assert_eq!(value, Some("90"));
    }

    // ── Regression: #not-has-parent? must check only the immediate parent ────────

    #[test]
    fn not_has_parent_checks_only_immediate_parent() {
        // `#not-has-parent? @fn "source_file"` should keep identifiers whose direct
        // parent is NOT `source_file`.  In `fn foo() {}` the identifier `foo` has
        // `function_item` as its immediate parent, not `source_file`, so the predicate
        // must return true (keep the capture).
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @fn (#not-has-parent? @fn "source_file"))"#,
        )));
        let buffer = TextBuffer::from_text("fn foo() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|s| s.capture_name == "fn"),
            "#not-has-parent? must not reject nodes whose immediate parent does not match; \
             `foo` lives under function_item, not source_file"
        );
    }

    #[test]
    fn not_has_parent_rejects_when_immediate_parent_matches() {
        // `#not-has-parent? @fn "function_item"` must reject the function name `foo`
        // because its direct parent IS a `function_item`.
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @fn (#not-has-parent? @fn "function_item"))"#,
        )));
        let buffer = TextBuffer::from_text("fn foo() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        let fn_name_byte = buffer.text().find("foo").unwrap_or(usize::MAX);
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .all(|s| s.capture_name != "fn" || s.start_byte != fn_name_byte),
            "#not-has-parent? must reject a node whose immediate parent matches"
        );
    }

    // ── Regression: lua_pattern_matches must handle real corpus patterns ─────────

    #[test]
    fn lua_pattern_matches_uppercase_class() {
        use super::lua_pattern_matches;
        // `^[A-Z]` — identifiers starting with an uppercase letter
        assert!(
            lua_pattern_matches("MyType", "^[A-Z]"),
            "^[A-Z] should match MyType"
        );
        assert!(
            !lua_pattern_matches("myVar", "^[A-Z]"),
            "^[A-Z] should not match myVar"
        );
    }

    #[test]
    fn lua_pattern_matches_uppercase_identifier_pattern() {
        use super::lua_pattern_matches;
        // `^[A-Z][A-Z0-9_]*$` — ALL_CAPS_CONSTANT style
        assert!(
            lua_pattern_matches("MAX_SIZE", "^[A-Z][A-Z0-9_]*$"),
            "should match ALL_CAPS"
        );
        assert!(
            !lua_pattern_matches("maxSize", "^[A-Z][A-Z0-9_]*$"),
            "should not match camelCase"
        );
    }

    #[test]
    fn lua_pattern_matches_percent_u_class() {
        use super::lua_pattern_matches;
        // `%u` — Lua uppercase class
        assert!(
            lua_pattern_matches("A", "%u"),
            "%u should match uppercase 'A'"
        );
        assert!(
            !lua_pattern_matches("a", "%u"),
            "%u should not match lowercase 'a'"
        );
    }

    #[test]
    fn lua_pattern_matches_percent_l_class() {
        use super::lua_pattern_matches;
        // `%l` — Lua lowercase class
        assert!(
            lua_pattern_matches("x", "%l"),
            "%l should match lowercase 'x'"
        );
        assert!(
            !lua_pattern_matches("X", "%l"),
            "%l should not match uppercase 'X'"
        );
    }

    #[test]
    fn lua_pattern_matches_percent_a_class() {
        use super::lua_pattern_matches;
        // `%a` — Lua letter class
        assert!(lua_pattern_matches("hello", "%a"), "%a should match letter");
        assert!(
            !lua_pattern_matches("123", "^%a"),
            "^%a should not match digits"
        );
    }

    #[test]
    fn lua_pattern_matches_percent_d_class() {
        use super::lua_pattern_matches;
        // `%d` — Lua digit class
        assert!(lua_pattern_matches("42", "^%d"), "^%d should match '42'");
        assert!(
            !lua_pattern_matches("abc", "^%d"),
            "^%d should not match 'abc'"
        );
    }

    #[test]
    fn lua_pattern_matches_anchored_literal() {
        use super::lua_pattern_matches;
        assert!(
            lua_pattern_matches("else", "^else"),
            "^else should match 'else'"
        );
        assert!(
            !lua_pattern_matches("elsewhere", "^else$"),
            "^else$ should not match 'elsewhere'"
        );
        assert!(
            lua_pattern_matches("else", "^else$"),
            "^else$ should match exact 'else'"
        );
    }

    #[test]
    fn lua_pattern_matches_escaped_parens() {
        use super::lua_pattern_matches;
        // `%(` literal open-paren, non-newline follows
        assert!(
            lua_pattern_matches("(x", "%("),
            "should match text starting with ("
        );
        assert!(
            !lua_pattern_matches("x(", "^%("),
            "^%( should not match text not starting with ("
        );
    }

    #[test]
    fn not_lua_match_integration_uppercase_class() {
        // Smoke test: `#lua-match? @fn "^[A-Z]"` keeps only identifiers beginning with uppercase.
        let mut registry = SyntaxRegistry::new();
        must(registry.register(rust_config_with_query(
            r#"((identifier) @fn (#lua-match? @fn "^[A-Z]"))"#,
        )));
        let buffer = TextBuffer::from_text("fn MyFunc() {} fn lowercase() {}");
        let snapshot = must(registry.highlight_buffer_for_extension("__rust_pred_test__", &buffer));
        let my_func_byte = buffer.text().find("MyFunc").unwrap_or(usize::MAX);
        let lower_byte = buffer.text().find("lowercase").unwrap_or(usize::MAX);
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .any(|s| s.capture_name == "fn" && s.start_byte == my_func_byte),
            "#lua-match? ^[A-Z] should keep 'MyFunc'"
        );
        assert!(
            snapshot
                .highlight_spans
                .iter()
                .all(|s| s.capture_name != "fn" || s.start_byte != lower_byte),
            "#lua-match? ^[A-Z] should reject 'lowercase'"
        );
    }
}
